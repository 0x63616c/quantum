//! GPUI owns tooltip timing; placement follows the control's painted bounds.
use super::{Hint, HintText, Theme};
use gpui::{prelude::*, *};
use quantum_ui::tokens::{metrics, space};
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Copy)]
pub(super) enum TooltipSide {
    Right,
    /// Retain the established mirror tooltip position while the button becomes square.
    Mirror(usize),
    Below,
    Above,
}

struct AnchoredHint {
    hint: Entity<Hint>,
    bounds: Rc<Cell<Bounds<Pixels>>>,
    side: TooltipSide,
}

impl Render for AnchoredHint {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let bounds = self.bounds.get();
        let (anchor, position) = match self.side {
            TooltipSide::Right => (
                Anchor::LeftCenter,
                point(bounds.right() + px(space::MD), bounds.center().y),
            ),
            TooltipSide::Mirror(axis) => (
                Anchor::LeftCenter,
                point(
                    bounds.right() + px(space::MD + metrics::MIRROR_HINT_X),
                    bounds.center().y
                        + px(metrics::MIRROR_HINT_Y + axis as f32 * metrics::MIRROR_HINT_ROW),
                ),
            ),
            TooltipSide::Below => (
                Anchor::TopCenter,
                point(bounds.center().x, bounds.bottom() + px(space::MD)),
            ),
            TooltipSide::Above => (
                Anchor::BottomCenter,
                point(bounds.center().x, bounds.top() - px(space::MD)),
            ),
        };
        anchored()
            .anchor(anchor)
            .position(position)
            .snap_to_window_with_margin(px(space::XS))
            .child(self.hint.clone())
    }
}

pub(super) trait AnchoredTooltip {
    fn anchored_hint(self, text: impl Into<HintText>, theme: Theme, side: TooltipSide) -> Self;
}

impl AnchoredTooltip for Stateful<Div> {
    fn anchored_hint(self, text: impl Into<HintText>, theme: Theme, side: TooltipSide) -> Self {
        let bounds = Rc::new(Cell::new(Bounds::default()));
        let measured = bounds.clone();
        let text = text.into();
        self.relative()
            .child(
                canvas(move |bounds, _, _| measured.set(bounds), |_, _, _, _| {})
                    .absolute()
                    .inset_0()
                    .size_full(),
            )
            .tooltip(move |_, cx| {
                let hint = cx.new(|_| Hint {
                    text: text.clone(),
                    theme,
                });
                cx.new(|_| AnchoredHint {
                    hint,
                    bounds: bounds.clone(),
                    side,
                })
                .into()
            })
    }
}
