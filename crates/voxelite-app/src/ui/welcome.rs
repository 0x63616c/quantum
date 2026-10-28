//! Native project launcher and recent-file integration. The editor remains the
//! sole document owner; requests made here reuse its unsaved-change flow.
use super::{EditorView, hint, settings, welcome_style as layout};
use crate::{
    actions::{self, Op, OpenRecentProject},
    shortcuts::{ShortcutPlatform, navigation_key},
    theme::Theme,
};
use gpui::{prelude::*, *};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::PathBuf,
};
use voxelite_app::{
    editor::Editor,
    recents::{RecentProject, Recents, folder_label, modified_label},
    samples::Sample,
    thumbnails::{SamplePreview, ThumbnailCache},
};
use voxelite_ui::{
    Button, SidebarItem, icon, row,
    tokens::{metrics, radius, space, type_size},
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
    preferences: voxelite_app::editor::EditorResult<voxelite_app::preferences::Preferences>,
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
    history_at(change, voxelite_app::local_settings::data_dir(), cx);
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
    let cache = ThumbnailCache::new(&voxelite_app::local_settings::data_dir());
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
    while queue.len() > voxelite_app::recents::CAPACITY {
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
            app_id: Some(voxelite_app::APP_ID.into()),
            ..Default::default()
        },
        |window, cx| cx.new(|cx| EditorView::new(editor, None, socket, window, cx)),
    )?;
    cx.global_mut::<Launcher>().editor = Some(handle);
    project_ready(cx);
    Ok(())
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
            app_id: Some(voxelite_app::APP_ID.into()),
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
    previews: HashMap<PathBuf, PathBuf>,
    revision: u64,
    thumbnails: Option<Task<()>>,
    busy: bool,
    error: Option<String>,
}
impl WelcomeView {
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
        cx.observe_window_activation(window, |_, window, cx| {
            if window.is_window_active() {
                history(HistoryChange::Reload, cx);
            }
            cx.notify();
        })
        .detach();
        let mut view = Self::initial(cx);
        window.focus(&view.focus, cx);
        view.refresh_previews(cx);
        let cache = ThumbnailCache::new(&voxelite_app::local_settings::data_dir());
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
        self.revision = state.revision;
        let revision = self.revision;
        let projects = state.projects.clone();
        let paths: Vec<_> = projects.iter().map(|p| p.path.clone()).collect();
        self.selection = reconcile_selection(&self.project_paths, &paths, self.selection.take());
        self.project_paths = paths;
        // An invalidated or corrupt file must never retain its old artwork.
        self.previews.clear();
        let cache = ThumbnailCache::new(&voxelite_app::local_settings::data_dir());
        self.thumbnails = Some(cx.spawn(async move |this, cx| {
            // One preview at a time bounds memory and progressively fills placeholders.
            for project in projects {
                let path = project.path.clone();
                let cache = cache.clone();
                let result = preview_job(cx.background_executor().clone(), move || {
                    cache.ensure(&path)
                })
                .await;
                if this
                    .update(cx, |this, cx| {
                        if this.revision == revision {
                            if let Ok(preview) = result {
                                this.previews.insert(project.path, preview);
                            }
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
        let task = cx.background_executor().spawn(async move {
            voxelite_format::session::DocumentSession::open(&path, voxelite_core::Limits::default())
                .map_err(voxelite_app::editor::EditorError::from)
        });
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
        result: voxelite_app::editor::EditorResult<Editor>,
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

impl WelcomeView {
    fn sample_card(
        &self,
        index: usize,
        compact: bool,
        focused: bool,
        t: Theme,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let sample = Sample::ALL[index];
        let preview = self.samples.get(&sample);
        let selected = self.selected_item() == Selection::Sample(sample);
        Button::custom(("sample", index), sample.name())
            .tab_stop(false)
            .flex_1()
            .min_w(px(space::NONE))
            .flex()
            .flex_col()
            .items_start()
            .text_color(t.text())
            .role(Role::ListBoxOption)
            .aria_selected(selected)
            .when(selected, |d| d.aria_active_descendant())
            .when(!self.busy, |d| {
                d.on_click(cx.listener(move |this, _, window, cx| {
                    this.selection = Some(Selection::Sample(sample));
                    window.focus(&this.list_focus, cx);
                    this.open_sample(sample, cx);
                }))
            })
            .tooltip(hint(sample.name(), t))
            .child(
                div()
                    .w_full()
                    .h(px(if compact {
                        layout::SAMPLE_SMALL
                    } else {
                        layout::SAMPLE_LARGE
                    }))
                    .child(
                        layout::thumbnail(
                            preview.map(|p| &p.path),
                            t,
                            true,
                            if compact {
                                layout::SAMPLE_SMALL
                            } else {
                                layout::SAMPLE_LARGE
                            },
                        )
                        .when(selected && (!compact || focused), |d| {
                            d.border_color(t.accent())
                        })
                        .when(selected && focused, |d| {
                            d.shadow(voxelite_ui::tokens::focus_ring(t))
                        }),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .mt(px(space::MD))
                    .text_size(px(if compact {
                        type_size::BODY
                    } else {
                        type_size::BODY_LG
                    }))
                    .h(px(type_size::LINE_SM))
                    .line_height(px(type_size::LINE_SM))
                    .font_weight(if compact {
                        type_size::NORMAL
                    } else {
                        type_size::MEDIUM
                    })
                    .when(compact, |d| d.text_color(t.muted()))
                    .truncate()
                    .child(sample.name()),
            )
            .when(!compact, |d| {
                d.child(
                    div()
                        .w_full()
                        .mt(px(space::XXS))
                        .text_size(px(type_size::CAPTION))
                        .h(px(type_size::LINE_SM))
                        .line_height(px(type_size::LINE_SM))
                        .text_color(t.muted())
                        .truncate()
                        .child(
                            preview
                                .map(|p| {
                                    format!(
                                        "{} voxels · {} {}",
                                        grouped_count(p.voxels),
                                        p.layers,
                                        if p.layers == 1 { "layer" } else { "layers" }
                                    )
                                })
                                .unwrap_or_default(),
                        ),
                )
            })
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
        if event.keystroke.key == "w"
            && ShortcutPlatform::current().is_primary(event.keystroke.modifiers)
        {
            window.remove_window();
            cx.stop_propagation();
            return;
        }
        let Some(key) = navigation_key(&event.keystroke) else {
            return;
        };
        if key == "tab" {
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
        } else if key == "escape" {
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
                "up" | "left" => self.selection = Some(selected.step(&self.project_paths, false)),
                "down" | "right" => self.selection = Some(selected.step(&self.project_paths, true)),
                "enter" => match selected {
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

impl Render for WelcomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let paths: HashSet<_> = self
            .previews
            .values()
            .cloned()
            .chain(self.samples.values().map(|p| p.path.clone()))
            .collect();
        for path in self.image_paths.difference(&paths) {
            self.images.update(cx, |images, cx| {
                images.remove(&Resource::Path(path.clone().into()), window, cx)
            });
        }
        self.image_paths = paths;
        let settings = cx.global::<settings::SettingsState>();
        let t = settings::theme(settings.values.appearance, window);
        let show_welcome = settings.values.show_welcome;
        let projects = cx.global::<Launcher>().projects.clone();
        let error = self
            .error
            .clone()
            .or_else(|| cx.global::<Launcher>().error.clone())
            .or(settings.error.clone());
        if cfg!(target_os = "macos") && window.is_window_active() {
            let native_menus = actions::menus_for_window(
                window.is_fullscreen(),
                |_| false,
                |op| {
                    !self.busy
                        && matches!(
                            op,
                            Op::New
                                | Op::Open
                                | Op::OpenSample
                                | Op::OpenRecent
                                | Op::ClearRecents
                                | Op::Welcome
                                | Op::Quit
                                | Op::Hide
                                | Op::HideOthers
                                | Op::ShowAll
                                | Op::Minimize
                                | Op::ZoomWindow
                        )
                },
            );
            cx.set_menus(menus(native_menus, !self.busy, cx));
        }
        let home = std::env::var_os("HOME").map(PathBuf::from);
        let empty = projects.is_empty();
        let mut sidebar = div()
            .relative()
            .w(px(layout::SIDEBAR))
            .h_full()
            .flex_shrink_0()
            .border_r_1()
            .border_color(t.line())
            .pt(px(layout::SIDEBAR_TOP))
            .px(px(space::XL))
            .pb(px(space::XXL))
            .flex()
            .flex_col()
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h(px(layout::SIDEBAR_TOP))
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move()),
            )
            .child(
                img("welcome-mark.png")
                    .size(px(layout::MARK))
                    .ml(-px(space::XS)),
            )
            .child(
                div()
                    .mt(px(space::XL))
                    .text_size(px(type_size::DISPLAY))
                    .line_height(px(layout::NAME_LINE))
                    .font_weight(type_size::SEMIBOLD)
                    .child("Quantum"),
            )
            .child(
                div()
                    .mt(px(space::XS))
                    .font_family(type_size::MONO)
                    .text_color(t.muted())
                    .text_size(px(type_size::CAPTION))
                    .line_height(px(type_size::LINE_SM))
                    .child(format!("Version {}", env!("CARGO_PKG_VERSION"))),
            );
        let mut actions = div()
            .mt(px(space::XXXL))
            .mx(-px(space::MD))
            .flex()
            .flex_col()
            .gap(px(space::XXS));
        for (tab, (op, label)) in [
            (Op::New, "New Project"),
            (Op::Open, "Open…"),
            (Op::OpenSample, "Open Sample…"),
        ]
        .into_iter()
        .enumerate()
        {
            actions = actions.child(
                Button::custom(op.command_id(), label)
                    .track_focus(&self.sidebar_focus[tab])
                    .tab_index(0)
                    .tooltip(hint(
                        format!(
                            "{label} ({})",
                            crate::shortcuts::shortcut_tokens(
                                op.key().unwrap_or(""),
                                ShortcutPlatform::current()
                            )
                            .iter()
                            .map(|t| t.text())
                            .collect::<String>()
                        ),
                        t,
                    ))
                    .h(px(layout::ACTION))
                    .flex_shrink_0()
                    .px(px(space::MD))
                    .rounded(px(radius::CONTROL))
                    .flex()
                    .items_center()
                    .gap(px(space::INSET))
                    .text_size(px(type_size::BODY_LG))
                    .when(op == Op::New, |d| d.bg(t.hover()))
                    .hover(move |d| d.bg(t.hover()))
                    .focus_visible(move |d| {
                        d.bg(t.hover()).shadow(voxelite_ui::tokens::focus_ring(t))
                    })
                    .when(self.busy, |d| d.opacity(0.5))
                    .child(
                        icon(op.icon())
                            .size(px(metrics::ICON_CHECK))
                            .text_color(t.muted())
                            .flex_shrink_0(),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(space::NONE))
                            .truncate()
                            .child(label),
                    )
                    .child(layout::caps(op.key().unwrap_or(""), t))
                    .when(!self.busy, |d| {
                        d.on_click(
                            cx.listener(move |this, _, window, cx| this.run(op, None, window, cx)),
                        )
                    }),
            );
        }
        sidebar = sidebar.child(actions).child(div().flex_1()).child(
            Button::custom("show-at-launch", "Show at launch")
                .track_focus(&self.sidebar_focus[3])
                .tab_index(0)
                .tooltip(hint("Show Welcome when Quantum launches", t))
                .role(Role::CheckBox)
                .aria_toggled(if show_welcome {
                    Toggled::True
                } else {
                    Toggled::False
                })
                .flex()
                .items_center()
                .gap(px(space::MD))
                .text_color(t.muted())
                .focus_visible(move |d| d.shadow(voxelite_ui::tokens::focus_ring(t)))
                .child(
                    div()
                        .size(px(metrics::ICON_DISCLOSURE))
                        .rounded(px(radius::SM))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(if show_welcome { t.tint() } else { t.hover() })
                        .when(!show_welcome, |d| d.border_1().border_color(t.line()))
                        .when(show_welcome, |d| {
                            d.child(
                                icon("check")
                                    .size(px(metrics::ICON_TINY))
                                    .text_color(t.accent_fg()),
                            )
                        }),
                )
                .child("Show at launch")
                .on_click(cx.listener(|_, _, _, cx| {
                    settings::change(cx, |p| p.show_welcome = !p.show_welcome)
                })),
        );
        let list_focused = self.list_focus.is_focused(window);
        let mut main = div()
            .id("welcome-projects")
            .track_focus(&self.list_focus)
            .tab_index(0)
            .role(Role::ListBox)
            .aria_label("Recent projects and samples. Use Up and Down to select, Enter to open.")
            .flex_1()
            .min_w(px(space::NONE))
            .h_full()
            .flex()
            .flex_col()
            .pt(px(layout::MAIN_TOP))
            .px(px(space::XL))
            .pb(px(space::XL));
        if !empty {
            main = main.child(
                layout::header("Recent", t).child(div().flex_1()).child(
                    row()
                        .gap(px(space::COMPACT))
                        .child(layout::caps("up", t))
                        .child(layout::caps("down", t))
                        .child(layout::caps("enter", t)),
                ),
            );
            // The list owns spare height; samples stay at the bottom for every
            // history length. Long histories scroll above the fixed strip.
            let mut list = div()
                .id("recent-projects")
                .flex_1()
                .min_h(px(space::NONE))
                .flex_shrink(1.)
                .overflow_y_scroll()
                .track_scroll(&self.scroll);
            for (index, project) in projects.iter().enumerate() {
                let path = project.path.clone();
                let selected = self.selected_item() == Selection::Recent(path.clone());
                list = list.child(
                    SidebarItem::custom(("recent", index), project.name.clone(), true, t)
                        .tab_stop(false)
                        .role(Role::ListBoxOption)
                        .aria_selected(selected)
                        .when(selected, |d| d.aria_active_descendant())
                        .when(selected && list_focused, |d| {
                            d.shadow(voxelite_ui::tokens::focus_ring(t))
                        })
                        .text_color(t.text())
                        .h(px(layout::ROW))
                        .min_h(px(layout::ROW))
                        .px(px(space::MD))
                        .py(px(space::SM))
                        .gap(px(space::LG))
                        .border_1()
                        .border_color(if selected {
                            t.selected()
                                .blend(t.tint().opacity(0.14))
                                .blend(t.accent().opacity(0.10))
                        } else {
                            transparent_black()
                        })
                        .bg(if selected {
                            t.selected()
                        } else {
                            t.background()
                        })
                        .hover(move |d| d.bg(if selected { t.selected() } else { t.hover() }))
                        .child(
                            div()
                                .w(px(layout::THUMB_WIDTH))
                                .h(px(layout::THUMB_HEIGHT))
                                .flex_shrink_0()
                                .child(layout::thumbnail(
                                    self.previews.get(&path),
                                    t,
                                    false,
                                    layout::THUMB_HEIGHT,
                                )),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(space::NONE))
                                .child(
                                    div()
                                        .text_size(px(type_size::BODY_LG))
                                        .line_height(px(type_size::LINE_SM))
                                        .font_weight(type_size::MEDIUM)
                                        .truncate()
                                        .child(project.name.clone()),
                                )
                                .child(
                                    div()
                                        .mt(px(space::COMPACT))
                                        .text_size(px(type_size::BODY))
                                        .line_height(px(type_size::LINE_SM))
                                        .text_color(t.muted())
                                        .truncate()
                                        .child(folder_label(&path, home.as_deref())),
                                ),
                        )
                        .child(
                            div()
                                .flex_shrink_0()
                                .text_size(px(type_size::BODY))
                                .text_color(if selected {
                                    t.accent().blend(t.text().opacity(0.30))
                                } else {
                                    t.muted()
                                })
                                .child(modified_label(
                                    project.modified,
                                    std::time::SystemTime::now(),
                                )),
                        )
                        .tooltip(hint(path.display().to_string(), t))
                        .when(!self.busy, |d| {
                            d.on_click(cx.listener(move |this, _, window, cx| {
                                this.selection = Some(Selection::Recent(path.clone()));
                                window.focus(&this.list_focus, cx);
                                this.run(Op::Open, Some(path.clone()), window, cx);
                            }))
                        }),
                );
            }
            main = main.child(list);
        }
        let mut samples = div()
            .id("samples")
            .flex_shrink_0()
            .when(!empty, |d| {
                d.mt(px(space::XL))
                    .pt(px(space::XL))
                    .border_t_1()
                    .border_color(t.line())
            })
            .child(layout::header("Samples", t).when(empty, |d| {
                d.child(div().flex_1()).child(
                    div()
                        .text_size(px(type_size::BODY))
                        .font_weight(type_size::NORMAL)
                        .child("No recent projects yet"),
                )
            }));
        let samples_focused = list_focused;
        if empty {
            for indices in [[0, 1], [2, 3]] {
                samples = samples.child(
                    div()
                        .flex()
                        .gap(px(space::LG))
                        .px(px(space::MD))
                        .when(indices[0] > 0, |d| d.mt(px(space::LG)))
                        .children(
                            indices.map(|i| self.sample_card(i, false, samples_focused, t, cx)),
                        ),
                );
            }
        } else {
            samples =
                samples.child(
                    div().flex().gap(px(space::MD)).px(px(space::MD)).children(
                        (0..4).map(|i| self.sample_card(i, true, samples_focused, t, cx)),
                    ),
                );
        }
        main = main.child(samples).when_some(error, |d, error| {
            d.child(div().mt(px(space::MD)).text_color(t.danger()).child(error))
        });
        div()
            .image_cache(self.images.clone())
            .id("welcome")
            .key_context("Editor")
            .track_focus(&self.focus)
            .tab_stop(false)
            .size_full()
            .flex()
            .overflow_hidden()
            .bg(t.background())
            .text_color(t.text())
            .font_family(type_size::UI)
            .text_size(px(type_size::BODY))
            .line_height(px(type_size::LINE_SM))
            .on_action(cx.listener(|this, op: &Op, window, cx| this.run(*op, None, window, cx)))
            .on_action(cx.listener(|this, op: &OpenRecentProject, window, cx| {
                this.run(Op::Open, Some(op.path.clone()), window, cx)
            }))
            .on_key_down(cx.listener(Self::key_down))
            .child(sidebar)
            .child(main)
    }
}

#[cfg(test)]
mod tests {
    use super::{Selection, grouped_count, queue_preview, reconcile_selection};
    use std::{collections::VecDeque, path::PathBuf};
    use voxelite_app::samples::Sample;

    #[test]
    fn selection_tracks_paths_and_survives_reorder_removal_and_empty_history() {
        let paths: Vec<PathBuf> = (0..12).map(|i| format!("{i}.voxl").into()).collect();
        let selection = Some(Selection::Recent(paths[11].clone()));
        let mut reordered = paths.clone();
        reordered.swap(0, 11);
        assert_eq!(
            reconcile_selection(&paths, &reordered, selection.clone()),
            selection
        );
        assert_eq!(
            reconcile_selection(&paths, &paths[..11], selection.clone()),
            Some(Selection::Recent(paths[10].clone()))
        );
        assert_eq!(
            reconcile_selection(&paths, &[], selection),
            Some(Selection::Sample(Sample::Autumn))
        );
        assert_eq!(reconcile_selection(&[], &paths, None), None);
        let sample = Some(Selection::Sample(Sample::Snow));
        assert_eq!(reconcile_selection(&paths, &[], sample.clone()), sample);
    }

    #[test]
    fn arrows_visit_every_recent_and_sample_in_display_order_and_clamp() {
        for count in [0, 1, 5, 12] {
            let paths: Vec<PathBuf> = (0..count).map(|i| format!("{i}.voxl").into()).collect();
            let expected: Vec<_> = paths
                .iter()
                .cloned()
                .map(Selection::Recent)
                .chain(Sample::ALL.into_iter().map(Selection::Sample))
                .collect();
            let mut selection = Selection::at(&paths, 0);
            assert_eq!(selection.step(&paths, false), selection);
            for item in &expected {
                assert_eq!(&selection, item);
                selection = selection.step(&paths, true);
            }
            assert_eq!(&selection, expected.last().unwrap());
            for item in expected.iter().rev() {
                assert_eq!(&selection, item);
                selection = selection.step(&paths, false);
            }
            assert_eq!(selection, expected[0]);
        }
    }

    fn window_fixture(
        cx: &mut gpui::TestAppContext,
        count: usize,
    ) -> (gpui::WindowHandle<super::WelcomeView>, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let paths: Vec<PathBuf> = (0..count)
            .map(|i| dir.path().join(format!("{i}.voxl")))
            .collect();
        cx.update(|cx| {
            crate::actions::install(cx);
            super::settings::init_at(
                None,
                Ok(Default::default()),
                dir.path().join("settings.json"),
                cx,
            );
            cx.set_global(super::Launcher {
                editor: None,
                welcome: None,
                socket: None,
                projects: paths
                    .iter()
                    .map(|path| voxelite_app::recents::RecentProject {
                        path: path.clone(),
                        name: "Project".into(),
                        modified: std::time::SystemTime::UNIX_EPOCH,
                    })
                    .collect(),
                error: None,
                work: None,
                notification: None,
                continuation: None,
                revision: 0,
                preview_work: None,
                preview_queue: VecDeque::new(),
            });
        });
        let window = cx.add_window(|window, cx| {
            let mut view = super::WelcomeView::initial(cx);
            view.project_paths = paths;
            window.focus(&view.focus, cx);
            view
        });
        cx.run_until_parked();
        (window, dir)
    }

    fn keys(
        cx: &mut gpui::TestAppContext,
        handle: gpui::WindowHandle<super::WelcomeView>,
        input: &str,
    ) {
        for key in input.split_whitespace() {
            use gpui::{AppContext, InputEvent};
            // simulate_keystrokes emits key-down only; buttons activate on key-up.
            cx.update_window(handle.into(), |_, window, cx| window.draw(cx).clear(cx))
                .unwrap();
            cx.simulate_keystrokes(handle.into(), key);
            let _ = cx.update_window(handle.into(), |_, window, cx| {
                window.dispatch_event(
                    gpui::KeyUpEvent {
                        keystroke: gpui::Keystroke::parse(key).unwrap(),
                    }
                    .to_platform_input(),
                    cx,
                );
            });
            cx.run_until_parked();
        }
    }

    #[gpui::test]
    fn shutdown_drains_ordered_history_writes_without_ui_callbacks(cx: &mut gpui::TestAppContext) {
        let (_handle, dir) = window_fixture(cx, 0);
        let path = dir.path().join("saved.voxl");
        std::fs::write(&path, b"project").unwrap();
        cx.update(|cx| {
            super::observe_quit(cx);
            super::history_at(
                super::HistoryChange::Record(path.clone(), "First".into()),
                dir.path().into(),
                cx,
            );
            super::history_at(super::HistoryChange::Clear, dir.path().into(), cx);
            super::history_at(
                super::HistoryChange::Record(path.clone(), "Last".into()),
                dir.path().into(),
                cx,
            );
        });
        cx.quit();
        let projects = voxelite_app::recents::Recents::new(dir.path())
            .load()
            .unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Last");
        assert_eq!(projects[0].path, path.canonicalize().unwrap());
    }

    #[gpui::test]
    fn tab_cycles_sidebar_toggle_and_list_in_both_directions(cx: &mut gpui::TestAppContext) {
        let (handle, dir) = window_fixture(cx, 0);
        for index in [0, 1, 2, 3, 4, 0] {
            keys(cx, handle, "tab");
            handle
                .update(cx, |view, window, _| {
                    let focus = if index < 4 {
                        &view.sidebar_focus[index]
                    } else {
                        &view.list_focus
                    };
                    assert!(focus.is_focused(window), "Tab stop {index}");
                })
                .unwrap();
        }
        for index in [4, 3, 2, 1, 0, 4] {
            keys(cx, handle, "shift-tab");
            handle
                .update(cx, |view, window, _| {
                    let focus = if index < 4 {
                        &view.sidebar_focus[index]
                    } else {
                        &view.list_focus
                    };
                    assert!(focus.is_focused(window), "reverse Tab stop {index}");
                })
                .unwrap();
        }
        keys(cx, handle, "shift-tab space");
        assert!(
            !voxelite_app::preferences::Preferences::load(&dir.path().join("settings.json"))
                .unwrap()
                .show_welcome
        );
        keys(cx, handle, "enter");
        assert!(
            voxelite_app::preferences::Preferences::load(&dir.path().join("settings.json"))
                .unwrap()
                .show_welcome
        );
    }

    #[gpui::test]
    fn real_keys_cross_recents_and_samples_without_stealing_sidebar_activation(
        cx: &mut gpui::TestAppContext,
    ) {
        let (handle, _dir) = window_fixture(cx, 1);
        keys(cx, handle, "down down down down down");
        handle
            .update(cx, |view, window, _| {
                assert_eq!(view.selected_item(), Selection::Sample(Sample::Snow));
                assert!(view.list_focus.is_focused(window));
            })
            .unwrap();
        keys(cx, handle, "up up up up up");
        handle
            .update(cx, |view, _, _| {
                assert!(matches!(view.selected_item(), Selection::Recent(_)))
            })
            .unwrap();
        keys(cx, handle, "ctrl-down alt-up");
        handle
            .update(cx, |view, _, _| {
                assert!(matches!(view.selected_item(), Selection::Recent(_)))
            })
            .unwrap();
        // Tab wraps from the list to New; arrows there must leave list selection alone.
        keys(cx, handle, "tab down tab tab enter");
        handle
            .update(cx, |view, window, _| {
                assert_eq!(view.selected_item(), Selection::Sample(Sample::Autumn));
                assert!(view.list_focus.is_focused(window));
            })
            .unwrap();
        keys(cx, handle, "down");
        keys(
            cx,
            handle,
            &crate::shortcuts::ShortcutPlatform::current().binding_key("secondary-shift-o"),
        );
        handle
            .update(cx, |view, window, _| {
                assert_eq!(view.selected_item(), Selection::Sample(Sample::Autumn));
                assert!(view.list_focus.is_focused(window));
            })
            .unwrap();
        keys(cx, handle, "escape");
        cx.read(|cx| assert!(handle.read(cx).is_err()));
    }

    #[gpui::test]
    fn enter_on_a_removed_file_preserves_welcome_and_reports_the_error(
        cx: &mut gpui::TestAppContext,
    ) {
        let (handle, _dir) = window_fixture(cx, 1);
        keys(cx, handle, "enter");
        handle
            .update(cx, |view, _, _| {
                assert!(!view.busy);
                assert!(view.error.is_some());
            })
            .unwrap();
    }

    #[test]
    fn preview_backlog_coalesces_saves_and_bounds_distinct_paths() {
        let mut queue = VecDeque::new();
        for i in 0..100 {
            queue_preview(&mut queue, format!("{i}.voxl").into());
        }
        assert_eq!(queue.len(), voxelite_app::recents::CAPACITY);
        for _ in 0..100 {
            queue_preview(&mut queue, "99.voxl".into());
        }
        assert_eq!(queue.len(), voxelite_app::recents::CAPACITY);
        assert_eq!(queue.back(), Some(&PathBuf::from("99.voxl")));
        assert_eq!(queue.front(), Some(&PathBuf::from("88.voxl")));
    }

    #[test]
    fn sample_counts_use_thousands_separators() {
        for (count, label) in [
            (0, "0"),
            (999, "999"),
            (1000, "1,000"),
            (27817, "27,817"),
            (1000000, "1,000,000"),
        ] {
            assert_eq!(grouped_count(count), label);
        }
    }
}
