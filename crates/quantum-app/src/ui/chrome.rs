//! Action controls, application menus, titlebar and tool rail.
use super::*;
use quantum_ui::ColorToken;
use quantum_ui::Space;
use quantum_ui::TextSize;
use quantum_ui::TextStyle;

// Explicit presentation; element identifiers are only identity, never style policy.
#[derive(Clone, Copy)]
pub(super) enum ActionPresentation {
    Icon(TooltipSide),
    Rail,
    Mirror,
    Segment,
    Frame,
    Search { style: ButtonStyle, compact: bool },
}

impl EditorView {
    pub(super) fn button(
        &self,
        id: &'static str,
        name: &'static str,
        label: &'static str,
        op: Op,
        presentation: ActionPresentation,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let theme = self.theme;
        let enabled = self.enabled(op);
        // Frame is an action; the camera selector owns persistent view state.
        let selected = self.selected(op) && !matches!(presentation, ActionPresentation::Frame);
        let disabled_reason = self.disabled_reason(op);
        let element = match presentation {
            ActionPresentation::Rail | ActionPresentation::Mirror => {
                let mut rail = IconButton::rail(id, op.icon(), op.label(), theme)
                    .disabled(!enabled)
                    .selected(selected);
                if let ActionPresentation::Mirror = presentation {
                    rail = rail.badge(label);
                }
                rail.build()
            }
            ActionPresentation::Segment => Segment::design(id, label, theme)
                .icon(name)
                .disabled(!enabled)
                .selected(selected)
                .build(),
            ActionPresentation::Icon(_) | ActionPresentation::Frame => {
                IconButton::new(id, name, op.label(), theme)
                    .disabled(!enabled)
                    .selected(selected)
                    .build()
            }
            ActionPresentation::Search { style, .. } => {
                let mut button =
                    Button::design(id, if label.is_empty() { op.label() } else { label }, theme)
                        .style(style)
                        .disabled(!enabled)
                        .selected(selected);
                if matches!(
                    presentation,
                    ActionPresentation::Search { compact: true, .. }
                ) {
                    button = button.variant(ButtonVariant::Icon);
                }
                if !name.is_empty() {
                    button = button.icon(name);
                }
                button.build()
            }
        };
        element
            .aria_keyshortcuts(op.shortcut())
            .aria_label(op.label())
            .anchored_hint(
                if enabled {
                    HintText::from(op.label()).action(op)
                } else {
                    format!("{} · {}", op.label(), disabled_reason).into()
                },
                theme,
                match presentation {
                    ActionPresentation::Mirror | ActionPresentation::Rail => TooltipSide::Right,
                    ActionPresentation::Segment | ActionPresentation::Frame => TooltipSide::Above,
                    ActionPresentation::Icon(side) => side,
                    _ => TooltipSide::Below,
                },
                &self.overlays,
            )
            .when(enabled, |d| {
                d.on_click(cx.listener(move |this, _, window, cx| {
                    this.invoke(op, window, cx);
                }))
            })
    }
    pub(super) fn menubar(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        row()
            .h(px(MENUBAR_HEIGHT))
            .flex_shrink_0()
            .px(px(Space::S6.px()))
            .border_b_1()
            .border_color(t.color(ColorToken::BorderSubtle))
            .gap(px(Space::S2.px()))
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_style(TextSize::Title)
                    .mr(px(Space::S5.px()))
                    .child("Quantum"),
            )
            .children(MENUS.iter().enumerate().map(|(i, (name, _))| {
                Button::design(("menu", i), *name, t)
                    .style(ButtonStyle::Ghost)
                    .selected(self.menu == Some(i))
                    .build()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.menu = if this.menu == Some(i) { None } else { Some(i) };
                        this.menu_index = 0;
                        this.commands.close();
                        cx.notify();
                    }))
            }))
            .child(div().flex_1())
    }
    /// Persistent tool and mirror controls use the same actions as menus and keys.
    pub(super) fn tool_rail(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        Panel::rail(t)
            .children(
                [
                    ("rail-place", Op::Place),
                    ("rail-erase", Op::Erase),
                    ("rail-paint", Op::Paint),
                    ("rail-fill", Op::Fill),
                    ("rail-box", Op::Box),
                    ("rail-picker", Op::Picker),
                ]
                .into_iter()
                .map(|(id, op)| self.button(id, op.icon(), "", op, ActionPresentation::Rail, cx)),
            )
            .child(
                div()
                    .w(px(Space::S6.px()))
                    .h(px(metrics::EDGE))
                    .my(px(Space::S3.px()))
                    .bg(t.color(ColorToken::BorderSubtle)),
            )
            .children(
                [
                    ("rail-mirror-x", "X", Op::MirrorX),
                    ("rail-mirror-y", "Y", Op::MirrorY),
                    ("rail-mirror-z", "Z", Op::MirrorZ),
                ]
                .into_iter()
                .map(|(id, label, op)| {
                    self.button(id, op.icon(), label, op, ActionPresentation::Mirror, cx)
                }),
            )
    }
    pub(super) fn toolbar(&self, compact: bool, cx: &Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        Toolbar::row(t)
            .id("main-titlebar")
            .on_mouse_down(MouseButton::Left, |event, window, _| {
                if event.click_count == 2 {
                    window.zoom_window();
                } else {
                    window.start_window_move();
                }
            })
            .when(cfg!(target_os = "macos"), |d| {
                d.pl(px(metrics::TOOLBAR_TRAFFIC_INSET))
            })
            .child(
                icon("cube")
                    .text_color(t.color(ColorToken::TextSecondary))
                    .size(px(metrics::KEYCAP)),
            )
            .child(
                div()
                    .w(px(if compact {
                        metrics::DOCUMENT_COMPACT_WIDTH
                    } else {
                        metrics::DOCUMENT_WIDTH
                    }))
                    .min_w(px(Space::S0.px()))
                    .flex_shrink_0()
                    .child(
                        row()
                            .gap(px(Space::S2.px()))
                            .text_style(TextSize::Title)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(
                                div()
                                    .min_w(px(Space::S0.px()))
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(format!("{}.voxl", self.editor.display_snapshot().name)),
                            )
                            .when(self.editor.engine.is_dirty(), |d| {
                                d.child(
                                    div()
                                        .flex_shrink_0()
                                        .text_style(TextSize::Body)
                                        .font_weight(FontWeight::NORMAL)
                                        .text_color(t.color(ColorToken::TextSecondary))
                                        .child("— Edited"),
                                )
                            }),
                    ),
            )
            .child(div().flex_1())
            .child(
                self.button(
                    "commands",
                    "search",
                    if compact { "" } else { "Search commands" },
                    Op::Commands,
                    ActionPresentation::Search {
                        style: ButtonStyle::Ghost,
                        compact,
                    },
                    cx,
                )
                .h(px(metrics::RAIL_BUTTON))
                .gap(px(Space::S4.px()))
                .child(action_shortcut(Op::Commands)),
            )
    }
    pub(super) fn action_row(
        &self,
        op: Op,
        index: usize,
        selected: bool,
        window: &Window,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let t = self.theme;
        let enabled = self.enabled(op);
        quantum_ui::MenuItem::new(
            ("action-row", index),
            op.display_label(window.is_fullscreen()),
            t,
        )
        .icon(op.icon())
        .selected(selected)
        .disabled(!enabled)
        .on_click(cx.listener(move |this, _, window, cx| {
            this.invoke(op, window, cx);
        }))
        .build()
        .tab_stop(false)
        .aria_selected(selected)
        .aria_keyshortcuts(op.shortcut())
        .when(selected, |d| d.aria_active_descendant())
        .when(self.selected(op), |d| {
            d.child(icon("check").text_color(t.color(ColorToken::AccentText)))
        })
        .child(
            div()
                .max_w(px(metrics::ACTION_HINT_WIDTH))
                .text_style(TextSize::Caption)
                .text_color(t.color(ColorToken::TextSecondary))
                .when(enabled, |d| d.child(action_shortcut(op)))
                .when(!enabled, |d| {
                    d.whitespace_normal().child(self.disabled_reason(op))
                }),
        )
    }
}
