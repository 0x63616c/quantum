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
    /// Create a stable keyed spinner. Set reduced motion from the app preference.
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
    /// Freeze the indicator without scheduling repaint timers.
    pub fn reduced_motion(mut self, reduced: bool) -> Self {
        self.reduced = reduced;
        self
    }
    /// Exact deterministic capture time; also disables timers.
    pub fn elapsed(mut self, elapsed: Duration) -> Self {
        self.elapsed = Some(elapsed);
        self
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
        let phase = if self.reduced {
            0
        } else {
            (elapsed / 100.) as usize % 8
        };
        div()
            .id(self.id)
            .aria_label("Loading")
            .size(px(geometry::ICON))
            .flex_shrink_0()
            .child(
                canvas(
                    move |bounds, window, cx| {
                        schedule(
                            &clock,
                            bounds,
                            !self.reduced && self.elapsed.is_none(),
                            window,
                            cx,
                        );
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
