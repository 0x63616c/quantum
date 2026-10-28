//! Concept A presentation using shared UI components and geometry tokens.
use crate::{
    shortcuts::{ShortcutPlatform, ShortcutToken, compact_shortcut_tokens},
    theme::Theme,
};
use gpui::{prelude::*, *};
use voxelite_ui::{
    Kbd, row,
    tokens::{radius, space, type_size},
};

pub(super) use voxelite_ui::tokens::metrics::welcome::*;

pub(super) fn caps(binding: &str, theme: Theme) -> Div {
    row().gap(px(space::COMPACT)).flex_shrink_0().children(
        compact_shortcut_tokens(binding, ShortcutPlatform::current())
            .into_iter()
            .map(|token| {
                let cap = Kbd::frame(theme.muted())
                    .min_w(px(space::SECTION))
                    .h(px(space::SECTION))
                    .px(px(space::COMPACT))
                    .bg(theme.hover())
                    .border_color(theme.line())
                    .font_family(type_size::MONO)
                    .text_color(theme.muted())
                    .text_size(px(type_size::CAPTION))
                    .line_height(px(type_size::LINE_SM));
                match token {
                    ShortcutToken::Text(text) => cap.child(text),
                    ShortcutToken::Symbol(symbol) => cap.child(
                        svg()
                            .path(symbol.path())
                            .size(px(KEY_GLYPH))
                            .text_color(theme.muted()),
                    ),
                }
            }),
    )
}

pub(super) fn header(label: &'static str, theme: Theme) -> Div {
    row()
        .h(px(space::PANEL))
        .flex_shrink_0()
        .px(px(space::MD))
        .mb(px(space::LG))
        .text_size(px(type_size::CAPTION))
        .font_weight(type_size::MEDIUM)
        .text_color(theme.muted())
        .child(label)
}

pub(super) fn thumbnail(
    path: Option<&std::path::PathBuf>,
    theme: Theme,
    large: bool,
    height: f32,
) -> Div {
    div()
        .relative()
        .size_full()
        .overflow_hidden()
        .rounded(px(if large { radius::LG } else { radius::CONTROL }))
        // The reference's quiet raised black is halfway between the base and hover.
        .bg(theme.background().blend(theme.hover().opacity(0.5)))
        .border_1()
        .border_color(theme.line())
        .when_some(path, |d, path| {
            // HTML reference fills 112% of the artwork frame. Give GPUI an
            // explicit height so the PNG's intrinsic aspect cannot stretch it.
            d.child(
                img(path.clone())
                    .absolute()
                    .left(relative(-0.06))
                    .top(px(-height * 0.06))
                    .w(relative(1.12))
                    .h(px(height * 1.12))
                    .object_fit(ObjectFit::Contain),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::Theme;
    use gpui::{Hsla, Rgba};

    fn luminance(color: Hsla) -> f32 {
        let color = Rgba::from(color);
        let linear = |value: f32| {
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
    }

    #[test]
    fn secondary_text_meets_wcag_aa_on_every_welcome_surface() {
        for theme in [Theme::Dark, Theme::Light] {
            for (foreground, background) in [
                (theme.muted(), theme.background()),
                (theme.muted(), theme.hover()),
                (theme.muted(), theme.selected()),
                (
                    theme.accent().blend(theme.text().opacity(0.30)),
                    theme.selected(),
                ),
            ] {
                let a = luminance(foreground);
                let b = luminance(background);
                let contrast = (a.max(b) + 0.05) / (a.min(b) + 0.05);
                assert!(
                    contrast >= 4.5,
                    "{theme:?}: secondary text contrast {contrast}"
                );
            }
        }
    }
}
