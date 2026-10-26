//! Compact viewport telemetry. Timings describe CPU renders, not display refresh.
use super::{EditorView, STATUSBAR_HEIGHT, hint, icon, mono, row};
use gpui::{prelude::*, *};
use voxelite_core::Pos;

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
            |p| format!("X {}  Y {}  Z {}", p[0], p[1], p[2]),
        );
        let selection = self
            .editor
            .box_start
            .map(|start| selection_metrics(start, self.hover.unwrap_or(self.cursor)));
        let selection_label = selection.map_or_else(
            || "Sel 0×0×0 · 0".into(),
            |(size, count)| format!("Sel {}×{}×{} · {}", size[0], size[1], size[2], count),
        );
        let zoom = self.zoom_reference_span / self.display_camera.span * 100.0;
        let performance = self.render_ms.map_or_else(
            || "CPU — FPS / — ms".into(),
            |ms| format!("CPU {:.1} FPS / {:.1} ms", 1000.0 / ms.max(0.001), ms),
        );
        let separator = || div().h(px(12.0)).w(px(1.0)).flex_shrink_0().bg(t.line());
        let number = |text: String| mono(text).text_size(px(11.0)).flex_shrink_0();
        row().h(px(STATUSBAR_HEIGHT)).flex_shrink_0().border_t_1().border_color(t.line())
            .px(px(12.0)).gap(px(if compact { 10.0 } else { 16.0 })).text_size(px(11.0)).text_color(t.muted())
            .child(row().gap(px(12.0)).flex_shrink_0()
                .child(number(position))
                .child(separator())
                .child(row().id("status-selection").tooltip(hint("Box selection preview: width × height × depth and cell count. No persistent selection is active after the box is committed.", t)).child(number(selection_label)))
                .child(separator())
                .child(row().id("status-layer").gap(px(5.0)).tooltip(hint(format!("Active layer: {layer}"), t))
                    .child(icon("layer").size(px(12.0)))
                    .child(div().max_w(px(if compact { 76.0 } else { 120.0 })).overflow_hidden().text_ellipsis().child(layer.to_owned())))
                .child(div().child(self.editor.tool.name())))
            .child(div().id("status-message").flex_1().min_w(px(0.0)).overflow_hidden().text_ellipsis()
                .text_color(if self.error { t.danger() } else { t.muted() })
                .tooltip(hint(self.status.clone(), t))
                .when(!compact, |d| d.child(self.status.clone()))
                .when(compact && self.error, |d| d.child("!")))
            .child(row().gap(px(12.0)).flex_shrink_0()
                .child(row().id("status-zoom").tooltip(hint("Zoom relative to the last fitted view", t)).child(number(format!("{zoom:.0}%"))))
                .when(self.preferences.show_performance, |d| d.child(row().id("status-performance")
                    .tooltip(hint("Last completed CPU render: reciprocal throughput and elapsed frame time. The viewport renders on demand; this is not display refresh rate.", t))
                    .text_color(if self.rendering { t.accent() } else { t.muted() }).child(number(performance))))
                .child(separator())
                .child(row().id("status-dimensions").tooltip(hint(format!("Model dimensions · {voxels} occupied voxels"), t))
                    .child(number(format!("{}×{}×{}", dimensions[0], dimensions[1], dimensions[2])))))
    }
}
#[cfg(test)]
mod tests {
    use super::selection_metrics;
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
