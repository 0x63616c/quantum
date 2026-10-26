//! Pointer interaction and painting for the six-ended camera gizmo.
use super::*;
use voxelite_app::gizmo::{self, AxisEnd, HIT_RADIUS, SIZE};

pub(super) struct GizmoDrag {
    origin: Point<Pixels>,
    last: Point<Pixels>,
    end: Option<AxisEnd>,
    moved: bool,
}

impl EditorView {
    fn gizmo_hit(&self, position: Point<Pixels>) -> Option<AxisEnd> {
        let bounds = self.gizmo_bounds.get();
        if !bounds.contains(&position) {
            return None;
        }
        let local = position - bounds.origin;
        gizmo::hit_test(self.display_camera, [local.x.into(), local.y.into()])
    }

    pub(super) fn move_gizmo(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        let Some(drag) = &mut self.gizmo_drag else {
            return;
        };
        if event.pressed_button != Some(MouseButton::Left) {
            self.gizmo_drag = None;
            cx.notify();
            return;
        }
        let travel = event.position - drag.origin;
        drag.moved |= f32::from(travel.x).hypot(f32::from(travel.y)) >= 3.0;
        if drag.moved {
            let delta = event.position - drag.last;
            self.editor
                .camera
                .orbit(f64::from(f32::from(delta.x)), f64::from(f32::from(delta.y)));
            self.agent.follow = false;
            self.camera_fitted = false;
            self.camera_view = None;
            self.gizmo_hover = None;
        }
        drag.last = event.position;
        if drag.moved {
            self.request_navigation(cx);
        }
        self.hover = None;
        cx.notify();
    }

    pub(super) fn release_gizmo(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        let Some(drag) = self.gizmo_drag.take() else {
            return;
        };
        if !drag.moved
            && let Some(end) = drag.end
            && self.gizmo_hit(position) == Some(end)
        {
            end.snap(&mut self.editor.camera);
            self.agent.follow = false;
            self.camera_fitted = false;
            self.camera_view = match (end.axis, end.positive) {
                (0, true) => Some(Op::Right),
                (1, true) => Some(Op::Top),
                (2, true) => Some(Op::Front),
                _ => None,
            };
            self.request_render(cx);
        }
        cx.notify();
    }

    pub(super) fn orientation(&self, cx: &Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        let ends = gizmo::project(self.display_camera);
        let bounds_cell = self.gizmo_bounds.clone();
        let hovered = self.gizmo_hover;
        div().id("orientation").absolute().top(px(12.0)).right(px(12.0)).size(px(SIZE))
            .aria_label("Camera orientation: drag to orbit, click an axis for its view, double-click for isometric")
            .cursor(if self.gizmo_drag.is_some() { CursorStyle::ClosedHand } else { CursorStyle::OpenHand })
            .child(canvas(
                move |bounds, _, _| bounds_cell.set(bounds),
                move |bounds, _, window, _| {
                    let center = bounds.center();
                    for end in ends {
                        let mut path = PathBuilder::stroke(px(if hovered == Some(end.end) { 2.0 } else { 1.0 }));
                        path.move_to(center);
                        path.line_to(bounds.origin + point(px(end.position[0]), px(end.position[1])));
                        if let Ok(path) = path.build() { window.paint_path(path, t.axis(end.end.axis)); }
                    }
                },
            ).size_full())
            .children(ends.into_iter().map(|projected| {
                let end = projected.end;
                let color = t.axis(end.axis);
                let highlighted = hovered == Some(end);
                row().absolute()
                    .left(px(projected.position[0] - HIT_RADIUS))
                    .top(px(projected.position[1] - HIT_RADIUS))
                    .size(px(HIT_RADIUS * 2.0)).justify_center().rounded_full()
                    .border_2().border_color(if highlighted { t.text() } else { color })
                    .bg(if end.positive { color } else { t.background() })
                    .text_color(t.background()).text_size(px(12.0)).font_weight(FontWeight::SEMIBOLD)
                    .when(end.positive, |d| d.child(["X", "Y", "Z"][end.axis]))
                    .when(highlighted, |d| d.shadow(focus_ring(t)))
            }))
            .on_mouse_down(MouseButton::Left, cx.listener(|this, event: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                window.focus(&this.focus, cx);
                this.hover = None;
                if event.click_count == 2 {
                    this.gizmo_drag = None;
                    this.invoke(Op::Frame, window, cx);
                } else {
                    this.gizmo_drag = Some(GizmoDrag {
                        origin: event.position, last: event.position,
                        end: this.gizmo_hit(event.position), moved: false,
                    });
                }
                cx.notify();
            }))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                cx.stop_propagation();
                this.hover = None;
                if this.gizmo_drag.is_some() { this.move_gizmo(event, cx); }
                else {
                    let hovered = this.gizmo_hit(event.position);
                    if hovered != this.gizmo_hover { this.gizmo_hover = hovered; cx.notify(); }
                }
            }))
            .on_hover(cx.listener(|this, hovered, _, cx| {
                if !hovered { this.gizmo_hover = None; cx.notify(); }
            }))
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
    }
}
