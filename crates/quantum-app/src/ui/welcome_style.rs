//! Concept A presentation using shared UI components and geometry tokens.
use crate::{
    shortcuts::{ShortcutPlatform, ShortcutToken, compact_shortcut_tokens},
    theme::Theme,
};
use gpui::{prelude::*, *};
use quantum_ui::ColorToken;
use quantum_ui::Radius;
use quantum_ui::Space;
use quantum_ui::TextSize;
use quantum_ui::TextStyle;
use quantum_ui::{Kbd, row, tokens::type_size};

pub(super) use quantum_ui::tokens::metrics::welcome::*;

pub(super) fn caps(binding: &str, theme: Theme) -> Div {
    row().gap(px(Space::S2.px())).flex_shrink_0().children(
        compact_shortcut_tokens(binding, ShortcutPlatform::current())
            .into_iter()
            .map(|token| {
                let cap = Kbd::frame(theme.color(ColorToken::TextSecondary))
                    .min_w(px(Space::S5.px()))
                    .h(px(Space::S5.px()))
                    .px(px(Space::S2.px()))
                    .bg(theme.hover())
                    .border_color(theme.color(ColorToken::BorderSubtle))
                    .font_family(type_size::MONO)
                    .text_color(theme.color(ColorToken::TextSecondary))
                    .text_style(TextSize::Caption);
                match token {
                    ShortcutToken::Text(text) => cap.child(text),
                    ShortcutToken::Symbol(symbol) => cap.child(
                        svg()
                            .path(symbol.path())
                            .size(px(KEY_GLYPH))
                            .text_color(theme.color(ColorToken::TextSecondary)),
                    ),
                }
            }),
    )
}

pub(super) fn header(label: &'static str, theme: Theme) -> Div {
    row()
        .h(px(Space::S5.px()))
        .flex_shrink_0()
        .px(px(Space::S3.px()))
        .mb(px(Space::S4.px()))
        .text_style(TextSize::Caption)
        .font_weight(type_size::MEDIUM)
        .text_color(theme.color(ColorToken::TextSecondary))
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
        .rounded(px(if large {
            Radius::Md.px()
        } else {
            Radius::Sm.px()
        }))
        // The reference's quiet raised black is halfway between the base and hover.
        .bg(theme
            .color(ColorToken::Canvas)
            .blend(theme.hover().opacity(0.5)))
        .border_1()
        .border_color(theme.color(ColorToken::BorderSubtle))
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
    use quantum_ui::ColorToken;

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
                (
                    theme.color(ColorToken::TextSecondary),
                    theme.color(ColorToken::Canvas),
                ),
                (theme.color(ColorToken::TextSecondary), theme.hover()),
                (
                    theme.color(ColorToken::TextSecondary),
                    theme.color(ColorToken::Canvas).blend(theme.selected()),
                ),
                (
                    theme.color(ColorToken::TextSecondary),
                    theme.color(ColorToken::Surface).blend(theme.selected()),
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
