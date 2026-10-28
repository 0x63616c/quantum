//! Passive viewport diagnostics; text formatting stays in the portable library.
use super::*;
use voxelite_ui::tokens::{radius, space, type_size};

impl EditorView {
    pub(super) fn debug_overlay(&self) -> Div {
        let meter = self.viewport.frame_meter();
        // A plain Div with no listeners, focus, cursor, tooltip or occlusion
        // creates no mouse hitbox. Viewport gestures pass through the panel.
        div()
            .absolute()
            .top(px(if self.editor.preview.is_some() {
                96.
            } else {
                52.
            }))
            .left(px(24.))
            .p(px(space::SM))
            .rounded(px(radius::SM))
            .bg(self.theme.background().opacity(0.82))
            .text_color(self.theme.text())
            .font_family(type_size::MONO)
            .text_size(px(type_size::CAPTION))
            .line_height(px(type_size::LINE_SM))
            .children(
                voxelite_app::debug_overlay::lines(
                    meter.renderer(),
                    if self.capture_state.is_some() {
                        None
                    } else {
                        meter.average()
                    },
                    self.viewport.frame_stats(),
                )
                .into_iter()
                .map(|line| div().whitespace_nowrap().child(line)),
            )
    }
}
