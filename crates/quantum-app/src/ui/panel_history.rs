//! Compact recent changes panel and empty state.
use super::*;

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
            .py(px(if compact { space::MD } else { space::XL }))
            .gap(px(space::XL))
            .child(
                row()
                    .justify_between()
                    .child(section_title("Recent changes"))
                    .child(
                        row()
                            .gap(px(space::XXS))
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
                    .min_h(px(space::NONE))
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
        let t = self.theme;
        row()
            .items_start()
            .gap(px(space::LG))
            .py(px(space::XS))
            .child(
                row()
                    .size(px(space::XXXL))
                    .flex_shrink_0()
                    .justify_center()
                    .rounded(px(radius::LG))
                    .bg(t.hover())
                    .child(icon("history").text_color(t.muted())),
            )
            .child(
                div()
                    .flex_1()
                    .child(div().font_weight(FontWeight::MEDIUM).child("No edits yet"))
                    .child(
                        div()
                            .mt(px(space::XS))
                            .text_size(px(type_size::BODY))
                            .line_height(px(type_size::LINE_DENSE))
                            .text_color(t.muted())
                            .child("Your changes will appear here."),
                    )
                    .when(!compact, |d| {
                        d.child(
                            self.button(
                                "history-guide",
                                "",
                                "Keyboard guide",
                                Op::Help,
                                ActionPresentation::Button(ButtonStyle::Ghost),
                                cx,
                            )
                            .mt(px(space::MD))
                            .px(px(space::NONE))
                            .text_color(t.text())
                            .child(
                                icon("arrow-right")
                                    .size(px(space::LG))
                                    .text_color(t.muted()),
                            ),
                        )
                    }),
            )
    }

    fn history_row(&self, i: usize, entry: &quantum_core::HistoryEntry) -> Stateful<Div> {
        let t = self.theme;
        row()
            .id(("history", i))
            .py(px(space::INSET))
            .gap(px(space::LG))
            .items_start()
            .child(
                icon("check")
                    .mt(px(space::XXS))
                    .size(px(space::XL))
                    .text_color(t.muted()),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(space::NONE))
                    .child(
                        div()
                            .text_ellipsis()
                            .overflow_hidden()
                            .font_weight(FontWeight::MEDIUM)
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
                        .text_color(t.muted())
                        .mt(px(space::XS)),
                    ),
            )
    }
}
