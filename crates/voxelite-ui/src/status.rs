//! Fixed-width telemetry slots. Formatting never changes the layout width.
use crate::{
    icon, row,
    tokens::{Theme, metrics, space, type_size},
};
use gpui::{prelude::*, *};

/// One sign and ten digits, covering every i32 coordinate without grouping.
pub const COORDINATE_CELLS: usize = 11;
/// Ten digits cover an inclusive dimension spanning the entire i32 domain.
pub const DIMENSION_CELLS: usize = 10;
/// Twenty-nine digits cover the largest inclusive box: 2^96 cells.
pub const COUNT_CELLS: usize = 29;

/// Right-align a value in exactly `cells` Unicode scalar cells. Overflow ends in
/// `+`, keeping the most significant prefix. Missing values use an em dash.
pub fn fixed_slot(value: Option<impl ToString>, cells: usize) -> String {
    if cells == 0 {
        return String::new();
    }
    let value = value.map_or_else(|| "—".into(), |v| v.to_string());
    let len = value.chars().count();
    if len > cells {
        value.chars().take(cells - 1).chain(['+']).collect()
    } else {
        format!("{}{value}", "\u{00a0}".repeat(cells - len))
    }
}
/// Exact coordinate string (no thousands grouping).
pub fn coordinate_slot(value: Option<i32>) -> String {
    fixed_slot(value, COORDINATE_CELLS)
}
/// Logical width for a Geist Mono slot, independent of content.
pub fn slot_width(cells: usize) -> f32 {
    cells as f32 * type_size::CAPTION * type_size::MONO_ADVANCE
}
/// A bottom bar whose children opt into fixed segments or a flexible message area.
pub struct StatusBar;
impl StatusBar {
    /// Build the outer row. Insert the message between fixed left/right clusters.
    pub fn row(theme: Theme) -> Div {
        row()
            .h(px(metrics::STATUS_HEIGHT))
            .flex_shrink_0()
            .border_t_1()
            .border_color(theme.line())
            .px(px(space::LG))
            .gap(px(space::LG))
            .text_size(px(type_size::CAPTION))
            .text_color(theme.muted())
            .overflow_hidden()
    }
}
/// Label and fixed-width value. Labels have their own fixed cells; icon and tooltip
/// hooks do not affect the value width. Text is always Geist Mono.
pub struct StatusSegment {
    label: SharedString,
    value: String,
    cells: usize,
    glyph: Option<SharedString>,
}
impl StatusSegment {
    /// Construct a slot. Values are padded or truncated before rendering.
    pub fn new(label: impl Into<SharedString>, value: Option<impl ToString>, cells: usize) -> Self {
        Self {
            label: label.into(),
            value: fixed_slot(value, cells),
            cells,
            glyph: None,
        }
    }
    /// Add a fixed-size leading icon.
    pub fn icon(mut self, glyph: impl Into<SharedString>) -> Self {
        self.glyph = Some(glyph.into());
        self
    }
    /// Build; add `id` and a tooltip to this row without changing its geometry.
    pub fn build(self) -> Div {
        row()
            .flex_shrink_0()
            .font_family(type_size::MONO)
            .text_size(px(type_size::CAPTION))
            .whitespace_nowrap()
            .when_some(self.glyph, |d, glyph| {
                d.child(icon(glyph).size(px(metrics::ICON_SM)).mr(px(space::XS)))
            })
            .when(!self.label.is_empty(), |d| {
                d.child(
                    div()
                        .w(px(slot_width(self.label.chars().count())))
                        .flex_shrink_0()
                        .child(self.label),
                )
            })
            .child(
                div()
                    .w(px(slot_width(self.cells)))
                    .min_w(px(slot_width(self.cells)))
                    .flex_shrink_0()
                    .overflow_hidden()
                    .text_right()
                    .child(self.value),
            )
    }
}
#[cfg(test)]
mod tests {
    use super::{COORDINATE_CELLS, COUNT_CELLS, DIMENSION_CELLS, coordinate_slot, fixed_slot};
    #[test]
    fn coordinates_keep_the_same_cells_for_sign_extremes_and_placeholders() {
        for value in (i32::MIN..i32::MIN + 4096)
            .chain(-10000..10000)
            .chain(i32::MAX - 4096..=i32::MAX)
        {
            assert_eq!(
                coordinate_slot(Some(value)).chars().count(),
                COORDINATE_CELLS
            );
            assert_eq!(coordinate_slot(Some(value)).trim(), value.to_string());
        }
        assert_eq!(coordinate_slot(None).chars().count(), COORDINATE_CELLS);
    }
    #[test]
    fn box_extremes_fit_and_overflow_preserves_width_including_unicode() {
        assert_eq!(
            fixed_slot(Some(1_u128 << 96), COUNT_CELLS).chars().count(),
            COUNT_CELLS
        );
        assert!(!fixed_slot(Some(1_u128 << 96), COUNT_CELLS).ends_with('+'));
        assert!(!fixed_slot(Some(1_u64 << 32), DIMENSION_CELLS).ends_with('+'));
        assert_eq!(fixed_slot(Some("abcdef"), 4), "abc+");
        assert_eq!(fixed_slot(Some("猫猫猫"), 2), "猫+");
        assert_eq!(fixed_slot(Some(42), 1), "+");
        assert_eq!(fixed_slot(Some(42), 0), "");
        for cells in 1..40 {
            assert_eq!(fixed_slot(None::<u32>, cells).chars().count(), cells);
        }
    }
}
