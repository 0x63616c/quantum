//! Deterministic review surface for shared controls, only entered with `--show`.
use super::*;
use quantum_ui::{ColorToken as C, ControlState, Space, TextSize, TextStyle, tokens::geometry};

impl EditorView {
    pub(super) fn control_states(&self) -> AnyElement {
        let t = self.theme;
        let states = [
            ("Rest", ControlState::Rest),
            ("Hover", ControlState::Hovered),
            ("Pressed", ControlState::Pressed),
            ("Keyboard focus", ControlState::Focused),
            ("Disabled", ControlState::Disabled),
            ("Loading", ControlState::Loading),
        ];
        let long_label = quantum_ui::ListRow::new(
            "long-label",
            "A deliberately long project name that must truncate without displacing the trailing action or changing the row height",
            t,
        )
        .icon("cube")
        .meta("A long folder path also stays within the available space")
        .trailing(Button::design("open-long-label", "Open", t).build());
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(t.app_scrim())
            .child(
                Popover::design(t)
                    .w(px(geometry::CONTROL_AUDIT_WIDTH))
                    .p(px(Space::S6.px()))
                    .flex()
                    .flex_col()
                    .gap(px(Space::S4.px()))
                    .child(div().text_style(TextSize::Title).child("Control states"))
                    .children(
                        states
                            .into_iter()
                            .enumerate()
                            .map(|(index, (label, state))| state_row(t, index, label, state)),
                    )
                    .child(long_label)
                    .child(quantum_ui::InlineError::message(
                        "This preview uses the production controls and fixed states.",
                        quantum_ui::FeedbackKind::Info,
                        t,
                    )),
            )
            .into_any_element()
    }
}

fn state_row(t: Theme, index: usize, label: &'static str, state: ControlState) -> Div {
    let styles = [
        (ButtonStyle::Primary, "Connect agent"),
        (ButtonStyle::Secondary, "Open project"),
        (ButtonStyle::Ghost, "Duplicate"),
        (ButtonStyle::Danger, "Delete"),
    ];
    row()
        .gap(px(Space::S4.px()))
        .child(
            div()
                .w(px(geometry::STATE_LABEL_WIDTH))
                .flex_shrink_0()
                .text_style(TextSize::Small)
                .child(label),
        )
        .children(
            styles
                .into_iter()
                .enumerate()
                .map(|(style_index, (style, label))| {
                    Button::design(("state-action", index * 4 + style_index), label, t)
                        .style(style)
                        .state(state)
                        .on_surface(C::Overlay)
                        .build()
                        .flex_1()
                }),
        )
}
