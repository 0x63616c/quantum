//! Palette swatches and brush color controls in the sidebar.
use super::*;
use quantum_ui::ColorToken;
use quantum_ui::Radius;
use quantum_ui::Space;
use quantum_ui::TextSize;
use quantum_ui::TextStyle;
use quantum_ui::tokens::state;

impl EditorView {
    pub(super) fn active_palette(&self) -> Palette {
        self.editor
            .palette_view()
            .map(|v| v.palette)
            .unwrap_or_default()
    }
    pub(super) fn palette_panel(&self, compact: bool, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let palette = self.active_palette();
        let color = self.editor.active_color().rgba;
        // Eight 28px swatches, 8px gaps and 2px focus-ring padding per row.
        let visible_rows = if compact { 2 } else { 5 };
        let swatch_height = visible_rows as f32 * metrics::SWATCH_PITCH - Space::S1.px();
        let overflowing = palette.order.len() > visible_rows * 8;
        let hex = format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
        Panel::section(t)
            .flex_shrink_0()
            .px(px(PANEL_GUTTER))
            .pt(px(if compact {
                Space::S3.px()
            } else {
                Space::S5.px()
            }))
            .pb(px(if compact {
                Space::S3.px()
            } else {
                Space::S5.px()
            }))
            .child(
                row()
                    .justify_between()
                    .mb(px(if compact {
                        Space::S3.px()
                    } else {
                        Space::S4.px()
                    }))
                    .child(
                        quantum_ui::ListRow::frame(
                            "palette-switch",
                            "Choose palette",
                            t,
                            self.palette_open,
                        )
                        .tab_index(0)
                        .h(px(Space::S7.px()))
                        .gap(px(Space::S3.px()))
                        .rounded(px(Radius::Sm.px()))
                        .cursor_pointer()
                        .anchored_hint(
                            "Choose a project or preset palette",
                            t,
                            TooltipSide::Below,
                            &self.overlays,
                        )
                        .child(
                            div()
                                .font_weight(type_size::SEMIBOLD)
                                .child(self.palette_name()),
                        )
                        .child(
                            icon("chevron")
                                .size(px(Space::S5.px()))
                                .text_color(t.color(ColorToken::TextPrimary)),
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            if this.palette_open {
                                this.palette_open = false;
                            } else {
                                this.run(Op::Palette, cx);
                            }
                            cx.notify();
                        })),
                    )
                    .child(
                        div()
                            .text_style(TextSize::Small)
                            .text_color(t.color(ColorToken::TextPrimary))
                            .child(format!("{} colors", palette.order.len())),
                    ),
            )
            .child(self.palette_swatches(&palette, color, swatch_height, cx))
            .child(self.palette_color_footer(compact, overflowing, hex, cx))
    }

    #[expect(clippy::expect_used, reason = "internal invariant: valid palette")]
    fn palette_swatches(
        &self,
        palette: &Palette,
        color: [u8; 4],
        swatch_height: f32,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let t = self.theme;
        div()
            .id("swatch-scroll")
            .track_focus(&self.palette_focus)
            .focus(move |d| {
                d.bg(t
                    .color(ColorToken::AccentText)
                    .opacity(state::PALETTE_FOCUS))
            })
            .track_scroll(&self.swatch_scroll)
            .max_h(px(swatch_height))
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(Space::S3.px()))
                    .p(px(Space::S1.px()))
                    .children(palette.order.iter().enumerate().map(|(i, slot)| {
                        let swatch = palette.get(*slot).expect("valid palette").clone();
                        let c = swatch.rgba;
                        let selected = c == color;
                        let hex = format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2]);
                        let label = swatch
                            .name
                            .as_ref()
                            .map(|name| format!("{name} · {hex}"))
                            .unwrap_or(hex);
                        quantum_ui::Swatch::canvas(("swatch", i), t, color_fill(c), selected)
                            .role(Role::Button)
                            .aria_label(label.clone())
                            .anchored_hint(label, t, TooltipSide::Below, &self.overlays)
                            .when(selected, |d| {
                                d.child(
                                    icon("check")
                                        .size(px(Space::S5.px()))
                                        .text_color(swatch_ink(c)),
                                )
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.palette_request(PaletteAction::SelectSwatch(i), cx);
                                this.error = false;
                                cx.notify();
                            }))
                    })),
            )
    }
}

impl EditorView {
    fn palette_color_footer(
        &self,
        compact: bool,
        overflowing: bool,
        hex: String,
        cx: &Context<Self>,
    ) -> Div {
        let t = self.theme;
        row()
            .mt(px(if compact {
                Space::S3.px()
            } else {
                Space::S4.px()
            }))
            .gap(px(Space::S3.px()))
            .text_style(TextSize::Small)
            .text_color(t.color(ColorToken::TextPrimary))
            .child(hex)
            .child(div().flex_1())
            .when(!overflowing, |d| d.child("Brush color"))
            .when(overflowing, |d| {
                d.child("Scroll colors")
                    .child(
                        self.control("colors-up", "arrow-up", "", cx, |this, _, cx| {
                            let offset = this.swatch_scroll.offset();
                            this.swatch_scroll.set_offset(point(
                                px(Space::S0.px()),
                                offset.y + px(metrics::PALETTE_SCROLL_STEP),
                            ));
                            cx.notify();
                        })
                        .size(px(Space::S6.px()))
                        .px(px(Space::S0.px()))
                        .justify_center()
                        .border_0()
                        .anchored_hint(
                            "Scroll to previous colors",
                            t,
                            TooltipSide::Below,
                            &self.overlays,
                        ),
                    )
                    .child(
                        self.control("colors-down", "chevron", "", cx, |this, _, cx| {
                            let offset = this.swatch_scroll.offset();
                            this.swatch_scroll.set_offset(point(
                                px(Space::S0.px()),
                                offset.y - px(metrics::PALETTE_SCROLL_STEP),
                            ));
                            cx.notify();
                        })
                        .size(px(Space::S6.px()))
                        .px(px(Space::S0.px()))
                        .justify_center()
                        .border_0()
                        .anchored_hint(
                            "Scroll to more colors",
                            t,
                            TooltipSide::Below,
                            &self.overlays,
                        ),
                    )
            })
    }
}
