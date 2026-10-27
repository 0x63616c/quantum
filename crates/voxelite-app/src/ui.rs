//! Native GPUI shell. Portable editing logic lives in the library.
mod about;
#[cfg(not(target_family = "wasm"))]
mod agent;
#[cfg(target_family = "wasm")]
#[path = "ui/agent_web.rs"]
mod agent;
#[cfg(target_family = "wasm")]
mod browser;
mod camera_anim;
#[cfg(target_os = "linux")]
mod capture;
mod command_palette;
mod comments;
mod history;
mod keyboard_guide;
mod viewport;
use keyboard_guide::keyboard_guide;
mod overlays;
use comments::SidebarTab;
mod palette_layout;
mod palettes;
mod save_flow;
mod settings;
mod shortcut_hint;
mod tooltip;
#[cfg(not(target_family = "wasm"))]
pub(crate) mod welcome;
use tooltip::{AnchoredTooltip, TooltipSide};
mod status;
use crate::{
    actions::{CommandContext, EditorState, MENUS, Op},
    shortcuts::{ShortcutPlatform, navigation_key},
    theme::Theme,
};
use glam::DVec3;
use gpui::{prelude::*, *};
use shortcut_hint::{HintText, action_shortcut, shortcut};
use std::{cell::Cell, path::PathBuf, rc::Rc, sync::Arc};
use voxelite_app::editor::{Editor, EditorResult, OrbitCamera, Tool};
use voxelite_core::{Palette, Pos};
use voxelite_plugin_api::{PaletteAction, PalettePresentation};
use voxelite_render::{PresetView, RenderOptions, View};
use voxelite_ui::tokens::{focus_ring, metrics, radius, space, type_size};
use voxelite_ui::{Button, ButtonVariant, Dialog, Toolbar, ToolbarSeparator, icon, mono, row};

// Render sizing and native layout use the same shared logical-pixel metrics.
use voxelite_ui::tokens::metrics::{
    MENUBAR_HEIGHT, PANEL_GUTTER, RAIL_WIDTH as TOOL_RAIL_WIDTH, SIDEBAR_WIDTH,
    STATUS_HEIGHT as STATUSBAR_HEIGHT, TOOLBAR_HEIGHT,
};
use voxelite_ui::{IconButton, Panel, Popover, Segment, SegmentedControl, Tab, TabBar};

pub(crate) fn updates_available() -> bool {
    #[cfg(target_os = "macos")]
    {
        crate::updater::is_available()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

// Both windows use Sparkle's own progress and result dialogs when available.
fn check_for_updates() {
    #[cfg(target_os = "macos")]
    if updates_available() {
        crate::updater::check_for_updates();
    }
}

pub struct EditorView {
    palettes: palettes::PaletteManager,
    discussion: comments::Discussion,
    panel: SidebarTab,
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
    modal_focus: FocusHandle,
    sidebar_scroll: ScrollHandle,
    focus: FocusHandle,
    viewport: viewport::Viewport,
    last_title: String,
    last_edited: Option<bool>,
    zoom_reference_span: f64,
    camera_view: Option<Op>,
    camera_fitted: bool,
    camera_anim: voxelite_render::CameraAnimator,
    anim_epoch: web_time::Instant,
    hover: Option<Pos>,
    cursor: Pos,
    drag: Option<Point<Pixels>>,
    overlay_drag: Option<(String, voxelite_plugin_api::OverlayDrag)>,
    overlay_hover: Option<(String, u32)>,
    overlay_bounds: Rc<std::cell::RefCell<std::collections::BTreeMap<String, Bounds<Pixels>>>>,
    menu: Option<usize>,
    menu_index: usize,
    commands: command_palette::CommandPalette,
    command_input: Entity<crate::text_field::TextField>,
    palette_open: bool,
    palette_cursor: usize,
    palette_scroll: ScrollHandle,
    swatch_scroll: ScrollHandle,
    save_flow: save_flow::SaveFlow,
    info: Option<(&'static str, InfoContent)>,
    status: HintText,
    error: bool,
}
struct Hint {
    text: HintText,
    theme: Theme,
}
impl Render for Hint {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        voxelite_ui::tooltip(self.theme).child(self.text.clone())
    }
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

enum InfoContent {
    About,
    Acknowledgements,
    Text(String),
    Guide,
}

fn section_title(text: &'static str) -> Div {
    Panel::title(text)
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
    Theme::contrast_ink(luminance > 0.179)
}
impl EditorView {
    pub fn new(
        mut editor: Editor,
        override_theme: Option<Theme>,
        socket: Option<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        settings::init(override_theme, cx);
        #[cfg(not(target_family = "wasm"))]
        {
            welcome::record(&editor, cx);
            welcome::observe_editor(cx);
        }
        let preferences = cx.global::<settings::SettingsState>().values.clone();
        let theme = settings::theme(preferences.appearance, window);
        editor
            .set_grid(preferences.grid)
            .expect("declared boolean grid setting");
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
                match this
                    .save_flow
                    .request_close(|| this.editor.engine.is_dirty())
                {
                    save_flow::CloseRequest::Allow => true,
                    save_flow::CloseRequest::Blocked => false,
                    save_flow::CloseRequest::Confirm => {
                        cx.notify();
                        false
                    }
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
        let command_input = crate::text_field::TextField::new("Type a command…", cx);
        command_input.update(cx, |field, _| field.embed());
        cx.subscribe_in(
            &command_input,
            window,
            |this, field, event, window, cx| match event {
                voxelite_ui::TextFieldEvent::Changed => {
                    this.commands.set_query(field.read(cx).text());
                    cx.notify();
                }
                voxelite_ui::TextFieldEvent::Committed => {
                    this.command_palette_key(
                        &Keystroke::parse("enter").expect("constant keystroke"),
                        window,
                        cx,
                    );
                }
                _ => {}
            },
        )
        .detach();
        let palettes = palettes::PaletteManager::new(window, cx);
        let library_error = editor
            .load_palette_library()
            .err()
            .map(|error| error.to_string());
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
        #[cfg(target_os = "macos")]
        Self::monitor_gpu(cx);
        Self {
            agent,
            palettes,
            discussion: comments::Discussion::new(window, cx),
            panel: SidebarTab::Session,
            timeline: history::Timeline::new(window, cx),
            cursor: editor.snapshot().bounds.min,
            camera_view: Some(Op::Frame),
            camera_fitted: true,
            camera_anim: voxelite_render::CameraAnimator::new(),
            anim_epoch: web_time::Instant::now(),
            zoom_reference_span: editor.camera.span,
            viewport: viewport::Viewport::new(editor.camera),
            editor,
            preferences,
            #[cfg(not(target_family = "wasm"))]
            settings_window: None,
            #[cfg(target_family = "wasm")]
            settings_modal: None,
            palette_focus: cx.focus_handle(),
            modal_focus: cx.focus_handle(),
            sidebar_scroll: ScrollHandle::new(),
            theme,
            focus,
            last_title: String::new(),
            last_edited: None,
            hover: None,
            drag: None,
            overlay_drag: None,
            overlay_hover: None,
            overlay_bounds: Rc::default(),
            menu: None,
            menu_index: 0,
            commands: command_palette::CommandPalette::default(),
            command_input,
            palette_open: false,
            palette_cursor: 0,
            palette_scroll: ScrollHandle::new(),
            swatch_scroll: ScrollHandle::new(),
            save_flow: save_flow::SaveFlow::default(),
            info: None,
            status: library_error
                .clone()
                .or(agent_error.clone())
                .unwrap_or_default()
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
        if !self.viewport.is_current(&self.editor) {
            self.request_render(cx);
        }
        cx.notify();
    }
    fn replace_editor(&mut self, mut editor: Editor, cx: &mut Context<Self>) {
        self.cancel_camera_animation();
        #[cfg(not(target_family = "wasm"))]
        {
            welcome::record(&editor, cx);
            welcome::project_ready(cx);
        }
        self.pause_history(cx);
        self.timeline.dragging = false;
        self.timeline.expanded = false;
        self.zoom_reference_span = editor.camera.span;
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
        editor.comments_state.region = self.editor.comments_state.region;
        editor.comments_state.filter = self.editor.comments_state.filter;

        editor.retain_palette_library(&self.editor);
        self.editor = editor;
        self.editor
            .set_grid(self.preferences.grid)
            .expect("declared boolean grid setting");
        self.overlay_drag = None;
        self.overlay_hover = None;
        self.overlay_bounds.borrow_mut().clear();
        self.hover = None;
        self.drag = None;
        self.cursor = self.editor.display_snapshot().bounds.min;
        self.camera_view = Some(Op::Frame);
        self.camera_fitted = true;
        self.viewport.replace_editor(&mut self.editor);
        self.error = false;
        self.request_render(cx);
    }
    fn active_palette(&self) -> Palette {
        self.editor
            .palette_view()
            .map(|v| v.palette)
            .unwrap_or_default()
    }
    fn availability(&self, op: Op) -> Result<(), &'static str> {
        op.availability_in(
            &self.editor.plugin_host(),
            CommandContext::Editor(EditorState {
                dialog_open: self.save_flow.dialog_open(),
                pending_confirmation: self.save_flow.needs_confirmation(),
                modal_open: self.info.is_some() || self.palettes.open,
                preview: self.editor.preview.is_some(),
                history_playing: self.editor.history_playing(),
                can_undo: self.editor.engine.head().is_some(),
                can_redo: self.editor.engine.redo_head().is_some(),
                updates_available: updates_available(),
            }),
        )
    }
    fn enabled(&self, op: Op) -> bool {
        self.availability(op).is_ok()
    }
    fn disabled_reason(&self, op: Op) -> &'static str {
        self.availability(op).err().unwrap_or_default()
    }
    fn selected(&self, op: Op) -> bool {
        match op {
            Op::Place => self.editor.tool == Tool::Place,
            Op::Erase => self.editor.tool == Tool::Erase,
            Op::Paint => self.editor.tool == Tool::Paint,
            Op::Fill => self.editor.tool == Tool::Fill,
            Op::Box => self.editor.tool == Tool::Box,
            Op::Picker => self.editor.tool == Tool::Picker,
            Op::Grid => self.editor.grid(),
            Op::Frame | Op::Front | Op::Right | Op::Top => self.camera_view == Some(op),
            Op::MirrorX => self.editor.mirror_axes()[0],
            Op::MirrorY => self.editor.mirror_axes()[1],
            Op::MirrorZ => self.editor.mirror_axes()[2],
            _ => false,
        }
    }
    fn invoke(&mut self, op: Op, window: &mut Window, cx: &mut Context<Self>) {
        let enabled = self.enabled(op);
        if matches!(
            op,
            Op::Undo
                | Op::Redo
                | Op::Place
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
                | Op::New
                | Op::Open
                | Op::Quit
        ) {
            self.pause_history(cx);
        }
        if !enabled {
            return;
        }
        self.menu = None;
        self.commands.close();
        self.palette_open = false;
        if self
            .save_flow
            .confirm_if_dirty(op, || self.editor.engine.is_dirty())
        {
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
            Op::FullScreen => {
                #[cfg(not(target_family = "wasm"))]
                window.toggle_fullscreen();
                cx.notify();
            }
            Op::ShowPalette => {
                if !self
                    .palette_request(PaletteAction::Command(op.command_id().into()), cx)
                    .is_some_and(|out| out.presentation == Some(PalettePresentation::Focus))
                {
                    return;
                }
                self.panel = SidebarTab::Session;
                self.sidebar_scroll.scroll_to_bottom();
                window.focus(&self.palette_focus, cx);
                self.status = "Palette focused · Arrow keys choose a brush color · Escape returns to the viewport".into();
                cx.notify();
                return;
            }
            _ => self.run(op, cx),
        }
        if self.commands.is_open() {
            self.command_input
                .update(cx, |field, cx| field.focus(window, cx));
            return;
        }
        window.focus(
            if self.info.is_some() || self.palettes.open {
                &self.modal_focus
            } else {
                &self.focus
            },
            cx,
        );
    }
    fn run(&mut self, op: Op, cx: &mut Context<Self>) {
        match op {
            Op::Settings | Op::Theme | Op::Minimize | Op::ZoomWindow | Op::FullScreen | Op::ShowPalette => {},
            Op::About => self.info = Some(("About Voxelite", InfoContent::About)),
            Op::CheckUpdates => check_for_updates(),
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
            #[cfg(not(target_family = "wasm"))]
            Op::OpenRecent | Op::Welcome => {
                cx.defer(|cx| { if let Err(error) = welcome::show(cx) { eprintln!("Could not open welcome window: {error}"); } });
            }
            #[cfg(not(target_family = "wasm"))]
            Op::ClearRecents => welcome::clear(cx),
            Op::Save => self.save(false, cx),
            Op::SaveAs => self.save(true, cx),
            #[cfg(target_family = "wasm")]
            Op::ExportGlb => self.export_browser(false, cx),
            #[cfg(target_family = "wasm")]
            Op::ExportPng => self.export_browser(true, cx),
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
                let b = self.viewport.bounds();
                let aspect = (f32::from(b.size.width) / f32::from(b.size.height).max(1.0)) as f64;
                let view = match op {
                    Op::Front => PresetView::Front,
                    Op::Right => PresetView::Right,
                    Op::Top => PresetView::Top,
                    _ => PresetView::Iso,
                };
                let destination =
                    OrbitCamera::fit_view(self.editor.display_snapshot(), aspect, view);
                self.zoom_reference_span = destination.span;
                self.camera_view = Some(op);
                self.camera_fitted = true;
                self.animate_camera_to(destination, cx);
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
                if let Err(error) = self.editor.set_mirror_axis(a, !self.editor.mirror_axes()[a]) {
                    self.status = error.to_string().into();
                    self.error = true;
                }
            }
            Op::AddLayer => {
                let r = self.editor.add_layer();
                self.report(r, "Layer added", cx);
            }
            Op::ManagePalettes => self.manage_palettes(cx),
            Op::Palette => {
                if self.palette_request(PaletteAction::Command(op.command_id().into()), cx)
                    .is_some_and(|out| out.presentation == Some(PalettePresentation::Choose)) {
                    self.palette_open = true;
                    self.palette_cursor = self.palette_index();
                }
            }
            Op::Commands | Op::ToolPicker => {
                self.commands.open(op == Op::ToolPicker);
                self.command_input.update(cx, |field, cx| {
                    field.placeholder(
                        if self.commands.tools_only() { "Search tools…" } else { "Type a command…" },
                        cx,
                    );
                    field.set(String::new(), cx);
                });
            }
            Op::Agent => {
                self.agent.open = true;
            }
            Op::Extensions => {
                self.info = Some((
                    "Extensions",
                    InfoContent::Text("Place, Erase, Paint, Fill, Box and Picker are included. Custom extensions are not available in this build.".into()),
                ))
            }
            Op::Help => {
                self.info = Some((
                    "Make yourself at home",
                    InfoContent::Guide,
                ))
            }
        }
        cx.notify();
    }
    fn pointer_move(&mut self, e: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.overlay_drag.is_some() {
            return;
        }
        if self.commands.is_open()
            || self.menu.is_some()
            || self.palette_open
            || self.info.is_some()
            || self.save_flow.needs_confirmation()
            || self.save_flow.dialog_open()
        {
            return;
        }
        if matches!(
            e.pressed_button,
            Some(MouseButton::Right | MouseButton::Middle)
        ) {
            if let Some(previous) = self.drag {
                self.cancel_camera_animation();
                let dx = f64::from(f32::from(e.position.x - previous.x));
                let dy = f64::from(f32::from(e.position.y - previous.y));
                if e.pressed_button == Some(MouseButton::Middle) || e.modifiers.shift {
                    self.editor.camera.pan(
                        dx,
                        dy,
                        f64::from(f32::from(self.viewport.bounds().size.height)),
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
            self.hover = self
                .viewport
                .ray(&self.editor, e.position)
                .and_then(|r| self.editor.target(r));
        }
        cx.notify();
    }
    fn key_down(&mut self, e: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        #[cfg(target_os = "linux")]
        if self.capture_status(e, cx) {
            return;
        }
        self.reconcile_palette_panel(window, cx);
        #[cfg(target_family = "wasm")]
        if self.settings_modal.is_some() {
            return;
        }
        if self.save_flow.needs_confirmation() {
            self.pending_key(&e.keystroke, cx);
            return;
        }
        // The palette dialog owns cancellation, text commits and grid navigation.
        // Its descendants must receive these keys before viewport shortcuts.
        if self.palettes.open && !self.save_flow.dialog_open() {
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
                if let Some(next) = self
                    .palette_request(PaletteAction::StepSwatch(delta), cx)
                    .and_then(|out| out.selected)
                {
                    self.swatch_scroll.set_offset(point(
                        px(space::NONE),
                        -px((next / 8) as f32 * metrics::SWATCH_PITCH),
                    ));
                    cx.notify();
                }
                cx.stop_propagation();
                return;
            }
            if !matches!(e.keystroke.key.as_str(), "escape" | "tab") {
                // Choosing a brush color must never apply the viewport tool.
                return;
            }
        }
        if self.save_flow.dialog_open() {
            cx.stop_propagation();
            return;
        }
        let key = navigation_key(&e.keystroke).unwrap_or_default();
        // Shared editors own their keys and native text input, including fields
        // created inside controlled NumericField elements.
        if crate::text_field::focused(window) {
            return;
        }

        if key == "tab" && !self.save_flow.needs_confirmation() && self.menu.is_none() {
            if self.palettes.open
                || self.agent.open
                || self.info.is_some()
                || self.commands.is_open()
            {
                Dialog::trap_tab(&self.modal_focus, e.keystroke.modifiers.shift, window, cx);
            } else if e.keystroke.modifiers.shift {
                window.focus_prev(cx);
            } else {
                window.focus_next(cx);
            }
            cx.stop_propagation();
            return;
        }
        if key != "escape"
            && (self.palettes.field_focused(window, cx)
                || self.discussion.text.read(cx).focused(window)
                || self.timeline.name.read(cx).focused(window))
        {
            return;
        }
        if !self.focus.is_focused(window)
            && !self.commands.is_open()
            && matches!(key, "enter" | "space")
        {
            return;
        }
        if key == "escape" {
            self.discussion_action(voxelite_plugin_api::CommentsAction::Cancel, cx);
            if self.editor.preview.is_some() {
                self.editor.return_live();
                self.refresh_preview(cx);
            }
            self.agent.open = false;

            self.palettes.open = false;
            window.focus(&self.focus, cx);
            self.menu = None;
            self.commands.close();
            self.palette_open = false;
            self.info = None;
            self.save_flow.cancel_confirmation();
            self.editor.box_start = None;
            self.drag = None;
            self.overlay_drag = None;
            self.overlay_hover = None;
            self.hover = None;
            self.status = HintText::default();
            self.error = false;
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if self.palettes.open || self.agent.open {
            cx.stop_propagation();
            return;
        }
        if self.palette_open {
            let choices = self.palette_choices();
            if choices.is_empty() {
                self.palette_open = false;
                return;
            }
            match key {
                "down" => self.palette_cursor = (self.palette_cursor + 1) % choices.len(),
                "up" => {
                    self.palette_cursor = (self.palette_cursor + choices.len() - 1) % choices.len()
                }
                "enter" => {
                    self.palette_request(PaletteAction::ChoosePreset(self.palette_cursor), cx);
                    self.palette_open = false;
                }
                _ => {}
            }
            self.palette_scroll.scroll_to_item(self.palette_cursor);
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if self.commands.is_open() {
            self.command_palette_key(&e.keystroke, window, cx);
            return;
        }
        if let Some(menu) = self.menu {
            let items = crate::actions::menu_items(menu, &self.editor.plugin_host());
            self.menu_index = self.menu_index.min(items.len().saturating_sub(1));
            match key {
                "down" if !items.is_empty() => {
                    self.menu_index = (self.menu_index + 1) % items.len()
                }
                "up" if !items.is_empty() => {
                    self.menu_index = (self.menu_index + items.len() - 1) % items.len()
                }
                "right" => {
                    self.menu = Some((menu + 1) % MENUS.len());
                    self.menu_index = 0;
                }
                "left" => {
                    self.menu = Some((menu + MENUS.len() - 1) % MENUS.len());
                    self.menu_index = 0;
                }
                "enter" if !items.is_empty() => self.invoke(items[self.menu_index], window, cx),
                _ => {}
            }
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if self.info.is_some() || self.save_flow.needs_confirmation() || self.palette_open {
            cx.stop_propagation();
            return;
        }
        if self.panel == SidebarTab::History && key == "space" {
            if !e.is_held {
                self.toggle_history(cx);
            }
            cx.stop_propagation();
            return;
        }
        if self.panel == SidebarTab::History && matches!(key, "left" | "right") {
            let count = self.editor.timeline_entries().count();
            let index = self.editor.timeline_index();
            self.scrub(
                if key == "left" {
                    index.saturating_sub(1)
                } else {
                    (index + 1).min(count.saturating_sub(1))
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
        if key == "enter" && self.editor.comments_state.placing {
            self.anchor_comment(self.cursor, cx);
            return;
        }
        if key == "enter" {
            let r = self.editor.apply_at(self.cursor);
            self.report(r, "Tool applied", cx);
            cx.stop_propagation();
        }
        if key == "[" || key == "]" {
            self.palette_request(
                PaletteAction::StepSwatch(if key == "]" { 1 } else { -1 }),
                cx,
            );
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
        let variant = if primary {
            ButtonVariant::Primary
        } else if label.is_empty() {
            ButtonVariant::Icon
        } else {
            ButtonVariant::Ghost
        };
        let mut button = Button::new(id, if label.is_empty() { op.label() } else { label }, theme)
            .variant(variant)
            .disabled(!enabled)
            .selected(selected);
        if !name.is_empty() {
            button = button.icon(name);
        }
        let element = if id.starts_with("rail-") {
            let mut rail = IconButton::rail(id, op.icon(), op.label(), theme)
                .disabled(!enabled)
                .selected(selected);
            if !label.is_empty() {
                rail = rail.badge(label);
            }
            rail.build()
        } else if matches!(id, "view-iso" | "view-front" | "view-right" | "view-top") {
            Segment::new(id, label, theme)
                .icon(name)
                .disabled(!enabled)
                .selected(selected)
                .build()
        } else if label.is_empty() && id != "commands" {
            IconButton::new(id, name, op.label(), theme)
                .disabled(!enabled)
                .selected(selected)
                .build()
        } else {
            button.build()
        };
        element
            .aria_keyshortcuts(op.shortcut())
            .aria_label(op.label())
            .anchored_hint(
                if enabled {
                    HintText::from(op.label()).action(op)
                } else {
                    format!("{} · {}", op.label(), disabled_reason).into()
                },
                theme,
                if id.starts_with("rail-mirror-") {
                    TooltipSide::Mirror(match op {
                        Op::MirrorY => 1,
                        Op::MirrorZ => 2,
                        _ => 0,
                    })
                } else if id.starts_with("rail-") {
                    TooltipSide::Right
                } else if id.starts_with("view-") {
                    TooltipSide::Above
                } else {
                    TooltipSide::Below
                },
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                this.invoke(op, window, cx);
            }))
    }
    fn menubar(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        row()
            .h(px(MENUBAR_HEIGHT))
            .flex_shrink_0()
            .px(px(space::XXL))
            .border_b_1()
            .border_color(t.line())
            .gap(px(space::COMPACT))
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(px(type_size::TITLE))
                    .mr(px(space::SECTION))
                    .child("Voxelite"),
            )
            .children(MENUS.iter().enumerate().map(|(i, (name, _))| {
                Button::custom(("menu", i), *name)
                    .h(px(space::XXXL))
                    .px(px(space::INSET))
                    .rounded(px(radius::CONTROL))
                    .text_color(t.muted())
                    .cursor_pointer()
                    .child(*name)
                    .when(self.menu == Some(i), |d| d.bg(t.hover()))
                    .hover(move |d| d.bg(t.hover()))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.menu = if this.menu == Some(i) { None } else { Some(i) };
                        this.menu_index = 0;
                        this.commands.close();
                        cx.notify();
                    }))
            }))
            .child(div().flex_1())
    }
    /// Persistent tool and mirror controls use the same actions as menus and keys.
    fn tool_rail(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        Panel::rail(t)
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
                .map(|(id, op)| self.button(id, op.icon(), "", op, cx)),
            )
            .child(
                div()
                    .w(px(space::XXL))
                    .h(px(metrics::EDGE))
                    .my(px(space::SM))
                    .bg(t.line()),
            )
            .children(
                [
                    ("rail-mirror-x", "X", Op::MirrorX),
                    ("rail-mirror-y", "Y", Op::MirrorY),
                    ("rail-mirror-z", "Z", Op::MirrorZ),
                ]
                .into_iter()
                .map(|(id, label, op)| self.button(id, op.icon(), label, op, cx)),
            )
    }
    fn toolbar(&self, compact: bool, cx: &Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        Toolbar::row(t)
            .id("main-titlebar")
            .on_mouse_down(MouseButton::Left, |event, window, _| {
                if event.click_count == 2 {
                    window.zoom_window();
                } else {
                    window.start_window_move();
                }
            })
            .when(cfg!(target_os = "macos"), |d| {
                d.pl(px(metrics::TOOLBAR_TRAFFIC_INSET))
            })
            .child(icon("cube").text_color(t.muted()).size(px(metrics::KEYCAP)))
            .child(
                div()
                    .w(px(if compact {
                        metrics::DOCUMENT_COMPACT_WIDTH
                    } else {
                        metrics::DOCUMENT_WIDTH
                    }))
                    .min_w(px(space::NONE))
                    .flex_shrink_0()
                    .child(
                        row()
                            .gap(px(space::GLYPH))
                            .text_size(px(type_size::DOCUMENT))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(
                                div()
                                    .min_w(px(space::NONE))
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(format!("{}.voxl", self.editor.display_snapshot().name)),
                            )
                            .when(self.editor.engine.is_dirty(), |d| {
                                d.child(
                                    div()
                                        .flex_shrink_0()
                                        .text_size(px(type_size::BODY_LG))
                                        .font_weight(FontWeight::NORMAL)
                                        .text_color(t.muted())
                                        .child("— Edited"),
                                )
                            }),
                    ),
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
                .h(px(metrics::RAIL_BUTTON))
                .gap(px(space::LG))
                .child(action_shortcut(Op::Commands)),
            )
    }
    fn viewport(&self, cx: &Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        let surface = self.viewport_surface(cx);
        div()
            .id("viewport")
            .relative()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .cursor_crosshair()
            .child(surface)
            .child(
                div()
                    .absolute()
                    .top(px(space::PANEL))
                    .left(px(space::XXL))
                    .child(row().gap(px(space::INSET)).text_color(t.muted()).child(
                        match self.camera_view {
                            Some(Op::Front) => "Front",
                            Some(Op::Right) => "Right",
                            Some(Op::Top) => "Top",
                            Some(Op::Frame) => "Isometric",
                            _ => "Orthographic",
                        },
                    )),
            )
            .children(self.comment_pins(cx))
            .when(self.editor.preview.is_some(), |d| {
                d.child(
                    row()
                        .id("preview-banner")
                        .absolute()
                        .top(px(metrics::COMMAND_SEARCH_HEIGHT))
                        .left(px(space::XXL))
                        .px(px(space::LG))
                        .py(px(space::SM))
                        .rounded(px(radius::CONTROL))
                        .bg(t.selected())
                        .text_color(t.accent())
                        .gap(px(space::MD))
                        .child(icon("lock").text_color(t.accent()))
                        .child("Read-only preview")
                        .child(
                            self.control(
                                "preview-exit",
                                "log-out",
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
            .children(self.viewport_widgets(cx))
            .child(
                row()
                    .absolute()
                    .bottom(px(space::PANEL))
                    .left(px(space::XXL))
                    .gap(px(space::XS))
                    .child(self.button("view-settings", "settings", "", Op::Settings, cx))
                    .when(self.preferences.navigation_hints, |d| {
                        d.child(self.button("view-help", "keyboard", "", Op::Help, cx))
                    }),
            )
            .child(
                div()
                    .absolute()
                    .bottom(px(space::XL))
                    .right(px(space::XXL))
                    .child(
                        SegmentedControl::row(t)
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
                            .child(ToolbarSeparator::line(t).mx(px(space::XS)))
                            .child(self.button("view-fit", "fit", "", Op::Frame, cx)),
                    ),
            )
            .on_mouse_move(cx.listener(Self::pointer_move))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, e: &MouseDownEvent, _, cx| {
                    if this.save_flow.dialog_open() {
                        return;
                    }
                    this.drag = Some(e.position);
                    cx.stop_propagation();
                }),
            )
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(|this, e: &MouseDownEvent, _, cx| {
                    if this.save_flow.dialog_open() {
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
                    if this.commands.is_open()
                        || this.info.is_some()
                        || this.save_flow.needs_confirmation()
                        || this.save_flow.dialog_open()
                    {
                        return;
                    }
                    if this.editor.preview.is_some() {
                        this.pause_history(cx);
                        this.status =
                            "History preview is read-only · Return to live to edit".into();
                        cx.notify();
                        return;
                    }
                    if let Some(ray) = this.viewport.ray(&this.editor, e.position) {
                        if this.editor.comments_state.placing && this.editor.comments_enabled() {
                            this.place_comment(
                                Some(ray),
                                voxelite_plugin_api::ToolGesture::Click,
                                cx,
                            );
                            return;
                        }
                        let r = this.editor.click(ray, e.modifiers.alt);
                        let status = if this.editor.box_start.is_some() {
                            HintText::from("First corner set · click opposite corner · ")
                                .key("escape")
                                .text(" cancels")
                        } else {
                            HintText::default()
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
                if this.save_flow.dialog_open()
                    || this.commands.is_open()
                    || this.palette_open
                    || this.menu.is_some()
                    || this.info.is_some()
                    || this.save_flow.needs_confirmation()
                {
                    return;
                }
                let delta = match e.delta {
                    ScrollDelta::Pixels(p) => f64::from(f32::from(p.y)),
                    ScrollDelta::Lines(p) => f64::from(p.y) * 40.0,
                };
                this.cancel_camera_animation();
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
        let color = self.editor.active_color().rgba;
        // Eight 32px swatches, 6px gaps and 2px focus-ring padding per row.
        let visible_rows = if compact { 2 } else { 5 };
        let swatch_height = visible_rows as f32 * metrics::SWATCH_PITCH - space::XXS;
        let overflowing = palette.order.len() > visible_rows * 8;
        let hex = format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
        Panel::section(t)
            .flex_shrink_0()
            .px(px(PANEL_GUTTER))
            .pt(px(if compact { space::MD } else { space::XL }))
            .pb(px(if compact { space::MD } else { space::PANEL }))
            .child(
                row()
                    .justify_between()
                    .mb(px(if compact { space::MD } else { space::INSET }))
                    .child(
                        Button::custom("palette-switch", "Choose palette")
                            .tab_index(0)
                            .focus_visible(move |d| d.bg(t.hover()).shadow(focus_ring(t)))
                            .active(move |d| d.bg(t.selected()))
                            .h(px(space::XXXL))
                            .gap(px(space::MD))
                            .rounded(px(radius::CONTROL))
                            .cursor_pointer()
                            .tooltip(hint("Choose a project or preset palette", t))
                            .hover(move |d| d.bg(t.hover()))
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(self.palette_name()),
                            )
                            .child(icon("chevron").size(px(space::FOOTER)).text_color(t.text()))
                            .on_click(cx.listener(|this, _, _, cx| {
                                if this.palette_open {
                                    this.palette_open = false;
                                } else {
                                    this.run(Op::Palette, cx);
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .text_size(px(type_size::BODY))
                            .text_color(t.text())
                            .child(format!("{} colors", palette.order.len())),
                    ),
            )
            .child(
                div()
                    .id("swatch-scroll")
                    .track_focus(&self.palette_focus)
                    .focus(move |d| d.bg(t.accent().opacity(0.22)))
                    .track_scroll(&self.swatch_scroll)
                    .max_h(px(swatch_height))
                    .overflow_y_scroll()
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(space::SM))
                            .p(px(space::XXS))
                            .children(palette.order.iter().enumerate().map(|(i, slot)| {
                                let swatch = palette.get(*slot).expect("valid palette").clone();
                                let c = swatch.rgba;
                                let selected = c == color;
                                let hex = format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2]);
                                let label = swatch
                                    .name
                                    .as_ref()
                                    .map(|name| format!("{name} · {hex}"))
                                    .unwrap_or(hex);
                                voxelite_ui::Swatch::canvas(
                                    ("swatch", i),
                                    t,
                                    color_fill(c),
                                    selected,
                                )
                                .role(Role::Button)
                                .aria_label(label.clone())
                                .tooltip(hint(label, t))
                                .when(selected, |d| {
                                    d.child(
                                        icon("check").size(px(space::XL)).text_color(swatch_ink(c)),
                                    )
                                })
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.palette_request(PaletteAction::SelectSwatch(i), cx);
                                        this.error = false;
                                        cx.notify();
                                    },
                                ))
                            })),
                    ),
            )
            .child(
                row()
                    .mt(px(if compact { space::MD } else { space::LG }))
                    .gap(px(space::MD))
                    .text_size(px(type_size::BODY))
                    .text_color(t.text())
                    .child(hex)
                    .child(div().flex_1())
                    .when(!overflowing, |d| d.child("Brush color"))
                    .when(overflowing, |d| {
                        d.child("Scroll colors")
                            .child(
                                self.control("colors-up", "arrow-up", "", cx, |this, _, cx| {
                                    let offset = this.swatch_scroll.offset();
                                    this.swatch_scroll.set_offset(point(
                                        px(space::NONE),
                                        offset.y + px(metrics::PALETTE_SCROLL_STEP),
                                    ));
                                    cx.notify();
                                })
                                .size(px(space::XXL))
                                .px(px(space::NONE))
                                .justify_center()
                                .border_0()
                                .tooltip(hint("Scroll to previous colors", t)),
                            )
                            .child(
                                self.control("colors-down", "chevron", "", cx, |this, _, cx| {
                                    let offset = this.swatch_scroll.offset();
                                    this.swatch_scroll.set_offset(point(
                                        px(space::NONE),
                                        offset.y - px(metrics::PALETTE_SCROLL_STEP),
                                    ));
                                    cx.notify();
                                })
                                .size(px(space::XXL))
                                .px(px(space::NONE))
                                .justify_center()
                                .border_0()
                                .tooltip(hint("Scroll to more colors", t)),
                            )
                    }),
            )
    }
    fn layers_panel(&self, compact: bool, cx: &Context<Self>) -> Div {
        let t = self.theme;
        Panel::section(t)
            .flex_1()
            .min_h(px(if compact {
                metrics::LAYERS_COMPACT_MIN_HEIGHT
            } else {
                metrics::COLOR_EDITOR_WIDTH
            }))
            .px(px(PANEL_GUTTER - space::MD))
            .py(px(if compact { space::MD } else { space::XL }))
            .border_t_1()
            .border_color(t.line())
            .child(
                row()
                    .pl(px(space::MD))
                    .mb(px(if compact { space::XS } else { space::MD }))
                    .justify_between()
                    .child(section_title("Layers"))
                    .child(self.button("add-layer", "plus", "", Op::AddLayer, cx)),
            )
            .child(
                div()
                    .id("layer-scroll")
                    .max_h(px(if compact {
                        metrics::LAYERS_COMPACT_SCROLL_HEIGHT
                    } else {
                        metrics::LAYERS_SCROLL_HEIGHT
                    }))
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
                                Button::custom(("layer", i), layer.name.clone())
                                    .tab_index(0)
                                    .focus_visible(move |d| d.bg(t.hover()).shadow(focus_ring(t)))
                                    .active(move |d| d.bg(t.selected()))
                                    .h(px(if compact {
                                        voxelite_ui::tokens::ControlSize::Md.height()
                                    } else {
                                        metrics::RAIL_BUTTON
                                    }))
                                    .pl(px(space::SM))
                                    .pr(px(space::MD))
                                    .gap(px(space::MD))
                                    .rounded(px(radius::CONTROL))
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
                                    .child(icon("layer").size(px(space::XL)).text_color(t.muted()))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(space::NONE))
                                            .overflow_hidden()
                                            .text_ellipsis()
                                            .child(layer.name.clone()),
                                    )
                                    .child(
                                        IconButton::custom(
                                            ("eye", i),
                                            if visible { "Hide layer" } else { "Show layer" },
                                            voxelite_ui::tokens::ControlSize::Md,
                                        )
                                        .tab_index(0)
                                        .focus_visible(move |d| {
                                            d.bg(t.hover()).shadow(focus_ring(t))
                                        })
                                        .active(move |d| d.bg(t.selected()))
                                        .w(px(space::XXXL))
                                        .h(px(space::XXXL))
                                        .justify_center()
                                        .rounded(px(radius::CONTROL))
                                        .tooltip(hint(
                                            if visible { "Hide layer" } else { "Show layer" },
                                            t,
                                        ))
                                        .hover(move |d| d.bg(t.selected()))
                                        .child(
                                            icon(if visible { "eye" } else { "eye-off" })
                                                .size(px(metrics::ICON_DENSE))
                                                .text_color(t.muted()),
                                        )
                                        .on_click(
                                            cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                let r = this.editor.update_layer(
                                                    id,
                                                    Some(!visible),
                                                    None,
                                                );
                                                this.report(r, "Visibility updated", cx);
                                            }),
                                        ),
                                    )
                                    .child(
                                        IconButton::custom(
                                            ("lock", i),
                                            if locked { "Unlock layer" } else { "Lock layer" },
                                            voxelite_ui::tokens::ControlSize::Md,
                                        )
                                        .tab_index(0)
                                        .focus_visible(move |d| {
                                            d.bg(t.hover()).shadow(focus_ring(t))
                                        })
                                        .active(move |d| d.bg(t.selected()))
                                        .w(px(space::XXXL))
                                        .h(px(space::XXXL))
                                        .justify_center()
                                        .rounded(px(radius::CONTROL))
                                        .tooltip(hint(
                                            if locked { "Unlock layer" } else { "Lock layer" },
                                            t,
                                        ))
                                        .when(locked, |d| d.bg(t.selected()))
                                        .hover(move |d| d.bg(t.selected()))
                                        .child(
                                            icon(if locked { "lock" } else { "unlock" })
                                                .size(px(metrics::ICON_DENSE))
                                                .text_color(if locked {
                                                    t.accent()
                                                } else {
                                                    t.muted()
                                                }),
                                        )
                                        .on_click(
                                            cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                let r = this.editor.update_layer(
                                                    id,
                                                    None,
                                                    Some(!locked),
                                                );
                                                this.report(r, "Lock updated", cx);
                                            }),
                                        ),
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
        Panel::section(t)
            .flex_1()
            .min_h(px(if compact {
                metrics::PALETTE_CHOOSER_TOP
            } else {
                metrics::HISTORY_MIN_HEIGHT
            }))
            .flex()
            .flex_col()
            .px(px(PANEL_GUTTER))
            .py(px(if compact { space::MD } else { space::XL }))
            .gap(px(space::XL))
            .child(
                row()
                    .justify_between()
                    .child(section_title("Recent changes"))
                    .child(
                        row()
                            .gap(px(space::XXS))
                            .child(self.button("history-undo", "undo", "", Op::Undo, cx))
                            .child(self.button("history-redo", "redo", "", Op::Redo, cx)),
                    ),
            )
            .child(
                div()
                    .id("history-scroll")
                    .min_h(px(space::NONE))
                    .flex_1()
                    .overflow_y_scroll()
                    .when(empty, |d| {
                        d.child(
                            row()
                                .items_start()
                                .gap(px(space::LG))
                                .py(px(space::XS))
                                .child(
                                    row()
                                        .size(px(space::XXXL))
                                        .flex_shrink_0()
                                        .justify_center()
                                        .rounded(px(radius::LG))
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
                                                .mt(px(space::XS))
                                                .text_size(px(type_size::BODY))
                                                .line_height(px(type_size::LINE_DENSE))
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
                                                .mt(px(space::MD))
                                                .px(px(space::NONE))
                                                .text_color(t.text())
                                                .child(
                                                    icon("arrow-right")
                                                        .size(px(space::LG))
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
                                    .py(px(space::INSET))
                                    .gap(px(space::LG))
                                    .items_start()
                                    .child(
                                        icon("check")
                                            .mt(px(space::XXS))
                                            .size(px(space::XL))
                                            .text_color(t.muted()),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(space::NONE))
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
                                                .mt(px(space::XS)),
                                            ),
                                    )
                            }),
                    ),
            )
    }
    fn sidebar(&self, compact: bool, cx: &Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        Panel::sidebar(t)
            .id("sidebar-scroll")
            .overflow_y_scroll()
            .track_scroll(&self.sidebar_scroll)
            .child(self.panel_tabs(cx))
            .child(match self.panel {
                SidebarTab::Comments => self.comments_panel(cx),
                SidebarTab::History => self.timeline_panel(cx),
                SidebarTab::Session => self.agent_feed(compact, cx),
            })
            .when(self.panel == SidebarTab::Session, |d| {
                d.child(self.layers_panel(compact || self.agent.capture.is_some(), cx))
                    .children({
                        let panels: Vec<_> = self
                            .editor
                            .plugin_host()
                            .registry()
                            .panels()
                            .map(|(_, p)| p.id.clone())
                            .collect();
                        panels
                            .into_iter()
                            .filter(|id| id == voxelite_plugin_palette::PANEL_ID)
                            .map(|_| {
                                self.palette_panel(compact || self.agent.capture.is_some(), cx)
                            })
                    })
            })
    }
    fn action_row(
        &self,
        op: Op,
        index: usize,
        selected: bool,
        window: &Window,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let t = self.theme;
        let enabled = self.enabled(op);
        voxelite_ui::SidebarItem::row(
            ("action-row", index),
            op.display_label(window.is_fullscreen()),
            op.icon(),
            false,
            t,
        )
        .tab_stop(false)
        .role(Role::MenuItem)
        .aria_selected(selected)
        .aria_keyshortcuts(op.shortcut())
        .when(selected, |d| d.bg(t.hover()).aria_active_descendant())
        .text_color(if enabled { t.text() } else { t.disabled() })
        .when(self.selected(op), |d| {
            d.child(
                icon("check")
                    .size(px(metrics::ICON_SM))
                    .text_color(t.accent()),
            )
        })
        .child(
            div()
                .max_w(px(metrics::ACTION_HINT_WIDTH))
                .text_size(px(type_size::CAPTION))
                .text_color(t.muted())
                .when(enabled, |d| d.child(action_shortcut(op)))
                .when(!enabled, |d| {
                    d.whitespace_normal().child(self.disabled_reason(op))
                }),
        )
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(cx.listener(move |this, _, window, cx| {
            this.invoke(op, window, cx);
        }))
    }
    fn floating(&self) -> Stateful<Div> {
        Popover::canvas(self.theme)
            .build()
            .id("floating-panel")
            .role(Role::Dialog)
            .aria_label("Dialog")
            .track_focus(&self.modal_focus)
            .tab_group()
            .tab_stop(false)
            .occlude()
            .w(px(metrics::DIALOG_WIDTH))
            .max_w(relative(0.95))
            .max_h(relative(0.95))
            .flex()
            .flex_col()
            .text_color(self.theme.text())
            .font_family(type_size::UI)
            .text_size(px(type_size::BODY_LG))
            .p(px(metrics::POPOVER_PADDING))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
    }

    fn overlay(&self, window: &Window, cx: &Context<Self>) -> Option<AnyElement> {
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
                    .bg(t.settings_scrim())
                    .child(
                        div()
                            .w(px(metrics::SETTINGS_WIDTH))
                            .max_w_full()
                            .h(px(metrics::SETTINGS_HEIGHT))
                            .max_h_full()
                            .rounded(px(radius::WINDOW))
                            .shadow_lg()
                            .child(settings.clone()),
                    )
                    .into_any_element(),
            );
        }
        if self.agent.open && !self.save_flow.dialog_open() {
            return Some(self.agent_popover(cx));
        }
        if self.palettes.open && !self.save_flow.dialog_open() {
            return Some(self.palette_manager(cx));
        }
        if self.save_flow.dialog_open() {
            return Some(
                div()
                    .absolute()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(t.app_scrim())
                    .child(self.floating().p(px(space::XXL)).child(self.status.clone()))
                    .into_any_element(),
            );
        }
        if self.save_flow.needs_confirmation() {
            return Some(
                div()
                    .absolute()
                    .size_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .bg(t.app_scrim())
                    .child(
                        self.floating()
                            .w(px(metrics::CONFIRM_WIDTH))
                            .p(px(space::XXL))
                            .child(
                                div()
                                    .text_size(px(type_size::DIALOG_TITLE))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(format!(
                                        "Do you want to save the changes you made to “{}”?",
                                        self.editor.snapshot().name
                                    )),
                            )
                            .child(
                                div()
                                    .my(px(space::XL))
                                    .text_color(t.muted())
                                    .child("Your changes will be lost if you don’t save them."),
                            )
                            .child(
                                row().mt(px(space::XXL)).gap(px(space::MD)).children(
                                    ["Don’t Save", "Cancel", "Save"]
                                        .into_iter()
                                        .enumerate()
                                        .map(|(i, label)| {
                                            Button::custom(("pending-choice", i), label)
                                                .role(Role::Button)
                                                .aria_label(label)
                                                .aria_keyshortcuts(
                                                    crate::shortcuts::format_shortcut(
                                                        ["secondary-d", "escape", "enter"][i],
                                                        ShortcutPlatform::current(),
                                                    ),
                                                )
                                                .h(px(space::XXXL))
                                                .min_w(px(metrics::ACTION_MIN_WIDTH))
                                                .justify_center()
                                                .px(px(space::LG))
                                                .when(i == 0, |d| d.mr_auto())
                                                .rounded(px(radius::CONTROL))
                                                .border_1()
                                                .border_color(
                                                    if self.save_flow.choice_index() == i {
                                                        t.accent()
                                                    } else {
                                                        t.line()
                                                    },
                                                )
                                                .bg(t.background())
                                                .when(i == 2, |d| {
                                                    d.bg(t.accent()).text_color(t.background())
                                                })
                                                .when(self.save_flow.choice_index() == i, |d| {
                                                    d.shadow(focus_ring(t))
                                                })
                                                .cursor_pointer()
                                                .hover(move |d| {
                                                    d.bg(if i == 2 {
                                                        t.accent().opacity(0.88)
                                                    } else {
                                                        t.hover()
                                                    })
                                                })
                                                .child(label)
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.resolve_pending(i, cx)
                                                }))
                                        }),
                                ),
                            ),
                    )
                    .into_any_element(),
            );
        }
        if let Some((_, InfoContent::About | InfoContent::Acknowledgements)) = &self.info {
            return Some(self.about_dialog(
                matches!(self.info, Some((_, InfoContent::Acknowledgements))),
                cx,
            ));
        }
        if let Some((title, content)) = &self.info {
            return Some(
                div()
                    .absolute()
                    .size_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .bg(t.app_scrim())
                    .child(
                        self.floating()
                            .w(px(if matches!(content, InfoContent::Guide) {
                                metrics::GUIDE_WIDTH
                            } else {
                                metrics::DIALOG_WIDTH
                            }))
                            .p(px(space::XXL))
                            .child(
                                row()
                                    .justify_between()
                                    .mb(px(space::SECTION))
                                    .child(
                                        div()
                                            .text_size(px(type_size::DIALOG_TITLE))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(*title),
                                    )
                                    .child(
                                        voxelite_ui::IconButton::button(
                                            "close-info",
                                            "close",
                                            "Close dialog",
                                            t,
                                        )
                                        .build()
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.info = None;
                                                cx.notify();
                                            }),
                                        ),
                                    ),
                            )
                            .child(match content {
                                InfoContent::Text(text) => div()
                                    .text_color(t.muted())
                                    .line_height(px(type_size::LINE_PROSE))
                                    .child(text.clone()),
                                InfoContent::Guide => keyboard_guide(t),
                                InfoContent::About | InfoContent::Acknowledgements => {
                                    unreachable!("handled above")
                                }
                            }),
                    )
                    .into_any_element(),
            );
        }
        if self.commands.is_open() {
            let matches = self.commands.filtered_actions(&self.editor.plugin_host());
            return Some(
                Dialog::scrim(t, false)
                    .child(
                        Dialog::panel("command-palette", "Search commands", t, &self.modal_focus)
                            .h(px(metrics::COMMAND_HEIGHT))
                            .p(px(space::SM))
                            .flex()
                            .flex_col()
                            .child(
                                voxelite_ui::FieldFrame::search_row(t, true)
                                    .rounded(px(radius::concentric(
                                        radius::XL,
                                        metrics::EDGE + space::SM,
                                    )))
                                    .h(px(metrics::COMMAND_SEARCH_HEIGHT))
                                    .flex_shrink_0()
                                    .child(icon("search").text_color(t.muted()))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(space::NONE))
                                            .child(self.command_input.clone()),
                                    )
                                    .child(shortcut("escape").text_color(t.muted())),
                            )
                            .child(
                                div()
                                    .id("command-results")
                                    .track_scroll(self.commands.scroll())
                                    .flex_1()
                                    .overflow_y_scroll()
                                    .py(px(space::SM))
                                    .children(matches.iter().enumerate().map(|(i, op)| {
                                        self.action_row(
                                            *op,
                                            i,
                                            self.commands.index() == i,
                                            window,
                                            cx,
                                        )
                                    }))
                                    .when(matches.is_empty(), |d| {
                                        d.child(
                                            div()
                                                .p(px(space::XXL))
                                                .text_color(t.muted())
                                                .child("No matching commands"),
                                        )
                                    }),
                            )
                            .child(
                                row()
                                    .h(px(space::XXXL))
                                    .px(px(space::FOOTER))
                                    .text_color(t.muted())
                                    .text_size(px(type_size::BODY))
                                    .border_t_1()
                                    .border_color(t.line())
                                    .child(if self.command_input.read(cx).focused(window) {
                                        HintText::default()
                                            .key("enter")
                                            .text(" to run   ·   ")
                                            .key("escape")
                                            .text(" to browse results")
                                    } else {
                                        HintText::default()
                                            .key("up")
                                            .text(" ")
                                            .key("down")
                                            .text(" to navigate   ·   ")
                                            .key("enter")
                                            .text(" to run   ·   ")
                                            .key("escape")
                                            .text(" to close")
                                    }),
                            ),
                    )
                    .into_any_element(),
            );
        }
        if self.palette_open {
            return Some(
                div()
                    .absolute()
                    .top(px(metrics::PALETTE_CHOOSER_TOP))
                    .right(px(space::XL))
                    .child(
                        self.floating()
                            .w(px(metrics::PALETTE_CHOOSER_WIDTH))
                            .child(
                                row()
                                    .px(px(space::LG))
                                    .h(px(voxelite_ui::tokens::ControlSize::Lg.height()))
                                    .justify_between()
                                    .child(section_title("Choose palette"))
                                    .child(shortcut("escape").text_color(t.muted())),
                            )
                            .child(
                                div()
                                    .id("palette-results")
                                    .track_scroll(&self.palette_scroll)
                                    .max_h(px(metrics::PALETTE_CHOOSER_HEIGHT))
                                    .overflow_y_scroll()
                                    .children(self.palette_choices().iter().enumerate().map(
                                        |(i, choice)| {
                                            let p = &choice.palette;
                                            Button::tile(("preset", i), choice.name.clone())
                                                .p(px(space::INSET))
                                                .rounded(px(radius::CONTROL))
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
                                                        .gap(px(space::MD))
                                                        .mb(px(space::MD))
                                                        .child(
                                                            div()
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .child(choice.name.clone()),
                                                        )
                                                        .when(i == self.palette_index(), |d| {
                                                            d.child(
                                                                icon("check")
                                                                    .size(px(space::FOOTER))
                                                                    .text_color(t.accent()),
                                                            )
                                                        }),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_wrap()
                                                        .gap(px(space::COMPACT))
                                                        .children(p.order.iter().take(32).map(
                                                            |slot| {
                                                                div()
                                                                    .w(px(metrics::ICON_DENSE))
                                                                    .h(px(space::FOOTER))
                                                                    .rounded(px(radius::PREVIEW))
                                                                    .border_1()
                                                                    .border_color(t.swatch_edge())
                                                                    .bg(color_fill(
                                                                        p.get(*slot)
                                                                            .expect("valid palette")
                                                                            .rgba,
                                                                    ))
                                                            },
                                                        )),
                                                )
                                                .child(
                                                    div()
                                                        .mt(px(space::SM))
                                                        .text_size(px(type_size::BODY))
                                                        .text_color(t.muted())
                                                        .child(format!(
                                                            "{} colors{}{}",
                                                            p.order.len(),
                                                            if i != 0 {
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
                                                    this.palette_request(
                                                        PaletteAction::ChoosePreset(i),
                                                        cx,
                                                    );
                                                    this.palette_open = false;
                                                    cx.notify();
                                                }))
                                        },
                                    )),
                            )
                            .child(
                                div()
                                    .px(px(space::LG))
                                    .py(px(space::LG))
                                    .mt(px(space::XS))
                                    .border_t_1()
                                    .border_color(t.line())
                                    .text_size(px(type_size::BODY))
                                    .text_color(t.muted())
                                    .child(self.control(
                                        "manage-palettes",
                                        "palette",
                                        "Manage palettes…",
                                        cx,
                                        |this, window, cx| {
                                            this.palette_open = false;
                                            this.manage_palettes(cx);
                                            window.focus(&this.modal_focus, cx);
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
                    voxelite_ui::Menu::canvas("application-menu", t, &self.modal_focus)
                        .w(px(metrics::APP_MENU_WIDTH))
                        .children(
                            crate::actions::menu_items(i, &self.editor.plugin_host())
                                .iter()
                                .enumerate()
                                .map(|(j, op)| {
                                    self.action_row(*op, j, j == self.menu_index, window, cx)
                                }),
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
        if let Some(view) = self.editor.comments_view() {
            self.editor.comments_state = view.state;
        } else {
            if self.panel == SidebarTab::Comments {
                self.panel = SidebarTab::Session;
            }
            self.editor.comments_state.placing = false;
            self.editor.comments_state.first = None;
            self.editor.comments_state.draft = None;
        }
        for image in self.agent.retired.drain(..) {
            let _ = window.drop_image(image);
        }
        self.viewport.release_retired_image(window);
        self.palettes.appearance(self.theme, cx);
        self.discussion
            .text
            .update(cx, |f, cx| f.appearance(self.theme, cx));
        self.timeline
            .name
            .update(cx, |f, cx| f.appearance(self.theme, cx));
        self.command_input
            .update(cx, |field, cx| field.appearance(self.theme, cx));
        for field in [&self.palettes.name, &self.palettes.hex, &self.command_input] {
            field.update(cx, |field, _| {
                field.focus_policy(Some(self.modal_focus.clone()), self.modal_focus.clone())
            });
        }
        for field in [&self.discussion.text, &self.timeline.name] {
            field.update(cx, |field, _| field.focus_policy(None, self.focus.clone()));
        }
        let size = window.viewport_size();
        self.layout_viewport(window, cx);
        let title = format!(
            "{}{} — Voxelite",
            self.editor.display_snapshot().name,
            if self.editor.engine.is_dirty() && !cfg!(target_os = "macos") {
                " •"
            } else {
                ""
            }
        );
        if title != self.last_title {
            window.set_window_title(&title);
            self.last_title = title;
        }
        let edited = self.editor.engine.is_dirty();
        if self.last_edited != Some(edited) {
            window.set_window_edited(edited);
            self.last_edited = Some(edited);
        }
        if cfg!(target_os = "macos") && window.is_window_active() {
            let menus = crate::actions::menus_from_host(
                &self.editor.plugin_host(),
                window.is_fullscreen(),
                |op| self.selected(op),
                |op| {
                    (matches!(op, Op::Undo | Op::Redo) && crate::text_field::focused(window))
                        || self.enabled(op)
                },
            );
            #[cfg(not(target_family = "wasm"))]
            let menus = welcome::menus(menus, self.enabled(Op::Open), cx);
            cx.set_menus(menus);
        }
        #[cfg(target_family = "wasm")]
        let settings_open = self.settings_modal.is_some();
        #[cfg(not(target_family = "wasm"))]
        let settings_open = false;
        if self
            .overlay_drag
            .as_ref()
            .is_some_and(|(id, _)| self.editor.plugin_host().registry().get(id).is_none())
        {
            self.overlay_drag = None;
            self.overlay_hover = None;
        }
        if self.overlay_drag.is_none() {
            self.overlay_hover = self.overlay_hit(window.mouse_position());
        }
        self.reconcile_palette_panel(window, cx);
        let overlay = self.overlay(window, cx);
        div()
            .id("editor-root")
            .key_context(if settings_open {
                "Settings"
            } else if self.command_input.read(cx).focused(window)
                || self.palettes.field_focused(window, cx)
                || self.discussion.text.read(cx).focused(window)
                || self.timeline.name.read(cx).focused(window)
            {
                "TextField"
            } else if self.palette_focus.is_focused(window) {
                "Palette"
            } else if self.commands.is_open() {
                "CommandPalette"
            } else if self.menu.is_some()
                || self.palette_open
                || self.save_flow.needs_confirmation()
                || self.info.is_some()
                || self.save_flow.dialog_open()
                || self.palettes.open
                || self.agent.open
            {
                "Modal"
            } else {
                "Editor"
            })
            .track_focus(&self.focus)
            .tab_stop(false)
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(self.theme.background())
            .text_color(self.theme.text())
            .font(gpui::font("Geist"))
            .text_size(px(type_size::BODY_LG))
            .capture_key_down(cx.listener(Self::key_down))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if this.overlay_drag.is_some() {
                    this.move_overlay(event, cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, event: &MouseUpEvent, _, cx| {
                    this.release_overlay(event.position, cx);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.overlay_drag = None;
                    this.overlay_hover = None;
                    cx.notify();
                }),
            )
            .on_action(cx.listener(|this, op: &Op, window, cx| {
                if crate::text_field::route_history(*op, window, cx) {
                    return;
                }
                this.invoke(*op, window, cx);
            }))
            .map(|d| {
                #[cfg(not(target_family = "wasm"))]
                let d = d.on_action(cx.listener(
                    |this, op: &crate::actions::OpenRecentProject, window, cx| {
                        this.open_recent(op.path.clone(), window, cx);
                    },
                ));
                d
            })
            .when(!cfg!(target_os = "macos"), |d| d.child(self.menubar(cx)))
            .child(self.toolbar(f32::from(size.width) < 1200.0, cx))
            .child(
                row()
                    .flex_1()
                    .min_h(px(space::NONE))
                    .items_stretch()
                    .child(self.tool_rail(cx))
                    .child(self.viewport(cx))
                    .child(self.sidebar(f32::from(size.height) < 760.0, cx)),
            )
            .child(self.statusbar(f32::from(size.width) < metrics::STATUS_COMPACT_WIDTH))
            .children(overlay.map(|overlay| {
                div()
                    .id("overlay")
                    .absolute()
                    .size_full()
                    .occlude()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            if this.palettes.open {
                                this.close_palettes(window, cx);
                                cx.stop_propagation();
                                return;
                            }
                            window.focus(&this.focus, cx);
                            #[cfg(target_family = "wasm")]
                            {
                                this.settings_modal = None;
                            }
                            this.agent.open = false;

                            this.palettes.open = false;
                            this.menu = None;
                            this.palette_open = false;
                            this.commands.close();
                            this.info = None;
                            this.save_flow.cancel_confirmation();
                            this.drag = None;
                            cx.stop_propagation();
                            cx.notify();
                        }),
                    )
                    .child(overlay)
            }))
    }
}
