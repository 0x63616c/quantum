//! Viewport composition and pointer navigation.
use super::*;
use quantum_ui::ColorToken;
use quantum_ui::Radius;
use quantum_ui::Space;

impl EditorView {
    pub(super) fn pointer_move(
        &mut self,
        e: &MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
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
    pub(super) fn viewport(&self, cx: &Context<Self>) -> Stateful<Div> {
        let surface = self.viewport_surface(cx);
        let surface = div()
            .id("viewport")
            .relative()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .cursor_crosshair()
            .when(
                self.capture_state != Some(launch::LaunchState::SceneLoading),
                |d| d.child(surface),
            )
            .when(self.scene_is_loading(), |d| d.child(self.scene_loading()))
            .child(self.scene_heading())
            .children(self.comment_pins(cx))
            .when(self.editor.preview.is_some(), |d| {
                d.child(self.preview_banner(cx))
            })
            .when(self.preferences.show_debug_overlay, |d| {
                d.child(self.debug_overlay())
            })
            .children(self.viewport_widgets(cx))
            .child(self.viewport_help(cx))
            .child(self.camera_controls(cx));
        self.viewport_input(surface, cx)
    }

    fn preview_banner(&self, cx: &Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        row()
            .id("preview-banner")
            .absolute()
            .top(px(metrics::COMMAND_SEARCH_HEIGHT))
            .left(px(Space::S6.px()))
            .px(px(Space::S4.px()))
            .py(px(Space::S3.px()))
            .rounded(px(Radius::Sm.px()))
            .bg(t.selected())
            .text_color(t.color(ColorToken::AccentText))
            .gap(px(Space::S3.px()))
            .child(icon("lock").text_color(t.color(ColorToken::AccentText)))
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
            )
    }

    fn viewport_help(&self, cx: &Context<Self>) -> Div {
        row()
            .absolute()
            .bottom(px(Space::S5.px()))
            .left(px(Space::S6.px()))
            .gap(px(Space::S2.px()))
            .child(self.button(
                "view-settings",
                "settings",
                "",
                Op::Settings,
                ActionPresentation::Icon(TooltipSide::Above),
                cx,
            ))
            .when(self.preferences.navigation_hints, |d| {
                d.child(self.button(
                    "view-help",
                    "keyboard",
                    "",
                    Op::Help,
                    ActionPresentation::Icon(TooltipSide::Above),
                    cx,
                ))
            })
    }

    fn camera_controls(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        div()
            .absolute()
            .bottom(px(Space::S5.px()))
            .right(px(Space::S6.px()))
            .child(
                SegmentedControl::design(t)
                    .children(
                        [
                            ("view-iso", "Iso", Op::Frame),
                            ("view-front", "Front", Op::Front),
                            ("view-right", "Right", Op::Right),
                            ("view-top", "Top", Op::Top),
                        ]
                        .map(|(id, label, op)| {
                            self.button(id, op.icon(), label, op, ActionPresentation::Segment, cx)
                                .border_b_2()
                                .border_color(transparent_black())
                                .when(self.camera_view == Some(op), |d| {
                                    d.bg(t.selected())
                                        .text_color(t.color(ColorToken::AccentText))
                                        .border_color(t.color(ColorToken::AccentText))
                                })
                        }),
                    )
                    .child(ToolbarSeparator::line(t).mx(px(Space::S2.px())))
                    .child(self.button(
                        "view-fit",
                        "fit",
                        "",
                        Op::Frame,
                        ActionPresentation::Frame,
                        cx,
                    )),
            )
    }

    fn viewport_click(&mut self, e: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        if self.menu.take().is_some() || self.palette_open {
            self.palette_open = false;
            cx.notify();
            return;
        }
        if self.commands.is_open()
            || self.info.is_some()
            || self.save_flow.needs_confirmation()
            || self.save_flow.dialog_open()
        {
            return;
        }
        if self.editor.preview.is_some() {
            self.pause_history(cx);
            self.status = "History preview is read-only · Return to live to edit".into();
            cx.notify();
            return;
        }
        if let Some(ray) = self.viewport.ray(&self.editor, e.position) {
            if self.editor.comments_state.placing && self.editor.comments_enabled() {
                self.place_comment(Some(ray), quantum_plugin_api::ToolGesture::Click, cx);
                return;
            }
            let r = self.editor.click(ray, e.modifiers.alt);
            let status = if self.editor.box_start.is_some() {
                HintText::from("First corner set · click opposite corner · ")
                    .key("escape")
                    .text(" cancels")
            } else {
                HintText::default()
            };
            self.report(r, status, cx);
        }
    }

    fn viewport_input(&self, surface: Stateful<Div>, cx: &Context<Self>) -> Stateful<Div> {
        surface
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
            .on_mouse_down(MouseButton::Left, cx.listener(Self::viewport_click))
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
}
