//! Layer selection, visibility and locking in the sidebar.
use super::*;

impl EditorView {
    pub(super) fn layers_panel(&self, compact: bool, cx: &Context<Self>) -> Div {
        let t = self.theme;
        Panel::section(t)
            .flex_1()
            .min_h(px(if compact {
                metrics::LAYERS_COMPACT_MIN_HEIGHT
            } else {
                metrics::COLOR_EDITOR_WIDTH
            }))
            .px(px(PANEL_GUTTER - space::MD))
            .py(px(if compact { space::MD } else { space::XL }))
            .border_t_1()
            .border_color(t.line())
            .child(
                row()
                    .pl(px(space::MD))
                    .mb(px(if compact { space::XS } else { space::MD }))
                    .justify_between()
                    .child(section_title("Layers"))
                    .child(self.button(
                        "add-layer",
                        "plus",
                        "",
                        Op::AddLayer,
                        ActionPresentation::Icon(TooltipSide::Below),
                        cx,
                    )),
            )
            .child(
                div()
                    .id("layer-scroll")
                    .max_h(px(if compact {
                        metrics::LAYERS_COMPACT_SCROLL_HEIGHT
                    } else {
                        metrics::LAYERS_SCROLL_HEIGHT
                    }))
                    .overflow_y_scroll()
                    .children(
                        self.editor
                            .display_snapshot()
                            .layers
                            .iter()
                            .rev()
                            .enumerate()
                            .map(|(i, layer)| self.layer_row(i, layer, compact, cx)),
                    ),
            )
    }

    fn layer_row(
        &self,
        i: usize,
        layer: &quantum_core::Layer,
        compact: bool,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let t = self.theme;
        let id = layer.id;
        let visible = layer.visible;
        let locked = layer.locked;
        Button::custom(("layer", i), layer.name.clone())
            .tab_index(0)
            .focus_visible(move |d| d.bg(t.hover()).shadow(focus_ring(t)))
            .active(move |d| d.bg(t.selected()))
            .h(px(if compact {
                quantum_ui::tokens::ControlSize::Md.height()
            } else {
                metrics::RAIL_BUTTON
            }))
            .pl(px(space::SM))
            .pr(px(space::MD))
            .gap(px(space::MD))
            .rounded(px(radius::CONTROL))
            .cursor_pointer()
            .border_l_2()
            .border_color(if id == self.editor.layer {
                t.accent()
            } else {
                t.background()
            })
            .when(id == self.editor.layer, |d| {
                d.bg(t.hover()).font_weight(FontWeight::MEDIUM)
            })
            .hover(move |d| d.bg(t.hover()))
            .text_color(if visible { t.text() } else { t.muted() })
            .child(icon("layer").size(px(space::XL)).text_color(t.muted()))
            .child(
                div()
                    .flex_1()
                    .min_w(px(space::NONE))
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(layer.name.clone()),
            )
            .child(self.layer_visibility(i, id, visible, cx))
            .child(self.layer_lock(i, id, locked, cx))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.editor.layer = id;
                this.editor.box_start = None;
                cx.notify();
            }))
    }

    fn layer_visibility(
        &self,
        i: usize,
        id: quantum_core::Uuid,
        visible: bool,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let t = self.theme;
        IconButton::custom(
            ("eye", i),
            if visible { "Hide layer" } else { "Show layer" },
            quantum_ui::tokens::ControlSize::Md,
        )
        .tab_index(0)
        .focus_visible(move |d| d.bg(t.hover()).shadow(focus_ring(t)))
        .active(move |d| d.bg(t.selected()))
        .w(px(space::XXXL))
        .h(px(space::XXXL))
        .justify_center()
        .rounded(px(radius::CONTROL))
        .tooltip(hint(if visible { "Hide layer" } else { "Show layer" }, t))
        .hover(move |d| d.bg(t.selected()))
        .child(
            icon(if visible { "eye" } else { "eye-off" })
                .size(px(metrics::ICON_DENSE))
                .text_color(t.muted()),
        )
        .on_click(cx.listener(move |this, _, _, cx| {
            cx.stop_propagation();
            let r = this.editor.update_layer(id, Some(!visible), None);
            this.report(r, "Visibility updated", cx);
        }))
    }

    fn layer_lock(
        &self,
        i: usize,
        id: quantum_core::Uuid,
        locked: bool,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let t = self.theme;
        IconButton::custom(
            ("lock", i),
            if locked { "Unlock layer" } else { "Lock layer" },
            quantum_ui::tokens::ControlSize::Md,
        )
        .tab_index(0)
        .focus_visible(move |d| d.bg(t.hover()).shadow(focus_ring(t)))
        .active(move |d| d.bg(t.selected()))
        .w(px(space::XXXL))
        .h(px(space::XXXL))
        .justify_center()
        .rounded(px(radius::CONTROL))
        .tooltip(hint(if locked { "Unlock layer" } else { "Lock layer" }, t))
        .when(locked, |d| d.bg(t.selected()))
        .hover(move |d| d.bg(t.selected()))
        .child(
            icon(if locked { "lock" } else { "unlock" })
                .size(px(metrics::ICON_DENSE))
                .text_color(if locked { t.accent() } else { t.muted() }),
        )
        .on_click(cx.listener(move |this, _, _, cx| {
            cx.stop_propagation();
            let r = this.editor.update_layer(id, None, Some(!locked));
            this.report(r, "Lock updated", cx);
        }))
    }
}
