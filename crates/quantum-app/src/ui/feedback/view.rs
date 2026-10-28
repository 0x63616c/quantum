use super::*;

impl FeedbackView {
    fn form(&self, t: Theme, cx: &Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap(px(space::MD))
            .child(div().text_color(t.muted()).child(PRIVACY))
            .when(!self.is_crash(), |d| {
                d.child(
                    row().gap(px(space::SM)).children(
                        [
                            ("feedback-bug", "Bug", FeedbackKind::Bug),
                            ("feedback-feature", "Feature", FeedbackKind::Feature),
                        ]
                        .map(|(id, label, kind)| {
                            let selected =
                                matches!(&self.report, Report::Feedback(r) if r.kind == kind);
                            Button::new(id, label, t)
                                .selected(selected)
                                .build()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if let Report::Feedback(report) = &mut this.report {
                                        report.kind = kind;
                                    }
                                    this.preview = None;
                                    this.error = None;
                                    cx.notify();
                                }))
                        }),
                    ),
                )
            })
            .child(if self.is_crash() {
                "Optional context"
            } else {
                "Description"
            })
            .child(self.input.clone())
            .child(
                Button::new("report-preview", "Update preview", t)
                    .build()
                    .on_click(cx.listener(|this, _, _, cx| this.prepare(cx))),
            )
    }

    fn footer(&self, t: Theme, cx: &Context<Self>) -> Div {
        Dialog::footer(t)
            .child(
                Button::new(
                    "report-close",
                    if self.is_crash() {
                        "Keep for later"
                    } else {
                        "Cancel"
                    },
                    t,
                )
                .variant(ButtonVariant::Ghost)
                .build()
                .on_click(|_, window, _| window.remove_window()),
            )
            .when(self.pending.is_some(), |d| {
                d.child(
                    Button::new("report-delete", "Delete report", t)
                        .variant(ButtonVariant::Ghost)
                        .build()
                        .on_click(cx.listener(|this, _, window, cx| this.delete(window, cx))),
                )
            })
            .when(self.preview.is_some(), |d| {
                d.child(
                    Button::new("report-copy", "Copy", t)
                        .variant(ButtonVariant::Ghost)
                        .build()
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(preview) = &this.preview {
                                cx.write_to_clipboard(ClipboardItem::new_string(preview.clone()));
                            }
                        })),
                )
                .child(
                    Button::new("report-save", "Save…", t)
                        .variant(ButtonVariant::Ghost)
                        .build()
                        .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                )
                .child(
                    Button::new("report-continue", "Continue to GitHub…", t)
                        .variant(ButtonVariant::Primary)
                        .build()
                        .on_click(cx.listener(|this, _, _, cx| this.continue_in_browser(cx))),
                )
            })
    }
}

impl Render for FeedbackView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = settings::theme(
            cx.global::<settings::SettingsState>().values.appearance,
            window,
        );
        self.input.update(cx, |field, cx| field.appearance(t, cx));
        self.sync_menu(window, cx);
        div()
            .size_full()
            .bg(t.surface())
            .text_color(t.text())
            .font_family(type_size::UI)
            .text_size(px(type_size::BODY))
            .flex()
            .flex_col()
            .key_context("ReportWindow")
            .on_action(cx.listener(|this, op: &Op, window, cx| this.menu_action(*op, window, cx)))
            .on_action(|_: &CloseReport, window, _| window.remove_window())
            .tab_group()
            .tab_stop(false)
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|_, event: &KeyDownEvent, window, cx| {
                use crate::shortcuts::EditorKey;
                match navigation_key(&event.keystroke).map(EditorKey::from_key) {
                    Some(EditorKey::Escape) => window.remove_window(),
                    Some(EditorKey::Tab) => {
                        if event.keystroke.modifiers.shift {
                            window.focus_prev(cx);
                        } else {
                            window.focus_next(cx);
                        }
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }))
            .child(Dialog::title(
                if self.is_crash() {
                    "Quantum stopped unexpectedly"
                } else {
                    "Send feedback"
                },
                t,
            ))
            .child(
                div()
                    .id("report-content")
                    .flex_1()
                    .min_h(px(space::NONE))
                    .overflow_y_scroll()
                    .p(px(space::XL))
                    .flex()
                    .flex_col()
                    .gap(px(space::LG))
                    .child(self.form(t, cx))
                    .when_some(self.error.clone(), |d, error| {
                        d.child(div().text_color(t.muted()).child(error))
                    })
                    .child(
                        div()
                            .font_weight(type_size::SEMIBOLD)
                            .child("Report preview · public GitHub issue"),
                    )
                    .child(
                        mono("").text_size(px(type_size::CAPTION)).child(
                            self.preview
                                .clone()
                                .unwrap_or_else(|| "Preview your report before continuing.".into()),
                        ),
                    ),
            )
            .child(self.footer(t, cx))
    }
}
