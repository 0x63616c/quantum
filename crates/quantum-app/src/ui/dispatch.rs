//! Editor command availability, selection and execution.
use super::*;

impl EditorView {
    pub(super) fn availability(&self, op: Op) -> Result<(), &'static str> {
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
    pub(super) fn enabled(&self, op: Op) -> bool {
        self.availability(op).is_ok()
    }
    pub(super) fn disabled_reason(&self, op: Op) -> &'static str {
        self.availability(op).err().unwrap_or_default()
    }
    pub(super) fn selected(&self, op: Op) -> bool {
        match op {
            Op::Place => self.editor.tool == Tool::Place,
            Op::Erase => self.editor.tool == Tool::Erase,
            Op::Paint => self.editor.tool == Tool::Paint,
            Op::Fill => self.editor.tool == Tool::Fill,
            Op::Box => self.editor.tool == Tool::Box,
            Op::Picker => self.editor.tool == Tool::Picker,
            Op::Grid => self.editor.grid(),
            Op::DebugOverlay => self.preferences.show_debug_overlay,
            Op::Frame | Op::Front | Op::Right | Op::Top => self.camera_view == Some(op),
            Op::MirrorX => self.editor.mirror_axes()[0],
            Op::MirrorY => self.editor.mirror_axes()[1],
            Op::MirrorZ => self.editor.mirror_axes()[2],
            _ => false,
        }
    }
    pub(super) fn invoke(&mut self, op: Op, window: &mut Window, cx: &mut Context<Self>) {
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
            .confirm_if_dirty(op, || self.editor.needs_save())
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
    pub(super) fn run(&mut self, op: Op, cx: &mut Context<Self>) {
        match op {
            Op::Settings | Op::Theme | Op::Minimize | Op::ZoomWindow | Op::FullScreen | Op::ShowPalette => {},
            Op::About => self.info = Some(("About Quantum", InfoContent::About)),
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
            Op::OpenSample => welcome::show_deferred(true, cx),
            Op::OpenRecent | Op::Welcome => welcome::show_deferred(false, cx),
            Op::ClearRecents => welcome::clear(cx),
            Op::Save => self.save(false, cx),
            Op::SaveAs => self.save(true, cx),
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
            Op::DebugOverlay => settings::change(cx, |p| p.show_debug_overlay = !p.show_debug_overlay),
            Op::Frame | Op::Front | Op::Right | Op::Top => self.select_camera(op, cx),
            Op::Place | Op::Erase | Op::Paint | Op::Fill | Op::Box | Op::Picker => self.select_tool(op, cx),
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

    fn select_camera(&mut self, op: Op, cx: &mut Context<Self>) {
        self.agent.follow = false;
        let b = self.viewport.bounds();
        let aspect = (f32::from(b.size.width) / f32::from(b.size.height).max(1.0)) as f64;
        let view = match op {
            Op::Front => PresetView::Front,
            Op::Right => PresetView::Right,
            Op::Top => PresetView::Top,
            _ => PresetView::Iso,
        };
        let destination = OrbitCamera::fit_view(self.editor.display_snapshot(), aspect, view);
        self.zoom_reference_span = destination.span;
        self.camera_view = Some(op);
        self.camera_fitted = true;
        self.animate_camera_to(destination, cx);
    }

    fn select_tool(&mut self, op: Op, _cx: &mut Context<Self>) {
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
                .key("escape")
                .text(" cancels.")
        } else {
            format!("{} tool", self.editor.tool.name()).into()
        };
        self.error = false;
    }
}
