//! Controlled HSB controls with pointer capture and commit boundaries.
use crate::{Hsb, Theme, tokens};
use gpui::{prelude::*, *};
use std::{cell::Cell, rc::Rc};

/// A picker change. Pointer motion previews; release and keyboard changes commit.
#[derive(Clone, Copy, Debug)]
pub struct ColorChange {
    /// Full precision HSB, including the hue of grey and black colors.
    pub color: Hsb,
    /// Whether this change ends an edit gesture.
    pub commit: bool,
}
type Listener = Rc<dyn Fn(&ColorChange, &mut Window, &mut App)>;

/// Shared gesture lifetime for a dialog's picker controls.
/// Cancel this before rolling back a preview so a later mouse-up cannot commit it.
#[derive(Clone, Default)]
pub struct ColorGesture(Rc<Cell<bool>>);
impl ColorGesture {
    /// End pointer tracking without emitting a color change.
    pub fn cancel(&self) {
        self.0.set(false);
    }
}

struct Interaction {
    dragging: Rc<Cell<bool>>,
    focus: FocusHandle,
}

/// Saturation/brightness plane at a controlled hue. Arrow keys adjust by 1%.
#[derive(IntoElement)]
pub struct ColorSquare(Picker);
impl ColorSquare {
    /// Create a square. Its parent controls the width and equal height.
    pub fn new(id: impl Into<ElementId>, theme: Theme, color: Hsb) -> Self {
        Self(Picker {
            id: id.into(),
            theme,
            color,
            hue: false,
            change: None,
            gesture: None,
        })
    }
    /// Receive preview and commit changes. Escape bubbles to the dialog owner.
    pub fn on_change(
        mut self,
        change: impl Fn(&ColorChange, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.change = Some(Rc::new(change));
        self
    }
    /// Let the owner synchronously cancel pointer tracking on Escape or dismissal.
    pub fn gesture(mut self, gesture: ColorGesture) -> Self {
        self.0.gesture = Some(gesture);
        self
    }
}
impl RenderOnce for ColorSquare {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.0.render(window, cx)
    }
}

/// A rainbow hue track, with the same preview/commit contract as `ColorSquare`.
#[derive(IntoElement)]
pub struct HueSlider(Picker);
impl HueSlider {
    /// Create a hue slider. Arrow keys adjust by one degree.
    pub fn new(id: impl Into<ElementId>, theme: Theme, color: Hsb) -> Self {
        Self(Picker {
            id: id.into(),
            theme,
            color,
            hue: true,
            change: None,
            gesture: None,
        })
    }
    /// Receive preview and commit changes. Escape bubbles to the dialog owner.
    pub fn on_change(
        mut self,
        change: impl Fn(&ColorChange, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.change = Some(Rc::new(change));
        self
    }
    /// Let the owner synchronously cancel pointer tracking on Escape or dismissal.
    pub fn gesture(mut self, gesture: ColorGesture) -> Self {
        self.0.gesture = Some(gesture);
        self
    }
}
impl RenderOnce for HueSlider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.0.render(window, cx)
    }
}

struct Picker {
    id: ElementId,
    theme: Theme,
    color: Hsb,
    hue: bool,
    change: Option<Listener>,
    gesture: Option<ColorGesture>,
}
fn fill_rgb(rgb: [u8; 3]) -> Rgba {
    gpui::rgb(u32::from(rgb[0]) << 16 | u32::from(rgb[1]) << 8 | u32::from(rgb[2]))
}
fn pointer_color(color: Hsb, hue: bool, position: Point<Pixels>, bounds: Bounds<Pixels>) -> Hsb {
    let x = f32::from(position.x - bounds.left()) / f32::from(bounds.size.width).max(1.);
    let y = f32::from(position.y - bounds.top()) / f32::from(bounds.size.height).max(1.);
    if hue {
        Hsb::new(x * 360., color.s, color.b)
    } else {
        color.at_square(x, y)
    }
}
fn keyboard_color(color: Hsb, hue: bool, key: &str, shift: bool) -> Option<Hsb> {
    let step = if shift { 10. } else { 1. };
    let mut next = color;
    match key {
        "left" | "down" if hue => next.h -= step,
        "right" | "up" if hue => next.h += step,
        "left" => next.s -= step / 100.,
        "right" => next.s += step / 100.,
        "up" => next.b += step / 100.,
        "down" => next.b -= step / 100.,
        "home" if hue => next.h = 0.,
        "end" if hue => next.h = 360.,
        _ => return None,
    }
    Some(Hsb::new(next.h, next.s, next.b))
}
impl Picker {
    fn render(self, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| Interaction {
            dragging: Rc::new(Cell::new(false)),
            focus: cx.focus_handle(),
        });
        let dragging = self
            .gesture
            .map_or_else(|| state.read(cx).dragging.clone(), |gesture| gesture.0);
        let focus = state.read(cx).focus.clone();
        let down = dragging.clone();
        let keyboard_drag = dragging.clone();
        let change = self.change.unwrap_or_else(|| Rc::new(|_, _, _| {}));
        let pointer_change = change.clone();
        let key_change = change.clone();
        let bounds = Rc::new(Cell::new(Bounds::default()));
        let painted = bounds.clone();
        let color = self.color;
        let hue = self.hue;
        let t = self.theme;
        let pointer_focus = focus.clone();
        div()
            .id(self.id)
            .relative()
            .w_full()
            .when(hue, |d| d.h(px(24.)))
            .when(!hue, |d| d.aspect_ratio(1.))
            .flex_shrink_0()
            .tab_index(0)
            .track_focus(&focus)
            .role(Role::Slider)
            .aria_label(if hue {
                "Hue"
            } else {
                "Saturation and brightness"
            })
            .aria_value(if hue {
                format!("{:.0} degrees", color.h)
            } else {
                format!(
                    "Saturation {:.0}%, brightness {:.0}%",
                    color.s * 100.,
                    color.b * 100.
                )
            })
            .rounded(px(tokens::radius::SM))
            .cursor_crosshair()
            .focus_visible(move |d| d.shadow(tokens::focus_ring(t)))
            .child(
                canvas(
                    move |b, _, _| painted.set(b),
                    move |b, _, window, _| {
                        if hue {
                            for i in 0..6 {
                                let start = fill_rgb(Hsb::new(i as f32 * 60., 1., 1.).to_rgb());
                                let end = fill_rgb(Hsb::new((i + 1) as f32 * 60., 1., 1.).to_rgb());
                                window.paint_quad(fill(
                                    Bounds::new(
                                        point(
                                            b.left() + b.size.width * (i as f32 / 6.),
                                            b.top() + px(5.),
                                        ),
                                        size(b.size.width / 6. + px(0.5), b.size.height - px(10.)),
                                    ),
                                    linear_gradient(
                                        90.,
                                        linear_color_stop(start, 0.),
                                        linear_color_stop(end, 1.),
                                    )
                                    .color_space(ColorSpace::Srgb),
                                ));
                            }
                        } else {
                            // Each row interpolates sRGB saturation at a fixed brightness.
                            // This avoids perceptual-gradient interpolation changing the picked RGB.
                            for i in 0..128 {
                                let brightness = 1. - i as f32 / 127.;
                                let start = fill_rgb(Hsb::new(color.h, 0., brightness).to_rgb());
                                let end = fill_rgb(Hsb::new(color.h, 1., brightness).to_rgb());
                                window.paint_quad(fill(
                                    Bounds::new(
                                        point(
                                            b.left(),
                                            b.top() + b.size.height * (i as f32 / 128.),
                                        ),
                                        size(b.size.width, b.size.height / 128. + px(0.5)),
                                    ),
                                    linear_gradient(
                                        90.,
                                        linear_color_stop(start, 0.),
                                        linear_color_stop(end, 1.),
                                    )
                                    .color_space(ColorSpace::Srgb),
                                ));
                            }
                        }
                        // Window listeners keep a gesture alive outside the control, including release.
                        let moving = dragging.clone();
                        let moved = change.clone();
                        window.on_mouse_event(move |e: &MouseMoveEvent, phase, window, cx| {
                            if phase == DispatchPhase::Capture
                                && moving.get()
                                && e.pressed_button == Some(MouseButton::Left)
                            {
                                moved(
                                    &ColorChange {
                                        color: pointer_color(color, hue, e.position, b),
                                        commit: false,
                                    },
                                    window,
                                    cx,
                                );
                            }
                        });
                        let releasing = dragging.clone();
                        let released = change.clone();
                        window.on_mouse_event(move |e: &MouseUpEvent, phase, window, cx| {
                            if phase == DispatchPhase::Capture
                                && e.button == MouseButton::Left
                                && releasing.replace(false)
                            {
                                released(
                                    &ColorChange {
                                        color: pointer_color(color, hue, e.position, b),
                                        commit: true,
                                    },
                                    window,
                                    cx,
                                );
                            }
                        });
                    },
                )
                .absolute()
                .size_full(),
            )
            .child(
                div()
                    .absolute()
                    .left(relative(if hue { color.h / 360. } else { color.s }))
                    .top(relative(if hue { 0.5 } else { 1. - color.b }))
                    .ml(px(-6.))
                    .mt(px(-6.))
                    .size(px(12.))
                    .rounded_full()
                    .border_2()
                    .border_color(white())
                    .shadow(vec![BoxShadow {
                        color: black().opacity(0.7),
                        offset: point(px(0.), px(0.)),
                        blur_radius: px(1.),
                        spread_radius: px(1.),
                        inset: false,
                    }]),
            )
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                down.set(true);
                window.focus(&pointer_focus, cx);
                pointer_change(
                    &ColorChange {
                        color: pointer_color(color, hue, event.position, bounds.get()),
                        commit: false,
                    },
                    window,
                    cx,
                );
                cx.stop_propagation();
            })
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "escape" {
                    keyboard_drag.set(false);
                    return;
                }
                let Some(next) = keyboard_color(color, hue, key, event.keystroke.modifiers.shift)
                else {
                    return;
                };
                key_change(
                    &ColorChange {
                        color: next,
                        commit: true,
                    },
                    window,
                    cx,
                );
                cx.stop_propagation();
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{ColorGesture, Hsb, keyboard_color, pointer_color};
    use gpui::{Bounds, point, px, size};

    #[test]
    fn pointer_coordinates_clamp_outside_bounds_and_keep_hue() {
        let bounds = Bounds::new(point(px(30.), px(50.)), size(px(100.), px(80.)));
        let color = Hsb::new(257., 0.3, 0.6);
        assert_eq!(
            pointer_color(color, false, point(px(80.), px(70.)), bounds),
            Hsb::new(257., 0.5, 0.75)
        );
        assert_eq!(
            pointer_color(color, false, point(px(-10.), px(200.)), bounds),
            Hsb::new(257., 0., 0.)
        );
        assert_eq!(
            pointer_color(color, true, point(px(180.), px(70.)), bounds),
            Hsb::new(360., 0.3, 0.6)
        );
        assert_eq!(
            pointer_color(color, true, point(px(-10.), px(70.)), bounds),
            Hsb::new(0., 0.3, 0.6)
        );
    }

    #[test]
    fn cancellation_is_shared_with_existing_release_listener() {
        let gesture = ColorGesture::default();
        let listener = gesture.0.clone();
        listener.set(true);
        gesture.cancel();
        assert!(!listener.replace(false));
    }

    #[test]
    fn keyboard_hue_never_changes_saturation_or_brightness() {
        let color = Hsb::new(210., 0.5, 0.8);
        for key in ["right", "up"] {
            assert_eq!(
                keyboard_color(color, true, key, false),
                Some(Hsb::new(211., 0.5, 0.8))
            );
        }
        for key in ["left", "down"] {
            assert_eq!(
                keyboard_color(color, true, key, true),
                Some(Hsb::new(200., 0.5, 0.8))
            );
        }
        assert_eq!(
            keyboard_color(color, true, "end", false),
            Some(Hsb::new(360., 0.5, 0.8))
        );
        assert_eq!(
            keyboard_color(Hsb::new(360., 0.5, 0.8), true, "up", true),
            Some(Hsb::new(360., 0.5, 0.8))
        );
        assert_eq!(
            keyboard_color(color, false, "left", true),
            Some(Hsb::new(210., 0.4, 0.8))
        );
        assert_eq!(
            keyboard_color(color, false, "up", true).unwrap().to_rgb(),
            Hsb::new(210., 0.5, 0.9).to_rgb()
        );
        assert_eq!(keyboard_color(color, false, "tab", false), None);
    }
}
