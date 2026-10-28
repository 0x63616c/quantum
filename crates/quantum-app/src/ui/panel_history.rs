//! Compact recent changes panel and empty state.
use super::*;
use quantum_ui::ColorToken;
use quantum_ui::Space;

impl EditorView {
    pub(super) fn history_panel(&self, compact: bool, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let empty = self.editor.engine.head().is_none();
        Panel::section(t)
            .flex_1()
            .min_h(px(if compact {
                metrics::PALETTE_CHOOSER_TOP
            } else {
                metrics::HISTORY_MIN_HEIGHT
            }))
            .flex()
            .flex_col()
            .px(px(PANEL_GUTTER))
            .py(px(if compact {
                Space::S3.px()
            } else {
                Space::S5.px()
            }))
            .gap(px(Space::S5.px()))
            .child(
                row()
                    .justify_between()
                    .child(section_title("Recent changes"))
                    .child(
                        row()
                            .gap(px(Space::S1.px()))
                            .child(self.button(
                                "history-undo",
                                "undo",
                                "",
                                Op::Undo,
                                ActionPresentation::Icon(TooltipSide::Below),
                                cx,
                            ))
                            .child(self.button(
                                "history-redo",
                                "redo",
                                "",
                                Op::Redo,
                                ActionPresentation::Icon(TooltipSide::Below),
                                cx,
                            )),
                    ),
            )
            .child(
                div()
                    .id("history-scroll")
                    .min_h(px(Space::S0.px()))
                    .flex_1()
                    .overflow_y_scroll()
                    .when(empty, |d| d.child(self.empty_history(compact, cx)))
                    .children(
                        self.editor
                            .engine
                            .history()
                            .rev()
                            .take(100)
                            .enumerate()
                            .map(|(i, entry)| self.history_row(i, entry)),
                    ),
            )
    }

    fn empty_history(&self, compact: bool, cx: &Context<Self>) -> Div {
        let empty = quantum_ui::EmptyState::new("history", "No edits yet", self.theme);
        if compact {
            empty.build()
        } else {
            empty
                .secondary(
                    Button::design("history-guide", "Keyboard guide", self.theme)
                        .style(ButtonStyle::Ghost)
                        .on_click(
                            cx.listener(|this, _, window, cx| this.invoke(Op::Help, window, cx)),
                        ),
                )
                .build()
        }
    }

    fn history_row(&self, i: usize, entry: &quantum_core::HistoryEntry) -> Stateful<Div> {
        let t = self.theme;
        row()
            .id(("history", i))
            .py(px(Space::S4.px()))
            .gap(px(Space::S4.px()))
            .items_start()
            .child(
                icon("check")
                    .mt(px(Space::S1.px()))
                    .size(px(Space::S5.px()))
                    .text_color(t.color(ColorToken::TextSecondary)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(Space::S0.px()))
                    .child(
                        div()
                            .text_ellipsis()
                            .overflow_hidden()
                            .font_weight(type_size::MEDIUM)
                            .child(entry.label.clone()),
                    )
                    .child(
                        mono(format!(
                            "{} · {}",
                            match entry.actor.origin {
                                quantum_core::Origin::Human => "Human",
                                quantum_core::Origin::Agent => "Agent",
                            },
                            history::change_summary(&entry.label, &entry.changes)
                        ))
                        .text_ellipsis()
                        .overflow_hidden()
                        .text_color(t.color(ColorToken::TextSecondary))
                        .mt(px(Space::S2.px())),
                    ),
            )
    }
}
