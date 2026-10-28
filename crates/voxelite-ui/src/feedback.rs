//! Compact feedback and progress, including reuse of the ASCII block shimmer.
use crate::{ColorToken as C, Radius, Space, TextSize, Theme, presentation, row, tokens::geometry};
use gpui::{prelude::*, *};
use std::time::Duration;

/// Feedback meaning, with a glyph as well as a color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackKind {
    /// Informational message.
    Info,
    /// Successful operation.
    Success,
    /// Error or destructive outcome.
    Danger,
}
impl FeedbackKind {
    fn color(self) -> C {
        match self {
            Self::Info => C::AccentText,
            Self::Success => C::Success,
            Self::Danger => C::Danger,
        }
    }
    fn glyph(self) -> &'static str {
        match self {
            Self::Info => "circle",
            Self::Success => "check",
            Self::Danger => "close",
        }
    }
}
/// An inline validation or operation message. Never relies on color alone.
pub struct InlineError;
impl InlineError {
    /// Build a wrapping, accessible message. Use Info/Success for non-error feedback.
    pub fn message(message: impl Into<SharedString>, kind: FeedbackKind, theme: Theme) -> Div {
        TextSize::Small
            .apply(row())
            .gap(px(Space::S3.px()))
            .items_start()
            .text_color(theme.color(kind.color()))
            .child(presentation::glyph(kind.glyph()).text_color(theme.color(kind.color())))
            .child(div().flex_1().whitespace_normal().child(message.into()))
    }
}
/// Nonmodal feedback surface. Lifetime and dismissal are controlled by the host.
pub struct Toast;
impl Toast {
    /// Build a floating notification; append an optional dismiss action.
    pub fn message(message: impl Into<SharedString>, kind: FeedbackKind, theme: Theme) -> Div {
        presentation::surface(theme, C::Overlay, Radius::Md)
            .p(px(Space::S4.px()))
            .shadow(crate::tokens::panel_shadow())
            .child(InlineError::message(message, kind, theme))
    }
}
/// Eight-dot inline progress indicator. Respects reduced motion and exact snapshots.
#[derive(IntoElement)]
pub struct Spinner {
    id: ElementId,
    theme: Theme,
    color: C,
    reduced: bool,
    elapsed: Option<Duration>,
}
impl Spinner {
    /// Create a stable keyed spinner that respects GPUI's reduced-motion preference.
    pub fn new(id: impl Into<ElementId>, theme: Theme) -> Self {
        Self {
            id: id.into(),
            theme,
            color: C::TextSecondary,
            reduced: false,
            elapsed: None,
        }
    }
    /// Semantic foreground, for example OnAccent inside a primary action.
    pub fn color(mut self, color: C) -> Self {
        self.color = color;
        self
    }
    /// Additionally freeze the indicator for a host preference. `false` never
    /// overrides GPUI's reduced-motion preference.
    pub fn reduced_motion(mut self, reduced: bool) -> Self {
        self.reduced = reduced;
        self
    }
    /// Exact deterministic capture time; also disables timers.
    pub fn elapsed(mut self, elapsed: Duration) -> Self {
        self.elapsed = Some(elapsed);
        self
    }

    fn frame(&self, elapsed_ms: f64, system_reduced: bool) -> (usize, bool) {
        let reduced = self.reduced || system_reduced;
        let phase = if reduced {
            0
        } else {
            (elapsed_ms / 100.) as usize % 8
        };
        (phase, !reduced && self.elapsed.is_none())
    }
}
impl RenderOnce for Spinner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use crate::ascii_reveal::clock::{Clock, Playback, schedule};
        let clock = window.use_keyed_state(self.id.clone(), cx, |_, _| Clock::new());
        let elapsed = clock.update(cx, |c, _| {
            c.elapsed(
                Playback {
                    start: None,
                    elapsed: self.elapsed,
                },
                true,
                false,
            )
        });
        let (phase, animate) = self.frame(elapsed, cx.reduce_motion());
        div()
            .id(self.id)
            .aria_label("Loading")
            .size(px(geometry::ICON))
            .flex_shrink_0()
            .child(
                canvas(
                    move |bounds, window, cx| {
                        schedule(&clock, bounds, animate, window, cx);
                    },
                    move |bounds, _, window, _| {
                        // Optical construction: eight equally spaced dots in a 16px box.
                        // The radius leaves a full dot's clearance at every edge.
                        let dot = px(Space::S1.px());
                        let radius = (bounds.size.width - dot * 2.) / 2.;
                        for index in 0..8 {
                            let angle = index as f32 * std::f32::consts::TAU / 8.;
                            let origin = bounds.center()
                                + point(radius * angle.cos(), radius * angle.sin())
                                - point(dot / 2., dot / 2.);
                            let opacity = (8 - (index + 8 - phase) % 8) as f32 / 8.;
                            window.paint_quad(quad(
                                Bounds::new(origin, size(dot, dot)),
                                dot / 2.,
                                self.theme.color(self.color).opacity(opacity),
                                px(Space::S0.px()),
                                transparent_black(),
                                BorderStyle::Solid,
                            ));
                        }
                    },
                )
                .size_full(),
            )
    }
}
/// The existing ASCII shimmer is the block-loading component; no second renderer.
pub type Shimmer = crate::AsciiSkeleton;

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn spinner_freezes_for_either_motion_preference_and_never_schedules_snapshots() {
        for system in [false, true] {
            for host in [false, true] {
                let spinner = Spinner::new("loading", Theme::Dark).reduced_motion(host);
                for (ms, phase) in [(0., 0), (99., 0), (100., 1), (799., 7), (800., 0)] {
                    assert_eq!(
                        spinner.frame(ms, system),
                        if system || host {
                            (0, false)
                        } else {
                            (phase, true)
                        }
                    );
                }
                let snapshot = spinner.elapsed(Duration::from_millis(350));
                assert_eq!(
                    snapshot.frame(350., system),
                    (if system || host { 0 } else { 3 }, false)
                );
            }
        }
    }

    #[test]
    fn feedback_has_distinct_severity_cues_and_only_toasts_cast_shadows() {
        for theme in [Theme::Dark, Theme::Light] {
            for (kind, color, glyph) in [
                (FeedbackKind::Info, C::AccentText, "circle"),
                (FeedbackKind::Success, C::Success, "check"),
                (FeedbackKind::Danger, C::Danger, "close"),
            ] {
                let mut inline = InlineError::message("Message", kind, theme);
                assert_eq!(inline.style().text.color, Some(theme.color(color)));
                assert_eq!(inline.style().text.font_size, Some(px(12.).into()));
                assert!(inline.style().box_shadow.is_none());
                assert_eq!(kind.glyph(), glyph);
                let mut toast = Toast::message("Message", kind, theme);
                assert_eq!(
                    toast.style().background,
                    Some(theme.color(C::Overlay).into())
                );
                assert!(toast.style().box_shadow.is_some());
            }
        }
    }
}
