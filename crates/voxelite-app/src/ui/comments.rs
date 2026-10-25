//! Model-space pins, thread popovers and discussion panel.
use super::*;
use voxelite_core::{Anchor, AnchorState, Origin, Uuid};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Panel {
    Session,
    Comments,
    History,
}
pub(super) struct Discussion {
    pub selected: Option<Uuid>,
    pub placing: bool,
    pub region: bool,
    pub first: Option<Pos>,
    pub draft: Option<Anchor>,
    pub filter: usize,
    pub text: Entity<crate::text_field::TextField>,
}
impl Discussion {
    pub fn new(cx: &mut App) -> Self {
        Self {
            selected: None,
            placing: false,
            region: false,
            first: None,
            draft: None,
            filter: 0,
            text: crate::text_field::TextField::new("Write a comment…", cx),
        }
    }
}
impl EditorView {
    pub(super) fn begin_comment(&mut self, cx: &mut Context<Self>) {
        self.panel = Panel::Comments;
        self.discussion.placing = true;
        self.discussion.first = None;
        self.discussion.selected = None;
        self.discussion.draft = None;
        self.status = if self.discussion.region {
            "Click two corners to anchor a region"
        } else {
            "Click a voxel to anchor a comment"
        }
        .into();
        cx.notify();
    }
    pub(super) fn anchor_comment(&mut self, at: Pos, cx: &mut Context<Self>) {
        let anchor = if self.discussion.region {
            if let Some(first) = self.discussion.first.take() {
                Anchor::Region {
                    min: std::array::from_fn(|a| first[a].min(at[a])),
                    max: std::array::from_fn(|a| first[a].max(at[a]) + 1),
                }
            } else {
                self.discussion.first = Some(at);
                self.status = "Click the opposite corner".into();
                cx.notify();
                return;
            }
        } else {
            Anchor::Voxel { at }
        };
        self.discussion.draft = Some(anchor);
        self.discussion.placing = false;
        self.discussion
            .text
            .update(cx, |f, cx| f.set(String::new(), cx));
        cx.notify();
    }
    fn post_comment(&mut self, cx: &mut Context<Self>) {
        let text = self.discussion.text.read(cx).text.trim().to_string();
        let result = if let Some(anchor) = self.discussion.draft.clone() {
            self.editor.comment(anchor, text).map(|id| {
                self.discussion.selected = Some(id);
                self.discussion.draft = None;
            })
        } else if let Some(id) = self.discussion.selected {
            self.editor.reply(id, text).map(|_| ())
        } else {
            return;
        };
        if result.is_ok() {
            self.discussion
                .text
                .update(cx, |f, cx| f.set(String::new(), cx));
        }
        self.report(result, "Comment saved", cx);
    }
    pub(super) fn panel_tabs(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        row()
            .px(px(16.0))
            .gap(px(4.0))
            .py(px(8.0))
            .border_b_1()
            .border_color(t.line())
            .children(
                [
                    (Panel::Session, "Session"),
                    (Panel::Comments, "Comments"),
                    (Panel::History, "History"),
                ]
                .into_iter()
                .enumerate()
                .map(|(i, (panel, label))| {
                    self.control(("panel-tab", i), label, cx, move |this, _, cx| {
                        this.panel = panel;
                        cx.notify();
                    })
                    .flex_1()
                    .justify_center()
                    .border_0()
                    .when(self.panel == panel, |d| {
                        d.bg(t.selected()).text_color(t.text())
                    })
                }),
            )
    }
    pub(super) fn comments_panel(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let comments = &self.editor.display_snapshot().comments;
        div()
            .flex_1()
            .min_h(px(120.0))
            .flex()
            .flex_col()
            .p(px(20.0))
            .gap(px(12.0))
            .child(
                row()
                    .justify_between()
                    .child(section_title("Comments"))
                    .child(self.control("new-comment", "+ Add", cx, |this, _, cx| {
                        this.begin_comment(cx)
                    })),
            )
            .child(
                row().gap(px(4.0)).children(
                    ["Open", "Resolved", "All"]
                        .into_iter()
                        .enumerate()
                        .map(|(i, label)| {
                            self.control(("comment-filter", i), label, cx, move |this, _, cx| {
                                this.discussion.filter = i;
                                cx.notify();
                            })
                            .border_0()
                            .when(self.discussion.filter == i, |d| d.bg(t.selected()))
                        }),
                ),
            )
            .child(
                self.control(
                    "region-anchor",
                    if self.discussion.region {
                        "Anchor: Region"
                    } else {
                        "Anchor: Voxel"
                    },
                    cx,
                    |this, _, cx| {
                        this.discussion.region = !this.discussion.region;
                        this.discussion.first = None;
                        cx.notify();
                    },
                )
                .tooltip(hint("Toggle one voxel or a two-corner region", t)),
            )
            .child(
                div()
                    .id("comments-scroll")
                    .flex_1()
                    .min_h(px(0.0))
                    .overflow_y_scroll()
                    .when(comments.is_empty(), |d| {
                        d.child(
                            div()
                                .py(px(16.0))
                                .text_color(t.muted())
                                .child("Pin a discussion to your model."),
                        )
                    })
                    .children(
                        comments
                            .iter()
                            .filter(|c| match self.discussion.filter {
                                0 => !c.resolved,
                                1 => c.resolved,
                                _ => true,
                            })
                            .enumerate()
                            .map(|(i, c)| {
                                let id = c.id;
                                let first = &c.messages[0];
                                div()
                                    .id(("thread", i))
                                    .p(px(12.0))
                                    .mb(px(8.0))
                                    .rounded(px(8.0))
                                    .border_1()
                                    .border_color(if self.discussion.selected == Some(id) {
                                        t.accent()
                                    } else {
                                        t.line()
                                    })
                                    .cursor_pointer()
                                    .hover(move |d| d.bg(t.hover()))
                                    .child(
                                        row()
                                            .gap(px(8.0))
                                            .child(
                                                icon(if c.resolved { "check" } else { "comment" })
                                                    .text_color(t.muted()),
                                            )
                                            .child(div().font_weight(FontWeight::MEDIUM).child(
                                                if first.origin == Origin::Human {
                                                    "Human"
                                                } else {
                                                    "Agent"
                                                },
                                            )),
                                    )
                                    .child(
                                        div().mt(px(8.0)).child(
                                            first.text.chars().take(160).collect::<String>(),
                                        ),
                                    )
                                    .child(
                                        mono(format!(
                                            "{} messages · {:?}",
                                            c.messages.len(),
                                            self.editor.display_snapshot().anchor_state(c)
                                        ))
                                        .mt(px(6.0))
                                        .text_color(t.muted()),
                                    )
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.discussion.selected = Some(id);
                                        this.discussion.draft = None;
                                        this.discussion.placing = false;
                                        this.discussion
                                            .text
                                            .update(cx, |f, cx| f.set(String::new(), cx));
                                        window.focus(&this.focus, cx);
                                        cx.notify();
                                    }))
                            }),
                    ),
            )
    }
    pub(super) fn comment_pins(&self, cx: &Context<Self>) -> Vec<AnyElement> {
        let t = self.theme;
        let b = self.viewport.get();
        let aspect = f64::from(f32::from(b.size.width) / f32::from(b.size.height).max(1.0));
        self.editor
            .snapshot()
            .comments
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.resolved || self.discussion.filter != 0)
            .filter_map(|(i, c)| {
                let (uv, occluded) = voxelite_app::comments::project_pin(
                    self.editor.display_snapshot(),
                    c,
                    self.display_camera,
                    aspect,
                )?;
                let id = c.id;
                let dim = occluded
                    || self.editor.display_snapshot().anchor_state(c) != AnchorState::Visible;
                Some(
                    row()
                        .id(("comment-pin", i))
                        .absolute()
                        .left(relative(uv[0] as f32))
                        .top(relative(uv[1] as f32))
                        .ml(px(-14.0))
                        .mt(px(-28.0))
                        .size(px(28.0))
                        .justify_center()
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(if self.discussion.selected == Some(id) {
                            t.accent()
                        } else {
                            t.line()
                        })
                        .bg(t.background())
                        .text_color(t.accent())
                        .opacity(if dim { 0.6 } else { 1.0 })
                        .shadow_sm()
                        .cursor_pointer()
                        .tooltip(hint(
                            format!(
                                "{}{}",
                                if dim { "Occluded / detached · " } else { "" },
                                c.messages[0].text.chars().take(100).collect::<String>()
                            ),
                            t,
                        ))
                        .child(if c.resolved {
                            "✓".to_string()
                        } else {
                            (i + 1).to_string()
                        })
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .on_mouse_move(|_, _, cx| cx.stop_propagation())
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.panel = Panel::Comments;
                            this.discussion.selected = Some(id);
                            this.discussion.draft = None;
                            this.discussion.placing = false;
                            cx.notify();
                        }))
                        .into_any_element(),
                )
            })
            .collect()
    }
    pub(super) fn thread_popover(&self, cx: &Context<Self>) -> Option<AnyElement> {
        let t = self.theme;
        let thread = self.discussion.selected.and_then(|id| {
            self.editor
                .display_snapshot()
                .comments
                .iter()
                .find(|c| c.id == id)
        });
        if thread.is_none() && self.discussion.draft.is_none() {
            return None;
        }
        Some(
            div()
                .absolute()
                .right(px(16.0))
                .top(px(120.0))
                .child(
                    self.floating()
                        .w(px(300.0))
                        .p(px(16.0))
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            row()
                                .justify_between()
                                .child(section_title(if thread.is_some() {
                                    "Discussion"
                                } else {
                                    "New comment"
                                }))
                                .child(self.control(
                                    "close-thread",
                                    "×",
                                    cx,
                                    |this, window, cx| {
                                        this.discussion.selected = None;
                                        this.discussion.draft = None;
                                        window.focus(&this.focus, cx);
                                        cx.notify();
                                    },
                                )),
                        )
                        .when_some(thread, |d, c| {
                            let id = c.id;
                            let resolved = c.resolved;
                            d.child(
                                mono(format!(
                                    "{:?}",
                                    c.anchor.region().map(|r| r.min).unwrap_or_default()
                                ))
                                .text_color(t.muted()),
                            )
                            .child(
                                div()
                                    .id("thread-messages")
                                    .max_h(px(200.0))
                                    .overflow_y_scroll()
                                    .children(c.messages.iter().map(|m| {
                                        div()
                                            .mb(px(12.0))
                                            .child(
                                                div()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(t.muted())
                                                    .child(if m.origin == Origin::Human {
                                                        "Human"
                                                    } else {
                                                        "Agent"
                                                    }),
                                            )
                                            .child(div().mt(px(4.0)).child(m.text.clone()))
                                    })),
                            )
                            .child(self.control(
                                "resolve-comment",
                                if resolved {
                                    "Reopen thread"
                                } else {
                                    "Resolve thread"
                                },
                                cx,
                                move |this, _, cx| {
                                    let r = this.editor.resolve_comment(id, !resolved);
                                    this.report(r, "Thread updated", cx);
                                },
                            ))
                        })
                        .child(self.discussion.text.clone())
                        .child(
                            self.control(
                                "post-comment",
                                if thread.is_some() {
                                    "Reply"
                                } else {
                                    "Post comment"
                                },
                                cx,
                                |this, window, cx| {
                                    this.post_comment(cx);
                                    window.focus(&this.focus, cx);
                                },
                            )
                            .justify_center()
                            .bg(t.text())
                            .text_color(t.background()),
                        ),
                )
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_mouse_move(|_, _, cx| cx.stop_propagation())
                .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                .into_any_element(),
        )
    }
}
