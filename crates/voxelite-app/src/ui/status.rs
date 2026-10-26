//! Compact viewport telemetry for newly painted content, not display refresh.
use super::{AnchoredTooltip, EditorView, STATUSBAR_HEIGHT, TooltipSide, icon, mono, row};
use gpui::{prelude::*, *};
use voxelite_core::Pos;

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
            .iter()
            .find(|layer| layer.id == self.editor.layer)
            .map_or("No layer", |layer| layer.name.as_str());
        let position = self.hover.map_or_else(
            || "X —  Y —  Z —".into(),
            |p| {
                format!(
                    "X {}  Y {}  Z {}",
                    grouped(p[0]),
                    grouped(p[1]),
                    grouped(p[2])
                )
            },
        );
        let selection = self
            .editor
            .box_start
            .map(|start| selection_metrics(start, self.hover.unwrap_or(self.cursor)));
        let selection_label = selection.map_or_else(
            || "Sel 0×0×0 · 0".into(),
            |(size, count)| {
                format!(
                    "Sel {}×{}×{} · {}",
                    grouped(size[0]),
                    grouped(size[1]),
                    grouped(size[2]),
                    grouped(count)
                )
            },
        );
        let zoom = self.zoom_reference_span / self.display_camera.span * 100.0;
        let renderer = self.frame_meter.renderer().label();
        let performance = self.frame_meter.average().map_or_else(
            || format!("{renderer} idle / — ms"),
            |average| {
                format!(
                    "{renderer} {} FPS / {} ms",
                    grouped(format!("{:.1}", average.fps)),
                    grouped(format!("{:.1}", average.frame_ms))
                )
            },
        );
        let separator = || div().h(px(12.0)).w(px(1.0)).flex_shrink_0().bg(t.line());
        let number = |text: String| mono(text).text_size(px(11.0)).flex_shrink_0();
        row().h(px(STATUSBAR_HEIGHT)).flex_shrink_0().border_t_1().border_color(t.line())
            .px(px(12.0)).gap(px(if compact { 10.0 } else { 16.0 })).text_size(px(11.0)).text_color(t.muted())
            .child(row().gap(px(12.0)).flex_shrink_0()
                .child(number(position))
                .child(separator())
                .child(row().id("status-selection").anchored_hint("Box selection preview: width × height × depth and cell count. No persistent selection is active after the box is committed.", t, TooltipSide::Above).child(number(selection_label)))
                .child(separator())
                .child(row().id("status-layer").gap(px(5.0)).anchored_hint(format!("Active layer: {layer}"), t, TooltipSide::Above)
                    .child(icon("layer").size(px(12.0)))
                    .child(div().max_w(px(if compact { 76.0 } else { 120.0 })).overflow_hidden().text_ellipsis().child(layer.to_owned())))
                .child(div().child(self.editor.tool.name())))
            .child(div().id("status-message").flex_1().min_w(px(0.0)).overflow_hidden().text_ellipsis()
                .text_color(if self.error { t.danger() } else { t.muted() })
                .anchored_hint(self.status.clone(), t, TooltipSide::Above)
                .when(!compact, |d| d.child(self.status.clone()))
                .when(compact && self.error, |d| d.child("!")))
            .child(row().gap(px(12.0)).flex_shrink_0()
                .child(row().id("status-zoom").anchored_hint("Zoom relative to the last fitted view", t, TooltipSide::Above).child(number(format!("{}%", grouped(format!("{zoom:.0}"))))))
                .when(self.preferences.show_performance, |d| d.child(row().id("status-performance")
                    .anchored_hint("Newly painted viewport frames: average FPS and time between frames over about 1 second. Holds the last average while idle; unchanged repaints are excluded.", t, TooltipSide::Above)
                    .text_color(if self.rendering { t.accent() } else { t.muted() }).child(number(performance))))
                .child(separator())
                .child(row().id("status-dimensions").anchored_hint(format!("Model dimensions · {} occupied voxels", grouped(voxels)), t, TooltipSide::Above)
                    .child(number(format!("{}×{}×{}", grouped(dimensions[0]), grouped(dimensions[1]), grouped(dimensions[2]))))))
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
