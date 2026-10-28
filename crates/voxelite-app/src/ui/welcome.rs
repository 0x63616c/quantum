//! Native project launcher and recent-file integration. The editor remains the
//! sole document owner; requests made here reuse its unsaved-change flow.
use super::{EditorView, hint, settings};
use crate::{
    actions::{self, Op, OpenRecentProject},
    shortcuts::{ShortcutPlatform, navigation_key},
    theme::Theme,
};
use gpui::{prelude::*, *};
use std::{collections::HashMap, path::PathBuf};
use voxelite_app::{
    editor::Editor,
    recents::{RecentProject, Recents, display_path, modified_label},
    thumbnails::ThumbnailCache,
};
use voxelite_ui::{
    AsciiReveal, Button, ButtonVariant, IconButton, SidebarItem, row,
    tokens::{metrics, space, type_size},
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
    revision: u64,
}
impl Global for Launcher {}

enum HistoryChange {
    Reload,
    Record(PathBuf, String),
    Clear,
}

pub(crate) fn init(theme: Option<Theme>, socket: Option<PathBuf>, cx: &mut App) {
    settings::init(theme, cx);
    cx.set_global(Launcher {
        editor: None,
        welcome: None,
        socket,
        projects: Vec::new(),
        error: None,
        work: None,
        revision: 0,
    });
    // Let GPUI's graceful shutdown drain metadata writes even when the user
    // closes the last window immediately after opening, saving, or clearing.
    // Thumbnails remain disposable and can be generated at the next launch.
    cx.on_app_quit(|cx| {
        let work = cx.global_mut::<Launcher>().work.take();
        async move {
            if let Some(work) = work {
                work.await;
            }
        }
    })
    .detach();
    history(HistoryChange::Reload, cx);
}

fn history(change: HistoryChange, cx: &mut App) {
    let previous = cx.global_mut::<Launcher>().work.take();
    let directory = voxelite_app::local_settings::data_dir();
    let executor = cx.background_executor().clone();
    let task = cx.spawn(async move |cx| {
        if let Some(previous) = previous {
            previous.await;
        }
        let result = executor
            .spawn(async move {
                let store = Recents::new(&directory);
                match change {
                    HistoryChange::Reload => store.load(),
                    HistoryChange::Record(path, name) => store.record(&path, &name),
                    HistoryChange::Clear => store.clear().and_then(|()| store.load()),
                }
            })
            .await;
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
    cx.global_mut::<Launcher>().work = Some(task);
}

pub(super) fn record(editor: &Editor, cx: &mut App) {
    let Some(path) = editor.path.clone() else {
        return;
    };
    history(
        HistoryChange::Record(path.clone(), editor.snapshot().name.clone()),
        cx,
    );
    let cache = ThumbnailCache::new(&voxelite_app::local_settings::data_dir());
    cx.background_executor()
        .spawn(async move {
            if let Err(error) = cache.ensure(&path) {
                eprintln!("Could not create project preview: {error}");
            }
        })
        .detach();
}

pub(super) fn observe_editor(cx: &mut Context<EditorView>) {
    cx.observe_global::<Launcher>(|_, cx| cx.notify()).detach();
}

// A Save & Quit continuation must not exit before the recent entry is durable.
pub(super) fn after_record(op: Op, cx: &mut Context<EditorView>) {
    let previous = cx.global_mut::<Launcher>().work.take();
    let task = cx.spawn(async move |this, cx| {
        if let Some(previous) = previous {
            previous.await;
        }
        let _ = this.update(cx, |this, cx| {
            this.save_flow.finish_dialog();
            this.run(op, cx);
        });
    });
    cx.global_mut::<Launcher>().work = Some(task);
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
    let bounds = Bounds::centered(None, size(px(1440.), px(900.)), cx);
    let handle = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: if cfg!(target_os = "macos") {
                Some(TitlebarOptions {
                    title: Some("Quantum".into()),
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(20.), px(24.))),
                })
            } else {
                None
            },
            app_owns_titlebar_drag: cfg!(target_os = "macos"),
            window_min_size: Some(size(px(1024.), px(640.))),
            app_id: Some(voxelite_app::APP_ID.into()),
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

pub(crate) fn show(cx: &mut App) -> anyhow::Result<WindowHandle<WelcomeView>> {
    if let Some(handle) = cx.global::<Launcher>().welcome
        && handle
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
    {
        return Ok(handle);
    }
    let bounds = Bounds::centered(None, size(px(760.), px(620.)), cx);
    let handle = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(px(640.), px(500.))),
            titlebar: Some(TitlebarOptions {
                title: Some("Welcome to Quantum".into()),
                appears_transparent: true,
                traffic_light_position: Some(point(px(18.), px(23.))),
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
    scroll: ScrollHandle,
    selected: usize,
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
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
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
                this.reduced_motion = voxelite_app::reduce_motion::prefers_reduced_motion();
                history(HistoryChange::Reload, cx);
            }
            cx.notify();
        })
        .detach();
        let mut view = Self {
            focus,
            list_focus: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            selected: 0,
            previews: HashMap::new(),
            preview_projects: Vec::new(),
            reduced_motion: voxelite_app::reduce_motion::prefers_reduced_motion(),
            retired_previews: Vec::new(),
            revision: u64::MAX,
            thumbnails: None,
            busy: false,
            error: None,
        };
        view.refresh_previews(cx);
        view
    }

    fn refresh_previews(&mut self, cx: &mut Context<Self>) {
        let state = cx.global::<Launcher>();
        if self.revision == state.revision {
            return;
        }
        let revision = state.revision;
        let projects = state.projects.clone();
        self.selected = self.selected.min(projects.len().saturating_sub(1));
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
        let cache = ThumbnailCache::new(&voxelite_app::local_settings::data_dir());
        self.thumbnails = Some(cx.spawn(async move |this, cx| {
            // One preview at a time bounds memory and progressively fills placeholders.
            for project in projects {
                let path = project.path.clone();
                let cache = cache.clone();
                let result = cx
                    .background_executor()
                    .spawn(async move { super::loading::recent(&cache.ensure(&path)?) })
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
        if matches!(op, Op::New | Op::Open | Op::Quit) && cx.global::<Launcher>().editor.is_some() {
            // A stale handle is cleared before retrying in this window.
            let handle = cx.global::<Launcher>().editor.unwrap();
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
            Op::OpenRecent | Op::Welcome => window.focus(&self.list_focus, cx),
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

impl Render for WelcomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = settings::theme(
            cx.global::<settings::SettingsState>().values.appearance,
            window,
        );
        for image in self.retired_previews.drain(..) {
            let _ = window.drop_image(image);
        }
        let projects = cx.global::<Launcher>().projects.clone();
        let error = self
            .error
            .clone()
            .or_else(|| cx.global::<Launcher>().error.clone());
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
        let mut list = div()
            .id("recent-projects")
            .track_focus(&self.list_focus)
            .tab_index(0)
            .flex_1()
            .min_h(px(0.))
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let len = cx.global::<Launcher>().projects.len();
                match navigation_key(&event.keystroke) {
                    Some("up") if len > 0 => {
                        this.selected = this.selected.saturating_sub(1);
                    }
                    Some("down") if len > 0 => {
                        this.selected = (this.selected + 1).min(len - 1);
                    }
                    Some("enter") if len > 0 => {
                        let path = cx.global::<Launcher>().projects[this.selected].path.clone();
                        this.run(Op::Open, Some(path), window, cx);
                    }
                    _ => return,
                }
                this.scroll.scroll_to_item(this.selected);
                cx.stop_propagation();
                cx.notify();
            }));
        if projects.is_empty() {
            list = list.child(
                div()
                    .py(px(space::XXL))
                    .text_color(t.muted())
                    .child("Your recent projects will appear here."),
            );
        }
        for (index, project) in projects.iter().enumerate() {
            let path = project.path.clone();
            let preview = div()
                .w(px(metrics::RECENT_PREVIEW_COLUMN))
                .h(px(metrics::RECENT_PREVIEW))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .bg(t.surface())
                .child(super::loading::recent_element(
                    self.previews.get(&path),
                    index,
                    project.name.clone(),
                    t,
                    self.reduced_motion,
                ));
            list = list.child(
                SidebarItem::custom(
                    ("recent", index),
                    project.name.clone(),
                    self.selected == index,
                    t,
                )
                .tab_stop(false)
                .py(px(space::SM))
                .child(preview)
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .child(div().text_ellipsis().child(project.name.clone()))
                        .child(
                            div()
                                .text_size(px(type_size::CAPTION))
                                .text_color(t.muted())
                                .text_ellipsis()
                                .child(display_path(&path, home.as_deref())),
                        ),
                )
                .child(
                    div()
                        .text_size(px(type_size::CAPTION))
                        .text_color(t.muted())
                        .child(modified_label(
                            project.modified,
                            std::time::SystemTime::now(),
                        )),
                )
                .tooltip(hint(path.display().to_string(), t))
                .on_click(cx.listener(
                    move |this, event: &ClickEvent, window, cx| {
                        this.selected = index;
                        window.focus(&this.list_focus, cx);
                        if event.click_count() == 2 {
                            this.run(Op::Open, Some(path.clone()), window, cx);
                        }
                        cx.notify();
                    },
                )),
            );
        }
        div()
            .id("welcome")
            .key_context("Editor")
            .track_focus(&self.focus)
            .tab_stop(false)
            .size_full()
            .flex()
            .flex_col()
            .bg(t.background())
            .text_color(t.text())
            .font(gpui::font(if cfg!(target_os = "macos") {
                ".SystemUIFont"
            } else {
                "Geist"
            }))
            .text_size(px(type_size::BODY))
            .on_action(cx.listener(|this, op: &Op, window, cx| this.run(*op, None, window, cx)))
            .on_action(cx.listener(|this, op: &OpenRecentProject, window, cx| {
                this.run(Op::Open, Some(op.path.clone()), window, cx)
            }))
            .on_key_down(cx.listener(|_, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "tab" {
                    if event.keystroke.modifiers.shift {
                        window.focus_prev(cx);
                    } else {
                        window.focus_next(cx);
                    }
                    cx.stop_propagation();
                } else if event.keystroke.key == "w"
                    && ShortcutPlatform::current().is_primary(event.keystroke.modifiers)
                {
                    window.remove_window();
                    cx.stop_propagation();
                }
            }))
            .child(
                div()
                    .h(px(52.))
                    .flex_shrink_0()
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move()),
            )
            .child(
                row()
                    .px(px(space::XXL))
                    .gap(px(space::XL))
                    .child(img("app-icon-256.png").size(px(80.)))
                    .child(
                        div()
                            .child(
                                div()
                                    .text_size(px(type_size::DISPLAY))
                                    .font_weight(type_size::BOLD)
                                    .child("Quantum"),
                            )
                            .child(
                                div()
                                    .text_color(t.muted())
                                    .child(format!("Version {}", env!("CARGO_PKG_VERSION"))),
                            ),
                    ),
            )
            .child(
                row()
                    .px(px(space::XXL))
                    .py(px(space::XL))
                    .gap(px(space::MD))
                    .children(
                        [
                            (Op::New, "New Project", ButtonVariant::Primary),
                            (Op::Open, "Open…", ButtonVariant::Secondary),
                        ]
                        .map(|(op, label, variant)| {
                            Button::new(op.command_id(), label, t)
                                .icon(op.icon())
                                .variant(variant)
                                .disabled(self.busy)
                                .build()
                                .child(div().text_size(px(type_size::CAPTION)).child(op.shortcut()))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.run(op, None, window, cx)
                                }))
                        }),
                    ),
            )
            .child(
                row()
                    .px(px(space::XXL))
                    .justify_between()
                    .child(div().font_weight(type_size::SEMIBOLD).child("Recent"))
                    .child(
                        IconButton::button("clear-recents", "trash-2", Op::ClearRecents.label(), t)
                            .disabled(projects.is_empty() || self.busy)
                            .build()
                            .tooltip(hint(Op::ClearRecents.label(), t))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.run(Op::ClearRecents, None, window, cx)
                            })),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.))
                    .px(px(space::XL))
                    .pb(px(space::LG))
                    .flex()
                    .flex_col()
                    .child(list),
            )
            .when(self.busy, |d| {
                d.child(
                    div()
                        .px(px(space::XXL))
                        .pb(px(space::LG))
                        .text_color(t.muted())
                        .child(
                            AsciiReveal::new("project-opening", "Opening project…", t)
                                .loading(true)
                                .reduced_motion(self.reduced_motion),
                        ),
                )
            })
            .when_some(error, |d, error| {
                d.child(
                    div()
                        .px(px(space::XXL))
                        .pb(px(space::LG))
                        .text_color(t.danger())
                        .child(error),
                )
            })
    }
}
