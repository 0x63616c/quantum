//! Native project launcher and recent-file integration. The editor remains the
//! sole document owner; requests made here reuse its unsaved-change flow.
use super::{EditorView, hint, settings, welcome_style as layout};
use crate::{
    actions::{self, Op, OpenRecentProject},
    shortcuts::{ShortcutPlatform, navigation_key},
    theme::Theme,
};
use gpui::{prelude::*, *};
use quantum_app::{
    editor::Editor,
    recents::{RecentProject, Recents, folder_label, modified_label},
    samples::Sample,
    thumbnails::{SamplePreview, ThumbnailCache},
};
use quantum_ui::{
    AsciiReveal, Button, SidebarItem, icon, row,
    tokens::{metrics, radius, space, type_size},
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::PathBuf,
};

struct Launcher {
    editor: Option<WindowHandle<EditorView>>,
    welcome: Option<WindowHandle<WelcomeView>>,
    socket: Option<PathBuf>,
    projects: Vec<RecentProject>,
    error: Option<String>,
    // Serialize metadata changes across windows, including Clear while an open
    // or save is finishing. Rendering has a separate bounded background worker.
    work: Option<Task<()>>,
    notification: Option<Task<()>>,
    continuation: Option<Task<()>>,
    revision: u64,
    preview_work: Option<Task<()>>,
    preview_queue: VecDeque<PathBuf>,
}
impl Global for Launcher {}

enum HistoryChange {
    Reload,
    Record(PathBuf, String),
    Clear,
}

pub(crate) fn init(
    theme: Option<Theme>,
    socket: Option<PathBuf>,
    preferences: quantum_app::editor::EditorResult<quantum_app::preferences::Preferences>,
    cx: &mut App,
) {
    settings::init_loaded(theme, preferences, cx);
    cx.set_global(Launcher {
        editor: None,
        welcome: None,
        socket,
        projects: Vec::new(),
        error: None,
        work: None,
        notification: None,
        continuation: None,
        revision: 0,
        preview_work: None,
        preview_queue: VecDeque::new(),
    });
    // Let GPUI's graceful shutdown drain metadata writes even when the user
    // closes the last window immediately after opening, saving, or clearing.
    // Thumbnails remain disposable and can be generated at the next launch.
    observe_quit(cx);
    history(HistoryChange::Reload, cx);
}

fn observe_quit(cx: &mut App) {
    cx.on_app_quit(|cx| {
        let state = cx.global_mut::<Launcher>();
        state.notification.take();
        state.continuation.take();
        state.preview_work.take();
        let work = state.work.take();
        async move {
            if let Some(work) = work {
                work.await;
            }
        }
    })
    .detach();
}

fn history(change: HistoryChange, cx: &mut App) {
    history_at(change, quantum_app::local_settings::data_dir(), cx);
}

fn history_at(change: HistoryChange, directory: PathBuf, cx: &mut App) {
    let previous = cx.global_mut::<Launcher>().work.take();
    let (done, receive) = futures_channel::oneshot::channel();
    // Serialize only disk work on the background executor. The quit hook can
    // drain this chain without borrowing the App or running UI continuations.
    let work = cx.background_executor().spawn(async move {
        if let Some(previous) = previous {
            previous.await;
        }
        let store = Recents::new(&directory);
        let result = match change {
            HistoryChange::Reload => store.load(),
            HistoryChange::Record(path, name) => store.record(&path, &name),
            HistoryChange::Clear => store.clear().and_then(|()| store.load()),
        };
        if let Err(error) = &result {
            eprintln!("Could not update recent projects: {error}");
        }
        let _ = done.send(result);
    });
    let notification = cx.spawn(async move |cx| {
        let Ok(result) = receive.await else {
            return;
        };
        cx.update(|cx| {
            cx.update_global::<Launcher, _>(|state, _| {
                match result {
                    Ok(projects) => {
                        state.projects = projects;
                        state.error = None;
                    }
                    Err(error) => {
                        state.error = Some(format!("Could not update recent projects: {error}"))
                    }
                }
                state.revision = state.revision.wrapping_add(1);
            })
        });
    });
    let state = cx.global_mut::<Launcher>();
    state.work = Some(work);
    state.notification = Some(notification);
}

pub(super) fn record(editor: &Editor, cx: &mut App) {
    let Some(path) = editor.path.clone() else {
        return;
    };
    history(
        HistoryChange::Record(path.clone(), editor.snapshot().name.clone()),
        cx,
    );
    let state = cx.global_mut::<Launcher>();
    queue_preview(&mut state.preview_queue, path);
    if state.preview_work.is_some() {
        return;
    }
    let cache = ThumbnailCache::new(&quantum_app::local_settings::data_dir());
    let task = cx.spawn(async move |cx| {
        loop {
            let path = cx.update(|cx| cx.global_mut::<Launcher>().preview_queue.pop_front());
            let Some(path) = path else {
                break;
            };
            let cache = cache.clone();
            if let Err(error) = preview_job(cx.background_executor().clone(), move || {
                cache.ensure(&path)
            })
            .await
            {
                eprintln!("Could not create project preview: {error}");
            }
        }
        cx.update(|cx| cx.global_mut::<Launcher>().preview_work = None);
    });
    cx.global_mut::<Launcher>().preview_work = Some(task);
}

// Coalesce repeat saves and bound pending work to the recent-history capacity.
fn queue_preview(queue: &mut VecDeque<PathBuf>, path: PathBuf) {
    queue.retain(|queued| queued != &path);
    queue.push_back(path);
    while queue.len() > quantum_app::recents::CAPACITY {
        queue.pop_front();
    }
}

// Wait asynchronously BEFORE occupying a background thread. The permit travels
// with the blocking job, so cancelling a view cannot release it mid-render.
static PREVIEWS: async_lock::Semaphore = async_lock::Semaphore::new(1);
async fn preview_job<T: Send + 'static>(
    executor: BackgroundExecutor,
    job: impl FnOnce() -> T + Send + 'static,
) -> T {
    let permit = PREVIEWS.acquire().await;
    executor
        .spawn(async move {
            let _permit = permit;
            job()
        })
        .await
}

pub(super) fn observe_editor(cx: &mut Context<EditorView>) {
    cx.observe_global::<Launcher>(|_, cx| cx.notify()).detach();
}

// A Save & Quit continuation must not exit before the recent entry is durable.
pub(super) fn after_record(op: Op, cx: &mut Context<EditorView>) {
    let previous = cx.global_mut::<Launcher>().work.take();
    let (done, receive) = futures_channel::oneshot::channel();
    let work = cx.background_executor().spawn(async move {
        if let Some(previous) = previous {
            previous.await;
        }
        let _ = done.send(());
    });
    let continuation = cx.spawn(async move |this, cx| {
        if receive.await.is_err() {
            return;
        }
        let _ = this.update(cx, |this, cx| {
            this.save_flow.finish_dialog();
            this.run(op, cx);
        });
    });
    let state = cx.global_mut::<Launcher>();
    state.work = Some(work);
    state.continuation = Some(continuation);
}

pub(super) fn clear(cx: &mut App) {
    history(HistoryChange::Clear, cx);
}

/// Fill the registry's submenu with the current snapshot, without disk I/O in render.
pub(super) fn menus(mut menus: Vec<Menu>, enabled: bool, cx: &App) -> Vec<Menu> {
    let state = cx.global::<Launcher>();
    for menu in &mut menus {
        for item in &mut menu.items {
            if let MenuItem::Submenu(submenu) = item
                && submenu.name == Op::OpenRecent.label()
            {
                submenu.items = state
                    .projects
                    .iter()
                    .map(|project| {
                        MenuItem::action(
                            project.name.clone(),
                            OpenRecentProject {
                                path: project.path.clone(),
                            },
                        )
                        .disabled(!enabled)
                    })
                    .collect();
                if !submenu.items.is_empty() {
                    submenu.items.push(MenuItem::separator());
                }
                submenu.items.push(
                    MenuItem::action(Op::ClearRecents.label(), Op::ClearRecents)
                        .disabled(!enabled || state.projects.is_empty()),
                );
            }
        }
    }
    menus
}

pub(crate) fn open_editor(editor: Editor, cx: &mut App) -> anyhow::Result<()> {
    let socket = cx.global::<Launcher>().socket.clone();
    let bounds = Bounds::centered(
        None,
        size(px(layout::EDITOR_WIDTH), px(layout::EDITOR_HEIGHT)),
        cx,
    );
    let handle = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: if cfg!(target_os = "macos") {
                Some(TitlebarOptions {
                    title: Some("Quantum".into()),
                    appears_transparent: true,
                    traffic_light_position: Some(point(
                        px(layout::EDITOR_TRAFFIC_LEFT),
                        px(space::XXL),
                    )),
                })
            } else {
                None
            },
            app_owns_titlebar_drag: cfg!(target_os = "macos"),
            window_min_size: Some(size(
                px(layout::EDITOR_MIN_WIDTH),
                px(metrics::SETTINGS_HEIGHT),
            )),
            app_id: Some(quantum_app::APP_ID.into()),
            ..Default::default()
        },
        |window, cx| cx.new(|cx| EditorView::new(editor, None, socket, window, cx)),
    )?;
    cx.global_mut::<Launcher>().editor = Some(handle);
    project_ready(cx);
    Ok(())
}

/// Apply a launch-only screenshot fixture to the editor just opened by main.
pub(crate) fn show_capture(state: super::launch::LaunchState, cx: &mut App) -> anyhow::Result<()> {
    let handle = cx
        .global::<Launcher>()
        .editor
        .ok_or_else(|| anyhow::anyhow!("capture requires an editor window"))?;
    handle.update(cx, |view, window, cx| view.show_capture(state, window, cx))?
}

pub(super) fn project_ready(cx: &mut App) {
    // Opening a window renders immediately; defer updates of the originating view.
    cx.defer(|cx| {
        if let Some(handle) = cx.global_mut::<Launcher>().welcome.take() {
            let _ = handle.update(cx, |_, window, _| window.remove_window());
        }
        if let Some(handle) = cx.global::<Launcher>().editor {
            let _ = handle.update(cx, |_, window, _| window.activate_window());
        }
    });
}

pub(crate) fn startup(cx: &mut App) -> anyhow::Result<()> {
    if cx.global::<settings::SettingsState>().values.show_welcome {
        show(cx).map(|_| ())
    } else {
        open_editor(Editor::blank()?, cx)
    }
}

pub(crate) fn show(cx: &mut App) -> anyhow::Result<WindowHandle<WelcomeView>> {
    if let Some(handle) = cx.global::<Launcher>().welcome
        && handle
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
    {
        return Ok(handle);
    }
    let bounds = Bounds::centered(None, size(px(layout::WIDTH), px(layout::HEIGHT)), cx);
    let handle = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(px(layout::WIDTH), px(layout::HEIGHT))),
            titlebar: Some(TitlebarOptions {
                title: (!cfg!(target_os = "macos")).then(|| "Quantum".into()),
                appears_transparent: true,
                traffic_light_position: Some(point(px(space::SECTION), px(metrics::TRAFFIC_TOP))),
            }),
            app_owns_titlebar_drag: cfg!(target_os = "macos"),
            window_background: if cfg!(target_os = "macos") {
                WindowBackgroundAppearance::Blurred
            } else {
                WindowBackgroundAppearance::Opaque
            },
            app_id: Some(quantum_app::APP_ID.into()),
            ..Default::default()
        },
        |window, cx| cx.new(|cx| WelcomeView::new(window, cx)),
    )?;
    cx.global_mut::<Launcher>().welcome = Some(handle);
    Ok(handle)
}

pub(super) fn request(op: Op, path: Option<PathBuf>, cx: &mut App) {
    cx.defer(move |cx| {
        if let Some(handle) = cx.global::<Launcher>().editor
            && handle
                .update(cx, |view, window, cx| {
                    window.activate_window();
                    if let Some(path) = path.clone() {
                        view.open_recent(path, window, cx);
                    } else {
                        view.invoke(op, window, cx);
                    }
                })
                .is_ok()
        {
            return;
        }
        if op == Op::Quit {
            cx.quit();
            return;
        }
        match show(cx) {
            Ok(handle) => {
                let _ = handle.update(cx, |view, window, cx| view.run(op, path, window, cx));
            }
            Err(error) => eprintln!("Could not open welcome window: {error}"),
        }
    });
}

pub(crate) struct WelcomeView {
    focus: FocusHandle,
    list_focus: FocusHandle,
    sidebar_focus: [FocusHandle; 4],
    samples: HashMap<Sample, SamplePreview>,
    sample_task: Option<Task<()>>,
    scroll: ScrollHandle,
    selection: Option<Selection>,
    project_paths: Vec<PathBuf>,
    images: Entity<RetainAllImageCache>,
    image_paths: HashSet<PathBuf>,
    previews: HashMap<PathBuf, Option<super::loading::Preview>>,
    preview_projects: Vec<(PathBuf, std::time::SystemTime)>,
    reduced_motion: bool,
    retired_previews: Vec<std::sync::Arc<RenderImage>>,
    revision: u64,
    thumbnails: Option<Task<()>>,
    busy: bool,
    error: Option<String>,
}
impl WelcomeView {
    pub(crate) fn show_error(&mut self, error: Option<String>, cx: &mut Context<Self>) {
        self.error = error;
        cx.notify();
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Launcher>(|this, cx| {
            this.refresh_previews(cx);
            cx.notify();
        })
        .detach();
        cx.observe_global::<settings::SettingsState>(|_, cx| cx.notify())
            .detach();
        cx.observe_window_appearance(window, |_, _, cx| cx.notify())
            .detach();
        cx.observe_window_activation(window, |this, window, cx| {
            if window.is_window_active() {
                this.reduced_motion = quantum_app::reduce_motion::prefers_reduced_motion();
                history(HistoryChange::Reload, cx);
            }
            cx.notify();
        })
        .detach();
        let mut view = Self::initial(cx);
        window.focus(&view.focus, cx);
        view.refresh_previews(cx);
        let cache = ThumbnailCache::new(&quantum_app::local_settings::data_dir());
        view.sample_task = Some(cx.spawn(async move |this, cx| {
            for sample in Sample::ALL {
                let cache = cache.clone();
                let result = preview_job(cx.background_executor().clone(), move || {
                    cache.sample(sample)
                })
                .await;
                if this
                    .update(cx, |this, cx| {
                        match result {
                            Ok(preview) => {
                                this.samples.insert(sample, preview);
                            }
                            Err(error) => eprintln!("Could not render sample preview: {error}"),
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        }));
        view
    }

    fn initial(cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            list_focus: cx.focus_handle(),
            sidebar_focus: std::array::from_fn(|_| cx.focus_handle()),
            samples: HashMap::new(),
            sample_task: None,
            scroll: ScrollHandle::new(),
            selection: None,
            project_paths: Vec::new(),
            images: RetainAllImageCache::new(cx),
            image_paths: HashSet::new(),
            previews: HashMap::new(),
            preview_projects: Vec::new(),
            reduced_motion: quantum_app::reduce_motion::prefers_reduced_motion(),
            retired_previews: Vec::new(),
            revision: u64::MAX,
            thumbnails: None,
            busy: false,
            error: None,
        }
    }

    fn refresh_previews(&mut self, cx: &mut Context<Self>) {
        let state = cx.global::<Launcher>();
        if self.revision == state.revision {
            return;
        }
        let revision = state.revision;
        let projects = state.projects.clone();
        let paths: Vec<_> = projects.iter().map(|p| p.path.clone()).collect();
        self.selection = reconcile_selection(&self.project_paths, &paths, self.selection.take());
        self.project_paths = paths;
        let stamps: Vec<_> = projects
            .iter()
            .map(|p| (p.path.clone(), p.modified))
            .collect();
        if self.preview_projects == stamps {
            return;
        }
        self.revision = revision;
        self.preview_projects = stamps;
        self.retired_previews.extend(
            self.previews
                .drain()
                .filter_map(|(_, preview)| preview.map(|p| p.image)),
        );
        let cache = ThumbnailCache::new(&quantum_app::local_settings::data_dir());
        self.thumbnails = Some(cx.spawn(async move |this, cx| {
            // One preview at a time bounds memory and progressively fills placeholders.
            for project in projects {
                let path = project.path.clone();
                let cache = cache.clone();
                let result = preview_job(cx.background_executor().clone(), move || {
                    super::loading::recent(&cache.ensure(&path)?)
                })
                .await;
                if this
                    .update(cx, |this, cx| {
                        if this.revision == revision {
                            this.previews
                                .insert(project.path, result.ok().map(|p| p.ready()));
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        }));
    }

    fn run(&mut self, op: Op, path: Option<PathBuf>, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        if matches!(op, Op::New | Op::Open | Op::Quit)
            && let Some(handle) = cx.global::<Launcher>().editor
        {
            // A stale handle is cleared before retrying in this window.
            if handle.read(cx).is_ok() {
                request(op, path, cx);
                return;
            }
            cx.global_mut::<Launcher>().editor = None;
        }
        match op {
            Op::New => self.finish(Editor::blank(), cx),
            Op::Open => {
                if let Some(path) = path {
                    self.load(path, cx);
                } else {
                    self.choose(cx);
                }
            }
            Op::OpenSample => self.focus_samples(window, cx),
            Op::OpenRecent | Op::Welcome => {
                window.focus(&self.list_focus, cx);
            }
            Op::ClearRecents => clear(cx),
            Op::Quit => cx.quit(),
            Op::Hide => cx.hide(),
            Op::HideOthers => cx.hide_other_apps(),
            Op::ShowAll => cx.unhide_other_apps(),
            Op::Minimize => window.minimize_window(),
            Op::ZoomWindow => window.zoom_window(),
            _ => {}
        }
        cx.notify();
    }

    pub(super) fn focus_samples(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.selection = Some(Selection::Sample(Sample::Autumn));
        window.focus(&self.list_focus, cx);
        cx.notify();
    }

    fn open_sample(&mut self, sample: Sample, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        if let Some(handle) = cx.global::<Launcher>().editor
            && handle.read(cx).is_ok()
        {
            cx.defer(move |cx| {
                let _ = handle.update(cx, |view, window, cx| {
                    window.activate_window();
                    view.open_sample(sample, window, cx);
                });
            });
            return;
        }
        self.busy = true;
        self.error = None;
        let task = cx
            .background_executor()
            .spawn(async move { sample.session() });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.finish(result.and_then(Editor::from_session), cx)
            });
        })
        .detach();
        cx.notify();
    }

    fn choose(&mut self, cx: &mut Context<Self>) {
        self.busy = true;
        self.error = None;
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open .voxl project".into()),
        });
        cx.spawn(async move |this, cx| {
            let result = receiver.await;
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(Ok(Some(paths))) if !paths.is_empty() => this.load(paths[0].clone(), cx),
                    Ok(Ok(_)) => {}
                    _ => this.error = Some("Could not show the Open dialog.".into()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn load(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.busy = true;
        self.error = None;
        let task = cx
            .background_executor()
            .spawn(async move { Editor::load_session(&path) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.finish(result.and_then(Editor::from_session), cx)
            });
        })
        .detach();
    }

    fn finish(
        &mut self,
        result: quantum_app::editor::EditorResult<Editor>,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(editor) => {
                self.busy = true;
                let this = cx.weak_entity();
                cx.defer(move |cx| {
                    if let Err(error) = open_editor(editor, cx) {
                        let _ = this.update(cx, |this, cx| {
                            this.busy = false;
                            this.error = Some(error.to_string());
                            cx.notify();
                        });
                    }
                });
            }
            Err(error) => {
                self.busy = false;
                self.error = Some(error.to_string());
            }
        }
        cx.notify();
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Selection {
    Recent(PathBuf),
    Sample(Sample),
}

impl Selection {
    fn at(paths: &[PathBuf], index: usize) -> Self {
        if let Some(path) = paths.get(index) {
            Self::Recent(path.clone())
        } else {
            Self::Sample(
                Sample::ALL[(index.saturating_sub(paths.len())).min(Sample::ALL.len() - 1)],
            )
        }
    }

    fn index(&self, paths: &[PathBuf]) -> usize {
        match self {
            Self::Recent(path) => paths.iter().position(|p| p == path).unwrap_or(0),
            Self::Sample(sample) => {
                paths.len() + Sample::ALL.iter().position(|s| s == sample).unwrap_or(0)
            }
        }
    }

    fn step(&self, paths: &[PathBuf], forward: bool) -> Self {
        let index = self.index(paths);
        Self::at(
            paths,
            if forward {
                (index + 1).min(paths.len() + Sample::ALL.len() - 1)
            } else {
                index.saturating_sub(1)
            },
        )
    }
}

fn reconcile_selection(
    old: &[PathBuf],
    new: &[PathBuf],
    selection: Option<Selection>,
) -> Option<Selection> {
    selection.map(|selection| match &selection {
        Selection::Recent(path) if !new.contains(path) => {
            let index = selection.index(old).min(new.len().saturating_sub(1));
            Selection::at(new, index)
        }
        _ => selection,
    })
}

impl WelcomeView {
    fn selected_item(&self) -> Selection {
        self.selection
            .clone()
            .unwrap_or_else(|| Selection::at(&self.project_paths, 0))
    }

    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if WelcomeKey::parse(&event.keystroke.key) == Some(WelcomeKey::Close)
            && ShortcutPlatform::current().is_primary(event.keystroke.modifiers)
        {
            window.remove_window();
            cx.stop_propagation();
            return;
        }
        let Some(key) = navigation_key(&event.keystroke).and_then(WelcomeKey::parse) else {
            return;
        };
        if key == WelcomeKey::Tab {
            let stops = [
                &self.sidebar_focus[0],
                &self.sidebar_focus[1],
                &self.sidebar_focus[2],
                &self.sidebar_focus[3],
                &self.list_focus,
            ];
            let current = stops.iter().position(|focus| focus.is_focused(window));
            let next = match (current, event.keystroke.modifiers.shift) {
                (Some(index), false) => (index + 1) % stops.len(),
                (Some(index), true) => (index + stops.len() - 1) % stops.len(),
                (None, false) => 0,
                (None, true) => stops.len() - 1,
            };
            window.focus(stops[next], cx);
        } else if key == WelcomeKey::Escape {
            // Welcome is also available while an editor is open. Escape returns
            // to that editor, or closes the standalone launcher like primary-W.
            if let Some(editor) = cx.global::<Launcher>().editor {
                cx.defer(move |cx| {
                    let _ = editor.update(cx, |_, window, _| window.activate_window());
                });
            }
            window.remove_window();
        } else {
            // Sidebar controls retain native Space/Enter activation. Arrows only
            // navigate the project list (or the initial, un-tabbed window focus).
            if self.busy || (!self.focus.is_focused(window) && !self.list_focus.is_focused(window))
            {
                return;
            }
            let selected = self.selected_item();
            match key {
                WelcomeKey::Previous => {
                    self.selection = Some(selected.step(&self.project_paths, false))
                }
                WelcomeKey::Next => self.selection = Some(selected.step(&self.project_paths, true)),
                WelcomeKey::Open => match selected {
                    Selection::Recent(path) => self.run(Op::Open, Some(path), window, cx),
                    Selection::Sample(sample) => self.open_sample(sample, cx),
                },
                _ => return,
            }
            if let Selection::Recent(_) = self.selected_item() {
                self.scroll
                    .scroll_to_item(self.selected_item().index(&self.project_paths));
            }
            window.focus(&self.list_focus, cx);
        }
        cx.stop_propagation();
        cx.notify();
    }
}

fn grouped_count(value: usize) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (index, c) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

mod view;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq)]
enum WelcomeKey {
    Previous,
    Next,
    Open,
    Tab,
    Escape,
    Close,
}
impl WelcomeKey {
    fn parse(key: &str) -> Option<Self> {
        const KEYS: &[(&str, WelcomeKey)] = &[
            ("up", WelcomeKey::Previous),
            ("left", WelcomeKey::Previous),
            ("down", WelcomeKey::Next),
            ("right", WelcomeKey::Next),
            ("enter", WelcomeKey::Open),
            ("tab", WelcomeKey::Tab),
            ("escape", WelcomeKey::Escape),
            ("w", WelcomeKey::Close),
        ];
        KEYS.iter()
            .find(|(name, _)| *name == key)
            .map(|(_, key)| *key)
    }
}

pub(super) fn show_deferred(samples: bool, cx: &mut App) {
    cx.defer(move |cx| match show(cx) {
        Ok(handle) if samples => {
            let _ = handle.update(cx, |view, window, cx| view.focus_samples(window, cx));
        }
        Ok(_) => {}
        Err(error) => eprintln!("Could not open welcome window: {error}"),
    });
}
