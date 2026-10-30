//! Model-space pins and docked, revision-aware discussion threads.
use super::*;
use voxelite_core::Origin;
use voxelite_plugin_api::{CommentsAction, CommentsOutcome, CommentsView, ToolGesture};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SidebarTab {
    Session,
    Comments,
    History,
}
pub(super) struct Discussion {
    pub scroll: ScrollHandle,
    pub text: Entity<crate::text_field::TextField>,
}
impl Discussion {
    pub fn new(window: &mut Window, cx: &mut Context<EditorView>) -> Self {
        let text = crate::text_field::TextField::new("Write a comment…", cx);
        cx.subscribe_in(
            &text,
            window,
            |this, _, event: &voxelite_ui::TextFieldEvent, window, cx| {
                if matches!(event, voxelite_ui::TextFieldEvent::Committed) {
                    this.post_comment(cx);
                    window.focus(&this.focus, cx);
                }
            },
        )
        .detach();
        Self {
            scroll: ScrollHandle::new(),
            text,
        }
    }
}
impl EditorView {
    fn discussion_outcome(
        &mut self,
        result: voxelite_app::editor::EditorResult<Option<CommentsOutcome>>,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(Some(out)) => {
                if out.open_panel {
                    self.panel = SidebarTab::Comments;
                }
                if let Some(status) = out.status {
                    self.status = status.into();
                }
                if out.clear_text {
                    self.discussion
                        .text
                        .update(cx, |f, cx| f.set(String::new(), cx));
                }
                if out.scroll_bottom {
                    self.discussion.scroll.scroll_to_bottom();
                }
            }
            Ok(None) => {}
            Err(error) => {
                self.status = error.to_string().into();
            }
        }
        cx.notify();
    }
    pub(super) fn discussion_action(&mut self, action: CommentsAction, cx: &mut Context<Self>) {
        let result = self.editor.comments_action(action);
        self.discussion_outcome(result, cx);
    }
    pub(super) fn begin_comment(&mut self, cx: &mut Context<Self>) {
        self.discussion_action(
            CommentsAction::Command(voxelite_plugin_comments::COMMAND_ID.into()),
            cx,
        );
    }
    pub(super) fn anchor_comment(&mut self, at: Pos, cx: &mut Context<Self>) {
        self.place_comment(None, ToolGesture::ApplyAt(at), cx);
    }
    pub(super) fn place_comment(
        &mut self,
        ray: Option<voxelite_app::editor::Ray>,
        gesture: ToolGesture,
        cx: &mut Context<Self>,
    ) {
        let result = self.editor.comment_gesture(ray, gesture);
        self.discussion_outcome(result, cx);
    }
    fn post_comment(&mut self, cx: &mut Context<Self>) {
        let text = self.discussion.text.read(cx).text().to_owned();
        match self.editor.comments_action(CommentsAction::Post(text)) {
            Ok(Some(outcome)) if !outcome.commands.is_empty() => {
                self.discussion_outcome(Ok(Some(outcome)), cx);
                self.report(Ok(()), "Comment saved", cx);
            }
            Ok(_) => {}
            Err(error) => self.report::<()>(Err(error), "Comment saved", cx),
        }
    }
    pub(super) fn panel_tabs(&self, cx: &Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        TabBar::row(t).children(
            [
                (SidebarTab::Session, "Session"),
                (SidebarTab::Comments, "Comments"),
                (SidebarTab::History, "History"),
            ]
            .into_iter()
            .filter(|(panel, _)| *panel != SidebarTab::Comments || self.editor.comments_enabled())
            .enumerate()
            .map(|(i, (panel, label))| {
                Tab::new(("panel-tab", i), label, t)
                    .selected(self.panel == panel)
                    .build()
                    .flex_1()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        window.focus(&this.focus, cx);
                        this.panel = panel;
                        cx.notify();
                    }))
            }),
        )
    }
    pub(super) fn comments_panel(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let Some(view) = self.editor.comments_view() else {
            return div();
        };
        let comments = &view.comments;
        if let Some(thread) = self.thread_panel(&view, cx) {
            return div()
                .flex_1()
                .min_h(px(space::NONE))
                .flex()
                .flex_col()
                .child(thread);
        }
        Panel::section(t)
            .flex_1()
            .min_h(px(metrics::COMMENTS_MIN_HEIGHT))
            .flex()
            .flex_col()
            .p(px(space::PANEL))
            .gap(px(space::LG))
            .child(
                row()
                    .justify_between()
                    .child(section_title("Comments"))
                    .child(
                        self.control("new-comment", "plus", "Add", cx, |this, _, cx| {
                            this.begin_comment(cx)
                        }),
                    ),
            )
            .child(
                SegmentedControl::inline().children(
                    ["Open", "Resolved", "All"]
                        .into_iter()
                        .enumerate()
                        .map(|(i, label)| {
                            Segment::new(("comment-filter", i), label, t)
                                .icon(["comment", "check-check", "layer"][i])
                                .build()
                                .aria_selected(self.editor.comments_state.filter == i)
                                .when(self.editor.comments_state.filter == i, |d| {
                                    d.bg(t.selected())
                                })
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    window.focus(&this.focus, cx);
                                    this.discussion_action(CommentsAction::Filter(i), cx);
                                    cx.notify();
                                }))
                        }),
                ),
            )
            .child(
                self.control(
                    "region-anchor",
                    "box",
                    if self.editor.comments_state.region {
                        "Anchor: Region"
                    } else {
                        "Anchor: Voxel"
                    },
                    cx,
                    |this, _, cx| {
                        this.discussion_action(CommentsAction::ToggleRegion, cx);
                        cx.notify();
                    },
                )
                .tooltip(hint("Toggle one voxel or a two-corner region", t)),
            )
            .child(
                div()
                    .id("comments-scroll")
                    .flex_1()
                    .min_h(px(space::NONE))
                    .overflow_y_scroll()
                    .when(comments.is_empty(), |d| {
                        d.child(div().py(px(space::XL)).text_color(t.muted()).child(
                            if view.empty {
                                "Pin a discussion to your model."
                            } else {
                                "No threads in this filter."
                            },
                        ))
                    })
                    .children(comments.iter().enumerate().map(|(i, c)| {
                        let id = c.id;
                        let first = &c.messages[0];
                        Button::tile(("thread", i), first.text.clone())
                            .tab_index(0)
                            .focus_visible(move |d| d.bg(t.hover()).shadow(focus_ring(t)))
                            .active(move |d| d.bg(t.selected()))
                            .p(px(space::LG))
                            .mb(px(space::MD))
                            .rounded(px(radius::LG))
                            .border_b_1()
                            .border_color(if self.editor.comments_state.selected == Some(id) {
                                t.accent()
                            } else {
                                t.line()
                            })
                            .cursor_pointer()
                            .hover(move |d| d.bg(t.hover()))
                            .child(
                                row()
                                    .gap(px(space::MD))
                                    .child(
                                        mono(
                                            self.editor
                                                .comments_state
                                                .numbers
                                                .get(&id)
                                                .copied()
                                                .unwrap_or(i + 1)
                                                .to_string(),
                                        )
                                        .text_color(t.accent()),
                                    )
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
                                div()
                                    .mt(px(space::MD))
                                    .child(first.text.chars().take(160).collect::<String>()),
                            )
                            .child(
                                mono(format!(
                                    "{} messages · {:?}",
                                    c.messages.len(),
                                    self.editor.display_snapshot().anchor_state(c)
                                ))
                                .mt(px(space::SM))
                                .text_color(t.muted()),
                            )
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.discussion_action(CommentsAction::Select(Some(id)), cx);
                                this.discussion
                                    .text
                                    .update(cx, |f, cx| f.set(String::new(), cx));
                                window.focus(&this.focus, cx);
                                cx.notify();
                            }))
                    })),
            )
    }
    pub(super) fn comment_pins(&self, cx: &Context<Self>) -> Vec<AnyElement> {
        // Pins are contextual annotations, not unexplained badges over the art.
        if self.panel != SidebarTab::Comments || !self.editor.comments_enabled() {
            return Vec::new();
        }
        let t = self.theme;
        let b = self.viewport.bounds();
        let aspect = f64::from(f32::from(b.size.width) / f32::from(b.size.height).max(1.0));
        self.editor
            .comment_pins(self.viewport.display_camera(), aspect)
            .into_iter()
            .map(|pin| {
                let id = pin.id;
                let i = pin.index;
                let uv = pin.uv;
                let dim = pin.dim;
                Button::custom(("comment-pin", i), pin.tooltip.clone())
                    .absolute()
                    .left(relative(uv[0] as f32))
                    .top(relative(uv[1] as f32))
                    .ml(px(-space::XL))
                    .mt(px(-space::XXXL))
                    .size(px(space::XXXL))
                    .justify_center()
                    .rounded(px(radius::LG))
                    .border_1()
                    .border_color(if pin.selected { t.accent() } else { t.line() })
                    .bg(t.background())
                    .text_color(if dim { t.muted() } else { t.accent() })
                    .shadow_sm()
                    .cursor_pointer()
                    .tooltip(hint(pin.tooltip, t))
                    .when(pin.resolved, |d| {
                        d.child(icon("check").text_color(t.accent()))
                    })
                    .when(!pin.resolved, |d| d.child(pin.number.to_string()))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_mouse_move(|_, _, cx| cx.stop_propagation())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        window.focus(&this.focus, cx);
                        this.panel = SidebarTab::Comments;
                        this.discussion_action(CommentsAction::Select(Some(id)), cx);
                        cx.notify();
                    }))
                    .into_any_element()
            })
            .collect()
    }
    /// Keep discussions in the sidebar so the artwork and its anchors remain visible.
    fn thread_panel(&self, view: &CommentsView, cx: &Context<Self>) -> Option<AnyElement> {
        let t = self.theme;
        let thread = view.thread.as_ref().map(|(i, c)| (*i, c));
        if thread.is_none() && self.editor.comments_state.draft.is_none() {
            return None;
        }
        let preview = self.editor.preview.is_some();
        Some(
            Panel::section(t)
                .flex_1()
                .min_h(px(space::NONE))
                .p(px(space::PANEL))
                .flex()
                .flex_col()
                .gap(px(space::XL))
                .child(
                    row()
                        .justify_between()
                        .child(
                            self.control(
                                "close-thread",
                                "arrow-left",
                                "All comments",
                                cx,
                                |this, window, cx| {
                                    this.discussion_action(CommentsAction::Select(None), cx);
                                    window.focus(&this.focus, cx);
                                    cx.notify();
                                },
                            )
                            .border_0(),
                        )
                        .child(
                            mono(thread.map_or("New".into(), |(i, c)| {
                                format!(
                                    "#{}",
                                    self.editor
                                        .comments_state
                                        .numbers
                                        .get(&c.id)
                                        .copied()
                                        .unwrap_or(i + 1)
                                )
                            }))
                            .text_color(t.muted()),
                        ),
                )
                .when_some(thread, |d, (_, c)| {
                    let id = c.id;
                    let resolved = c.resolved;
                    let anchor = c.anchor.region().ok();
                    d.child(
                        row()
                            .gap(px(space::MD))
                            .child(icon("comment").text_color(t.muted()))
                            .child(section_title("Discussion")),
                    )
                    .child(
                        mono(anchor.map_or_else(
                            || "Detached anchor".into(),
                            |r| format!("X {}   Y {}   Z {}", r.min[0], r.min[1], r.min[2]),
                        ))
                        .text_color(t.muted()),
                    )
                    .child(
                        div()
                            .id("thread-messages")
                            .track_scroll(&self.discussion.scroll)
                            .flex_1()
                            .min_h(px(metrics::THREAD_MIN_HEIGHT))
                            .overflow_y_scroll()
                            .children(c.messages.iter().map(|m| {
                                div()
                                    .mb(px(space::XXL))
                                    .child(
                                        row()
                                            .gap(px(space::MD))
                                            .child(
                                                icon(if m.origin == Origin::Human {
                                                    "user"
                                                } else {
                                                    "spark"
                                                })
                                                .text_color(t.muted()),
                                            )
                                            .child(div().font_weight(FontWeight::MEDIUM).child(
                                                if m.origin == Origin::Human {
                                                    "Human"
                                                } else {
                                                    "Agent"
                                                },
                                            )),
                                    )
                                    .child(
                                        div()
                                            .mt(px(space::MD))
                                            .line_height(px(type_size::LINE_COMMENT))
                                            .child(m.text.clone()),
                                    )
                            })),
                    )
                    .when(!preview, |d| {
                        d.child(self.control(
                            "resolve-comment",
                            if resolved { "rotate-ccw" } else { "check" },
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
                })
                .when(!preview, |d| {
                    d.child(self.discussion.text.clone()).child(
                        self.styled_control(
                            "post-comment",
                            "send",
                            if thread.is_some() {
                                "Reply"
                            } else {
                                "Post comment"
                            },
                            ButtonStyle::Primary,
                            cx,
                            |this, window, cx| {
                                this.post_comment(cx);
                                window.focus(&this.focus, cx);
                            },
                        )
                        .justify_center()
                        .bg(t.text())
                        .text_color(t.background()),
                    )
                })
                .when(preview, |d| {
                    d.child(
                        row()
                            .gap(px(space::MD))
                            .text_color(t.muted())
                            .child(icon("lock").text_color(t.muted()))
                            .child("Read-only history preview"),
                    )
                })
                .into_any_element(),
        )
    }
}
