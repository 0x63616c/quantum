//! Common token-only presentation shared by design-system builders.
use crate::{ColorToken as C, Radius, Space, TextSize, Theme, tokens::state};
use gpui::{prelude::*, *};

pub(crate) fn ellipsis(text: impl Into<SharedString>) -> Div {
    div()
        .min_w(px(Space::S0.px()))
        .overflow_hidden()
        .text_ellipsis()
        .whitespace_nowrap()
        .child(text.into())
}
pub(crate) fn surface(theme: Theme, color: C, radius: Radius) -> Div {
    TextSize::Body
        .apply(div())
        .bg(theme.color(color))
        .text_color(theme.color(C::TextPrimary))
        .rounded(px(radius.px()))
        .border_1()
        .border_color(theme.color(C::Border))
}
/// Two concentric shadows leave the 1px surface-colored gap outside the edge.
/// The host surface color is explicit so the offset stays correct on any surface.
pub(crate) fn ring(theme: Theme, surface: C) -> Vec<BoxShadow> {
    [
        (
            state::FOCUS_OFFSET + state::FOCUS_WIDTH,
            theme.color(C::AccentText),
        ),
        (state::FOCUS_OFFSET, theme.color(surface)),
    ]
    .into_iter()
    .map(|(spread, color)| BoxShadow {
        inset: false,
        color,
        offset: Point::default(),
        blur_radius: px(Space::S0.px()),
        spread_radius: px(spread),
    })
    .collect()
}
pub(crate) fn neutral(theme: Theme, base: C, opacity: f32) -> Hsla {
    theme
        .color(base)
        .blend(theme.color(C::OnAccent).opacity(opacity))
}
/// Source-over composition for state previews. GPUI's `blend` preserves the
/// destination alpha, which would make a hovered transparent ghost stay invisible.
pub(crate) fn over(base: Hsla, wash: Hsla, opacity: f32) -> Hsla {
    let base = Rgba::from(base);
    let wash = Rgba::from(wash);
    let alpha = opacity + base.a * (1. - opacity);
    if alpha == 0. {
        return transparent_black();
    }
    let channel = |a, b| (a * base.a * (1. - opacity) + b * opacity) / alpha;
    Rgba {
        r: channel(base.r, wash.r),
        g: channel(base.g, wash.g),
        b: channel(base.b, wash.b),
        a: alpha,
    }
    .into()
}
pub(crate) fn selected(theme: Theme) -> Hsla {
    theme.color(C::Accent).opacity(state::SELECTED)
}
pub(crate) fn glyph(name: impl AsRef<str>) -> Svg {
    crate::icon(name).size(px(crate::tokens::geometry::ICON))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    #[test]
    fn translucent_preview_matches_two_painted_layers_on_both_surfaces() {
        for theme in [Theme::Dark, Theme::Light] {
            for base in [transparent_black(), selected(theme), theme.color(C::Raised)] {
                for opacity in [state::HOVER, state::PRESSED] {
                    let wash = crate::interaction::wash(theme);
                    let composite = over(base, wash, opacity);
                    assert!(composite.a > 0.);
                    for surface in [C::Surface, C::Overlay] {
                        let expected = Rgba::from(
                            theme
                                .color(surface)
                                .blend(base)
                                .blend(wash.opacity(opacity)),
                        );
                        let actual = Rgba::from(theme.color(surface).blend(composite));
                        for (a, b) in [
                            (actual.r, expected.r),
                            (actual.g, expected.g),
                            (actual.b, expected.b),
                        ] {
                            assert!((a - b).abs() < 0.00001);
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn titles_ellipsize_in_a_shrinkable_single_line() {
        let mut title = ellipsis("An arbitrarily long layer or document name");
        let s = title.style();
        assert_eq!(s.min_size.width, Some(px(0.).into()));
        assert_eq!(s.overflow.x, Some(Overflow::Hidden));
        assert!(s.text.text_overflow.is_some());
        assert_eq!(s.text.white_space, Some(WhiteSpace::Nowrap));
    }
}
