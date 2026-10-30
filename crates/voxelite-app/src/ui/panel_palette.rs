//! Palette swatches and brush color controls in the sidebar.
use super::*;

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
        // Eight 32px swatches, 6px gaps and 2px focus-ring padding per row.
        let visible_rows = if compact { 2 } else { 5 };
        let swatch_height = visible_rows as f32 * metrics::SWATCH_PITCH - space::XXS;
        let overflowing = palette.order.len() > visible_rows * 8;
        let hex = format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
        Panel::section(t)
            .flex_shrink_0()
            .px(px(PANEL_GUTTER))
            .pt(px(if compact { space::MD } else { space::XL }))
            .pb(px(if compact { space::MD } else { space::PANEL }))
            .child(
                row()
                    .justify_between()
                    .mb(px(if compact { space::MD } else { space::INSET }))
                    .child(
                        Button::custom("palette-switch", "Choose palette")
                            .tab_index(0)
                            .focus_visible(move |d| d.bg(t.hover()).shadow(focus_ring(t)))
                            .active(move |d| d.bg(t.selected()))
                            .h(px(space::XXXL))
                            .gap(px(space::MD))
                            .rounded(px(radius::CONTROL))
                            .cursor_pointer()
                            .tooltip(hint("Choose a project or preset palette", t))
                            .hover(move |d| d.bg(t.hover()))
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(self.palette_name()),
                            )
                            .child(icon("chevron").size(px(space::FOOTER)).text_color(t.text()))
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
                            .text_size(px(type_size::BODY))
                            .text_color(t.text())
                            .child(format!("{} colors", palette.order.len())),
                    ),
            )
            .child(self.palette_swatches(&palette, color, swatch_height, cx))
            .child(
                row()
                    .mt(px(if compact { space::MD } else { space::LG }))
                    .gap(px(space::MD))
                    .text_size(px(type_size::BODY))
                    .text_color(t.text())
                    .child(hex)
                    .child(div().flex_1())
                    .when(!overflowing, |d| d.child("Brush color"))
                    .when(overflowing, |d| {
                        d.child("Scroll colors")
                            .child(
                                self.control("colors-up", "arrow-up", "", cx, |this, _, cx| {
                                    let offset = this.swatch_scroll.offset();
                                    this.swatch_scroll.set_offset(point(
                                        px(space::NONE),
                                        offset.y + px(metrics::PALETTE_SCROLL_STEP),
                                    ));
                                    cx.notify();
                                })
                                .size(px(space::XXL))
                                .px(px(space::NONE))
                                .justify_center()
                                .border_0()
                                .tooltip(hint("Scroll to previous colors", t)),
                            )
                            .child(
                                self.control("colors-down", "chevron", "", cx, |this, _, cx| {
                                    let offset = this.swatch_scroll.offset();
                                    this.swatch_scroll.set_offset(point(
                                        px(space::NONE),
                                        offset.y - px(metrics::PALETTE_SCROLL_STEP),
                                    ));
                                    cx.notify();
                                })
                                .size(px(space::XXL))
                                .px(px(space::NONE))
                                .justify_center()
                                .border_0()
                                .tooltip(hint("Scroll to more colors", t)),
                            )
                    }),
            )
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
            .focus(move |d| d.bg(t.accent().opacity(0.22)))
            .track_scroll(&self.swatch_scroll)
            .max_h(px(swatch_height))
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(space::SM))
                    .p(px(space::XXS))
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
                        voxelite_ui::Swatch::canvas(("swatch", i), t, color_fill(c), selected)
                            .role(Role::Button)
                            .aria_label(label.clone())
                            .tooltip(hint(label, t))
                            .when(selected, |d| {
                                d.child(icon("check").size(px(space::XL)).text_color(swatch_ink(c)))
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
