use super::*;

impl Overlay for CommentsPlugin {
    fn draw(&self, cx: &mut dyn OverlayCx, _: Option<u32>) {
        let Some(comments) = cx.comments() else {
            return;
        };
        let view = self.view(comments);
        let pins: Vec<_> = comments
            .snapshot()
            .comments
            .iter()
            .enumerate()
            .filter(|(_, c)| panel::matches(c, view.state.filter))
            .filter_map(|(index, c)| {
                let region = c.anchor.region().ok()?;
                let center =
                    std::array::from_fn(|a| f64::from(region.min[a] + region.max[a]) * 0.5);
                let uv = cx.project(center)?;
                if uv
                    .iter()
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
                {
                    return None;
                }
                let occluded = cx.pick(uv).is_some_and(|at| !region.contains(at));
                let dim = occluded || comments.snapshot().anchor_state(c) != AnchorState::Visible;
                Some(OverlayPin {
                    id: c.id,
                    index,
                    uv,
                    number: view.state.numbers.get(&c.id).copied().unwrap_or(index + 1),
                    resolved: c.resolved,
                    selected: view.state.selected == Some(c.id),
                    dim,
                    tooltip: format!(
                        "{}{}",
                        if dim { "Occluded / detached · " } else { "" },
                        c.messages[0].text.chars().take(100).collect::<String>()
                    ),
                })
            })
            .collect();
        for pin in pins {
            cx.emit(OverlayPrimitive::Pin(pin));
        }
    }
}
