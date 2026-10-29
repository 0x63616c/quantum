//! Paint-only interaction feedback. Layout, activation and focus stay with the control.
use crate::{ColorToken, Radius, Theme, tokens::state};
use gpui::{prelude::*, *};
use std::time::Instant;

/// Neutral ink must darken light surfaces, including pure white menus.
pub(crate) fn wash(theme: Theme) -> Hsla {
    theme.color(match theme {
        Theme::Dark => ColorToken::OnAccent,
        Theme::Light => ColorToken::TextPrimary,
    })
}

struct Transition {
    from: f32,
    target: f32,
    started: Instant,
    pressed: bool,
    hover: f32,
}
impl Transition {
    fn new() -> Self {
        Self {
            from: 0.,
            target: 0.,
            started: Instant::now(),
            pressed: false,
            hover: state::HOVER,
        }
    }
    fn value(&self, now: Instant, reduced: bool) -> (f32, bool) {
        let duration = if self.target == state::PRESSED {
            state::PRESS_DURATION
        } else {
            state::HOVER_DURATION
        };
        let progress = if reduced {
            1.
        } else {
            (now.saturating_duration_since(self.started).as_secs_f32() / duration.as_secs_f32())
                .min(1.)
        };
        let eased = 1. - (1. - progress).powi(3);
        (
            self.from + (self.target - self.from) * eased,
            progress < 1. && self.from != self.target,
        )
    }
    fn retarget(&mut self, hovered: bool, now: Instant, reduced: bool) -> bool {
        let target = if !hovered {
            0.
        } else if self.pressed {
            state::PRESSED
        } else {
            self.hover
        };
        if self.target == target {
            return false;
        }
        self.from = self.value(now, reduced).0;
        self.target = target;
        self.started = now;
        true
    }
}

/// Absolute underlay, inserted before content. It never changes hit targets or consumes input.
#[derive(IntoElement)]
pub(crate) struct InteractionFill {
    color: Hsla,
    radius: f32,
    hover: f32,
}
impl InteractionFill {
    pub(crate) fn new(color: Hsla) -> Self {
        Self {
            color,
            radius: Radius::Sm.px(),
            hover: state::HOVER,
        }
    }
    pub(crate) fn round(mut self) -> Self {
        self.radius = Radius::Full.px();
        self
    }
    pub(crate) fn primary(mut self, primary: bool) -> Self {
        if primary {
            self.hover = state::PRIMARY_HOVER;
        }
        self
    }
}
impl RenderOnce for InteractionFill {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let transition = window.use_keyed_state("interaction-fill", cx, |_, _| Transition::new());
        canvas(
            |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
            move |bounds, hitbox, window, cx| {
                let now = Instant::now();
                let reduced = cx.reduce_motion();
                let (opacity, active) = transition.update(cx, |state, _| {
                    state.hover = self.hover;
                    state.retarget(hitbox.is_hovered(window), now, reduced);
                    state.value(now, reduced)
                });
                window.paint_quad(quad(
                    bounds,
                    px(self.radius),
                    self.color.opacity(opacity),
                    px(0.),
                    transparent_black(),
                    BorderStyle::Solid,
                ));
                if active && window.is_visible() && bounds.intersects(&window.content_mask().bounds)
                {
                    window.request_animation_frame();
                }
                listen(&transition, hitbox, window);
            },
        )
        .absolute()
        .inset_0()
    }
}

fn listen(state: &Entity<Transition>, hitbox: Hitbox, window: &mut Window) {
    let moving = state.clone();
    let move_hitbox = hitbox.clone();
    window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
        if phase == DispatchPhase::Capture {
            moving.update(cx, |s, cx| {
                // Recover if a release happened outside the window.
                s.pressed &= event.pressed_button == Some(MouseButton::Left);
                if s.retarget(
                    move_hitbox.is_hovered(window),
                    Instant::now(),
                    cx.reduce_motion(),
                ) {
                    cx.notify();
                }
            });
        }
    });
    let down = state.clone();
    let down_hitbox = hitbox.clone();
    window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
        if phase == DispatchPhase::Capture
            && event.button == MouseButton::Left
            && down_hitbox.is_hovered(window)
        {
            down.update(cx, |s, cx| {
                s.pressed = true;
                s.retarget(true, Instant::now(), cx.reduce_motion());
                cx.notify();
            });
        }
    });
    let up = state.clone();
    window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
        if phase == DispatchPhase::Capture && event.button == MouseButton::Left {
            up.update(cx, |s, cx| {
                if s.pressed {
                    s.pressed = false;
                    s.retarget(
                        hitbox.is_hovered(window),
                        Instant::now(),
                        cx.reduce_motion(),
                    );
                    cx.notify();
                }
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use std::time::Duration;

    #[test]
    fn interrupted_hover_and_press_continue_from_the_painted_value_and_settle() {
        let start = Instant::now();
        let at = |ms| start + Duration::from_millis(ms);
        let mut s = Transition::new();
        assert!(!s.value(start, false).1);
        s.retarget(true, start, false);
        let halfway = s.value(at(60), false).0;
        assert!(halfway > 0. && halfway < state::HOVER);
        s.pressed = true;
        s.retarget(true, at(60), false);
        assert_eq!(s.value(at(60), false).0, halfway);
        assert_eq!(s.value(at(140), false), (state::PRESSED, false));
        s.retarget(false, at(140), false);
        assert_eq!(s.value(at(260), false), (0., false));
        // Re-entering while held restores the press, releasing returns to hover.
        s.retarget(true, at(260), false);
        assert_eq!(s.value(at(340), false), (state::PRESSED, false));
        s.pressed = false;
        s.retarget(true, at(340), false);
        assert_eq!(s.value(at(460), false), (state::HOVER, false));
    }

    #[test]
    fn reduced_motion_is_immediate_and_never_requests_a_frame() {
        let mut s = Transition::new();
        let now = Instant::now();
        s.retarget(true, now, true);
        assert_eq!(s.value(now, true), (state::HOVER, false));
        s.pressed = true;
        s.retarget(true, now, true);
        assert_eq!(s.value(now, true), (state::PRESSED, false));
        s.retarget(false, now, true);
        assert_eq!(s.value(now, true), (0., false));
    }
}
