//! Passive viewport diagnostics; text formatting stays in the portable library.
use super::*;
use quantum_ui::ColorToken;
use quantum_ui::Radius;
use quantum_ui::Space;
use quantum_ui::TextStyle;
use quantum_ui::tokens::state;

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
            .left(px(Space::S6.px()))
            .p(px(Space::S3.px()))
            .rounded(px(Radius::Xs.px()))
            .bg(self
                .theme
                .color(ColorToken::Canvas)
                .opacity(state::DEBUG_OVERLAY))
            .text_color(self.theme.color(ColorToken::TextPrimary))
            .text_style(quantum_ui::TextSize::Mono)
            .children(
                quantum_app::debug_overlay::lines(
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
