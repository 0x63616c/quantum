//! Compact viewport telemetry for newly painted content, not display refresh.
use super::{AnchoredTooltip, EditorView, TooltipSide, row};
use gpui::{prelude::*, *};
use quantum_core::Pos;
use quantum_ui::ColorToken;
use quantum_ui::Space;
use quantum_ui::tokens::type_size;
use quantum_ui::{
    COORDINATE_CELLS, COUNT_CELLS, DIMENSION_CELLS, StatusBar, StatusBarItem, ToolbarSeparator,
};

fn grouped(value: impl ToString) -> String {
    let value = value.to_string();
    let (integer, fraction) = value
        .split_once('.')
        .map_or((value.as_str(), None), |(a, b)| (a, Some(b)));
    let mut result = String::new();
    for (index, ch) in integer.chars().enumerate() {
        if index > 0
            && ch.is_ascii_digit()
            && (integer.len() - index).is_multiple_of(3)
            && integer.as_bytes()[index - 1].is_ascii_digit()
        {
            result.push(',');
        }
        result.push(ch);
    }
    if let Some(fraction) = fraction {
        result.push('.');
        result.push_str(fraction);
    }
    result
}

fn selection_metrics(start: Pos, end: Pos) -> ([u64; 3], u128) {
    let size = std::array::from_fn(|axis| {
        (i64::from(end[axis]) - i64::from(start[axis])).unsigned_abs() + 1
    });
    let count = size.into_iter().map(u128::from).product();
    (size, count)
}
impl EditorView {
    pub(super) fn statusbar(&self, compact: bool) -> Div {
        let t = self.theme;
        let snapshot = self.editor.display_snapshot();
        let dimensions: [i64; 3] = std::array::from_fn(|axis| {
            i64::from(snapshot.bounds.max[axis]) - i64::from(snapshot.bounds.min[axis])
        });
        let voxels: usize = snapshot.layers.iter().map(|layer| layer.voxels.len()).sum();
        let layer = snapshot
            .layers
            .by_uuid(self.editor.layer)
            .map_or("No layer", |layer| layer.name.as_str());
        let (hover, box_start) = (self.hover, self.editor.box_start);
        #[cfg(target_os = "linux")]
        let (hover, box_start) = super::capture::status_fixture()
            .map_or((hover, box_start), |position| {
                (position, position.map(|_| [0; 3]))
            });
        let selection = box_start
            .map(|start| selection_metrics(start, hover.unwrap_or(self.cursor)))
            .unwrap_or(([0; 3], 0));
        // Full selection slots cover inclusive dimensions and 2^96 box volume.
        // Coordinates and compact selection slots use '+' on overflow; tooltips
        // keep exact values. Only the window-size breakpoint changes geometry.
        let extent_cells = if compact { 3 } else { DIMENSION_CELLS };
        let count_cells = if compact { 6 } else { COUNT_CELLS };
        let slot = |label, value: String, cells| {
            StatusBarItem::new(label, Some(value), cells, t)
                .separator(false)
                .build()
        };
        let position = self.status_position(hover);
        let selection_row = row().id("status-selection").flex_shrink_0()
            .anchored_hint(format!("Box selection preview: {} × {} × {} · {} cells. No persistent selection remains after commit.", selection.0[0], selection.0[1], selection.0[2], selection.1), t, TooltipSide::Above, &self.overlays)
            .child(slot("Sel ", selection.0[0].to_string(), extent_cells))
            .child(slot("×", selection.0[1].to_string(), extent_cells))
            .child(slot("×", selection.0[2].to_string(), extent_cells))
            .child(slot(" · ", selection.1.to_string(), count_cells));
        let zoom = self.zoom_reference_span / self.viewport.display_camera().span * 100.0;
        let performance = self.status_performance();
        StatusBar::row(t)
            .child(
                row()
                    .gap(px(Space::S3.px()))
                    .flex_shrink_0()
                    .child(position)
                    .child(ToolbarSeparator::line(t))
                    .child(selection_row)
                    .child(ToolbarSeparator::line(t))
                    .child(
                        StatusBarItem::new("", Some(layer), if compact { 8 } else { 16 }, t)
                            .separator(false)
                            .icon("layer", t.color(ColorToken::TextSecondary))
                            .build()
                            .id("status-layer")
                            .anchored_hint(
                                format!("Active layer: {layer}"),
                                t,
                                TooltipSide::Above,
                                &self.overlays,
                            ),
                    )
                    .child(slot("", self.editor.tool.name().to_owned(), 6)),
            )
            .child(self.status_message(compact))
            .child(
                row()
                    .gap(px(Space::S3.px()))
                    .flex_shrink_0()
                    .child(
                        slot("", format!("{zoom:.0}%"), 6)
                            .id("status-zoom")
                            .anchored_hint(
                                "Zoom relative to the last fitted view",
                                t,
                                TooltipSide::Above,
                                &self.overlays,
                            ),
                    )
                    .when(self.preferences.show_performance, |d| d.child(performance))
                    .child(ToolbarSeparator::line(t))
                    .child(
                        row()
                            .id("status-dimensions")
                            .flex_shrink_0()
                            .anchored_hint(
                                format!(
                                    "Model dimensions: {} × {} × {} · {} occupied voxels",
                                    dimensions[0],
                                    dimensions[1],
                                    dimensions[2],
                                    grouped(voxels)
                                ),
                                t,
                                TooltipSide::Above,
                                &self.overlays,
                            )
                            .children((0..3).map(|axis| {
                                slot(
                                    if axis == 0 { "" } else { "×" },
                                    dimensions[axis].to_string(),
                                    if compact { 3 } else { 5 },
                                )
                            })),
                    ),
            )
    }
}

impl EditorView {
    fn status_message(&self, compact: bool) -> Stateful<Div> {
        let t = self.theme;
        div()
            .id("status-message")
            .flex_1()
            .min_w(px(Space::S0.px()))
            .overflow_hidden()
            .text_ellipsis()
            .font_family(type_size::UI)
            .text_color(if self.error {
                t.color(ColorToken::Danger)
            } else {
                t.color(ColorToken::TextSecondary)
            })
            .anchored_hint(self.status.clone(), t, TooltipSide::Above, &self.overlays)
            .when(!compact, |d| {
                if !self.error
                    && let Some(text) = self.status.plain_text()
                {
                    if self.save_flow.working() {
                        return d.child(
                            quantum_ui::AsciiReveal::new("io-progress", text.to_owned(), t)
                                .loading(true)
                                .reduced_motion(self.reduced_motion),
                        );
                    }
                    if let Some(at) = self.save_flow.completion(text) {
                        return d.child(
                            quantum_ui::AsciiReveal::new("io-progress", text.to_owned(), t)
                                .started_at(at)
                                .reduced_motion(self.reduced_motion),
                        );
                    }
                }
                d.child(self.status.clone())
            })
            .when(compact && self.error, |d| d.child("!"))
    }
}

impl EditorView {
    fn status_performance(&self) -> Stateful<Div> {
        let t = self.theme;
        let slot = |label, value: String, cells| {
            StatusBarItem::new(label, Some(value), cells, t)
                .separator(false)
                .build()
        };
        let renderer = self.viewport.frame_meter().renderer().label();
        let average = if self.capture_state.is_some() {
            None
        } else {
            self.viewport.frame_meter().average()
        };

        row().id("status-performance").flex_shrink_0()
            .anchored_hint("Newly painted viewport frames: average FPS and time between frames over about 1 second. Holds the last average while idle; unchanged repaints are excluded.", t, TooltipSide::Above, &self.overlays)
            .text_color(if self.viewport.rendering() { t.color(ColorToken::AccentText) } else { t.color(ColorToken::TextSecondary) })
            .child(slot("", renderer.to_owned(), 3))
            .child(StatusBarItem::new(" ", average.map(|a| format!("{:.1}", a.fps)), 6, t).separator(false).build())
            .child(StatusBarItem::new(" FPS / ", average.map(|a| format!("{:.1}", a.frame_ms)), 7, t).separator(false).build())
            .child(slot(" ms", String::new(), 0))
    }
}

impl EditorView {
    fn status_position(&self, hover: Option<Pos>) -> Stateful<Div> {
        let t = self.theme;
        row()
            .id("status-position")
            .flex_shrink_0()
            .gap(px(Space::S3.px()))
            .anchored_hint(
                hover.map_or_else(
                    || "No cursor position".to_owned(),
                    |[x, y, z]| format!("Cursor position: {x}, {y}, {z}"),
                ),
                t,
                TooltipSide::Above,
                &self.overlays,
            )
            .children((0..3).map(|axis| {
                row()
                    .flex_shrink_0()
                    .gap(px(Space::S2.px()))
                    .child(
                        StatusBarItem::new(["X", "Y", "Z"][axis], None::<u8>, 0, t)
                            .separator(false)
                            .build()
                            .text_color(t.axis(axis)),
                    )
                    .child(
                        StatusBarItem::new("", hover.map(|p| p[axis]), COORDINATE_CELLS, t)
                            .separator(false)
                            .left_aligned()
                            .build(),
                    )
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::{grouped, selection_metrics};
    #[test]
    fn groups_counts_and_signed_coordinates_without_changing_fractional_digits() {
        for (input, expected) in [
            ("0", "0"),
            ("999", "999"),
            ("1000", "1,000"),
            ("-1234567", "-1,234,567"),
            ("1000.5", "1,000.5"),
            ("-128", "-128"),
        ] {
            assert_eq!(grouped(input), expected);
        }
        assert_eq!(
            grouped(1_u128 << 96),
            "79,228,162,514,264,337,593,543,950,336"
        );
    }
    #[test]
    fn box_metrics_are_inclusive_and_direction_independent() {
        assert_eq!(selection_metrics([0, 0, 0], [0, 0, 0]), ([1, 1, 1], 1));
        assert_eq!(selection_metrics([-3, 2, 0], [2, 0, 3]), ([6, 3, 4], 72));
        assert_eq!(selection_metrics([2, 0, 3], [-3, 2, 0]), ([6, 3, 4], 72));
        assert_eq!(
            selection_metrics([i32::MIN; 3], [i32::MAX; 3]).1,
            1_u128 << 96
        );
    }
}
