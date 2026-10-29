//! Editor frame synchronization, root layout and event wiring.
use super::*;
use quantum_ui::ColorToken;
use quantum_ui::Space;
use quantum_ui::TextSize;
use quantum_ui::TextStyle;

impl Render for EditorView {
    #[cfg_attr(
        feature = "profiling",
        tracing::instrument(name = "editor.frame", skip_all)
    )]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        #[cfg(feature = "profiling")]
        tracing::event!(tracing::Level::INFO, tracy.frame_mark = true);
        self.prepare_frame(window, cx);
        let modal = self.palettes.open
            || self.save_flow.dialog_open()
            || self.save_flow.needs_confirmation()
            || self.info.is_some()
            || self.commands.is_open();
        self.overlays.update(cx, |state, cx| {
            state.set_open("editor-modal", quantum_ui::OverlayLayer::Dialog, modal, cx);
            state.set_open(
                "editor-popover",
                quantum_ui::OverlayLayer::Popover,
                self.menu.is_some() || self.palette_open || self.agent.open,
                cx,
            );
        });
        let size = window.viewport_size();
        let overlay = self.overlay(window, cx);
        div()
            .id("editor-root")
            .key_context(self.key_context(window, cx))
            .track_focus(&self.focus)
            .tab_stop(false)
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(self.theme.color(ColorToken::Canvas))
            .text_color(self.theme.color(ColorToken::TextPrimary))
            .font(gpui::font("Geist"))
            .text_style(TextSize::Body)
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
            .on_action(
                cx.listener(|this, op: &crate::actions::OpenRecentProject, window, cx| {
                    this.open_recent(op.path.clone(), window, cx);
                }),
            )
            .when(!cfg!(target_os = "macos"), |d| d.child(self.menubar(cx)))
            .child(self.toolbar(f32::from(size.width) < 1200.0, cx))
            .child(
                row()
                    .flex_1()
                    .min_h(px(Space::S0.px()))
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
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::dismiss_overlay))
                    .child(overlay)
            }))
    }
}

impl EditorView {
    fn prepare_frame(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
        self.layout_viewport(window, cx);
        let title = format!(
            "{}{} — Quantum",
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
        if crate::actions::sync_native_menus(window) {
            let menus = crate::actions::menus_from_host(
                &self.editor.plugin_host(),
                window.is_fullscreen(),
                |op| self.selected(op),
                |op| {
                    (matches!(op, Op::Undo | Op::Redo) && crate::text_field::focused(window, cx))
                        || self.enabled(op)
                },
            );
            let menus = welcome::menus(menus, self.enabled(Op::Open), cx);
            cx.set_menus(menus);
        }
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
    }

    fn key_context(&self, window: &Window, cx: &Context<Self>) -> &'static str {
        if self.command_input.read(cx).focused(window)
            || self.palettes.field_focused(window, cx)
            || self.discussion.text.read(cx).focused(window)
            || self.timeline.name.read(cx).focused(window)
        {
            "TextField"
        } else if self.palette_focus.contains_focused(window, cx) {
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
        }
    }

    fn dismiss_overlay(&mut self, _: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.palettes.open {
            self.close_palettes(window, cx);
            cx.stop_propagation();
            return;
        }
        window.focus(&self.focus, cx);
        self.agent.open = false;

        self.palettes.open = false;
        self.menu = None;
        self.palette_open = false;
        self.commands.close();
        self.info = None;
        self.save_flow.cancel_confirmation();
        self.drag = None;
        cx.stop_propagation();
        cx.notify();
    }
}
