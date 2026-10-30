//! Action controls, application menus, titlebar and tool rail.
use super::*;

// Explicit presentation; element identifiers are only identity, never style policy.
#[derive(Clone, Copy)]
pub(super) enum ActionPresentation {
    Button(ButtonStyle),
    Icon(TooltipSide),
    Rail,
    Mirror(usize),
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
            ActionPresentation::Rail | ActionPresentation::Mirror(_) => {
                let mut rail = IconButton::rail(id, op.icon(), op.label(), theme)
                    .disabled(!enabled)
                    .selected(selected);
                if let ActionPresentation::Mirror(_) = presentation {
                    rail = rail.badge(label);
                }
                rail.build()
            }
            ActionPresentation::Segment => Segment::new(id, label, theme)
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
            ActionPresentation::Button(style) | ActionPresentation::Search { style, .. } => {
                let mut button =
                    Button::new(id, if label.is_empty() { op.label() } else { label }, theme)
                        .legacy_style(style)
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
                    ActionPresentation::Mirror(row) => TooltipSide::Mirror(row),
                    ActionPresentation::Rail => TooltipSide::Right,
                    ActionPresentation::Segment | ActionPresentation::Frame => TooltipSide::Above,
                    ActionPresentation::Icon(side) => side,
                    _ => TooltipSide::Below,
                },
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                this.invoke(op, window, cx);
            }))
    }
    pub(super) fn menubar(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        row()
            .h(px(MENUBAR_HEIGHT))
            .flex_shrink_0()
            .px(px(space::XXL))
            .border_b_1()
            .border_color(t.line())
            .gap(px(space::COMPACT))
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(px(type_size::TITLE))
                    .mr(px(space::SECTION))
                    .child("Quantum"),
            )
            .children(MENUS.iter().enumerate().map(|(i, (name, _))| {
                Button::custom(("menu", i), *name)
                    .h(px(space::XXXL))
                    .px(px(space::INSET))
                    .rounded(px(radius::CONTROL))
                    .text_color(t.muted())
                    .cursor_pointer()
                    .child(*name)
                    .when(self.menu == Some(i), |d| d.bg(t.hover()))
                    .hover(move |d| d.bg(t.hover()))
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
                    .w(px(space::XXL))
                    .h(px(metrics::EDGE))
                    .my(px(space::SM))
                    .bg(t.line()),
            )
            .children(
                [
                    ("rail-mirror-x", "X", Op::MirrorX),
                    ("rail-mirror-y", "Y", Op::MirrorY),
                    ("rail-mirror-z", "Z", Op::MirrorZ),
                ]
                .into_iter()
                .enumerate()
                .map(|(index, (id, label, op))| {
                    self.button(
                        id,
                        op.icon(),
                        label,
                        op,
                        ActionPresentation::Mirror(index),
                        cx,
                    )
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
            .child(icon("cube").text_color(t.muted()).size(px(metrics::KEYCAP)))
            .child(
                div()
                    .w(px(if compact {
                        metrics::DOCUMENT_COMPACT_WIDTH
                    } else {
                        metrics::DOCUMENT_WIDTH
                    }))
                    .min_w(px(space::NONE))
                    .flex_shrink_0()
                    .child(
                        row()
                            .gap(px(space::GLYPH))
                            .text_size(px(type_size::DOCUMENT))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(
                                div()
                                    .min_w(px(space::NONE))
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(format!("{}.voxl", self.editor.display_snapshot().name)),
                            )
                            .when(self.editor.engine.is_dirty(), |d| {
                                d.child(
                                    div()
                                        .flex_shrink_0()
                                        .text_size(px(type_size::BODY_LG))
                                        .font_weight(FontWeight::NORMAL)
                                        .text_color(t.muted())
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
                .gap(px(space::LG))
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
        quantum_ui::SidebarItem::row(
            ("action-row", index),
            op.display_label(window.is_fullscreen()),
            op.icon(),
            false,
            t,
        )
        .tab_stop(false)
        .role(Role::MenuItem)
        .aria_selected(selected)
        .aria_keyshortcuts(op.shortcut())
        .when(selected, |d| d.bg(t.hover()).aria_active_descendant())
        .text_color(if enabled { t.text() } else { t.disabled() })
        .when(self.selected(op), |d| {
            d.child(
                icon("check")
                    .size(px(metrics::ICON_SM))
                    .text_color(t.accent()),
            )
        })
        .child(
            div()
                .max_w(px(metrics::ACTION_HINT_WIDTH))
                .text_size(px(type_size::CAPTION))
                .text_color(t.muted())
                .when(enabled, |d| d.child(action_shortcut(op)))
                .when(!enabled, |d| {
                    d.whitespace_normal().child(self.disabled_reason(op))
                }),
        )
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(cx.listener(move |this, _, window, cx| {
            this.invoke(op, window, cx);
        }))
    }
}
