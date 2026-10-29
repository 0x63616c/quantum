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
                    + Space::S4.px() * 2.
                    // Border and caret clearance keep a full-width value visible.
                    + Space::S2.px()))
            })
    }
}
impl Swatch {
    /// Keyboard-reachable swatch. Focus adds a contrasting inner stroke so it is
    /// distinguishable even when this swatch already has the selection ring.
    pub fn interactive(
        self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
    ) -> Stateful<Div> {
        let theme = self.theme;
        let ink = swatch_ink(theme, self.color);
        self.build()
            .id(id)
            .tab_index(0)
            .role(Role::Button)
            .aria_label(label)
            .cursor_pointer()
            .focus_visible(move |d| {
                let mut ring = presentation::ring(theme, C::Surface);
                ring.push(BoxShadow {
                    inset: true,
                    color: ink,
                    offset: Point::default(),
                    blur_radius: px(0.),
                    spread_radius: px(crate::tokens::state::FOCUS_WIDTH),
                });
                d.shadow(ring)
            })
    }
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
                d.child(presentation::glyph("lock").text_color(swatch_ink(self.theme, self.color)))
            })
    }
}

/// Keyboard navigation frame for a list or palette grid. The inset ring stays
/// visible inside scroll clips and never changes the content's geometry.
pub struct FocusGroup;
impl FocusGroup {
    /// The host owns arrow navigation, selection and the persistent focus handle.
    pub fn frame(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        theme: Theme,
        focus: &FocusHandle,
    ) -> Stateful<Div> {
        div()
            .id(id)
            .tab_index(0)
            .track_focus(focus)
            .role(Role::ListBox)
            .aria_label(label)
            .rounded(px(Radius::Sm.px()))
            .focus_visible(move |d| {
                d.shadow(vec![BoxShadow {
                    inset: true,
                    color: theme.color(C::AccentText),
                    offset: Point::default(),
                    blur_radius: px(0.),
                    spread_radius: px(crate::tokens::state::FOCUS_WIDTH),
                }])
            })
    }
}

fn swatch_ink(theme: Theme, color: Hsla) -> Hsla {
    // Swatches contain document colors. Choose ink using WCAG relative luminance,
    // including all three channels and the surface beneath translucent samples.
    let color = Rgba::from(theme.color(C::Surface).blend(color));
    let [r, g, b] = [color.r, color.g, color.b].map(|v| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    });
    Theme::contrast_ink(0.2126 * r + 0.7152 * g + 0.0722 * b > 0.179)
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
        let mut hex = FieldFrame::design(t).numeric(7).build();
        assert_eq!(hex.style().size.width, Some(px(78.4).into()));
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
    #[test]
    fn locked_swatch_ink_uses_luminance_instead_of_the_red_channel() {
        for theme in [Theme::Dark, Theme::Light] {
            for color in [0x00ff00, 0x00ffff, 0xffffff] {
                assert_eq!(swatch_ink(theme, rgb(color).into()), black());
            }
            for color in [0x0000ff, 0x800000, 0x000000] {
                assert_eq!(swatch_ink(theme, rgb(color).into()), white());
            }
        }
    }
}
