//! Concept A presentation using shared UI components and geometry tokens.
use crate::{
    shortcuts::{ShortcutPlatform, ShortcutSymbol, ShortcutToken, shortcut_tokens},
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
        shortcut_tokens(binding, ShortcutPlatform::current())
            .into_iter()
            .filter(|token| token.text() != "+")
            .map(|token| {
                let token = match token {
                    ShortcutToken::Text(ref text) if text == "Shift" => {
                        ShortcutToken::Symbol(ShortcutSymbol::Shift)
                    }
                    ShortcutToken::Text(ref text) if text == "Enter" => {
                        ShortcutToken::Symbol(ShortcutSymbol::Return)
                    }
                    token => token,
                };
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
