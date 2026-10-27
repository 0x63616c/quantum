//! Native GPUI shell. Portable editing logic lives in the library.
mod about;
#[cfg(not(target_family = "wasm"))]
mod agent;
#[cfg(target_family = "wasm")]
#[path = "ui/agent_web.rs"]
mod agent;
#[cfg(target_family = "wasm")]
mod browser;
#[cfg(target_os = "linux")]
mod capture;
mod command_palette;
mod comments;
mod history;
mod keyboard_guide;
mod viewport;
use keyboard_guide::keyboard_guide;
mod overlays;
use comments::Panel;
mod palette_layout;
mod palettes;
mod save_flow;
mod settings;
mod shortcut_hint;
mod tooltip;
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

// Keep render sizing and native layout on the same logical-pixel metrics.
const SIDEBAR_WIDTH: f32 = 352.0;
const TOOL_RAIL_WIDTH: f32 = 56.0;
const MENUBAR_HEIGHT: f32 = if cfg!(target_os = "macos") { 0.0 } else { 32.0 };
const TOOLBAR_HEIGHT: f32 = metrics::TOOLBAR_HEIGHT;
const STATUSBAR_HEIGHT: f32 = metrics::STATUS_HEIGHT;
const PANEL_GUTTER: f32 = 24.0;

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
    modal_focus: FocusHandle,
    sidebar_scroll: ScrollHandle,
    focus: FocusHandle,
    viewport: viewport::Viewport,
    last_title: String,
    last_edited: Option<bool>,
    zoom_reference_span: f64,
    camera_view: Option<Op>,
    camera_fitted: bool,
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
impl EditorView {
    pub fn new(
        mut editor: Editor,
        override_theme: Option<Theme>,
        socket: Option<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        settings::init(override_theme, cx);
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
            panel: Panel::Session,
            timeline: history::Timeline::new(window, cx),
            cursor: editor.snapshot().bounds.min,
            camera_view: Some(Op::Frame),
            camera_fitted: true,
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
                self.panel = Panel::Session;
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
                self.editor.camera =
                    OrbitCamera::fit_view(self.editor.display_snapshot(), aspect, view);
                self.zoom_reference_span = self.editor.camera.span;
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
                    self.swatch_scroll
                        .set_offset(point(px(0.0), -px((next / 8) as f32 * 38.0)));
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
        if self.panel == Panel::History && key == "space" {
            if !e.is_held {
                self.toggle_history(cx);
            }
            cx.stop_propagation();
            return;
        }
        if self.panel == Panel::History && matches!(key, "left" | "right") {
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
        button
            .build()
            .aria_keyshortcuts(op.shortcut())
            .aria_label(op.label())
            .anchored_hint(
                if enabled {
                    HintText::from(op.label()).action(op)
                } else {
                    format!("{} · {}", op.label(), disabled_reason).into()
                },
                theme,
                if id.starts_with("rail-") {
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
                        this.commands.close();
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
        Toolbar::row(t)
            .id("main-titlebar")
            .on_mouse_down(MouseButton::Left, |event, window, _| {
                if event.click_count == 2 {
                    window.zoom_window();
                } else {
                    window.start_window_move();
                }
            })
            .when(cfg!(target_os = "macos"), |d| d.pl(px(104.0)))
            .child(icon("cube").text_color(t.muted()).size(px(22.0)))
            .child(
                div()
                    .w(px(if compact { 260.0 } else { 340.0 }))
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
                                    .child(format!("{}.voxl", self.editor.display_snapshot().name)),
                            )
                            .when(self.editor.engine.is_dirty(), |d| {
                                d.child(
                                    div()
                                        .flex_shrink_0()
                                        .text_size(px(13.0))
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
                .h(px(36.0))
                .gap(px(12.0))
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
                    .bottom(px(20.0))
                    .left(px(24.0))
                    .gap(px(4.0))
                    .child(self.button("view-settings", "settings", "", Op::Settings, cx))
                    .when(self.preferences.navigation_hints, |d| {
                        d.child(self.button("view-help", "keyboard", "", Op::Help, cx))
                    }),
            )
            .child(
                div().absolute().bottom(px(16.0)).right(px(24.0)).child(
                    voxelite_ui::Segmented::row(t)
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
                            .tab_index(0)
                            .focus_visible(move |d| d.bg(t.hover()).shadow(focus_ring(t)))
                            .active(move |d| d.bg(t.selected()))
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
                            .child(icon("chevron").size(px(14.0)).text_color(t.text()))
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
                            .text_size(px(12.0))
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
                                .active(move |d| d.border_2().border_color(t.accent()))
                                .when(selected, |d| {
                                    d.child(icon("check").size(px(16.0)).text_color(swatch_ink(c)))
                                })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.palette_request(PaletteAction::SelectSwatch(i), cx);
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
                    .text_color(t.text())
                    .child(hex)
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
                                    .tab_index(0)
                                    .focus_visible(move |d| d.bg(t.hover()).shadow(focus_ring(t)))
                                    .active(move |d| d.bg(t.selected()))
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
                                            .tab_index(0)
                                            .focus_visible(move |d| {
                                                d.bg(t.hover()).shadow(focus_ring(t))
                                            })
                                            .active(move |d| d.bg(t.selected()))
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
                                            .tab_index(0)
                                            .focus_visible(move |d| {
                                                d.bg(t.hover()).shadow(focus_ring(t))
                                            })
                                            .active(move |d| d.bg(t.selected()))
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
            .child(self.panel_tabs(cx))
            .child(match self.panel {
                Panel::Comments => self.comments_panel(cx),
                Panel::History => self.timeline_panel(cx),
                Panel::Session => self.agent_feed(compact, cx),
            })
            .when(self.panel == Panel::Session, |d| {
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
                .max_w(px(240.))
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
        Dialog::panel("floating-panel", "Dialog", self.theme, &self.modal_focus)
            .bg(self.theme.background())
            .p(px(space::SM))
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
                    .bg(rgba(0x00000070))
                    .child(self.floating().p(px(24.0)).child(self.status.clone()))
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
                    .bg(rgba(0x00000070))
                    .child(
                        self.floating()
                            .w(px(480.0))
                            .p(px(24.0))
                            .child(
                                div()
                                    .text_size(px(18.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(format!(
                                        "Do you want to save the changes you made to “{}”?",
                                        self.editor.snapshot().name
                                    )),
                            )
                            .child(
                                div()
                                    .my(px(16.0))
                                    .text_color(t.muted())
                                    .child("Your changes will be lost if you don’t save them."),
                            )
                            .child(
                                row().mt(px(24.0)).gap(px(8.0)).children(
                                    ["Don’t Save", "Cancel", "Save"]
                                        .into_iter()
                                        .enumerate()
                                        .map(|(i, label)| {
                                            row()
                                                .id(("pending-choice", i))
                                                .role(Role::Button)
                                                .aria_label(label)
                                                .aria_keyshortcuts(
                                                    crate::shortcuts::format_shortcut(
                                                        ["secondary-d", "escape", "enter"][i],
                                                        ShortcutPlatform::current(),
                                                    ),
                                                )
                                                .h(px(32.0))
                                                .min_w(px(80.0))
                                                .justify_center()
                                                .px(px(12.0))
                                                .when(i == 0, |d| d.mr_auto())
                                                .rounded(px(6.0))
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
                    .bg(rgba(0x00000070))
                    .child(
                        self.floating()
                            .w(px(if matches!(content, InfoContent::Guide) {
                                720.0
                            } else {
                                560.0
                            }))
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
                                    .line_height(px(26.0))
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
                            .h(px(440.0))
                            .p(px(space::SM))
                            .flex()
                            .flex_col()
                            .child(
                                voxelite_ui::FieldFrame::search_row(t, true)
                                    .rounded(px(radius::concentric(
                                        radius::XL,
                                        metrics::EDGE + space::SM,
                                    )))
                                    .h(px(52.0))
                                    .flex_shrink_0()
                                    .child(icon("search").text_color(t.muted()))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(0.))
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
                                    .py(px(6.0))
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
                                    .children(self.palette_choices().iter().enumerate().map(
                                        |(i, choice)| {
                                            let p = &choice.palette;
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
                                                                .child(choice.name.clone()),
                                                        )
                                                        .when(i == self.palette_index(), |d| {
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
                    self.floating().w(px(310.0)).children(
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
            if self.panel == Panel::Comments {
                self.panel = Panel::Session;
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
            cx.set_menus(crate::actions::menus_from_host(
                &self.editor.plugin_host(),
                window.is_fullscreen(),
                |op| self.selected(op),
                |op| {
                    (matches!(op, Op::Undo | Op::Redo) && crate::text_field::focused(window))
                        || self.enabled(op)
                },
            ));
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
            .text_size(px(13.0))
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
