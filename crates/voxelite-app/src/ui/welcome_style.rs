//! Launcher-specific geometry from Concept A. Shared tokens/components remain
//! read-only while the UI library is being updated independently.
use crate::{
    shortcuts::{ShortcutPlatform, ShortcutSymbol, ShortcutToken, shortcut_tokens},
    theme::Theme,
};
use gpui::{prelude::*, *};
use voxelite_ui::{
    Kbd, row,
    tokens::{metrics, radius, space, type_size},
};

pub(super) const WIDTH: f32 = 760.;
pub(super) const HEIGHT: f32 = 620.;
pub(super) const SIDEBAR: f32 = 244.;
pub(super) const SIDEBAR_TOP: f32 = 76.;
pub(super) const MARK: f32 = 56.;
pub(super) const NAME_LINE: f32 = 29.;
pub(super) const MAIN_TOP: f32 = 22.;
pub(super) const ACTION: f32 = 34.;
pub(super) const ROW: f32 = 60.;
pub(super) const THUMB_WIDTH: f32 = 60.;
pub(super) const THUMB_HEIGHT: f32 = 46.;
pub(super) const SAMPLE_LARGE: f32 = 196.;
pub(super) const SAMPLE_SMALL: f32 = 124.;
// Existing editor geometry; keep it stable while shared tokens are read-only.
pub(super) const EDITOR_WIDTH: f32 = 1440.;
pub(super) const EDITOR_HEIGHT: f32 = 900.;
pub(super) const EDITOR_MIN_WIDTH: f32 = 1024.;
pub(super) const EDITOR_TRAFFIC_LEFT: f32 = 20.;

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
                            .size(px(metrics::ICON_TINY))
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
