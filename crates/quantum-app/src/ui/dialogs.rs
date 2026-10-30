//! Modal and popover precedence and their shell surfaces.
use super::*;

impl EditorView {
    pub(super) fn floating(&self) -> Stateful<Div> {
        Popover::canvas(self.theme)
            .build()
            .id("floating-panel")
            .role(Role::Dialog)
            .aria_label("Dialog")
            .track_focus(&self.modal_focus)
            .tab_group()
            .tab_stop(false)
            .occlude()
            .w(px(metrics::DIALOG_WIDTH))
            .max_w(relative(0.95))
            .max_h(relative(0.95))
            .flex()
            .flex_col()
            .text_color(self.theme.text())
            .font_family(type_size::UI)
            .text_size(px(type_size::BODY_LG))
            .p(px(metrics::POPOVER_PADDING))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
    }

    pub(super) fn overlay(&self, window: &Window, cx: &Context<Self>) -> Option<AnyElement> {
        let t = self.theme;
        if self.agent.open && !self.save_flow.dialog_open() {
            return Some(self.agent_popover(cx));
        }
        if self.palettes.open && !self.save_flow.dialog_open() {
            return Some(self.palette_manager(cx));
        }
        if self.save_flow.dialog_open() {
            return Some(self.working_dialog(cx));
        }
        if self.save_flow.needs_confirmation() {
            return Some(self.save_confirmation(cx));
        }
        if let Some((_, InfoContent::About | InfoContent::Acknowledgements)) = &self.info {
            return Some(self.about_dialog(
                matches!(self.info, Some((_, InfoContent::Acknowledgements))),
                cx,
            ));
        }
        if let Some((title, content)) = &self.info {
            return Some(self.info_dialog(title, content, cx));
        }
        if self.commands.is_open() {
            return Some(self.command_dialog(window, cx));
        }
        if self.palette_open {
            return Some(self.palette_chooser(cx));
        }
        self.menu.map(|i| {
            let left = 100.0
                + MENUS
                    .iter()
                    .take(i)
                    .map(|(n, _)| n.len() as f32 * 7.0 + 23.0)
                    .sum::<f32>();
            div()
                .absolute()
                .top(px(MENUBAR_HEIGHT))
                .left(px(left))
                .child(
                    quantum_ui::Menu::canvas("application-menu", t, &self.modal_focus)
                        .w(px(metrics::APP_MENU_WIDTH))
                        .children(
                            crate::actions::menu_items(i, &self.editor.plugin_host())
                                .iter()
                                .enumerate()
                                .map(|(j, op)| {
                                    self.action_row(*op, j, j == self.menu_index, window, cx)
                                }),
                        ),
                )
                .into_any_element()
        })
    }

    fn save_confirmation(&self, cx: &Context<Self>) -> AnyElement {
        let t = self.theme;
        div()
            .absolute()
            .size_full()
            .flex()
            .justify_center()
            .items_center()
            .bg(t.app_scrim())
            .child(
                self.floating()
                    .w(px(metrics::CONFIRM_WIDTH))
                    .p(px(space::XXL))
                    .child(
                        div()
                            .text_size(px(type_size::DIALOG_TITLE))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!(
                                "Do you want to save the changes you made to “{}”?",
                                self.editor.snapshot().name
                            )),
                    )
                    .child(
                        div()
                            .my(px(space::XL))
                            .text_color(t.muted())
                            .child("Your changes will be lost if you don’t save them."),
                    )
                    .child(
                        row().mt(px(space::XXL)).gap(px(space::MD)).children(
                            ["Don’t Save", "Cancel", "Save"]
                                .into_iter()
                                .enumerate()
                                .map(|(i, label)| {
                                    Button::custom(("pending-choice", i), label)
                                        .role(Role::Button)
                                        .aria_label(label)
                                        .aria_keyshortcuts(crate::shortcuts::format_shortcut(
                                            ["secondary-d", "escape", "enter"][i],
                                            ShortcutPlatform::current(),
                                        ))
                                        .h(px(space::XXXL))
                                        .min_w(px(metrics::ACTION_MIN_WIDTH))
                                        .justify_center()
                                        .px(px(space::LG))
                                        .when(i == 0, |d| d.mr_auto())
                                        .rounded(px(radius::CONTROL))
                                        .border_1()
                                        .border_color(if self.save_flow.choice_index() == i {
                                            t.accent()
                                        } else {
                                            t.line()
                                        })
                                        .bg(t.background())
                                        .when(i == 2, |d| {
                                            d.bg(t.accent()).text_color(t.background())
                                        })
                                        .when(self.save_flow.choice_index() == i, |d| {
                                            d.shadow(focus_ring(t))
                                        })
                                        .cursor_pointer()
                                        .hover(move |d| {
                                            d.bg(if i == 2 {
                                                t.accent().opacity(0.88)
                                            } else {
                                                t.hover()
                                            })
                                        })
                                        .child(label)
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.resolve_pending(i, cx)
                                        }))
                                }),
                        ),
                    ),
            )
            .into_any_element()
    }

    fn info_dialog(
        &self,
        title: &'static str,
        content: &InfoContent,
        cx: &Context<Self>,
    ) -> AnyElement {
        let t = self.theme;
        div()
            .absolute()
            .size_full()
            .flex()
            .justify_center()
            .items_center()
            .bg(t.app_scrim())
            .child(
                self.floating()
                    .w(px(if matches!(content, InfoContent::Guide) {
                        metrics::GUIDE_WIDTH
                    } else {
                        metrics::DIALOG_WIDTH
                    }))
                    .p(px(space::XXL))
                    .child(
                        row()
                            .justify_between()
                            .mb(px(space::SECTION))
                            .child(
                                div()
                                    .text_size(px(type_size::DIALOG_TITLE))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(title),
                            )
                            .child(
                                quantum_ui::IconButton::button(
                                    "close-info",
                                    "close",
                                    "Close dialog",
                                    t,
                                )
                                .build()
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.info = None;
                                        cx.notify();
                                    },
                                )),
                            ),
                    )
                    .child(match content {
                        InfoContent::Text(text) => div()
                            .text_color(t.muted())
                            .line_height(px(type_size::LINE_PROSE))
                            .child(text.clone()),
                        InfoContent::Guide => keyboard_guide(t),
                        InfoContent::About | InfoContent::Acknowledgements => {
                            unreachable!("handled above")
                        }
                    }),
            )
            .into_any_element()
    }

    fn command_dialog(&self, window: &Window, cx: &Context<Self>) -> AnyElement {
        let t = self.theme;
        let matches = self.commands.filtered_actions(&self.editor.plugin_host());
        Dialog::scrim(t, false)
            .child(
                Dialog::panel("command-palette", "Search commands", t, &self.modal_focus)
                    .h(px(metrics::COMMAND_HEIGHT))
                    .p(px(space::SM))
                    .flex()
                    .flex_col()
                    .child(
                        quantum_ui::FieldFrame::search_row(t, true)
                            .rounded(px(radius::concentric(
                                radius::XL,
                                metrics::EDGE + space::SM,
                            )))
                            .h(px(metrics::COMMAND_SEARCH_HEIGHT))
                            .flex_shrink_0()
                            .child(icon("search").text_color(t.muted()))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(space::NONE))
                                    .child(self.command_input.clone()),
                            )
                            .child(shortcut("escape").text_color(t.muted())),
                    )
                    .child(
                        div()
                            .id("command-results")
                            .track_scroll(self.commands.scroll())
                            .flex_1()
                            .overflow_y_scroll()
                            .py(px(space::SM))
                            .children(matches.iter().enumerate().map(|(i, op)| {
                                self.action_row(*op, i, self.commands.index() == i, window, cx)
                            }))
                            .when(matches.is_empty(), |d| {
                                d.child(
                                    div()
                                        .p(px(space::XXL))
                                        .text_color(t.muted())
                                        .child("No matching commands"),
                                )
                            }),
                    )
                    .child(
                        row()
                            .h(px(space::XXXL))
                            .px(px(space::FOOTER))
                            .text_color(t.muted())
                            .text_size(px(type_size::BODY))
                            .border_t_1()
                            .border_color(t.line())
                            .child(if self.command_input.read(cx).focused(window) {
                                HintText::default()
                                    .key("enter")
                                    .text(" to run   ·   ")
                                    .key("escape")
                                    .text(" to browse results")
                            } else {
                                HintText::default()
                                    .key("up")
                                    .text(" ")
                                    .key("down")
                                    .text(" to navigate   ·   ")
                                    .key("enter")
                                    .text(" to run   ·   ")
                                    .key("escape")
                                    .text(" to close")
                            }),
                    ),
            )
            .into_any_element()
    }

    fn palette_chooser(&self, cx: &Context<Self>) -> AnyElement {
        let t = self.theme;
        div()
            .absolute()
            .top(px(metrics::PALETTE_CHOOSER_TOP))
            .right(px(space::XL))
            .child(
                self.floating()
                    .w(px(metrics::PALETTE_CHOOSER_WIDTH))
                    .child(
                        row()
                            .px(px(space::LG))
                            .h(px(quantum_ui::tokens::ControlSize::Lg.height()))
                            .justify_between()
                            .child(section_title("Choose palette"))
                            .child(shortcut("escape").text_color(t.muted())),
                    )
                    .child(
                        div()
                            .id("palette-results")
                            .track_scroll(&self.palette_scroll)
                            .max_h(px(metrics::PALETTE_CHOOSER_HEIGHT))
                            .overflow_y_scroll()
                            .children(
                                self.palette_choices()
                                    .iter()
                                    .enumerate()
                                    .map(|(i, choice)| self.palette_choice(i, choice, cx)),
                            ),
                    )
                    .child(
                        div()
                            .px(px(space::LG))
                            .py(px(space::LG))
                            .mt(px(space::XS))
                            .border_t_1()
                            .border_color(t.line())
                            .text_size(px(type_size::BODY))
                            .text_color(t.muted())
                            .child(self.control(
                                "manage-palettes",
                                "palette",
                                "Manage palettes…",
                                cx,
                                |this, window, cx| {
                                    this.palette_open = false;
                                    this.manage_palettes(cx);
                                    window.focus(&this.modal_focus, cx);
                                    cx.notify();
                                },
                            )),
                    ),
            )
            .into_any_element()
    }

    #[expect(clippy::expect_used, reason = "internal invariant: valid palette")]
    fn palette_choice(
        &self,
        i: usize,
        choice: &quantum_plugin_api::PaletteChoice,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let t = self.theme;
        let p = &choice.palette;
        Button::tile(("preset", i), choice.name.clone())
            .p(px(space::INSET))
            .rounded(px(radius::CONTROL))
            .border_l_2()
            .border_color(if i == self.palette_cursor {
                t.accent()
            } else {
                t.background()
            })
            .cursor_pointer()
            .when(i == self.palette_cursor, |d| d.bg(t.hover()))
            .hover(move |d| d.bg(t.hover()))
            .child(
                row()
                    .gap(px(space::MD))
                    .mb(px(space::MD))
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .child(choice.name.clone()),
                    )
                    .when(i == self.palette_index(), |d| {
                        d.child(icon("check").size(px(space::FOOTER)).text_color(t.accent()))
                    }),
            )
            .child(div().flex().flex_wrap().gap(px(space::COMPACT)).children(
                p.order.iter().take(32).map(|slot| {
                    div()
                        .w(px(metrics::ICON_DENSE))
                        .h(px(space::FOOTER))
                        .rounded(px(radius::PREVIEW))
                        .border_1()
                        .border_color(t.swatch_edge())
                        .bg(color_fill(p.get(*slot).expect("valid palette").rgba))
                }),
            ))
            .child(
                div()
                    .mt(px(space::SM))
                    .text_size(px(type_size::BODY))
                    .text_color(t.muted())
                    .child(format!(
                        "{} colors{}{}",
                        p.order.len(),
                        if i != 0 {
                            " · Preset"
                        } else {
                            " · In this project"
                        },
                        if p.order.len() > 32 {
                            " · First 32 shown"
                        } else {
                            ""
                        }
                    )),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.palette_request(PaletteAction::ChoosePreset(i), cx);
                this.palette_open = false;
                cx.notify();
            }))
    }
}
