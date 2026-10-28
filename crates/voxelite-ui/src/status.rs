//! Fixed-width telemetry slots. Formatting never changes the layout width.
use crate::{
    icon, row,
    tokens::{Theme, metrics, space, type_size},
};
use gpui::{prelude::*, *};

/// One sign and four digits for typical cursor coordinates. Larger values end
/// in `+`; the position tooltip retains the exact i32 coordinates.
pub const COORDINATE_CELLS: usize = 5;
/// Ten digits cover an inclusive dimension spanning the entire i32 domain.
pub const DIMENSION_CELLS: usize = 10;
/// Twenty-nine digits cover the largest inclusive box: 2^96 cells.
pub const COUNT_CELLS: usize = 29;

/// Which end of a fixed-width slot receives padding.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SlotAlignment {
    /// Keep the value next to its leading label.
    Left,
    /// Align trailing digits, as used by the other telemetry segments.
    #[default]
    Right,
}

/// Right-align a value in exactly `cells` Unicode scalar cells. Overflow ends in
/// `+`, keeping the most significant prefix. Missing values use an em dash.
pub fn fixed_slot(value: Option<impl ToString>, cells: usize) -> String {
    fixed_slot_aligned(value, cells, SlotAlignment::Right)
}
/// Pad on the requested side; overflow always keeps a prefix and trailing `+`.
pub fn fixed_slot_aligned(
    value: Option<impl ToString>,
    cells: usize,
    alignment: SlotAlignment,
) -> String {
    if cells == 0 {
        return String::new();
    }
    let value = value.map_or_else(|| "—".into(), |v| v.to_string());
    let len = value.chars().count();
    if len > cells {
        value.chars().take(cells - 1).chain(['+']).collect()
    } else {
        let padding = "\u{00a0}".repeat(cells - len);
        match alignment {
            SlotAlignment::Left => format!("{value}{padding}"),
            SlotAlignment::Right => format!("{padding}{value}"),
        }
    }
}
/// Left-aligned coordinate slot (no grouping), with `+` on overflow.
pub fn coordinate_slot(value: Option<i32>) -> String {
    fixed_slot_aligned(value, COORDINATE_CELLS, SlotAlignment::Left)
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
    alignment: SlotAlignment,
    glyph: Option<(SharedString, Hsla)>,
}
impl StatusSegment {
    /// Construct a slot. Values are padded or truncated before rendering.
    pub fn new(label: impl Into<SharedString>, value: Option<impl ToString>, cells: usize) -> Self {
        Self {
            label: label.into(),
            value: value.map_or_else(|| "—".into(), |v| v.to_string()),
            cells,
            alignment: SlotAlignment::Right,
            glyph: None,
        }
    }
    /// Keep the value at the leading edge of its fixed-width slot.
    pub fn left_aligned(mut self) -> Self {
        self.alignment = SlotAlignment::Left;
        self
    }
    /// Add a fixed-size leading icon.
    pub fn icon(mut self, glyph: impl Into<SharedString>, color: impl Into<Hsla>) -> Self {
        self.glyph = Some((glyph.into(), color.into()));
        self
    }
    /// Build; add `id` and a tooltip to this row without changing its geometry.
    pub fn build(self) -> Div {
        row()
            .flex_shrink_0()
            .font_family(type_size::MONO)
            .text_size(px(type_size::CAPTION))
            .whitespace_nowrap()
            .when_some(self.glyph, |d, (glyph, color)| {
                d.child(
                    icon(glyph)
                        .text_color(color)
                        .size(px(metrics::ICON_SM))
                        .mr(px(space::XS)),
                )
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
                    .when(self.alignment == SlotAlignment::Right, |d| d.text_right())
                    .when(self.alignment == SlotAlignment::Left, |d| d.text_left())
                    .child(fixed_slot_aligned(
                        Some(self.value),
                        self.cells,
                        self.alignment,
                    )),
            )
    }
}

/// Proportional label with a fixed-width tabular value and optional separator.
/// Values cannot shift neighboring items as their number of digits changes.
pub struct StatusBarItem {
    label: SharedString,
    value: String,
    cells: usize,
    separator: bool,
    theme: Theme,
}
impl StatusBarItem {
    /// Reserve exactly `cells` Geist Mono digits, truncating overflow with `+`.
    pub fn new(
        label: impl Into<SharedString>,
        value: Option<impl ToString>,
        cells: usize,
        theme: Theme,
    ) -> Self {
        Self {
            label: label.into(),
            value: fixed_slot(value, cells),
            cells,
            separator: true,
            theme,
        }
    }
    /// Omit the separator for the first item in a cluster.
    pub fn separator(mut self, separator: bool) -> Self {
        self.separator = separator;
        self
    }
    /// Build a static-width value next to a proportional label.
    pub fn build(self) -> Div {
        use crate::{ColorToken as C, ControlHeight, Space, TextSize, tokens::geometry};
        TextSize::Small
            .apply(row())
            .h(px(ControlHeight::Dense.px()))
            .flex_shrink_0()
            .gap(px(Space::S3.px()))
            .text_color(self.theme.color(C::TextSecondary))
            .when(self.separator, |d| {
                d.child(
                    div()
                        .w(px(geometry::EDGE))
                        .h(px(Space::S4.px()))
                        .mr(px(Space::S2.px()))
                        .bg(self.theme.color(C::BorderSubtle)),
                )
            })
            .child(self.label)
            .child(
                TextSize::Mono
                    .apply(div())
                    .w(px(self.cells as f32
                        * TextSize::Mono.px()
                        * type_size::MONO_ADVANCE))
                    .flex_shrink_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(self.value),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        COORDINATE_CELLS, COUNT_CELLS, DIMENSION_CELLS, SlotAlignment, coordinate_slot, fixed_slot,
        fixed_slot_aligned,
    };
    #[test]
    fn design_status_values_keep_fixed_slots_for_signs_missing_values_and_overflow() {
        use super::StatusBarItem;
        use crate::Theme;
        use gpui::{Styled, px};
        for (value, expected) in [
            (Some(42), "\u{00a0}\u{00a0}\u{00a0}42"),
            (Some(-42), "\u{00a0}\u{00a0}-42"),
            (Some(i32::MAX), "2147+"),
            (None, "\u{00a0}\u{00a0}\u{00a0}\u{00a0}—"),
        ] {
            let item = StatusBarItem::new("Count", value, 5, Theme::Dark);
            assert_eq!(item.value, expected);
            assert_eq!(item.cells, 5);
            let mut row = item.build();
            assert_eq!(row.style().size.height, Some(px(24.).into()));
            assert_eq!(row.style().flex_shrink, Some(0.));
        }
    }
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
            if value.to_string().len() <= COORDINATE_CELLS {
                assert_eq!(coordinate_slot(Some(value)).trim(), value.to_string());
            } else {
                assert!(coordinate_slot(Some(value)).ends_with('+'));
            }
        }
        assert_eq!(coordinate_slot(None).chars().count(), COORDINATE_CELLS);
    }
    #[test]
    fn alignment_preserves_padding_and_overflow_prefixes() {
        assert_eq!(
            coordinate_slot(Some(4)),
            "4\u{00a0}\u{00a0}\u{00a0}\u{00a0}"
        );
        assert_eq!(coordinate_slot(Some(-20)), "-20\u{00a0}\u{00a0}");
        assert_eq!(coordinate_slot(None), "—\u{00a0}\u{00a0}\u{00a0}\u{00a0}");
        assert_eq!(coordinate_slot(Some(-9999)), "-9999");
        assert_eq!(coordinate_slot(Some(i32::MIN)), "-214+");
        assert_eq!(coordinate_slot(Some(i32::MAX)), "2147+");
        assert_eq!(fixed_slot(Some(4), 5), "\u{00a0}\u{00a0}\u{00a0}\u{00a0}4");
        for alignment in [SlotAlignment::Left, SlotAlignment::Right] {
            assert_eq!(fixed_slot_aligned(Some("猫猫猫"), 2, alignment), "猫+");
            assert_eq!(fixed_slot_aligned(Some(-10000), 5, alignment), "-100+");
            assert_eq!(fixed_slot_aligned(Some(42), 1, alignment), "+");
            assert_eq!(fixed_slot_aligned(Some(42), 0, alignment), "");
        }
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
