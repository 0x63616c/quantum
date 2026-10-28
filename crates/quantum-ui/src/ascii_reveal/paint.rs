// Ported from ascii-reveal by Quan Hoang (MIT), https://github.com/quanhoangindex/ascii-reveal — see crates/quantum-ui/NOTICE
//! Cached native shaping and allocation-free recoloring.

use crate::tokens::type_size;
use gpui::{
    App, DecorationRun, Hsla, LineLayout, Pixels, Point, ShapedLine, SharedString, TextAlign,
    TextRun, Window, font, px,
};

pub(super) fn shape(
    text: impl Into<SharedString>,
    font_size: f32,
    advance: Option<f32>,
    window: &Window,
) -> ShapedLine {
    let text = text.into();
    let run = TextRun {
        len: text.len(),
        font: font(type_size::MONO),
        color: Hsla::default(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window
        .text_system()
        .shape_line(text, px(font_size), &[run], advance.map(px))
}

pub(super) fn paint(
    line: &ShapedLine,
    origin: Point<Pixels>,
    height: Pixels,
    color: Hsla,
    window: &mut Window,
    cx: &mut App,
) {
    let decorations = [DecorationRun {
        len: line.len() as u32,
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    }];
    // Glyph failures are recoverable (e.g. a missing fallback face); never panic in paint.
    let _ = LineLayout::paint(
        line,
        origin,
        height,
        TextAlign::Left,
        None,
        &decorations,
        window,
        cx,
    );
}
