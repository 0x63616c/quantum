//! Deterministic review surface for shared controls, only entered with `--show`.
use super::*;
use quantum_ui::{ColorToken as C, ControlState, Space, TextSize, TextStyle, tokens::geometry};

impl EditorView {
    pub(super) fn control_states(&self) -> AnyElement {
        let t = self.theme;
        div().absolute().inset_0().flex().items_center().justify_center().bg(t.app_scrim())
            .child(Popover::design(t).w(px(geometry::CONTROL_AUDIT_WIDTH))
                .p(px(Space::S6.px())).flex().flex_col().gap(px(Space::S4.px()))
                .child(div().text_style(TextSize::Title).child("Control states"))
                .children([
                    ("Rest", ControlState::Rest), ("Hover", ControlState::Hovered),
                    ("Pressed", ControlState::Pressed), ("Keyboard focus", ControlState::Focused),
                    ("Disabled", ControlState::Disabled), ("Loading", ControlState::Loading),
                ].into_iter().enumerate().map(|(index, (label, state))| {
                    row().gap(px(Space::S4.px()))
                        .child(div().w(px(geometry::STATE_LABEL_WIDTH)).flex_shrink_0().text_style(TextSize::Small).child(label))
                        .children([ButtonStyle::Primary, ButtonStyle::Secondary, ButtonStyle::Ghost, ButtonStyle::Danger]
                            .into_iter().enumerate().map(move |(style_index, style)| {
                                Button::design(("state-action", index * 4 + style_index),
                                    ["Connect agent", "Open project", "Duplicate", "Delete"][style_index], t)
                                    .style(style).state(state).on_surface(C::Overlay).build().flex_1()
                            }))
                }))
                .child(quantum_ui::ListRow::new("long-label", "A deliberately long project name that must truncate without displacing the trailing action or changing the row height", t)
                    .icon("cube").meta("A long folder path also stays within the available space")
                    .trailing(Button::design("open-long-label", "Open", t).build()))
                .child(quantum_ui::InlineError::message("This preview uses the production controls and fixed states.", quantum_ui::FeedbackKind::Info, t)))
            .into_any_element()
    }
}
