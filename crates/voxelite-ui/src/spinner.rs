//! Compact indeterminate progress indicator.
use crate::tokens::{Theme, metrics, type_size};
use gpui::{prelude::*, *};
use std::time::Duration;

/// A fixed-width braille spinner, animated only while mounted.
/// GPUI automatically freezes the animation when reduced motion is enabled.
#[derive(IntoElement)]
pub struct Spinner {
    id: ElementId,
    theme: Theme,
}

impl Spinner {
    /// Create an indicator with a stable, unique animation ID.
    pub fn new(id: impl Into<ElementId>, theme: Theme) -> Self {
        Self {
            id: id.into(),
            theme,
        }
    }
}

impl RenderOnce for Spinner {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        div()
            .w(px(metrics::ICON_DENSE))
            .flex_none()
            .font_family(type_size::MONO)
            .text_size(px(type_size::BODY))
            .text_color(self.theme.muted())
            .with_animation(
                self.id,
                Animation::new(Duration::from_secs(1))
                    .repeat()
                    .with_max_fps(10.),
                |indicator, progress| {
                    indicator
                        .child(FRAMES[((progress * FRAMES.len() as f32) as usize) % FRAMES.len()])
                },
            )
    }
}
