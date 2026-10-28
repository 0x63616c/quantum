//! Layer selection, visibility and locking in the sidebar.
use super::*;
use quantum_ui::ColorToken;
use quantum_ui::Space;

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
            .px(px(PANEL_GUTTER - Space::S3.px()))
            .py(px(if compact {
                Space::S3.px()
            } else {
                Space::S5.px()
            }))
            .border_t_1()
            .border_color(t.color(ColorToken::BorderSubtle))
            .child(
                row()
                    .pl(px(Space::S3.px()))
                    .mb(px(if compact {
                        Space::S2.px()
                    } else {
                        Space::S3.px()
                    }))
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
        quantum_ui::ListRow::new(("layer", i), layer.name.clone(), t)
            .icon("layer")
            .selected(id == self.editor.layer)
            .navigation(true)
            .trailing(
                row()
                    .child(self.layer_visibility(i, id, visible, cx))
                    .child(self.layer_lock(i, id, locked, cx)),
            )
            .build()
            .when(compact, |d| d.py(px(Space::S0.px())))
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
        IconButton::new(
            ("eye", i),
            if visible { "eye" } else { "eye-off" },
            if visible { "Hide layer" } else { "Show layer" },
            t,
        )
        .height(quantum_ui::ControlHeight::Dense)
        .build()
        .anchored_hint(
            if visible { "Hide layer" } else { "Show layer" },
            t,
            TooltipSide::Below,
            &self.overlays,
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
        IconButton::new(
            ("lock", i),
            if locked { "lock" } else { "unlock" },
            if locked { "Unlock layer" } else { "Lock layer" },
            t,
        )
        .height(quantum_ui::ControlHeight::Dense)
        .build()
        .anchored_hint(
            if locked { "Unlock layer" } else { "Lock layer" },
            t,
            TooltipSide::Below,
            &self.overlays,
        )
        .on_click(cx.listener(move |this, _, _, cx| {
            cx.stop_propagation();
            let r = this.editor.update_layer(id, None, Some(!locked));
            this.report(r, "Lock updated", cx);
        }))
    }
}
