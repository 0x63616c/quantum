//! Design-system field and swatch presentation. Editing stays in TextField.
use crate::{
    ColorToken as C, ControlHeight, FieldFrame, Radius, Space, Swatch, TextSize, Theme,
    presentation, row,
};
use gpui::{prelude::*, *};
impl FieldFrame {
    /// New filled/bordered field; native focus-visible styling belongs to the editor.
    pub fn design(theme: Theme) -> Self {
        Self {
            design: true,
            ..Self::new(theme)
        }
    }
    /// Validation affects the border; place the message below with `InlineError`.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }
    pub(crate) fn build_design(self) -> Div {
        let size = if self.cells.is_some() {
            TextSize::Mono
        } else {
            TextSize::Body
        };
        size.apply(div())
            .h(px(ControlHeight::Default.px()))
            .w_full()
            .px(px(Space::S4.px()))
            .py(px(Space::S2.px()))
            .rounded(px(Radius::Sm.px()))
            .border_1()
            .border_color(
                self.theme
                    .color(if self.invalid { C::Danger } else { C::Border }),
            )
            .bg(self.theme.color(C::Raised))
            .text_color(self.theme.color(C::TextPrimary))
            .cursor_text()
            .when(self.embedded, |d| {
                d.px(px(Space::S0.px())).border_0().bg(transparent_black())
            })
            .when_some(self.cells, |d, cells| {
                d.w(px(cells as f32
                    * TextSize::Mono.px()
                    * crate::tokens::type_size::MONO_ADVANCE
                    + Space::S4.px() * 2.))
            })
    }
}
impl Swatch {
    /// Selected-ring-only swatch. Document colors remain data, never theme tokens.
    pub fn design(theme: Theme, color: impl Into<Hsla>) -> Self {
        Self {
            design: Some(ControlHeight::Default),
            ..Self::new(theme, color)
        }
    }
    /// Set a design-system square size.
    pub fn height(mut self, height: ControlHeight) -> Self {
        self.design = Some(height);
        self
    }
    pub(crate) fn build_design(self, height: ControlHeight) -> Div {
        row()
            .relative()
            .size(px(height.px()))
            .flex_shrink_0()
            .justify_center()
            .rounded(px(Radius::Xs.px()))
            .bg(self.color)
            .border_1()
            .border_color(self.theme.color(C::Border))
            .when(self.selected, |d| {
                d.shadow(presentation::ring(self.theme, C::Surface))
            })
            .when(self.locked, |d| {
                d.child(
                    presentation::glyph("lock")
                        .text_color(Theme::contrast_ink(gpui::Rgba::from(self.color).r > 0.5)),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    #[test]
    fn fields_use_fill_border_and_error_without_replacing_the_editor() {
        let t = Theme::Light;
        let mut normal = FieldFrame::design(t).build();
        let mut invalid = FieldFrame::design(t).invalid(true).build();
        assert_eq!(normal.style().size.height, Some(px(28.).into()));
        assert_eq!(normal.style().background, Some(t.color(C::Raised).into()));
        assert_eq!(normal.style().border_color, Some(t.color(C::Border)));
        assert_eq!(invalid.style().border_color, Some(t.color(C::Danger)));
        assert_eq!(normal.style().text.font_size, Some(px(13.).into()));
    }
    #[test]
    fn swatch_selection_never_changes_document_color_or_dimensions() {
        let t = Theme::Dark;
        let color = t.color(C::Danger);
        let mut plain = Swatch::design(t, color).build();
        let mut selected = Swatch::design(t, color).selected(true).build();
        assert_eq!(plain.style().size, selected.style().size);
        assert_eq!(plain.style().background, selected.style().background);
        assert_eq!(plain.style().border_widths, selected.style().border_widths);
        assert!(selected.style().box_shadow.is_some());
    }
}
