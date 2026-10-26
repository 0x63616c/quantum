//! Native GPUI shell. Portable editing logic lives in the library.
#[cfg(not(target_family = "wasm"))]
mod agent;
#[cfg(target_family = "wasm")]
#[path = "ui/agent_web.rs"]
mod agent;
#[cfg(target_family = "wasm")]
mod browser;
mod comments;
mod history;
use comments::Panel;
mod palettes;
mod settings;
mod share;
mod shortcut_hint;
use crate::{
    actions::{MENUS, Op},
    shortcuts::{ShortcutPlatform, navigation_key},
    theme::Theme,
};
use glam::DVec3;
use gpui::{prelude::*, *};
use shortcut_hint::{HintText, action_shortcut, shortcut};
use std::{cell::Cell, path::PathBuf, rc::Rc, sync::Arc};
use voxelite_app::editor::{Editor, EditorResult, OrbitCamera, Tool};
use voxelite_core::{Palette, Pos, Preset};
use voxelite_render::{PresetView, RenderOptions, View};

// Keep render sizing and native layout on the same logical-pixel metrics.
const SIDEBAR_WIDTH: f32 = 352.0;
const TOOL_RAIL_WIDTH: f32 = 56.0;
const MENUBAR_HEIGHT: f32 = if cfg!(target_os = "macos") { 0.0 } else { 32.0 };
const TOOLBAR_HEIGHT: f32 = 64.0;
const STATUSBAR_HEIGHT: f32 = 32.0;
const PANEL_GUTTER: f32 = 24.0;

/// Image identity includes the editor incarnation, since reopening the same ID
/// resets its revision. Camera and dimensions bind picking to displayed pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
struct FrameIdentity {
    generation: u64,
    revision: u64,
    camera: OrbitCamera,
    size: [u32; 2],
}

pub struct EditorView {
    palettes: palettes::PaletteManager,
    sharing: share::Sharing,
    discussion: comments::Discussion,
    panel: Panel,
    timeline: history::Timeline,
    agent: agent::Agent,
    editor: Editor,
    theme: Theme,
    preferences: voxelite_app::preferences::Preferences,
    #[cfg(not(target_family = "wasm"))]
    settings_window: Option<WindowHandle<settings::SettingsView>>,
    #[cfg(target_family = "wasm")]
    settings_modal: Option<Entity<settings::SettingsView>>,
    palette_focus: FocusHandle,
    sidebar_scroll: ScrollHandle,
    focus: FocusHandle,
    image: Option<Arc<RenderImage>>,
    painted_image: Option<Arc<RenderImage>>,
    last_title: String,
    rendering: bool,
    render_pending: bool,
    render_size: [u32; 2],
    display_camera: OrbitCamera,
    document_generation: u64,
    displayed: Option<FrameIdentity>,
    display_size: [u32; 2],
    camera_view: Option<Op>,
    camera_fitted: bool,
    navigation_epoch: u64,
    interactive_render: bool,
    viewport: Rc<Cell<Bounds<Pixels>>>,
    hover: Option<Pos>,
    cursor: Pos,
    drag: Option<Point<Pixels>>,
    grid: bool,
    menu: Option<usize>,
    menu_index: usize,
    palette_open: bool,
    palette_index: usize,
    command_open: bool,
    tools_only: bool,
    palette_cursor: usize,
    palette_scroll: ScrollHandle,
    swatch_scroll: ScrollHandle,
    pending_choice: usize,
    after_save: Option<Op>,
    query: String,
    command_index: usize,
    command_scroll: ScrollHandle,
    dialog_open: bool,
    info: Option<(&'static str, HintText)>,
    pending: Option<Op>,
    status: HintText,
    error: bool,
}
fn icon(name: &'static str) -> Svg {
    svg()
        .path(format!("icons/{name}.svg"))
        .size(px(17.0))
        .flex_shrink_0()
}
struct Hint {
    text: HintText,
    theme: Theme,
}
impl Render for Hint {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(10.0))
            .py(px(7.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(self.theme.line())
            .bg(self.theme.background())
            .text_color(self.theme.text())
            .font(gpui::font("Geist"))
            .text_size(px(12.0))
            .shadow_lg()
            .child(self.text.clone())
    }
}
fn row() -> Div {
    div().flex().items_center()
}
fn mono(text: impl Into<SharedString>) -> Div {
    div()
        .font(gpui::font("Geist Mono"))
        .text_size(px(12.0))
        .child(text.into())
}
fn hint(text: impl Into<HintText>, theme: Theme) -> impl Fn(&mut Window, &mut App) -> AnyView {
    let text = text.into();
    move |_, cx| {
        cx.new(|_| Hint {
            text: text.clone(),
            theme,
        })
        .into()
    }
}

fn keyboard_guide() -> HintText {
    HintText::default()
        .action(Op::Place)
        .text(" Place · ")
        .action(Op::Erase)
        .text(" Erase · ")
        .action(Op::Paint)
        .text(" Paint · ")
        .action(Op::Fill)
        .text(" Fill · ")
        .action(Op::Box)
        .text(" Box · ")
        .action(Op::Picker)
        .text(" Picker\nClick to apply · ")
        .key("alt-click")
        .text(" to sample a color\nRight-drag to orbit · Middle-drag or ")
        .key("shift-rightdrag")
        .text(" to pan\nScroll to zoom · ")
        .action(Op::Frame)
        .text(" to frame the model\nBox: click two corners · ")
        .key("escape")
        .text(" cancels\n")
        .key("left")
        .text("/")
        .key("right")
        .text("/")
        .key("up")
        .text("/")
        .key("down")
        .text(" move X/Z · ")
        .key("pageup")
        .text("/")
        .key("pagedown")
        .text(" move Y · ")
        .key("enter")
        .text(" applies\n")
        .key("[")
        .text(" / ")
        .key("]")
        .text(" cycle colors · ")
        .action(Op::Commands)
        .text(" searches commands\n")
        .action(Op::Undo)
        .text(" undoes · ")
        .action(Op::Redo)
        .text(" redoes\n")
        .action(Op::Open)
        .text(" opens · ")
        .action(Op::Save)
        .text(" saves · ")
        .action(Op::SaveAs)
        .text(" saves as")
}

fn section_title(text: &'static str) -> Div {
    div()
        .text_size(px(14.0))
        .font_weight(FontWeight::SEMIBOLD)
        .child(text)
}

fn color_fill(color: [u8; 4]) -> Hsla {
    rgb(u32::from(color[0]) << 16 | u32::from(color[1]) << 8 | u32::from(color[2])).into()
}

/// Choose a legible checkmark on the actual sRGB swatch, in either theme.
fn swatch_ink(color: [u8; 4]) -> Hsla {
    let linear = |v: u8| {
        let v = f32::from(v) / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let luminance =
        0.2126 * linear(color[0]) + 0.7152 * linear(color[1]) + 0.0722 * linear(color[2]);
    rgb(if luminance > 0.179 {
        0x000000
    } else {
        0xffffff
    })
    .into()
}
const PALETTES: [(&str, Option<Preset>); 8] = [
    ("Project colors", None),
    ("PICO-8", Some(Preset::Pico8)),
    ("ENDESGA 32", Some(Preset::Endesga32)),
    ("Sweetie 16", Some(Preset::Sweetie16)),
    ("Grayscale", Some(Preset::Grayscale)),
    ("Neutral", Some(Preset::Neutral)),
    ("Warm", Some(Preset::Warm)),
    ("Vivid", Some(Preset::Vivid)),
];

impl EditorView {
    pub fn new(
        editor: Editor,
        override_theme: Option<Theme>,
        socket: Option<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        settings::init(override_theme, cx);
        let preferences = cx.global::<settings::SettingsState>().values.clone();
        let theme = settings::theme(preferences.appearance, window);
        cx.observe_global_in::<settings::SettingsState>(window, |this, window, cx| {
            this.sync_preferences(window, cx)
        })
        .detach();
        cx.observe_window_appearance(window, |this, window, cx| this.sync_preferences(window, cx))
            .detach();
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let weak = cx.weak_entity();
        window.on_window_should_close(cx, move |_, cx| {
            weak.update(cx, |this, cx| {
                if this.dialog_open {
                    return false;
                }
                if this.editor.engine.is_dirty() {
                    this.pending = Some(Op::Quit);
                    this.pending_choice = 2;
                    cx.notify();
                    false
                } else {
                    true
                }
            })
            .unwrap_or(true)
        });
        cx.observe_window_activation(window, |this, window, cx| {
            if !window.is_window_active() {
                this.drag = None;
                this.hover = None;
            }
            cx.notify();
        })
        .detach();
        let mut palettes = palettes::PaletteManager::new(cx);
        let library = voxelite_app::palettes::PaletteLibrary::load(
            &voxelite_app::palettes::data_dir().join("palettes.json"),
        );
        let library_error = library.as_ref().err().map(ToString::to_string);
        if let Ok(library) = library {
            palettes.library = library;
        }
        let agent = agent::Agent::new(&editor, socket);
        #[cfg(not(target_family = "wasm"))]
        let agent_error = agent.as_ref().err().map(ToString::to_string);
        #[cfg(target_family = "wasm")]
        let agent_error: Option<String> = None;
        let agent = agent.unwrap_or_default();
        #[cfg(not(target_family = "wasm"))]
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(80))
                    .await;
                if this.update(cx, |this, cx| this.poll_agent(cx)).is_err() {
                    break;
                }
            }
        })
        .detach();
        Self {
            agent,
            palettes,
            sharing: share::Sharing::new(cx),
            discussion: comments::Discussion::new(cx),
            panel: Panel::Session,
            timeline: history::Timeline::new(cx),
            cursor: editor.snapshot().bounds.min,
            display_camera: editor.camera,
            document_generation: 0,
            displayed: None,
            display_size: [0, 0],
            camera_view: Some(Op::Frame),
            camera_fitted: true,
            navigation_epoch: 0,
            interactive_render: false,
            editor,
            grid: preferences.grid,
            preferences,
            #[cfg(not(target_family = "wasm"))]
            settings_window: None,
            #[cfg(target_family = "wasm")]
            settings_modal: None,
            palette_focus: cx.focus_handle(),
            sidebar_scroll: ScrollHandle::new(),
            theme,
            focus,
            image: None,
            painted_image: None,
            last_title: String::new(),
            rendering: false,
            render_pending: false,
            render_size: [0, 0],
            viewport: Rc::new(Cell::new(Bounds::default())),
            hover: None,
            drag: None,
            menu: None,
            menu_index: 0,
            palette_open: false,
            palette_index: 0,
            command_open: false,
            tools_only: false,
            palette_cursor: 0,
            palette_scroll: ScrollHandle::new(),
            swatch_scroll: ScrollHandle::new(),
            pending_choice: 2,
            after_save: None,
            query: String::new(),
            command_index: 0,
            command_scroll: ScrollHandle::new(),
            dialog_open: false,
            info: None,
            pending: None,
            status: library_error
                .clone()
                .or(agent_error.clone())
                .unwrap_or_else(|| "Ready".into())
                .into(),
            error: library_error.is_some() || agent_error.is_some(),
        }
    }
    fn report<T>(
        &mut self,
        result: EditorResult<T>,
        message: impl Into<HintText>,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(_) => {
                self.status = message.into();
                self.error = false;
            }
            Err(e) => {
                self.status = e.to_string().into();
                self.error = true;
            }
        }
        // Sampling, empty clicks and rejected edits do not invalidate the image.
        // Avoid another CPU render (and its input wait) for unchanged content.
        if self.displayed != Some(self.frame_identity()) {
            self.request_render(cx);
        }
        cx.notify();
    }
    fn request_render(&mut self, cx: &mut Context<Self>) {
        if self.render_size[0] == 0 {
            return;
        }
        if self.rendering {
            self.render_pending = true;
            return;
        }
        self.rendering = true;
        self.render_pending = false;
        let snapshot = self.editor.display_snapshot().clone();
        let camera = self.editor.camera;
        let identity = self.frame_identity();
        let interactive = self.interactive_render;
        let options = RenderOptions {
            width: if interactive {
                self.render_size[0].div_ceil(2)
            } else {
                self.render_size[0]
            },
            height: if interactive {
                self.render_size[1].div_ceil(2)
            } else {
                self.render_size[1]
            },
            supersampling: if interactive || !self.preferences.antialiasing {
                1
            } else {
                2
            },
            view: View::Camera(self.editor.camera.camera()),
            background: [0; 4],
            ..Default::default()
        };
        let task = cx
            .background_executor()
            .spawn(async move { voxelite_render::render(&snapshot, &options) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.rendering = false;
                // Never install a render from a replaced document or an older edit.
                if this.document_generation == identity.generation
                    && this.editor.engine.revision() == identity.revision
                    && this.render_size == identity.size
                {
                    match result {
                        Ok(mut frame) => {
                            for pixel in frame.rgba.as_chunks_mut::<4>().0 {
                                pixel.swap(0, 2);
                            }
                            this.display_size = [frame.width, frame.height];
                            let buffer =
                                image::RgbaImage::from_raw(frame.width, frame.height, frame.rgba)
                                    .expect("renderer dimensions");
                            this.display_camera = camera;
                            this.displayed = Some(identity);
                            this.image =
                                Some(Arc::new(RenderImage::new(vec![image::Frame::new(buffer)])));
                        }
                        Err(e) => {
                            this.error = true;
                            this.status = e.to_string().into();
                        }
                    }
                }
                if this.render_pending {
                    this.request_render(cx);
                }
                cx.notify();
            });
        })
        .detach();
    }
    fn request_navigation(&mut self, cx: &mut Context<Self>) {
        self.navigation_epoch = self.navigation_epoch.wrapping_add(1);
        let epoch = self.navigation_epoch;
        self.interactive_render = true;
        self.request_render(cx);
        let timer = cx
            .background_executor()
            .timer(std::time::Duration::from_millis(180));
        cx.spawn(async move |this, cx| {
            timer.await;
            let _ = this.update(cx, |this, cx| {
                if this.navigation_epoch == epoch {
                    this.interactive_render = false;
                    this.request_render(cx);
                }
            });
        })
        .detach();
    }
    fn frame_identity(&self) -> FrameIdentity {
        FrameIdentity {
            generation: self.document_generation,
            revision: self.editor.engine.revision(),
            camera: self.editor.camera,
            size: self.render_size,
        }
    }
    fn replace_editor(&mut self, editor: Editor, cx: &mut Context<Self>) {
        #[cfg(not(target_family = "wasm"))]
        if let Some(session) = &mut self.agent.session
            && let Err(e) = session.replace(editor.project())
        {
            self.status = e.to_string().into();
            self.error = true;
        }
        self.agent.events.clear();
        self.agent.follow = false;
        self.agent.capture_generation = self.agent.capture_generation.wrapping_add(1);
        self.agent.capturing = false;
        if let Some(old) = self.agent.capture.take() {
            self.agent
                .retired
                .extend(old.views.into_iter().map(|v| v.2));
        }
        self.discussion.selected = None;
        self.discussion.numbers.clear();
        self.discussion.draft = None;
        self.discussion.placing = false;
        self.discussion.first = None;
        self.sharing.open = false;
        self.editor = editor;
        self.document_generation = self.document_generation.wrapping_add(1);
        self.image = None;
        self.displayed = None;
        self.hover = None;
        self.drag = None;
        self.cursor = self.editor.display_snapshot().bounds.min;
        self.palette_index = 0;
        self.camera_view = Some(Op::Frame);
        self.camera_fitted = true;
        let aspect = f64::from(self.render_size[0]) / f64::from(self.render_size[1].max(1));
        self.editor.camera = OrbitCamera::fit(self.editor.display_snapshot(), aspect.max(0.01));
        self.display_camera = self.editor.camera;
        self.error = false;
        self.request_render(cx);
    }
    fn active_palette(&self) -> Palette {
        if let Some(p) = self
            .palettes
            .custom
            .and_then(|id| self.palettes.library.entries.iter().find(|e| e.id == id))
        {
            return p.palette.clone();
        }
        PALETTES[self.palette_index]
            .1
            .map(|p| p.palette())
            .unwrap_or_else(|| self.editor.display_snapshot().palette.clone())
    }
    fn filtered_actions(&self) -> Vec<Op> {
        MENUS
            .iter()
            .flat_map(|(_, items)| items.iter().copied())
            .filter(|op| {
                *op != Op::Commands
                    && (!self.tools_only
                        || matches!(
                            op,
                            Op::Place | Op::Erase | Op::Paint | Op::Fill | Op::Box | Op::Picker
                        ))
                    && op
                        .label()
                        .to_lowercase()
                        .contains(&self.query.to_lowercase())
            })
            .collect()
    }
    fn enabled(&self, op: Op) -> bool {
        if self.dialog_open {
            return false;
        }
        if self.editor.preview.is_some()
            && matches!(
                op,
                Op::Place
                    | Op::Erase
                    | Op::Paint
                    | Op::Fill
                    | Op::Box
                    | Op::Picker
                    | Op::MirrorX
                    | Op::MirrorY
                    | Op::MirrorZ
                    | Op::AddLayer
                    | Op::Comment
                    | Op::ToolPicker
            )
        {
            return false;
        }
        match op {
            Op::Undo => self.editor.preview.is_none() && self.editor.engine.head().is_some(),
            Op::Redo => self.editor.preview.is_none() && self.editor.engine.redo_head().is_some(),
            _ => true,
        }
    }
    fn disabled_reason(&self, op: Op) -> &'static str {
        if self.editor.preview.is_some() {
            "Return to live to edit"
        } else {
            op.disabled_reason()
        }
    }
    fn selected(&self, op: Op) -> bool {
        match op {
            Op::Place => self.editor.tool == Tool::Place,
            Op::Erase => self.editor.tool == Tool::Erase,
            Op::Paint => self.editor.tool == Tool::Paint,
            Op::Fill => self.editor.tool == Tool::Fill,
            Op::Box => self.editor.tool == Tool::Box,
            Op::Picker => self.editor.tool == Tool::Picker,
            Op::Grid => self.grid,
            Op::Frame | Op::Front | Op::Right | Op::Top => self.camera_view == Some(op),
            Op::MirrorX => self.editor.mirror[0],
            Op::MirrorY => self.editor.mirror[1],
            Op::MirrorZ => self.editor.mirror[2],
            _ => false,
        }
    }
    fn invoke(&mut self, op: Op, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled(op) {
            return;
        }
        self.menu = None;
        self.command_open = false;
        self.palette_open = false;
        if self.pending.is_some() || self.dialog_open {
            return;
        }
        if matches!(op, Op::New | Op::Open | Op::Quit) && self.editor.engine.is_dirty() {
            self.pending = Some(op);
            self.pending_choice = 2;
            cx.notify();
            return;
        }
        match op {
            Op::Settings | Op::Theme => {
                self.open_settings(
                    if op == Op::Theme {
                        settings::Section::Appearance
                    } else {
                        settings::Section::General
                    },
                    window,
                    cx,
                );
                return;
            }
            Op::Minimize => window.minimize_window(),
            Op::ZoomWindow => window.zoom_window(),
            Op::ShowPalette => {
                self.panel = Panel::Session;
                self.sidebar_scroll.scroll_to_bottom();
                window.focus(&self.palette_focus, cx);
                self.status = "Palette focused · Arrow keys choose a brush color · Escape returns to the viewport".into();
                cx.notify();
                return;
            }
            _ => self.run(op, cx),
        }
        window.focus(&self.focus, cx);
    }
    fn run(&mut self, op: Op, cx: &mut Context<Self>) {
        match op {
            Op::Settings | Op::Theme | Op::Minimize | Op::ZoomWindow | Op::ShowPalette => {},
            Op::About => self.info = Some(("About Voxelite", format!("Voxelite {}\nA voxel workspace by Calum.\nBuilt with Rust and GPUI.", env!("CARGO_PKG_VERSION")).into())),
            Op::CheckUpdates => {
                #[cfg(target_os = "macos")]
                crate::updater::check_for_updates();
                self.status = "Update integration is not available in this build.".into();
            }
            Op::Hide => cx.hide(),
            Op::HideOthers => cx.hide_other_apps(),
            Op::ShowAll => cx.unhide_other_apps(),
            Op::Quit => cx.quit(),
            Op::New => match Editor::blank() {
                Ok(editor) => {
                    self.replace_editor(editor, cx);
                    self.status = "New project".into();
                }
                Err(e) => {
                    self.status = e.to_string().into();
                    self.error = true;
                }
            },
            Op::Open => self.open_dialog(cx),
            Op::Save => self.save(false, cx),
            Op::SaveAs => self.save(true, cx),
            #[cfg(target_family = "wasm")]
            Op::ExportGlb => self.export_browser(false, cx),
            #[cfg(target_family = "wasm")]
            Op::ExportPng => self.export_browser(true, cx),
            Op::Share => self.open_share(cx),
            Op::Comment => self.begin_comment(cx),
            Op::Undo => {
                let r = self.editor.undo();
                self.report(r, "Undone", cx);
            }
            Op::Redo => {
                let r = self.editor.redo();
                self.report(r, "Redone", cx);
            }
            Op::Grid => settings::change(cx, |p| p.grid = !p.grid),
            Op::Frame | Op::Front | Op::Right | Op::Top => {
                self.agent.follow = false;
                let b = self.viewport.get();
                let aspect = (f32::from(b.size.width) / f32::from(b.size.height).max(1.0)) as f64;
                let view = match op {
                    Op::Front => PresetView::Front,
                    Op::Right => PresetView::Right,
                    Op::Top => PresetView::Top,
                    _ => PresetView::Iso,
                };
                self.editor.camera =
                    OrbitCamera::fit_view(self.editor.display_snapshot(), aspect, view);
                self.camera_view = Some(op);
                self.camera_fitted = true;
                self.request_render(cx);
            }
            Op::Place | Op::Erase | Op::Paint | Op::Fill | Op::Box | Op::Picker => {
                self.editor.tool = match op {
                    Op::Erase => Tool::Erase,
                    Op::Paint => Tool::Paint,
                    Op::Fill => Tool::Fill,
                    Op::Box => Tool::Box,
                    Op::Picker => Tool::Picker,
                    _ => Tool::Place,
                };
                self.editor.box_start = None;
                self.status = if op == Op::Box {
                    HintText::from("Box: click the first corner, then the opposite corner. ")
                        .key("escape").text(" cancels.")
                } else {
                    format!("{} tool", self.editor.tool.name()).into()
                };
                self.error = false;
            }
            Op::MirrorX | Op::MirrorY | Op::MirrorZ => {
                let a = match op {
                    Op::MirrorX => 0,
                    Op::MirrorY => 1,
                    _ => 2,
                };
                self.editor.mirror[a] = !self.editor.mirror[a];
            }
            Op::AddLayer => {
                let r = self.editor.add_layer();
                self.report(r, "Layer added", cx);
            }
            Op::ManagePalettes => self.manage_palettes(cx),
            Op::Palette => {
                self.palette_open = true;
                self.palette_cursor = self.palette_index;
            }
            Op::Commands | Op::ToolPicker => {
                self.tools_only = op == Op::ToolPicker;
                self.command_open = true;
                self.query.clear();
                self.command_index = 0;
            }
            Op::Agent => {
                self.agent.open = true;
            }
            Op::Extensions => {
                self.info = Some((
                    "Extensions",
                    "Place, Erase, Paint, Fill, Box and Picker are included. Custom extensions are not available in this build.".into(),
                ))
            }
            Op::Help => {
                self.info = Some((
                    "Make yourself at home",
                    keyboard_guide(),
                ))
            }
        }
        cx.notify();
    }
    #[cfg(not(target_family = "wasm"))]
    fn open_dialog(&mut self, cx: &mut Context<Self>) {
        self.dialog_open = true;
        self.status = "Choose a project to open…".into();
        self.drag = None;
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open .voxl project".into()),
        });
        cx.spawn(async move |this, cx| {
            let result = receiver.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(Ok(Some(paths))) if !paths.is_empty() => {
                        let path = paths[0].clone();
                        this.status = "Opening project…".into();
                        let task = cx.background_executor().spawn(async move { Editor::open(&path) });
                        cx.spawn(async move |this, cx| {
                            let result = task.await;
                            let _ = this.update(cx, |this, cx| {
                                this.dialog_open = false;
                                match result {
                                    Ok(editor) => { this.replace_editor(editor, cx); this.status = "Project opened".into(); }
                                    Err(error) => { this.status = error.to_string().into(); this.error = true; }
                                }
                                cx.notify();
                            });
                        }).detach();
                    }
                    Ok(Ok(_)) => { this.dialog_open = false; }
                    _ => {
                        this.dialog_open = false;
                        this.status = "Open dialog unavailable. Enable a desktop file portal or open a .voxl path from the command line.".into();
                        this.error = true;
                    }
                }
                cx.notify();
            });
        }).detach();
    }
    #[cfg(not(target_family = "wasm"))]
    fn write_project(&mut self, path: PathBuf, copy: bool, cx: &mut Context<Self>) {
        self.dialog_open = true;
        self.status = "Saving project…".into();
        let save = self.editor.prepare_save(&path);
        let task = cx.background_executor().spawn(async move { save.write() });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.dialog_open = false;
                let next = this.after_save.take();
                match result {
                    Ok(saved) => {
                        if copy {
                            this.status = "Project copy saved".into();
                        } else {
                            this.editor.finish_save(saved);
                            this.status = "Saved".into();
                        }
                        this.error = false;
                        if let Some(op) = next {
                            this.run(op, cx);
                        }
                    }
                    Err(error) => {
                        this.status = error.to_string().into();
                        this.error = true;
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
    #[cfg(not(target_family = "wasm"))]
    fn save(&mut self, save_as: bool, cx: &mut Context<Self>) {
        if !save_as && let Some(path) = self.editor.path.clone() {
            self.write_project(path, false, cx);
            return;
        }
        self.choose_save_path(false, cx);
    }
    /// A shared copy must not adopt its destination or clear the working file's dirty state.
    #[cfg(not(target_family = "wasm"))]
    fn choose_save_path(&mut self, copy: bool, cx: &mut Context<Self>) {
        let directory = self
            .editor
            .path
            .as_ref()
            .and_then(|p| p.parent())
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let suggested = format!(
            "{}.voxl",
            self.editor.snapshot().name.replace(['/', '\\'], "-")
        );
        self.dialog_open = true;
        self.status = "Choose where to save this project…".into();
        let receiver = cx.prompt_for_new_path(&directory, Some(&suggested));
        cx.spawn(async move |this, cx| {
            let result = receiver.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(Ok(Some(mut path))) => {
                        if path.extension().is_none() { path.set_extension("voxl"); }
                        this.write_project(path, copy, cx);
                    }
                    Ok(Ok(None)) => { this.dialog_open = false; this.after_save = None; }
                    _ => {
                        this.dialog_open = false;
                        this.after_save = None;
                        this.status = "Save dialog unavailable. Enable a desktop file portal; edits are still open.".into();
                        this.error = true;
                    }
                }
                cx.notify();
            });
        }).detach();
    }
    fn resolve_pending(&mut self, choice: usize, cx: &mut Context<Self>) {
        let Some(op) = self.pending.take() else {
            return;
        };
        match choice {
            0 => self.run(op, cx),
            2 => {
                self.after_save = Some(op);
                self.save(false, cx);
            }
            _ => {}
        }
        cx.notify();
    }
    fn ray(&self, p: Point<Pixels>) -> Option<voxelite_app::editor::Ray> {
        // Pointer edits must use the exact camera of the image the user can see.
        if self.rendering
            || self.interactive_render
            || self.image.is_none()
            || self.displayed != Some(self.frame_identity())
        {
            return None;
        }
        let b = self.viewport.get();
        let w = f64::from(f32::from(b.size.width));
        let h = f64::from(f32::from(b.size.height));
        self.display_camera.ray(
            [
                f64::from(f32::from(p.x - b.origin.x)) / w,
                f64::from(f32::from(p.y - b.origin.y)) / h,
            ],
            f64::from(self.render_size[0]) / f64::from(self.render_size[1].max(1)),
        )
    }
    fn pointer_move(&mut self, e: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.command_open
            || self.menu.is_some()
            || self.palette_open
            || self.info.is_some()
            || self.pending.is_some()
            || self.dialog_open
        {
            return;
        }
        if matches!(
            e.pressed_button,
            Some(MouseButton::Right | MouseButton::Middle)
        ) {
            if let Some(previous) = self.drag {
                let dx = f64::from(f32::from(e.position.x - previous.x));
                let dy = f64::from(f32::from(e.position.y - previous.y));
                if e.pressed_button == Some(MouseButton::Middle) || e.modifiers.shift {
                    self.editor.camera.pan(
                        dx,
                        dy,
                        f64::from(f32::from(self.viewport.get().size.height)),
                    );
                } else {
                    self.editor.camera.orbit(dx, dy);
                }
                self.agent.follow = false;
                self.camera_view = None;
                self.camera_fitted = false;
                self.request_navigation(cx);
            }
            self.drag = Some(e.position);
            self.hover = None;
        } else {
            self.drag = None;
            self.hover = self.ray(e.position).and_then(|r| self.editor.target(r));
        }
        cx.notify();
    }
    fn key_down(&mut self, e: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        #[cfg(target_family = "wasm")]
        if self.settings_modal.is_some() {
            return;
        }
        if self.palette_focus.is_focused(window) {
            let delta = match e.keystroke.key.as_str() {
                "left" => Some(-1),
                "right" => Some(1),
                "up" => Some(-8),
                "down" => Some(8),
                _ => None,
            };
            if let Some(delta) = delta {
                let palette = self.active_palette();
                if !palette.order.is_empty() {
                    let current = palette
                        .order
                        .iter()
                        .position(|slot| {
                            palette
                                .get(*slot)
                                .is_some_and(|color| color.rgba == self.editor.color.rgba)
                        })
                        .unwrap_or(0);
                    let next = (current as isize + delta).rem_euclid(palette.order.len() as isize)
                        as usize;
                    self.editor.color = palette
                        .get(palette.order[next])
                        .expect("palette slot")
                        .clone();
                    self.swatch_scroll
                        .set_offset(point(px(0.0), -px((next / 8) as f32 * 38.0)));
                    cx.notify();
                }
                cx.stop_propagation();
                return;
            }
            if e.keystroke.key != "escape" {
                // Choosing a brush color must never apply the viewport tool.
                return;
            }
        }
        if self.dialog_open {
            cx.stop_propagation();
            return;
        }
        let key = navigation_key(&e.keystroke).unwrap_or_default();
        if key == "tab"
            && self.pending.is_none()
            && !self.command_open
            && self.menu.is_none()
            && !self.agent.open
            && self.info.is_none()
        {
            let fields = if self.sharing.open {
                vec![self.sharing.person.focus_handle(cx)]
            } else if self.palettes.open {
                vec![
                    self.palettes.name.focus_handle(cx),
                    self.palettes.hex.focus_handle(cx),
                ]
            } else if self.panel == Panel::Comments
                && self.editor.preview.is_none()
                && (self.discussion.selected.is_some() || self.discussion.draft.is_some())
            {
                vec![self.discussion.text.focus_handle(cx)]
            } else if self.panel == Panel::History && self.editor.preview.is_none() {
                vec![self.timeline.name.focus_handle(cx)]
            } else {
                vec![]
            };
            if !fields.is_empty() {
                let i = fields
                    .iter()
                    .position(|f| f.is_focused(window))
                    .map_or(0, |i| {
                        (i + if e.keystroke.modifiers.shift {
                            fields.len() - 1
                        } else {
                            1
                        }) % fields.len()
                    });
                window.focus(&fields[i], cx);
                cx.stop_propagation();
                cx.notify();
                return;
            }
        }

        if self.sharing.open && self.sharing.person.read(cx).focused(window) && key == "enter" {
            self.add_share_person(cx);
            cx.stop_propagation();
            return;
        }
        if key != "escape"
            && (self.palettes.name.read(cx).focused(window)
                || self.palettes.hex.read(cx).focused(window)
                || self.discussion.text.read(cx).focused(window)
                || self.timeline.name.read(cx).focused(window)
                || self.sharing.person.read(cx).focused(window))
        {
            return;
        }
        if key == "escape" {
            self.discussion.placing = false;
            self.discussion.draft = None;
            self.discussion.selected = None;
            if self.editor.preview.is_some() {
                self.editor.return_live();
                self.refresh_preview(cx);
            }
            self.agent.open = false;
            self.sharing.open = false;
            self.palettes.open = false;
            window.focus(&self.focus, cx);
            self.menu = None;
            self.command_open = false;
            self.palette_open = false;
            self.info = None;
            self.pending = None;
            self.editor.box_start = None;
            self.drag = None;
            self.hover = None;
            self.status = "Ready".into();
            self.error = false;
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if self.palettes.open || self.sharing.open || self.agent.open {
            cx.stop_propagation();
            return;
        }
        if self.pending.is_some() {
            match key {
                "left" => self.pending_choice = (self.pending_choice + 2) % 3,
                "right" | "tab" => {
                    self.pending_choice =
                        (self.pending_choice + if e.keystroke.modifiers.shift { 2 } else { 1 }) % 3
                }
                "enter" => self.resolve_pending(self.pending_choice, cx),
                _ => {}
            }
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if self.palette_open {
            match key {
                "down" => self.palette_cursor = (self.palette_cursor + 1) % PALETTES.len(),
                "up" => {
                    self.palette_cursor =
                        (self.palette_cursor + PALETTES.len() - 1) % PALETTES.len()
                }
                "enter" => {
                    self.palette_index = self.palette_cursor;
                    self.palettes.custom = None;
                    self.palette_open = false;
                }
                _ => {}
            }
            self.palette_scroll.scroll_to_item(self.palette_cursor);
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if self.command_open {
            let count = self.filtered_actions().len();
            match key {
                "backspace" => {
                    self.query.pop();
                    self.command_index = 0;
                }
                "down" => {
                    self.command_index = (self.command_index + 1).min(count.saturating_sub(1))
                }
                "up" => self.command_index = self.command_index.saturating_sub(1),
                "enter" => {
                    if let Some(op) = self.filtered_actions().get(self.command_index).copied() {
                        self.invoke(op, window, cx);
                    }
                }
                _ => {
                    if let Some(text) = &e.keystroke.key_char
                        && self.query.len() + text.len() <= 128
                        && !text.chars().any(char::is_control)
                        && ShortcutPlatform::current().allows_text(e.keystroke.modifiers)
                    {
                        self.query.push_str(text);
                        self.command_index = 0;
                    }
                }
            }
            self.command_scroll.scroll_to_item(self.command_index);
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if let Some(menu) = self.menu {
            let items = MENUS[menu].1;
            match key {
                "down" => self.menu_index = (self.menu_index + 1) % items.len(),
                "up" => self.menu_index = (self.menu_index + items.len() - 1) % items.len(),
                "right" => {
                    self.menu = Some((menu + 1) % MENUS.len());
                    self.menu_index = 0;
                }
                "left" => {
                    self.menu = Some((menu + MENUS.len() - 1) % MENUS.len());
                    self.menu_index = 0;
                }
                "enter" => self.invoke(items[self.menu_index], window, cx),
                _ => {}
            }
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if self.info.is_some() || self.pending.is_some() || self.palette_open {
            cx.stop_propagation();
            return;
        }
        if self.panel == Panel::History && matches!(key, "left" | "right") {
            let ids: Vec<_> = std::iter::once(None)
                .filter(|_| self.editor.engine.initial_retained())
                .chain(self.editor.engine.timeline().map(|e| Some(e.id)))
                .collect();
            let index = self
                .editor
                .preview
                .as_ref()
                .and_then(|p| {
                    ids.iter().position(|id| {
                        p.source == voxelite_app::history::PreviewSource::History(*id)
                    })
                })
                .unwrap_or_else(|| {
                    ids.iter()
                        .position(|id| *id == self.editor.engine.head())
                        .unwrap_or(0)
                });
            self.scrub(
                if key == "left" {
                    index.saturating_sub(1)
                } else {
                    (index + 1).min(ids.len().saturating_sub(1))
                },
                cx,
            );
            cx.stop_propagation();
            return;
        }
        let movement = match key {
            "left" => Some((0, -1)),
            "right" => Some((0, 1)),
            "up" => Some((2, -1)),
            "down" => Some((2, 1)),
            "pageup" => Some((1, 1)),
            "pagedown" => Some((1, -1)),
            _ => None,
        };
        if let Some((a, delta)) = movement {
            self.cursor[a] = (self.cursor[a] + delta).clamp(
                self.editor.display_snapshot().bounds.min[a],
                self.editor.display_snapshot().bounds.max[a] - 1,
            );
            self.hover = Some(self.cursor);
            cx.stop_propagation();
        }
        if key == "enter" && self.discussion.placing {
            self.anchor_comment(self.cursor, cx);
            return;
        }
        if key == "enter" {
            let r = self.editor.apply_at(self.cursor);
            self.report(r, "Tool applied", cx);
            cx.stop_propagation();
        }
        if key == "[" || key == "]" {
            let palette = self.active_palette();
            let index = palette
                .order
                .iter()
                .position(|slot| {
                    palette
                        .get(*slot)
                        .is_some_and(|s| s.rgba == self.editor.color.rgba)
                })
                .unwrap_or(0);
            let index = if key == "]" {
                (index + 1) % palette.order.len()
            } else {
                (index + palette.order.len() - 1) % palette.order.len()
            };
            self.editor.color = palette
                .get(palette.order[index])
                .expect("valid palette")
                .clone();
            cx.stop_propagation();
        }
        cx.notify();
    }
    fn button(
        &self,
        id: &'static str,
        name: &'static str,
        label: &'static str,
        op: Op,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let theme = self.theme;
        let enabled = self.enabled(op);
        // Frame is an action; the camera selector owns the persistent view state.
        let selected = self.selected(op) && id != "view-fit";
        let primary = id == "save";
        let disabled_reason = self.disabled_reason(op);
        row()
            .id(id)
            .aria_keyshortcuts(op.shortcut())
            .tooltip(move |_, cx| {
                cx.new(|_| Hint {
                    text: if enabled {
                        HintText::from(op.label()).text("  ").action(op)
                    } else {
                        format!("{} · {}", op.label(), disabled_reason).into()
                    },
                    theme,
                })
                .into()
            })
            .h(px(32.0))
            .min_w(px(32.0))
            .px(px(9.0))
            .gap(px(8.0))
            .rounded(px(6.0))
            .when(enabled, |d| d.cursor_pointer())
            .text_color(if enabled {
                theme.muted()
            } else {
                theme.disabled()
            })
            .when(selected, |d| {
                d.bg(theme.selected()).text_color(theme.accent())
            })
            .when(enabled, |d| {
                d.hover(move |d| {
                    if primary {
                        d.bg(theme.text().opacity(0.88))
                            .text_color(theme.background())
                    } else if selected {
                        d.bg(theme.selected())
                    } else {
                        d.bg(theme.hover()).text_color(theme.text())
                    }
                })
            })
            .when(!name.is_empty(), |d| {
                d.child(icon(name).text_color(if !enabled {
                    theme.disabled()
                } else if primary {
                    theme.background()
                } else if selected {
                    theme.accent()
                } else {
                    theme.muted()
                }))
            })
            .when(!label.is_empty(), |d| d.child(label))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |this, _, window, cx| {
                if enabled {
                    this.invoke(op, window, cx);
                }
            }))
    }
    fn menubar(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        row()
            .h(px(MENUBAR_HEIGHT))
            .flex_shrink_0()
            .px(px(24.0))
            .border_b_1()
            .border_color(t.line())
            .gap(px(3.0))
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(px(14.0))
                    .mr(px(18.0))
                    .child("Voxelite"),
            )
            .children(MENUS.iter().enumerate().map(|(i, (name, _))| {
                row()
                    .id(("menu", i))
                    .h(px(32.0))
                    .px(px(10.0))
                    .rounded(px(6.0))
                    .text_color(t.muted())
                    .cursor_pointer()
                    .child(*name)
                    .when(self.menu == Some(i), |d| d.bg(t.hover()))
                    .hover(move |d| d.bg(t.hover()))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.menu = if this.menu == Some(i) { None } else { Some(i) };
                        this.menu_index = 0;
                        this.command_open = false;
                        cx.notify();
                    }))
            }))
            .child(div().flex_1())
    }
    /// Persistent tool and mirror controls use the same actions as menus and keys.
    fn tool_rail(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        div()
            .w(px(TOOL_RAIL_WIDTH))
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .py(px(12.0))
            .gap(px(6.0))
            .border_r_1()
            .border_color(t.line())
            .children(
                [
                    ("rail-place", Op::Place),
                    ("rail-erase", Op::Erase),
                    ("rail-paint", Op::Paint),
                    ("rail-fill", Op::Fill),
                    ("rail-box", Op::Box),
                    ("rail-picker", Op::Picker),
                ]
                .into_iter()
                .map(|(id, op)| {
                    self.button(id, op.icon(), "", op, cx)
                        .size(px(36.0))
                        .justify_center()
                        .px(px(0.0))
                        .border_1()
                        .border_color(if self.selected(op) {
                            t.accent().opacity(0.4)
                        } else {
                            t.background()
                        })
                }),
            )
            .child(div().w(px(24.0)).h(px(1.0)).my(px(6.0)).bg(t.line()))
            .children(
                [
                    ("rail-mirror-x", "X", Op::MirrorX),
                    ("rail-mirror-y", "Y", Op::MirrorY),
                    ("rail-mirror-z", "Z", Op::MirrorZ),
                ]
                .into_iter()
                .map(|(id, label, op)| {
                    self.button(id, op.icon(), label, op, cx)
                        .w(px(44.0))
                        .h(px(34.0))
                        .justify_center()
                        .px(px(0.0))
                        .gap(px(4.0))
                        .font_family("Geist Mono")
                        .text_size(px(12.0))
                }),
            )
    }
    fn toolbar(&self, compact: bool, cx: &Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        row()
            .id("main-titlebar")
            .on_mouse_down(MouseButton::Left, |event, window, _| {
                if event.click_count == 2 {
                    window.zoom_window();
                } else {
                    window.start_window_move();
                }
            })
            .h(px(TOOLBAR_HEIGHT))
            .flex_shrink_0()
            .px(px(24.0))
            .when(cfg!(target_os = "macos"), |d| d.pl(px(104.0)))
            .gap(px(16.0))
            .border_b_1()
            .border_color(t.line())
            .child(icon("cube").text_color(t.muted()).size(px(22.0)))
            .child(
                div()
                    .w(px(if compact { 184.0 } else { 232.0 }))
                    .min_w(px(0.0))
                    .flex_shrink_0()
                    .child(
                        row()
                            .gap(px(5.0))
                            .text_size(px(15.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(
                                div()
                                    .min_w(px(0.0))
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(self.editor.display_snapshot().name.clone()),
                            )
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::NORMAL)
                                    .text_color(t.muted())
                                    .child(".voxl"),
                            ),
                    )
                    .child(div().text_size(px(12.0)).text_color(t.muted()).child(
                        if self.editor.engine.is_dirty() {
                            "Unsaved changes"
                        } else if self.editor.path.is_some() {
                            "Saved locally"
                        } else {
                            "Not saved yet"
                        },
                    )),
            )
            .child(div().flex_1())
            .child(
                self.button(
                    "commands",
                    "search",
                    if compact { "" } else { "Search commands" },
                    Op::Commands,
                    cx,
                )
                .h(px(36.0))
                .gap(px(12.0))
                .child(
                    action_shortcut(Op::Commands)
                        .px(px(5.0))
                        .py(px(2.0))
                        .rounded(px(4.0))
                        .border_1()
                        .border_color(t.line()),
                ),
            )
            .child(div().w(px(1.0)).h(px(26.0)).bg(t.line()))
            .child(self.button("settings", "sliders-horizontal", "", Op::Settings, cx))
            .child(
                self.button("share", "share-2", "Share", Op::Share, cx)
                    .w(px(96.0))
                    .h(px(36.0))
                    .justify_center()
                    .flex_shrink_0()
                    .border_1()
                    .border_color(t.line()),
            )
            .child(
                self.button("save", "save", "Save", Op::Save, cx)
                    .w(px(96.0))
                    .h(px(36.0))
                    .justify_center()
                    .flex_shrink_0()
                    .border_1()
                    .border_color(t.text())
                    .font_weight(FontWeight::MEDIUM)
                    .bg(t.text())
                    .text_color(t.background()),
            )
    }
    /// Project the world axes with the displayed camera, not a fixed illustration.
    fn orientation(&self, cx: &Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        let (right, up, _) = self.display_camera.basis();
        let axes = [
            ("X", DVec3::X, Op::Right),
            ("Y", DVec3::Y, Op::Top),
            ("Z", DVec3::Z, Op::Front),
        ]
        .map(|(label, axis, op)| {
            (
                label,
                [axis.dot(right) as f32 * 34.0, -axis.dot(up) as f32 * 34.0],
                op,
            )
        });
        div()
            .id("orientation")
            .absolute()
            .top(px(12.0))
            .right(px(12.0))
            .size(px(104.0))
            .cursor_default()
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        let center = bounds.origin + point(px(52.0), px(52.0));
                        let mut path = PathBuilder::stroke(px(1.0));
                        for (_, delta, _) in axes {
                            path.move_to(center - point(px(delta[0] * 0.5), px(delta[1] * 0.5)));
                            path.line_to(center + point(px(delta[0]), px(delta[1])));
                        }
                        if let Ok(path) = path.build() {
                            window.paint_path(path, t.muted().opacity(0.55));
                        }
                    },
                )
                .size_full(),
            )
            .children(axes.into_iter().enumerate().map(|(i, (label, delta, op))| {
                row()
                    .id(("axis", i))
                    .absolute()
                    .left(px(36.0 + delta[0]))
                    .top(px(36.0 + delta[1]))
                    .size(px(32.0))
                    .justify_center()
                    .rounded_full()
                    .cursor_pointer()
                    .tooltip(hint(op.label(), t))
                    .hover(move |d| d.bg(t.hover()))
                    .child(
                        row()
                            .size(px(22.0))
                            .justify_center()
                            .rounded_full()
                            .border_1()
                            .border_color(t.line())
                            .bg(t.background())
                            .text_size(px(12.0))
                            .text_color(t.muted())
                            .child(label),
                    )
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(cx.listener(move |this, _, window, cx| this.invoke(op, window, cx)))
            }))
            .on_mouse_move(|_, _, cx| cx.stop_propagation())
    }

    fn viewport(&self, cx: &Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        let compact = self.render_size[0] < 800;
        let bounds_cell = self.viewport.clone();
        let image = self.image.clone();
        let camera = self.display_camera;
        let frame_size = self.displayed.map(|_| self.display_size);
        let hover = self.hover;
        let grid = self.grid;
        let doc_bounds = self.editor.display_snapshot().bounds;
        let box_start = self.editor.box_start;
        let comment_region = self
            .discussion
            .draft
            .as_ref()
            .or_else(|| {
                self.discussion
                    .selected
                    .and_then(|id| {
                        self.editor
                            .display_snapshot()
                            .comments
                            .iter()
                            .find(|c| c.id == id)
                    })
                    .map(|c| &c.anchor)
            })
            .and_then(|a| a.region().ok());
        let surface = canvas(
            move |bounds, _, _| {
                bounds_cell.set(bounds);
            },
            move |bounds, _, window, _| {
                let aspect = frame_size
                    .map(|s| f64::from(s[0]) / f64::from(s[1].max(1)))
                    .unwrap_or_else(|| {
                        f64::from(f32::from(bounds.size.width) / f32::from(bounds.size.height))
                    });
                let project = |p: DVec3| {
                    let uv = camera.project(p, aspect);
                    point(
                        bounds.origin.x + bounds.size.width * uv[0] as f32,
                        bounds.origin.y + bounds.size.height * uv[1] as f32,
                    )
                };
                if grid {
                    let mut path = PathBuilder::stroke(px(0.6));
                    let y = f64::from(doc_bounds.min[1]) - 0.05;
                    let extent = doc_bounds.min[0].min(doc_bounds.min[2]) - 16;
                    let end = doc_bounds.max[0].max(doc_bounds.max[2]) + 16;
                    for a in (extent..=end).step_by(4) {
                        path.move_to(project(DVec3::new(f64::from(a), y, f64::from(extent))));
                        path.line_to(project(DVec3::new(f64::from(a), y, f64::from(end))));
                        path.move_to(project(DVec3::new(f64::from(extent), y, f64::from(a))));
                        path.line_to(project(DVec3::new(f64::from(end), y, f64::from(a))));
                    }
                    if let Ok(path) = path.build() {
                        window.paint_path(path, t.line().opacity(0.24));
                    }
                }
                if let Some(image) = image {
                    let _ = window.paint_image(bounds, bounds, Corners::default(), image, 0, false);
                }
                if let Some(at) = comment_region.map(|r| r.min).or(hover).or(box_start) {
                    let lo = box_start
                        .map(|s| std::array::from_fn(|a| s[a].min(at[a])))
                        .unwrap_or(at);
                    let hi: Pos = box_start
                        .map(|s| std::array::from_fn(|a| s[a].max(at[a]) + 1))
                        .unwrap_or(at.map(|v| v + 1));
                    let mut path = PathBuilder::stroke(px(1.4));
                    let (lo, hi) = comment_region.map(|r| (r.min, r.max)).unwrap_or((lo, hi));
                    for bits in 0..8 {
                        for a in 0..3 {
                            if bits & (1 << a) == 0 {
                                let p: [f64; 3] = std::array::from_fn(|j| {
                                    f64::from(if bits & (1 << j) == 0 { lo[j] } else { hi[j] })
                                });
                                let mut q = p;
                                q[a] = f64::from(hi[a]);
                                path.move_to(project(p.into()));
                                path.line_to(project(q.into()));
                            }
                        }
                    }
                    if let Ok(path) = path.build() {
                        window.paint_path(path, t.accent());
                    }
                }
            },
        )
        .size_full();
        div()
            .id("viewport")
            .relative()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .cursor_crosshair()
            .child(surface)
            .child(
                div().absolute().top(px(20.0)).left(px(24.0)).child(
                    row()
                        .gap(px(10.0))
                        .text_color(t.muted())
                        .child(match self.camera_view {
                            Some(Op::Front) => "Front",
                            Some(Op::Right) => "Right",
                            Some(Op::Top) => "Top",
                            Some(Op::Frame) => "Isometric",
                            _ => "Orthographic",
                        }),
                ),
            )
            .children(self.comment_pins(cx))
            .when(self.editor.preview.is_some(), |d| {
                d.child(
                    row()
                        .id("preview-banner")
                        .absolute()
                        .top(px(52.0))
                        .left(px(24.0))
                        .px(px(12.0))
                        .py(px(6.0))
                        .rounded(px(6.0))
                        .bg(t.selected())
                        .text_color(t.accent())
                        .gap(px(8.0))
                        .child(icon("lock").text_color(t.accent()))
                        .child("Read-only preview")
                        .child(
                            self.control(
                                "preview-exit",
                                "play",
                                "Return to live",
                                cx,
                                |this, window, cx| {
                                    this.editor.return_live();
                                    this.refresh_preview(cx);
                                    window.focus(&this.focus, cx);
                                },
                            )
                            .border_0(),
                        ),
                )
            })
            .child(self.orientation(cx))
            .when(self.preferences.navigation_hints, |d| {
                d.child(
                    div().absolute().bottom(px(20.0)).left(px(24.0)).child(
                        row()
                            .id("camera-hints")
                            .h(px(32.0))
                            .gap(px(12.0))
                            .text_size(px(12.0))
                            .text_color(t.muted())
                            .child(
                                row()
                                    .id("orbit-hint")
                                    .gap(px(4.0))
                                    .tooltip(hint("Right-drag to orbit", t))
                                    .child(icon("mouse").text_color(t.muted()))
                                    .child(icon("orbit").text_color(t.muted()))
                                    .when(!compact, |d| d.child("Orbit")),
                            )
                            .child(div().w(px(1.0)).h(px(12.0)).bg(t.line()))
                            .child(
                                row()
                                    .id("pan-hint")
                                    .gap(px(4.0))
                                    .tooltip(hint(
                                        HintText::default()
                                            .key("shift-rightdrag")
                                            .text(" or middle-drag to pan"),
                                        t,
                                    ))
                                    .child(icon("arrow-up").text_color(t.muted()))
                                    .child(icon("mouse").text_color(t.muted()))
                                    .child(icon("move").text_color(t.muted()))
                                    .when(!compact, |d| d.child("Pan")),
                            )
                            .child(div().w(px(1.0)).h(px(12.0)).bg(t.line()))
                            .child(
                                row()
                                    .id("zoom-hint")
                                    .gap(px(4.0))
                                    .tooltip(hint(
                                        HintText::from("Scroll to zoom · ")
                                            .action(Op::Frame)
                                            .text(" to frame the model"),
                                        t,
                                    ))
                                    .child(icon("mouse").text_color(t.muted()))
                                    .child(icon("zoom-in").text_color(t.muted()))
                                    .when(!compact, |d| d.child("Zoom")),
                            ),
                    ),
                )
            })
            .child(
                div().absolute().bottom(px(16.0)).right(px(24.0)).child(
                    row()
                        .gap(px(4.0))
                        .p(px(4.0))
                        .bg(t.background())
                        .border_1()
                        .border_color(t.line())
                        .rounded(px(8.0))
                        .children(
                            [
                                ("view-iso", "Iso", Op::Frame),
                                ("view-front", "Front", Op::Front),
                                ("view-right", "Right", Op::Right),
                                ("view-top", "Top", Op::Top),
                            ]
                            .map(|(id, label, op)| {
                                self.button(id, op.icon(), label, op, cx)
                                    .border_b_2()
                                    .border_color(transparent_black())
                                    .when(self.camera_view == Some(op), |d| {
                                        d.bg(t.selected())
                                            .text_color(t.accent())
                                            .border_color(t.accent())
                                    })
                            }),
                        )
                        .child(div().h(px(20.0)).w(px(1.0)).mx(px(4.0)).bg(t.line()))
                        .child(self.button("view-fit", "fit", "", Op::Frame, cx)),
                ),
            )
            .on_mouse_move(cx.listener(Self::pointer_move))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, e: &MouseDownEvent, _, cx| {
                    if this.dialog_open {
                        return;
                    }
                    this.drag = Some(e.position);
                    cx.stop_propagation();
                }),
            )
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(|this, e: &MouseDownEvent, _, cx| {
                    if this.dialog_open {
                        return;
                    }
                    this.drag = Some(e.position);
                    cx.stop_propagation();
                }),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, e: &MouseDownEvent, window, cx| {
                    window.focus(&this.focus, cx);
                    if this.menu.take().is_some() || this.palette_open {
                        this.palette_open = false;
                        cx.notify();
                        return;
                    }
                    if this.command_open
                        || this.info.is_some()
                        || this.pending.is_some()
                        || this.dialog_open
                    {
                        return;
                    }
                    if this.editor.preview.is_some() {
                        this.status =
                            "History preview is read-only · Return to live to edit".into();
                        cx.notify();
                        return;
                    }
                    if let Some(ray) = this.ray(e.position) {
                        if this.discussion.placing {
                            if let Some(hit) =
                                voxelite_app::editor::raycast(this.editor.display_snapshot(), ray)
                            {
                                this.anchor_comment(hit.at, cx);
                            }
                            return;
                        }
                        let r = this.editor.click(ray, e.modifiers.alt);
                        let status = if this.editor.box_start.is_some() {
                            HintText::from("First corner set · click opposite corner · ")
                                .key("escape")
                                .text(" cancels")
                        } else {
                            "Ready".into()
                        };
                        this.report(r, status, cx);
                    }
                }),
            )
            .on_mouse_up(
                MouseButton::Right,
                cx.listener(|this, _, _, _| this.drag = None),
            )
            .on_mouse_up(
                MouseButton::Middle,
                cx.listener(|this, _, _, _| this.drag = None),
            )
            .on_mouse_up_out(
                MouseButton::Right,
                cx.listener(|this, _, _, _| this.drag = None),
            )
            .on_mouse_up_out(
                MouseButton::Middle,
                cx.listener(|this, _, _, _| this.drag = None),
            )
            .on_scroll_wheel(cx.listener(|this, e: &ScrollWheelEvent, _, cx| {
                if this.dialog_open
                    || this.command_open
                    || this.palette_open
                    || this.menu.is_some()
                    || this.info.is_some()
                    || this.pending.is_some()
                {
                    return;
                }
                let delta = match e.delta {
                    ScrollDelta::Pixels(p) => f64::from(f32::from(p.y)),
                    ScrollDelta::Lines(p) => f64::from(p.y) * 40.0,
                };
                this.agent.follow = false;
                this.editor.camera.zoom(delta);
                this.camera_fitted = false;
                this.request_navigation(cx);
                cx.notify();
                cx.stop_propagation();
            }))
    }
    fn palette_panel(&self, compact: bool, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let palette = self.active_palette();
        let color = self.editor.color.rgba;
        // Eight 32px swatches, 6px gaps and 2px focus-ring padding per row.
        let visible_rows = if compact { 2 } else { 5 };
        let swatch_height = visible_rows as f32 * 38.0 - 2.0;
        let overflowing = palette.order.len() > visible_rows * 8;
        let hex = format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
        div()
            .flex_shrink_0()
            .px(px(PANEL_GUTTER))
            .pt(px(if compact { 8.0 } else { 16.0 }))
            .pb(px(if compact { 8.0 } else { 20.0 }))
            .child(
                row()
                    .justify_between()
                    .mb(px(if compact { 8.0 } else { 10.0 }))
                    .child(
                        row()
                            .id("palette-switch")
                            .h(px(32.0))
                            .gap(px(8.0))
                            .rounded(px(6.0))
                            .cursor_pointer()
                            .tooltip(hint("Choose a project or preset palette", t))
                            .hover(move |d| d.bg(t.hover()))
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(self.palette_name()),
                            )
                            .child(icon("chevron").size(px(14.0)).text_color(t.muted()))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.palette_open = !this.palette_open;
                                this.palette_cursor = this.palette_index;
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(t.muted())
                            .child(format!("{} colors", palette.order.len())),
                    ),
            )
            .child(
                div()
                    .id("swatch-scroll")
                    .track_focus(&self.palette_focus)
                    .focus(move |d| d.border_1().border_color(t.accent()))
                    .track_scroll(&self.swatch_scroll)
                    .max_h(px(swatch_height))
                    .overflow_y_scroll()
                    .child(div().flex().flex_wrap().gap(px(6.0)).p(px(2.0)).children(
                        palette.order.iter().enumerate().map(|(i, slot)| {
                            let swatch = palette.get(*slot).expect("valid palette").clone();
                            let c = swatch.rgba;
                            let selected = c == color;
                            let hex = format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2]);
                            let label = swatch
                                .name
                                .as_ref()
                                .map(|name| format!("{name} · {hex}"))
                                .unwrap_or(hex);
                            row()
                                .id(("swatch", i))
                                .size(px(32.0))
                                .justify_center()
                                .rounded(px(6.0))
                                .border_1()
                                .border_color(t.swatch_edge())
                                .bg(color_fill(c))
                                .when(selected, |d| d.border_2().border_color(t.text()))
                                .cursor_pointer()
                                .tooltip(hint(label, t))
                                .hover(move |d| d.border_color(t.text()))
                                .when(selected, |d| {
                                    d.child(icon("check").size(px(16.0)).text_color(swatch_ink(c)))
                                })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.editor.color = swatch.clone();
                                    this.error = false;
                                    cx.notify();
                                }))
                        }),
                    )),
            )
            .child(
                row()
                    .mt(px(if compact { 8.0 } else { 12.0 }))
                    .gap(px(8.0))
                    .text_size(px(12.0))
                    .text_color(t.muted())
                    .child(mono(hex))
                    .child(div().flex_1())
                    .when(!overflowing, |d| d.child("Brush color"))
                    .when(overflowing, |d| {
                        d.child("Scroll colors")
                            .child(
                                self.control("colors-up", "arrow-up", "", cx, |this, _, cx| {
                                    let offset = this.swatch_scroll.offset();
                                    this.swatch_scroll
                                        .set_offset(point(px(0.0), offset.y + px(76.0)));
                                    cx.notify();
                                })
                                .size(px(24.0))
                                .px(px(0.0))
                                .justify_center()
                                .border_0()
                                .tooltip(hint("Scroll to previous colors", t)),
                            )
                            .child(
                                self.control("colors-down", "chevron", "", cx, |this, _, cx| {
                                    let offset = this.swatch_scroll.offset();
                                    this.swatch_scroll
                                        .set_offset(point(px(0.0), offset.y - px(76.0)));
                                    cx.notify();
                                })
                                .size(px(24.0))
                                .px(px(0.0))
                                .justify_center()
                                .border_0()
                                .tooltip(hint("Scroll to more colors", t)),
                            )
                    }),
            )
    }
    fn layers_panel(&self, compact: bool, cx: &Context<Self>) -> Div {
        let t = self.theme;
        div()
            .flex_1()
            .min_h(px(if compact { 180.0 } else { 216.0 }))
            .px(px(PANEL_GUTTER - 8.0))
            .py(px(if compact { 8.0 } else { 16.0 }))
            .border_t_1()
            .border_color(t.line())
            .child(
                row()
                    .pl(px(8.0))
                    .mb(px(if compact { 4.0 } else { 8.0 }))
                    .justify_between()
                    .child(section_title("Layers"))
                    .child(self.button("add-layer", "plus", "", Op::AddLayer, cx)),
            )
            .child(
                div()
                    .id("layer-scroll")
                    .max_h(px(if compact { 128.0 } else { 192.0 }))
                    .overflow_y_scroll()
                    .children(
                        self.editor
                            .display_snapshot()
                            .layers
                            .iter()
                            .rev()
                            .enumerate()
                            .map(|(i, layer)| {
                                let id = layer.id;
                                let visible = layer.visible;
                                let locked = layer.locked;
                                row()
                                    .id(("layer", i))
                                    .h(px(if compact { 32.0 } else { 36.0 }))
                                    .pl(px(6.0))
                                    .pr(px(8.0))
                                    .gap(px(8.0))
                                    .rounded(px(6.0))
                                    .cursor_pointer()
                                    .border_l_2()
                                    .border_color(if id == self.editor.layer {
                                        t.accent()
                                    } else {
                                        t.background()
                                    })
                                    .when(id == self.editor.layer, |d| {
                                        d.bg(t.hover()).font_weight(FontWeight::MEDIUM)
                                    })
                                    .hover(move |d| d.bg(t.hover()))
                                    .text_color(if visible { t.text() } else { t.muted() })
                                    .child(icon("layer").size(px(16.0)).text_color(t.muted()))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(0.0))
                                            .overflow_hidden()
                                            .text_ellipsis()
                                            .child(layer.name.clone()),
                                    )
                                    .child(
                                        row()
                                            .id(("eye", i))
                                            .w(px(32.0))
                                            .h(px(32.0))
                                            .justify_center()
                                            .rounded(px(6.0))
                                            .tooltip(hint(
                                                if visible { "Hide layer" } else { "Show layer" },
                                                t,
                                            ))
                                            .hover(move |d| d.bg(t.selected()))
                                            .child(
                                                icon(if visible { "eye" } else { "eye-off" })
                                                    .size(px(15.0))
                                                    .text_color(t.muted()),
                                            )
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                let r = this.editor.update_layer(
                                                    id,
                                                    Some(!visible),
                                                    None,
                                                );
                                                this.report(r, "Visibility updated", cx);
                                            })),
                                    )
                                    .child(
                                        row()
                                            .id(("lock", i))
                                            .w(px(32.0))
                                            .h(px(32.0))
                                            .justify_center()
                                            .rounded(px(6.0))
                                            .tooltip(hint(
                                                if locked { "Unlock layer" } else { "Lock layer" },
                                                t,
                                            ))
                                            .when(locked, |d| d.bg(t.selected()))
                                            .hover(move |d| d.bg(t.selected()))
                                            .child(
                                                icon(if locked { "lock" } else { "unlock" })
                                                    .size(px(15.0))
                                                    .text_color(if locked {
                                                        t.accent()
                                                    } else {
                                                        t.muted()
                                                    }),
                                            )
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                let r = this.editor.update_layer(
                                                    id,
                                                    None,
                                                    Some(!locked),
                                                );
                                                this.report(r, "Lock updated", cx);
                                            })),
                                    )
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.editor.layer = id;
                                        this.editor.box_start = None;
                                        cx.notify();
                                    }))
                            }),
                    ),
            )
    }
    fn history_panel(&self, compact: bool, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let empty = self.editor.engine.head().is_none();
        div()
            .flex_1()
            .min_h(px(if compact { 112.0 } else { 204.0 }))
            .flex()
            .flex_col()
            .px(px(PANEL_GUTTER))
            .py(px(if compact { 8.0 } else { 16.0 }))
            .gap(px(16.0))
            .child(
                row()
                    .justify_between()
                    .child(section_title("Recent changes"))
                    .child(
                        row()
                            .gap(px(2.0))
                            .child(self.button("history-undo", "undo", "", Op::Undo, cx))
                            .child(self.button("history-redo", "redo", "", Op::Redo, cx)),
                    ),
            )
            .child(
                div()
                    .id("history-scroll")
                    .min_h(px(0.0))
                    .flex_1()
                    .overflow_y_scroll()
                    .when(empty, |d| {
                        d.child(
                            row()
                                .items_start()
                                .gap(px(12.0))
                                .py(px(4.0))
                                .child(
                                    row()
                                        .size(px(32.0))
                                        .flex_shrink_0()
                                        .justify_center()
                                        .rounded(px(8.0))
                                        .bg(t.hover())
                                        .child(icon("history").text_color(t.muted())),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .child(
                                            div()
                                                .font_weight(FontWeight::MEDIUM)
                                                .child("No edits yet"),
                                        )
                                        .child(
                                            div()
                                                .mt(px(4.0))
                                                .text_size(px(12.0))
                                                .line_height(px(18.0))
                                                .text_color(t.muted())
                                                .child("Your changes will appear here."),
                                        )
                                        .when(!compact, |d| {
                                            d.child(
                                                self.button(
                                                    "history-guide",
                                                    "",
                                                    "Keyboard guide",
                                                    Op::Help,
                                                    cx,
                                                )
                                                .mt(px(8.0))
                                                .px(px(0.0))
                                                .text_color(t.text())
                                                .child(
                                                    icon("arrow-right")
                                                        .size(px(12.0))
                                                        .text_color(t.muted()),
                                                ),
                                            )
                                        }),
                                ),
                        )
                    })
                    .children(
                        self.editor
                            .engine
                            .history()
                            .rev()
                            .take(100)
                            .enumerate()
                            .map(|(i, entry)| {
                                row()
                                    .id(("history", i))
                                    .py(px(10.0))
                                    .gap(px(12.0))
                                    .items_start()
                                    .child(
                                        icon("check")
                                            .mt(px(2.0))
                                            .size(px(16.0))
                                            .text_color(t.muted()),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(0.0))
                                            .child(
                                                div()
                                                    .text_ellipsis()
                                                    .overflow_hidden()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .child(entry.label.clone()),
                                            )
                                            .child(
                                                mono(format!(
                                                    "{} · +{} −{} · {} painted",
                                                    match entry.actor.origin {
                                                        voxelite_core::Origin::Human => "Human",
                                                        voxelite_core::Origin::Agent => "Agent",
                                                    },
                                                    entry.changes.added,
                                                    entry.changes.removed,
                                                    entry.changes.recolored
                                                ))
                                                .text_color(t.muted())
                                                .mt(px(4.0)),
                                            ),
                                    )
                            }),
                    ),
            )
    }
    fn sidebar(&self, compact: bool, cx: &Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        div()
            .id("sidebar-scroll")
            .overflow_y_scroll()
            .track_scroll(&self.sidebar_scroll)
            .w(px(SIDEBAR_WIDTH))
            .flex_shrink_0()
            .h_full()
            .border_l_1()
            .border_color(t.line())
            .flex()
            .flex_col()
            .child(
                row()
                    .h(px(if compact { 52.0 } else { 72.0 }))
                    .flex_shrink_0()
                    .px(px(PANEL_GUTTER))
                    .gap(px(12.0))
                    .border_b_1()
                    .border_color(t.line())
                    .child(
                        row()
                            .size(px(32.0))
                            .justify_center()
                            .rounded(px(8.0))
                            .bg(t.hover())
                            .child(icon("spark").text_color(t.muted())),
                    )
                    .child(
                        div().flex_1().child(section_title("Local session")).child(
                            div()
                                .mt(px(2.0))
                                .text_size(px(12.0))
                                .text_color(t.muted())
                                .child(if self.agent.clients > 0 {
                                    format!(
                                        "{} agent connection{}",
                                        self.agent.clients,
                                        if self.agent.clients == 1 { "" } else { "s" }
                                    )
                                } else if !self.agent.events.is_empty() {
                                    "Agent disconnected".into()
                                } else {
                                    "Editing on this device".into()
                                }),
                        ),
                    )
                    .child(
                        div()
                            .id("revision")
                            .tooltip(hint("Current document revision", t))
                            .child(
                                mono(format!("r{:03}", self.editor.engine.revision()))
                                    .text_color(t.muted()),
                            ),
                    ),
            )
            .child(self.panel_tabs(cx))
            .child(match self.panel {
                Panel::Comments => self.comments_panel(cx),
                Panel::History => self.timeline_panel(cx),
                Panel::Session => self.agent_feed(compact, cx),
            })
            .when(self.panel == Panel::Session, |d| {
                d.child(self.layers_panel(compact || self.agent.capture.is_some(), cx))
                    .child(self.palette_panel(compact || self.agent.capture.is_some(), cx))
            })
    }
    fn statusbar(&self) -> Div {
        let t = self.theme;
        let s = self.editor.display_snapshot();
        let size: [i32; 3] = std::array::from_fn(|a| s.bounds.max[a] - s.bounds.min[a]);
        row()
            .h(px(STATUSBAR_HEIGHT))
            .flex_shrink_0()
            .border_t_1()
            .border_color(t.line())
            .px(px(24.0))
            .gap(px(16.0))
            .text_size(px(12.0))
            .text_color(t.muted())
            .child(mono(format!(
                "{} voxels",
                s.layers.iter().map(|l| l.voxels.len()).sum::<usize>()
            )))
            .child(mono(format!("{} × {} × {}", size[0], size[1], size[2])))
            .child(div().h(px(12.0)).w(px(1.0)).bg(t.line()))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_color(if self.error { t.danger() } else { t.muted() })
                    .child(self.status.clone()),
            )
            .when_some(self.hover, |d, p| {
                d.child(mono(format!("X {}   Y {}   Z {}", p[0], p[1], p[2])))
            })
            .when(self.rendering, |d| d.child("Rendering…"))
    }
    fn action_row(
        &self,
        op: Op,
        index: usize,
        selected: bool,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let t = self.theme;
        let enabled = self.enabled(op);
        row()
            .id(("action-row", index))
            .aria_keyshortcuts(op.shortcut())
            .h(px(36.0))
            .px(px(10.0))
            .gap(px(10.0))
            .rounded(px(6.0))
            .when(enabled, |d| d.cursor_pointer())
            .when(selected, |d| d.bg(t.hover()))
            .text_color(if enabled { t.text() } else { t.disabled() })
            .hover(move |d| d.bg(t.hover()))
            .child(div().w(px(14.0)).when(self.selected(op), |d| {
                d.child(icon("check").size(px(14.0)).text_color(t.accent()))
            }))
            .child(icon(op.icon()).text_color(if enabled { t.muted() } else { t.disabled() }))
            .child(op.label())
            .child(div().flex_1())
            .child(
                div()
                    .when(enabled, |d| d.child(action_shortcut(op)))
                    .when(!enabled, |d| d.child(mono(self.disabled_reason(op))))
                    .text_color(t.muted()),
            )
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |this, _, window, cx| {
                if enabled {
                    this.invoke(op, window, cx);
                }
            }))
    }
    fn floating(&self) -> Stateful<Div> {
        div()
            .id("floating-panel")
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .bg(self.theme.background())
            .border_1()
            .border_color(self.theme.line())
            .rounded(px(8.0))
            .shadow_lg()
            .p(px(6.0))
    }
    fn overlay(&self, cx: &Context<Self>) -> Option<AnyElement> {
        let t = self.theme;
        #[cfg(target_family = "wasm")]
        if let Some(settings) = &self.settings_modal {
            return Some(
                div()
                    .absolute()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgba(0x00000060))
                    .child(
                        div()
                            .w(px(880.0))
                            .max_w_full()
                            .h(px(640.0))
                            .max_h_full()
                            .rounded(px(14.0))
                            .shadow_lg()
                            .child(settings.clone()),
                    )
                    .into_any_element(),
            );
        }
        if self.agent.open && !self.dialog_open {
            return Some(self.agent_popover(cx));
        }
        if self.sharing.open && !self.dialog_open {
            return Some(self.share_popover(cx));
        }
        if self.palettes.open && !self.dialog_open {
            return Some(self.palette_manager(cx));
        }
        if self.dialog_open {
            return Some(
                div()
                    .absolute()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgba(0x00000070))
                    .child(self.floating().p(px(24.0)).child(self.status.clone()))
                    .into_any_element(),
            );
        }
        if self.pending.is_some() {
            return Some(
                div()
                    .absolute()
                    .size_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .bg(rgba(0x00000070))
                    .child(
                        self.floating()
                            .w(px(480.0))
                            .p(px(24.0))
                            .child(
                                div()
                                    .text_size(px(18.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Save your changes?"),
                            )
                            .child(div().my(px(16.0)).text_color(t.muted()).child(
                                "Save this project before continuing. Discarding cannot be undone.",
                            ))
                            .child(
                                row().justify_end().gap(px(8.0)).children(
                                    ["Discard", "Cancel", "Save & continue"]
                                        .into_iter()
                                        .enumerate()
                                        .map(|(i, label)| {
                                            row()
                                                .id(("pending-choice", i))
                                                .h(px(36.0))
                                                .px(px(12.0))
                                                .rounded(px(6.0))
                                                .border_1()
                                                .border_color(if self.pending_choice == i {
                                                    t.accent()
                                                } else {
                                                    t.line()
                                                })
                                                .text_color(if i == 0 {
                                                    t.danger()
                                                } else {
                                                    t.text()
                                                })
                                                .cursor_pointer()
                                                .hover(move |d| d.bg(t.hover()))
                                                .gap(px(8.0))
                                                .child(
                                                    icon(["trash-2", "close", "save"][i])
                                                        .text_color(t.muted()),
                                                )
                                                .child(label)
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.resolve_pending(i, cx)
                                                }))
                                        }),
                                ),
                            )
                            .child(
                                div()
                                    .mt(px(16.0))
                                    .text_size(px(12.0))
                                    .text_color(t.muted())
                                    .child(
                                        HintText::default()
                                            .key("tab")
                                            .text(" to choose · ")
                                            .key("enter")
                                            .text(" to confirm · ")
                                            .key("escape")
                                            .text(" to cancel"),
                                    ),
                            ),
                    )
                    .into_any_element(),
            );
        }
        if let Some((title, text)) = &self.info {
            return Some(
                div()
                    .absolute()
                    .size_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .bg(rgba(0x00000070))
                    .child(
                        self.floating()
                            .w(px(560.0))
                            .p(px(24.0))
                            .child(
                                row()
                                    .justify_between()
                                    .mb(px(18.0))
                                    .child(
                                        div()
                                            .text_size(px(18.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(*title),
                                    )
                                    .child(
                                        row()
                                            .id("close-info")
                                            .size(px(32.0))
                                            .justify_center()
                                            .cursor_pointer()
                                            .child(icon("close").text_color(t.muted()))
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.info = None;
                                                cx.notify();
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .text_color(t.muted())
                                    .line_height(px(26.0))
                                    .child(text.clone()),
                            ),
                    )
                    .into_any_element(),
            );
        }
        if self.command_open {
            let matches = self.filtered_actions();
            return Some(
                div()
                    .absolute()
                    .size_full()
                    .bg(rgba(0x00000070))
                    .flex()
                    .justify_center()
                    .pt(px(140.0))
                    .child(
                        self.floating()
                            .w(px(560.0))
                            .h(px(440.0))
                            .flex()
                            .flex_col()
                            .child(
                                row()
                                    .h(px(52.0))
                                    .px(px(14.0))
                                    .gap(px(12.0))
                                    .border_b_1()
                                    .border_color(t.line())
                                    .child(icon("search").text_color(t.muted()))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(0.0))
                                            .overflow_hidden()
                                            .text_ellipsis()
                                            .text_size(px(16.0))
                                            .child(if self.query.is_empty() {
                                                if self.tools_only {
                                                    "Search tools…"
                                                } else {
                                                    "Type a command…"
                                                }
                                                .to_string()
                                            } else {
                                                format!("{}│", self.query)
                                            }),
                                    )
                                    .child(shortcut("escape").text_color(t.muted())),
                            )
                            .child(
                                div()
                                    .id("command-results")
                                    .track_scroll(&self.command_scroll)
                                    .flex_1()
                                    .overflow_y_scroll()
                                    .py(px(6.0))
                                    .children(matches.iter().enumerate().map(|(i, op)| {
                                        self.action_row(*op, i, self.command_index == i, cx)
                                    }))
                                    .when(matches.is_empty(), |d| {
                                        d.child(
                                            div()
                                                .p(px(24.0))
                                                .text_color(t.muted())
                                                .child("No matching commands"),
                                        )
                                    }),
                            )
                            .child(
                                row()
                                    .h(px(32.0))
                                    .px(px(14.0))
                                    .text_color(t.muted())
                                    .text_size(px(12.0))
                                    .border_t_1()
                                    .border_color(t.line())
                                    .child(
                                        HintText::default()
                                            .key("up")
                                            .text(" ")
                                            .key("down")
                                            .text(" to navigate   ·   ")
                                            .key("enter")
                                            .text(" to run   ·   ")
                                            .key("escape")
                                            .text(" to close"),
                                    ),
                            ),
                    )
                    .into_any_element(),
            );
        }
        if self.palette_open {
            return Some(
                div()
                    .absolute()
                    .top(px(112.0))
                    .right(px(16.0))
                    .child(
                        self.floating()
                            .w(px(328.0))
                            .child(
                                row()
                                    .px(px(12.0))
                                    .h(px(40.0))
                                    .justify_between()
                                    .child(section_title("Choose palette"))
                                    .child(shortcut("escape").text_color(t.muted())),
                            )
                            .child(
                                div()
                                    .id("palette-results")
                                    .track_scroll(&self.palette_scroll)
                                    .max_h(px(360.0))
                                    .overflow_y_scroll()
                                    .children(PALETTES.iter().enumerate().map(
                                        |(i, (name, preset))| {
                                            let p =
                                                preset.map(|p| p.palette()).unwrap_or_else(|| {
                                                    self.editor.display_snapshot().palette.clone()
                                                });
                                            div()
                                                .id(("preset", i))
                                                .p(px(10.0))
                                                .rounded(px(6.0))
                                                .border_l_2()
                                                .border_color(if i == self.palette_cursor {
                                                    t.accent()
                                                } else {
                                                    t.background()
                                                })
                                                .cursor_pointer()
                                                .when(i == self.palette_cursor, |d| d.bg(t.hover()))
                                                .hover(move |d| d.bg(t.hover()))
                                                .child(
                                                    row()
                                                        .gap(px(8.0))
                                                        .mb(px(8.0))
                                                        .child(
                                                            div()
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .child(*name),
                                                        )
                                                        .when(i == self.palette_index, |d| {
                                                            d.child(
                                                                icon("check")
                                                                    .size(px(14.0))
                                                                    .text_color(t.accent()),
                                                            )
                                                        }),
                                                )
                                                .child(
                                                    div().flex().flex_wrap().gap(px(3.0)).children(
                                                        p.order.iter().take(32).map(|slot| {
                                                            div()
                                                                .w(px(15.0))
                                                                .h(px(14.0))
                                                                .rounded(px(3.0))
                                                                .border_1()
                                                                .border_color(t.swatch_edge())
                                                                .bg(color_fill(
                                                                    p.get(*slot)
                                                                        .expect("valid palette")
                                                                        .rgba,
                                                                ))
                                                        }),
                                                    ),
                                                )
                                                .child(
                                                    div()
                                                        .mt(px(6.0))
                                                        .text_size(px(12.0))
                                                        .text_color(t.muted())
                                                        .child(format!(
                                                            "{} colors{}{}",
                                                            p.order.len(),
                                                            if preset.is_some() {
                                                                " · Preset"
                                                            } else {
                                                                " · In this project"
                                                            },
                                                            if p.order.len() > 32 {
                                                                " · First 32 shown"
                                                            } else {
                                                                ""
                                                            }
                                                        )),
                                                )
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.palette_index = i;
                                                    this.palettes.custom = None;
                                                    this.palette_open = false;
                                                    cx.notify();
                                                }))
                                        },
                                    )),
                            )
                            .child(
                                div()
                                    .px(px(12.0))
                                    .py(px(12.0))
                                    .mt(px(4.0))
                                    .border_t_1()
                                    .border_color(t.line())
                                    .text_size(px(12.0))
                                    .text_color(t.muted())
                                    .child(self.control(
                                        "manage-palettes",
                                        "palette",
                                        "Manage palettes…",
                                        cx,
                                        |this, _, cx| {
                                            this.palette_open = false;
                                            this.manage_palettes(cx);
                                            cx.notify();
                                        },
                                    )),
                            ),
                    )
                    .into_any_element(),
            );
        }
        self.menu.map(|i| {
            let left = 100.0
                + MENUS
                    .iter()
                    .take(i)
                    .map(|(n, _)| n.len() as f32 * 7.0 + 23.0)
                    .sum::<f32>();
            div()
                .absolute()
                .top(px(MENUBAR_HEIGHT))
                .left(px(left))
                .child(
                    self.floating().w(px(310.0)).children(
                        MENUS[i]
                            .1
                            .iter()
                            .enumerate()
                            .map(|(j, op)| self.action_row(*op, j, j == self.menu_index, cx)),
                    ),
                )
                .into_any_element()
        })
    }
}
impl Render for EditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        #[cfg(target_family = "wasm")]
        crate::web::set_unsaved(self.editor.engine.is_dirty());
        for comment in self
            .editor
            .snapshot()
            .comments
            .iter()
            .chain(self.editor.display_snapshot().comments.iter())
        {
            let next = self.discussion.numbers.len() + 1;
            self.discussion.numbers.entry(comment.id).or_insert(next);
        }
        for image in self.agent.retired.drain(..) {
            let _ = window.drop_image(image);
        }
        if self.image.as_ref().map(|i| i.id) != self.painted_image.as_ref().map(|i| i.id) {
            if let Some(previous) = self.painted_image.take() {
                let _ = window.drop_image(previous);
            }
            self.painted_image = self.image.clone();
        }
        self.palettes
            .name
            .update(cx, |f, cx| f.appearance(self.theme, cx));
        self.palettes
            .hex
            .update(cx, |f, cx| f.appearance(self.theme, cx));
        self.discussion
            .text
            .update(cx, |f, cx| f.appearance(self.theme, cx));
        self.timeline
            .name
            .update(cx, |f, cx| f.appearance(self.theme, cx));
        self.sharing
            .person
            .update(cx, |field, cx| field.appearance(self.theme, cx));
        let size = window.viewport_size();
        let width = (f32::from(size.width) - SIDEBAR_WIDTH - TOOL_RAIL_WIDTH).max(1.0);
        let height =
            (f32::from(size.height) - MENUBAR_HEIGHT - TOOLBAR_HEIGHT - STATUSBAR_HEIGHT).max(1.0);
        let scale = (1024.0 / width.max(height)).min(1.0);
        let dimensions = [
            (width * scale).round() as u32,
            (height * scale).round() as u32,
        ];
        if self.render_size != dimensions {
            if self.camera_fitted {
                self.editor.camera = OrbitCamera::fit_view(
                    self.editor.display_snapshot(),
                    f64::from(dimensions[0]) / f64::from(dimensions[1].max(1)),
                    match self.camera_view {
                        Some(Op::Front) => PresetView::Front,
                        Some(Op::Right) => PresetView::Right,
                        Some(Op::Top) => PresetView::Top,
                        _ => PresetView::Iso,
                    },
                );
                if self.image.is_none() {
                    self.display_camera = self.editor.camera;
                }
            }
            self.render_size = dimensions;
            self.request_render(cx);
        }
        let title = format!(
            "{}{} — Voxelite",
            self.editor.display_snapshot().name,
            if self.editor.engine.is_dirty() {
                " •"
            } else {
                ""
            }
        );
        if title != self.last_title {
            window.set_window_title(&title);
            self.last_title = title;
        }
        if cfg!(target_os = "macos") && window.is_window_active() {
            cx.set_menus(crate::actions::menus(
                |op| self.selected(op),
                |op| self.enabled(op),
            ));
        }
        #[cfg(target_family = "wasm")]
        let settings_open = self.settings_modal.is_some();
        #[cfg(not(target_family = "wasm"))]
        let settings_open = false;
        let overlay = self.overlay(cx);
        div()
            .id("editor-root")
            .key_context(if settings_open {
                "Settings"
            } else if self.palettes.name.read(cx).focused(window)
                || self.palettes.hex.read(cx).focused(window)
                || self.discussion.text.read(cx).focused(window)
                || self.timeline.name.read(cx).focused(window)
                || self.sharing.person.read(cx).focused(window)
            {
                "TextField"
            } else if self.palette_focus.is_focused(window) {
                "Palette"
            } else if self.command_open {
                "CommandPalette"
            } else if self.menu.is_some()
                || self.palette_open
                || self.pending.is_some()
                || self.info.is_some()
                || self.dialog_open
                || self.palettes.open
                || self.sharing.open
                || self.agent.open
            {
                "Modal"
            } else {
                "Editor"
            })
            .track_focus(&self.focus)
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(self.theme.background())
            .text_color(self.theme.text())
            .font(gpui::font("Geist"))
            .text_size(px(13.0))
            .capture_key_down(cx.listener(Self::key_down))
            .on_action(cx.listener(|this, op: &Op, window, cx| {
                if matches!(
                    op,
                    Op::Settings | Op::Quit | Op::Hide | Op::HideOthers | Op::ShowAll
                ) || (this.info.is_none() && this.pending.is_none() && !this.palettes.open)
                {
                    this.invoke(*op, window, cx);
                }
            }))
            .when(!cfg!(target_os = "macos"), |d| d.child(self.menubar(cx)))
            .child(self.toolbar(f32::from(size.width) < 1200.0, cx))
            .child(
                row()
                    .flex_1()
                    .min_h(px(0.0))
                    .items_stretch()
                    .child(self.tool_rail(cx))
                    .child(self.viewport(cx))
                    .child(self.sidebar(f32::from(size.height) < 760.0, cx)),
            )
            .child(self.statusbar())
            .children(overlay.map(|overlay| {
                div()
                    .id("overlay")
                    .absolute()
                    .size_full()
                    .occlude()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            window.focus(&this.focus, cx);
                            #[cfg(target_family = "wasm")]
                            {
                                this.settings_modal = None;
                            }
                            this.agent.open = false;
                            this.sharing.open = false;
                            this.palettes.open = false;
                            this.menu = None;
                            this.palette_open = false;
                            this.command_open = false;
                            this.info = None;
                            this.pending = None;
                            this.drag = None;
                            cx.stop_propagation();
                            cx.notify();
                        }),
                    )
                    .child(overlay)
            }))
    }
}
