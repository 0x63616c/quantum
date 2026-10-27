//! GPUI adapter for enabled portable overlay primitives and pointer feedback.
use super::*;
use voxelite_plugin_api::{
    OverlayColor, OverlayGesture, OverlayInput, OverlayNavigation, OverlayOutcome,
    OverlayPrimitive, RenderPassSlot,
};

pub(super) fn color(theme: Theme, color: OverlayColor) -> Hsla {
    match color {
        OverlayColor::Axis(axis) => theme.axis(axis),
        OverlayColor::Line => theme.line(),
        OverlayColor::Background => theme.background(),
        OverlayColor::Text => theme.text(),
    }
}

impl EditorView {
    fn overlay_input(
        &self,
        id: &str,
        position: Point<Pixels>,
        gesture: OverlayGesture,
    ) -> OverlayInput {
        let origin = self
            .overlay_bounds
            .borrow()
            .get(id)
            .map_or(point(px(0.), px(0.)), |b| b.origin);
        let local = position - origin;
        OverlayInput {
            gesture,
            position: [local.x.into(), local.y.into()],
            pointer: [position.x.into(), position.y.into()],
            drag: self
                .overlay_drag
                .as_ref()
                .filter(|(owner, _)| owner == id)
                .map(|(_, drag)| *drag),
        }
    }

    pub(super) fn overlay_hit(&self, position: Point<Pixels>) -> Option<(String, u32)> {
        for (id, bounds) in self.overlay_bounds.borrow().iter() {
            if bounds.contains(&position) {
                let input = self.overlay_input(id, position, OverlayGesture::Hover);
                if let Ok(Some(outcome)) =
                    self.editor
                        .overlay_outcome(id, self.viewport.display_camera(), input)
                    && let Some(target) = outcome.hovered
                {
                    return Some((id.clone(), target));
                }
            }
        }
        None
    }

    fn interact_overlay(
        &mut self,
        id: &str,
        position: Point<Pixels>,
        gesture: OverlayGesture,
        cx: &mut Context<Self>,
    ) -> Option<OverlayOutcome> {
        let input = self.overlay_input(id, position, gesture);
        match self
            .editor
            .interact_overlay(id, self.viewport.display_camera(), input)
        {
            Ok(outcome) => {
                self.overlay_drag = outcome
                    .as_ref()
                    .and_then(|o| o.drag)
                    .map(|drag| (id.into(), drag));
                if let Some(outcome) = &outcome {
                    if !outcome.commands.is_empty() {
                        self.request_render(cx);
                    }
                    match &outcome.navigation {
                        Some(OverlayNavigation::Orbit(_)) => {
                            self.agent.follow = false;
                            self.camera_fitted = false;
                            self.camera_view = None;
                            self.overlay_hover = None;
                            self.request_navigation(cx);
                        }
                        Some(OverlayNavigation::Orient { view, .. }) => {
                            self.agent.follow = false;
                            self.camera_fitted = false;
                            self.camera_view = view.as_deref().and_then(Op::from_command_id);
                            self.request_render(cx);
                        }
                        _ => {}
                    }
                }
                outcome
            }
            Err(error) => {
                self.status = error.to_string().into();
                self.error = true;
                None
            }
        }
    }

    pub(super) fn move_overlay(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        let Some((id, _)) = self.overlay_drag.clone() else {
            return;
        };
        self.interact_overlay(
            &id,
            event.position,
            OverlayGesture::Move {
                primary_down: event.pressed_button == Some(MouseButton::Left),
            },
            cx,
        );
        self.hover = None;
        cx.notify();
    }

    pub(super) fn release_overlay(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        let Some((id, _)) = self.overlay_drag.clone() else {
            return;
        };
        self.interact_overlay(&id, position, OverlayGesture::Release, cx);
        cx.notify();
    }

    pub(super) fn viewport_widgets(&self, cx: &Context<Self>) -> Vec<AnyElement> {
        let hovered = self
            .overlay_hover
            .as_ref()
            .map(|(id, target)| (id.as_str(), *target));
        self.editor
            .overlays(self.viewport.display_camera(), hovered)
            .into_iter()
            .filter(|frame| frame.slot == RenderPassSlot::Overlay)
            .flat_map(|frame| {
                frame.primitives.into_iter().filter_map(move |primitive| {
                    let OverlayPrimitive::Widget(widget) = primitive else {
                        return None;
                    };
                    Some((frame.id.clone(), widget))
                })
            })
            .map(|(id, widget)| {
                let t = self.theme;
                let bounds_cell = self.overlay_bounds.clone();
                let bounds_id = id.clone();
                let press_id = id.clone();
                let lines = widget.lines;
                let dragging = self
                    .overlay_drag
                    .as_ref()
                    .is_some_and(|(owner, _)| owner == &id);
                div()
                    .id(SharedString::from(id))
                    .absolute()
                    .top(px(widget.inset))
                    .right(px(widget.inset))
                    .size(px(widget.size))
                    .aria_label(widget.label)
                    .cursor(if dragging {
                        CursorStyle::ClosedHand
                    } else {
                        CursorStyle::OpenHand
                    })
                    .child(
                        canvas(
                            move |bounds, _, _| {
                                bounds_cell.borrow_mut().insert(bounds_id.clone(), bounds);
                            },
                            move |bounds, _, window, _| {
                                for line in &lines {
                                    let mut path = PathBuilder::stroke(px(line.width));
                                    path.move_to(
                                        bounds.origin + point(px(line.from[0]), px(line.from[1])),
                                    );
                                    path.line_to(
                                        bounds.origin + point(px(line.to[0]), px(line.to[1])),
                                    );
                                    if let Ok(path) = path.build() {
                                        window.paint_path(path, color(t, line.color));
                                    }
                                }
                            },
                        )
                        .size_full(),
                    )
                    .children(widget.circles.into_iter().map(|circle| {
                        row()
                            .absolute()
                            .left(px(circle.center[0] - circle.radius))
                            .top(px(circle.center[1] - circle.radius))
                            .size(px(circle.radius * 2.0))
                            .justify_center()
                            .rounded_full()
                            .border_2()
                            .border_color(color(t, circle.border))
                            .bg(color(t, circle.fill))
                            .text_color(color(t, circle.text))
                            .text_size(px(12.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .children(circle.label)
                            .when(circle.highlighted, |d| d.shadow(focus_ring(t)))
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                            cx.stop_propagation();
                            window.focus(&this.focus, cx);
                            this.hover = None;
                            let outcome = this.interact_overlay(
                                &press_id,
                                event.position,
                                OverlayGesture::Press {
                                    double: event.click_count == 2,
                                },
                                cx,
                            );
                            if outcome
                                .is_some_and(|o| o.navigation == Some(OverlayNavigation::Frame))
                            {
                                this.invoke(Op::Frame, window, cx);
                            }
                            cx.notify();
                        }),
                    )
                    .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                        cx.stop_propagation();
                        this.hover = None;
                        if this.overlay_drag.is_some() {
                            this.move_overlay(event, cx);
                        } else {
                            let hovered = this.overlay_hit(event.position);
                            if hovered != this.overlay_hover {
                                this.overlay_hover = hovered;
                                cx.notify();
                            }
                        }
                    }))
                    .on_hover(cx.listener(|this, hovered, _, cx| {
                        if !hovered {
                            this.overlay_hover = None;
                            cx.notify();
                        }
                    }))
                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                    .into_any_element()
            })
            .collect()
    }
}
