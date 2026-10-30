//! Local share preview backed by persisted metadata, ready for a hosted backend.
use super::*;
use crate::text_field::TextField;
use voxelite_app::share::{Access, LocalShareBackend, Person, Share, ShareBackend};
use voxelite_core::Uuid;

pub(super) struct Sharing {
    pub open: bool,
    record: Option<Share>,
    pub person: Entity<TextField>,
    expanded: Option<Uuid>,
    copied: bool,
    error: Option<String>,
}
impl Sharing {
    pub fn new(cx: &mut App) -> Self {
        Self {
            open: false,
            record: None,
            person: TextField::new("Person or agent name", cx),
            expanded: None,
            copied: false,
            error: None,
        }
    }
    fn backend() -> LocalShareBackend {
        LocalShareBackend::new(voxelite_app::palettes::data_dir().join("shares.json"))
    }
}
impl EditorView {
    pub(super) fn open_share(&mut self, cx: &mut Context<Self>) {
        match Sharing::backend().get_or_create(self.editor.engine.document().id) {
            Ok(record) => {
                self.sharing.record = Some(record);
                self.sharing.open = true;
                self.sharing.copied = false;
                self.sharing.expanded = None;
                self.sharing.error = None;
                self.sharing
                    .person
                    .update(cx, |field, cx| field.set(String::new(), cx));
            }
            Err(error) => self.report::<()>(Err(error), "", cx),
        }
        cx.notify();
    }
    /// Publish UI state only after the backend accepts and persists the change.
    fn edit_share(&mut self, cx: &mut Context<Self>, edit: impl FnOnce(&mut Share)) -> bool {
        let Some(mut share) = self.sharing.record.clone() else {
            return false;
        };
        edit(&mut share);
        let result = Sharing::backend().save(share.clone());
        let saved = result.is_ok();
        self.sharing.error = result.as_ref().err().map(ToString::to_string);
        if saved {
            self.sharing.record = Some(share);
        }
        self.report(result, "Local share settings saved", cx);
        saved
    }
    pub(super) fn add_share_person(&mut self, cx: &mut Context<Self>) {
        let name = self.sharing.person.read(cx).text.trim().to_string();
        if self.edit_share(cx, |share| {
            share.people.push(Person {
                id: Uuid::new_v4(),
                name,
                access: Access::View,
            });
        }) {
            self.sharing
                .person
                .update(cx, |field, cx| field.set(String::new(), cx));
        }
    }
    fn share_access_choices(
        &self,
        selected: Access,
        person: Option<Uuid>,
        cx: &Context<Self>,
    ) -> Div {
        let t = self.theme;
        row()
            .w_full()
            .p(px(3.0))
            .gap(px(3.0))
            .rounded(px(7.0))
            .bg(t.hover())
            .children(Access::ALL.into_iter().enumerate().map(|(i, access)| {
                let active = selected == access;
                row()
                    .id((
                        if person.is_some() {
                            "person-access-choice"
                        } else {
                            "link-access-choice"
                        },
                        i,
                    ))
                    .flex_1()
                    .h(px(32.0))
                    .justify_center()
                    .gap(px(6.0))
                    .rounded(px(5.0))
                    .cursor_pointer()
                    .text_color(if active { t.text() } else { t.muted() })
                    .when(active, |d| {
                        d.bg(t.background())
                            .child(icon("check").size(px(14.0)).text_color(t.text()))
                    })
                    .hover(move |d| d.text_color(t.text()))
                    .child(access.label())
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        window.focus(&this.focus, cx);
                        if this.edit_share(cx, |share| {
                            if let Some(id) = person {
                                if let Some(p) = share.people.iter_mut().find(|p| p.id == id) {
                                    p.access = access;
                                }
                            } else {
                                share.access = access;
                            }
                        }) {
                            this.sharing.expanded = None;
                        }
                    }))
            }))
    }
    pub(super) fn share_popover(&self, cx: &Context<Self>) -> AnyElement {
        let t = self.theme;
        let Some(share) = &self.sharing.record else {
            return div().into_any_element();
        };
        div()
            .absolute()
            .top(px(88.0))
            .bottom(px(STATUSBAR_HEIGHT + 12.0))
            .right(px(24.0))
            .w(px(432.0))
            .flex()
            .flex_col()
            .child(
                self.floating()
                    .id("share-panel")
                    .w_full()
                    .max_h_full()
                    .overflow_hidden()
                    .p(px(20.0))
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    .child(
                        row()
                            .flex_shrink_0()
                            .justify_between()
                            .child(section_title("Share this project").text_size(px(16.0)))
                            .child(
                                self.control("close-share", "close", "", cx, |this, _, cx| {
                                    this.sharing.open = false;
                                    cx.notify();
                                })
                                .border_0()
                                .px(px(6.0))
                                .tooltip(hint("Close sharing · Esc", t)),
                            ),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_size(px(12.0))
                            .text_color(t.muted())
                            .child("Local preview — hosted sharing coming soon"),
                    )
                    .child(
                        div()
                            .id("share-content")
                            .min_h(px(0.0))
                            .overflow_y_scroll()
                            .flex()
                            .flex_col()
                            .gap(px(16.0))
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .flex()
                                    .flex_col()
                                    .gap(px(10.0))
                                    .child(
                                        row().justify_between().child("Link access").child(
                                            div()
                                                .text_size(px(12.0))
                                                .text_color(t.muted())
                                                .child("Local link"),
                                        ),
                                    )
                                    .child(self.share_access_choices(share.access, None, cx))
                                    .child(div().text_size(px(12.0)).text_color(t.muted()).child(
                                        match share.access {
                                            Access::View => {
                                                "View the project without making changes."
                                            }
                                            Access::Comment => {
                                                "View the project and join the discussion."
                                            }
                                            Access::Remix => {
                                                "View, comment and create an independent copy."
                                            }
                                        },
                                    )),
                            )
                            .child(
                                row()
                                    .flex_shrink_0()
                                    .h(px(44.0))
                                    .px(px(10.0))
                                    .gap(px(8.0))
                                    .border_1()
                                    .border_color(t.line())
                                    .rounded(px(6.0))
                                    .child(icon("link").text_color(t.muted()))
                                    .child(
                                        div()
                                            .id("share-link")
                                            .flex_1()
                                            .min_w(px(0.0))
                                            .text_size(px(12.0))
                                            .overflow_hidden()
                                            .text_ellipsis()
                                            .tooltip(hint(share.link(), t))
                                            .child(share.link()),
                                    )
                                    .child(
                                        self.control(
                                            "copy-share",
                                            if self.sharing.copied { "check" } else { "copy" },
                                            if self.sharing.copied {
                                                "Copied"
                                            } else {
                                                "Copy"
                                            },
                                            cx,
                                            |this, _, cx| {
                                                if let Some(share) = &this.sharing.record {
                                                    cx.write_to_clipboard(
                                                        ClipboardItem::new_string(share.link()),
                                                    );
                                                    this.sharing.copied = true;
                                                    this.status = "Local share link copied".into();
                                                    this.error = false;
                                                    cx.notify();
                                                }
                                            },
                                        )
                                        .border_0()
                                        .flex_shrink_0(),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .pt(px(16.0))
                                    .border_t_1()
                                    .border_color(t.line())
                                    .flex()
                                    .flex_col()
                                    .gap(px(10.0))
                                    .child(section_title("People & agents"))
                                    .child(div().text_size(px(12.0)).text_color(t.muted()).child(
                                        "Settings stay on this device. No invitations are sent.",
                                    ))
                                    .child(
                                        row()
                                            .h(px(36.0))
                                            .gap(px(10.0))
                                            .child(
                                                row()
                                                    .size(px(28.0))
                                                    .justify_center()
                                                    .rounded_full()
                                                    .bg(t.hover())
                                                    .child(icon("user").text_color(t.muted())),
                                            )
                                            .child(div().flex_1().child("You"))
                                            .child(div().text_color(t.muted()).child("Owner")),
                                    )
                                    .child(
                                        div()
                                            .id("share-people")
                                            .max_h(px(150.0))
                                            .overflow_y_scroll()
                                            .children(share.people.iter().enumerate().map(
                                                |(i, person)| {
                                                    let id = person.id;
                                                    div()
                                                        .mb(px(4.0))
                                                        .child(
                                                            row()
                                                                .gap(px(8.0))
                                                                .h(px(36.0))
                                                                .child(
                                                                    row()
                                                                        .size(px(28.0))
                                                                        .flex_shrink_0()
                                                                        .justify_center()
                                                                        .rounded_full()
                                                                        .bg(t.hover())
                                                                        .child(
                                                                            icon("user")
                                                                                .text_color(
                                                                                    t.muted(),
                                                                                ),
                                                                        ),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .flex_1()
                                                                        .min_w(px(0.0))
                                                                        .overflow_hidden()
                                                                        .text_ellipsis()
                                                                        .child(person.name.clone()),
                                                                )
                                                                .child(
                                                                    self.control(
                                                                        ("person-access", i),
                                                                        "chevron",
                                                                        format!(
                                                                            "Can {}",
                                                                            person
                                                                                .access
                                                                                .label()
                                                                                .to_lowercase()
                                                                        ),
                                                                        cx,
                                                                        move |this, _, cx| {
                                                                            this.sharing.expanded =
                                                                                if this
                                                                                    .sharing
                                                                                    .expanded
                                                                                    == Some(id)
                                                                                {
                                                                                    None
                                                                                } else {
                                                                                    Some(id)
                                                                                };
                                                                            cx.notify();
                                                                        },
                                                                    )
                                                                    .border_0()
                                                                    .flex_shrink_0(),
                                                                )
                                                                .child(
                                                                    self.control(
                                                                        ("person-remove", i),
                                                                        "close",
                                                                        "",
                                                                        cx,
                                                                        move |this, _, cx| {
                                                                            this.edit_share(
                                                                                cx,
                                                                                |share| {
                                                                                    share
                                                                            .people
                                                                            .retain(|p| p.id != id)
                                                                                },
                                                                            );
                                                                        },
                                                                    )
                                                                    .border_0()
                                                                    .px(px(6.0))
                                                                    .tooltip(hint(
                                                                        "Remove local participant",
                                                                        t,
                                                                    )),
                                                                ),
                                                        )
                                                        .when(
                                                            self.sharing.expanded == Some(id),
                                                            |d| {
                                                                d.child(self.share_access_choices(
                                                                    person.access,
                                                                    Some(id),
                                                                    cx,
                                                                ))
                                                            },
                                                        )
                                                },
                                            )),
                                    )
                                    .child(
                                        row()
                                            .gap(px(8.0))
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .min_w(px(0.0))
                                                    .child(self.sharing.person.clone()),
                                            )
                                            .child(self.control(
                                                "add-person",
                                                "plus",
                                                "Add",
                                                cx,
                                                |this, _, cx| this.add_share_person(cx),
                                            )),
                                    )
                                    .when_some(self.sharing.error.clone(), |d, error| {
                                        d.child(
                                            div()
                                                .text_size(px(12.0))
                                                .text_color(t.danger())
                                                .child(error),
                                        )
                                    }),
                            ),
                    )
                    .child(
                        row()
                            .flex_shrink_0()
                            .pt(px(12.0))
                            .border_t_1()
                            .border_color(t.line())
                            .child(self.control(
                                "share-copy",
                                "file-output",
                                "Save a copy…",
                                cx,
                                |this, _, cx| {
                                    this.sharing.open = false;
                                    this.choose_save_path(true, cx);
                                },
                            )),
                    ),
            )
            .into_any_element()
    }
}
