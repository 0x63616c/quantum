//! Accessible horizontal slider and pure range arithmetic.
use crate::{
    row,
    tokens::{self, ControlSize, Theme, metrics, space},
};
use gpui::{prelude::*, *};
use std::{cell::Cell, rc::Rc};

/// Finite slider domain with a positive keyboard step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderRange {
    min: f64,
    max: f64,
    step: f64,
}
impl SliderRange {
    /// Normalize reversed or degenerate ranges and reject non-finite inputs.
    pub fn new(min: f64, max: f64, step: f64) -> Self {
        let min = if min.is_finite() { min } else { 0. };
        let max = if max.is_finite() { max.max(min) } else { min };
        Self {
            min,
            max,
            step: if step.is_finite() && step > 0. {
                step
            } else {
                1.
            },
        }
    }
    /// Clamp and quantize a value to the nearest step.
    pub fn clamp(self, value: f64) -> f64 {
        if !value.is_finite() {
            return self.min;
        }
        (self.min + ((value.clamp(self.min, self.max) - self.min) / self.step).round() * self.step)
            .clamp(self.min, self.max)
    }
    /// Value at a pointer fraction, clamped to the track.
    pub fn at(self, fraction: f64) -> f64 {
        self.clamp(self.min + fraction.clamp(0., 1.) * (self.max - self.min))
    }
    /// Fraction occupied by a value; degenerate ranges stay at the start.
    pub fn fraction(self, value: f64) -> f32 {
        if self.max == self.min {
            0.
        } else {
            ((self.clamp(value) - self.min) / (self.max - self.min)) as f32
        }
    }
    /// Handle unmodified navigation keys, including Home and End.
    pub fn key(self, value: f64, key: &str) -> Option<f64> {
        Some(match key {
            "left" | "down" => self.clamp(value - self.step),
            "right" | "up" => self.clamp(value + self.step),
            "home" => self.min,
            "end" => self.max,
            _ => return None,
        })
    }
}
type ChangeListener = Rc<dyn Fn(&f64, &mut Window, &mut App)>;

struct SliderInteraction {
    bounds: Rc<Cell<Bounds<Pixels>>>,
    dragging: Rc<Cell<bool>>,
    focus: FocusHandle,
}

/// Controlled slider. The caller owns the value and receives changes in `on_change`.
#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    label: SharedString,
    theme: Theme,
    range: SliderRange,
    value: f64,
    track: Option<Background>,
    change: Option<ChangeListener>,
}
impl Slider {
    /// Create a slider from a label, range and current value.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        theme: Theme,
        range: SliderRange,
        value: f64,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            theme,
            range,
            value: range.clamp(value),
            track: None,
            change: None,
        }
    }
    /// Use any GPUI background, including a linear gradient, for the track.
    pub fn track(mut self, background: impl Into<Background>) -> Self {
        self.track = Some(background.into());
        self
    }
    /// Receive pointer and keyboard changes. Put a value label beside the slider
    /// using `labelled`. Drag state survives the parent's value-change renders.
    pub fn on_change(mut self, change: impl Fn(&f64, &mut Window, &mut App) + 'static) -> Self {
        self.change = Some(Rc::new(change));
        self
    }
    fn build(self, interaction: &SliderInteraction) -> Stateful<Div> {
        let bounds = interaction.bounds.clone();
        let painted = bounds.clone();
        let dragging = interaction.dragging.clone();
        let focus = interaction.focus.clone();
        let pointer_focus = focus.clone();
        let down = dragging.clone();
        let up = dragging.clone();
        let up_out = dragging.clone();
        let change = self.change.unwrap_or_else(|| Rc::new(|_, _, _| {}));
        let pointer_change = change.clone();
        let move_change = change.clone();
        let move_bounds = bounds.clone();
        let range = self.range;
        let value = self.value;
        let t = self.theme;
        row()
            .id(self.id)
            .tab_index(0)
            .track_focus(&focus)
            .role(Role::Slider)
            .aria_label(self.label)
            .aria_numeric_value(value)
            .aria_min_numeric_value(range.min)
            .aria_max_numeric_value(range.max)
            .aria_numeric_value_step(range.step)
            .relative()
            .h(px(ControlSize::Md.height()))
            .flex_1()
            .min_w(px(space::XXXL))
            .px(px(space::MD))
            .cursor_pointer()
            .focus_visible(move |d| d.bg(t.surface()).shadow(tokens::focus_ring(t)))
            .child(
                div()
                    .relative()
                    .w_full()
                    .h(px(metrics::TRACK))
                    .rounded_full()
                    .bg(self
                        .track
                        .unwrap_or_else(|| t.color(crate::ColorToken::Border).into()))
                    .child(
                        canvas(move |b, _, _| painted.set(b), |_, _, _, _| ())
                            .absolute()
                            .size_full(),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(relative(range.fraction(value)))
                            .top(px((metrics::TRACK - metrics::THUMB) / 2.))
                            .ml(px(-metrics::THUMB / 2.))
                            .size(px(metrics::THUMB))
                            .rounded_full()
                            .bg(t.accent_fg())
                            .border_1()
                            .border_color(t.separator())
                            .shadow(tokens::panel_shadow()),
                    ),
            )
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                down.set(true);
                window.focus(&pointer_focus, cx);
                let b = bounds.get();
                pointer_change(
                    &range.at(f64::from(
                        f32::from(event.position.x - b.left()) / f32::from(b.size.width).max(1.),
                    )),
                    window,
                    cx,
                );
                cx.stop_propagation();
            })
            .on_mouse_move(move |event, window, cx| {
                if dragging.get() && event.pressed_button == Some(MouseButton::Left) {
                    let b = move_bounds.get();
                    move_change(
                        &range.at(f64::from(
                            f32::from(event.position.x - b.left())
                                / f32::from(b.size.width).max(1.),
                        )),
                        window,
                        cx,
                    );
                    cx.stop_propagation();
                }
            })
            .on_mouse_up(MouseButton::Left, move |_, _, _| up.set(false))
            .on_mouse_up_out(MouseButton::Left, move |_, _, _| up_out.set(false))
            .on_key_down(move |event, window, cx| {
                if event.keystroke.modifiers == Modifiers::none()
                    && let Some(next) = range.key(value, &event.keystroke.key)
                {
                    change(&next, window, cx);
                    cx.stop_propagation();
                }
            })
    }
    /// Compose a track with a caller-provided value label slot.
    pub fn labelled(track: impl IntoElement, label: impl IntoElement) -> Div {
        row()
            .gap(px(space::MD))
            .child(track)
            .child(div().flex_shrink_0().child(label))
    }
}
impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let interaction = window.use_keyed_state(self.id.clone(), cx, |_, cx| SliderInteraction {
            bounds: Rc::new(Cell::new(Bounds::default())),
            dragging: Rc::new(Cell::new(false)),
            focus: cx.focus_handle().tab_stop(true),
        });
        self.build(interaction.read(cx))
    }
}

#[cfg(test)]
mod tests {
    use super::SliderRange;
    #[test]
    fn range_clamps_quantizes_and_handles_degenerate_input() {
        let r = SliderRange::new(0., 255., 1.);
        assert_eq!(r.at(-2.), 0.);
        assert_eq!(r.at(2.), 255.);
        assert_eq!(r.at(0.5), 128.);
        assert_eq!(r.key(255., "right"), Some(255.));
        assert_eq!(r.key(12., "home"), Some(0.));
        assert_eq!(r.key(12., "end"), Some(255.));
        assert_eq!(r.key(12., "a"), None);
        assert_eq!(SliderRange::new(8., 2., 0.).fraction(3.), 0.);
        assert_eq!(r.clamp(f64::NAN), 0.);
    }
}
