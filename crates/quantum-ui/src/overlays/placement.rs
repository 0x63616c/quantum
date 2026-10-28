use super::OverlayLayer;
use crate::{ColorToken as C, Popover, Radius, Space, Theme, presentation};
use gpui::{prelude::*, *};

/// Preferred side of a trigger; flips to the opposite side when it fits better.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverSide {
    /// Below the trigger.
    #[default]
    Below,
    /// Above the trigger.
    Above,
    /// Right of the trigger (submenus).
    Right,
    /// Left of the trigger.
    Left,
}
/// Resolve a 4px trigger gap, flip, then clamp to the viewport.
/// Oversized overlays align to the leading viewport edge and should scroll internally.
pub fn overlay_position(
    trigger: Bounds<Pixels>,
    popup: Size<Pixels>,
    viewport: Size<Pixels>,
    side: PopoverSide,
) -> Point<Pixels> {
    let gap = px(Space::S2.px());
    let origin = |side| match side {
        PopoverSide::Below => point(trigger.left(), trigger.bottom() + gap),
        PopoverSide::Above => point(trigger.left(), trigger.top() - popup.height - gap),
        PopoverSide::Right => point(trigger.right() + gap, trigger.top()),
        PopoverSide::Left => point(trigger.left() - popup.width - gap, trigger.top()),
    };
    let opposite = match side {
        PopoverSide::Below => PopoverSide::Above,
        PopoverSide::Above => PopoverSide::Below,
        PopoverSide::Right => PopoverSide::Left,
        PopoverSide::Left => PopoverSide::Right,
    };
    let overflow = |p: Point<Pixels>| {
        let zero = px(Space::S0.px());
        match side {
            PopoverSide::Below | PopoverSide::Above => {
                (-p.y).max(zero) + (p.y + popup.height - viewport.height).max(zero)
            }
            _ => (-p.x).max(zero) + (p.x + popup.width - viewport.width).max(zero),
        }
    };
    let a = origin(side);
    let b = origin(opposite);
    let p = if overflow(b) < overflow(a) { b } else { a };
    let zero = px(Space::S0.px());
    point(
        p.x.clamp(zero, (viewport.width - popup.width).max(zero)),
        p.y.clamp(zero, (viewport.height - popup.height).max(zero)),
    )
}
/// Measured anchored overlay; normally constructed by `Popover::anchored`.
pub struct PlacedOverlay {
    trigger: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    side: PopoverSide,
    child: AnyElement,
}
impl PlacedOverlay {
    pub(crate) fn measured(
        trigger: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
        side: PopoverSide,
        child: impl IntoElement,
    ) -> Self {
        Self {
            trigger,
            side,
            child: child.into_any_element(),
        }
    }

    pub(crate) fn new(trigger: Bounds<Pixels>, side: PopoverSide, child: impl IntoElement) -> Self {
        Self {
            trigger: std::rc::Rc::new(std::cell::Cell::new(trigger)),
            side,
            child: child.into_any_element(),
        }
    }
}
impl IntoElement for PlacedOverlay {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for PlacedOverlay {
    type RequestLayoutState = LayoutId;
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, LayoutId) {
        let child = self.child.request_layout(window, cx);
        (
            window.request_layout(
                Style {
                    position: Position::Absolute,
                    ..Style::default()
                },
                [child],
                cx,
            ),
            child,
        )
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        child: &mut LayoutId,
        window: &mut Window,
        cx: &mut App,
    ) {
        let popup = window.layout_bounds(*child).size;
        let origin = overlay_position(self.trigger.get(), popup, window.viewport_size(), self.side);
        window.with_element_offset(origin - bounds.origin, |window| {
            self.child.prepaint(window, cx);
        });
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut LayoutId,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}
impl Popover {
    /// Token-only floating surface with the shared overlay palette and corners.
    pub fn design(theme: Theme) -> Div {
        presentation::surface(theme, C::Overlay, Radius::Md)
            .occlude()
            .p(px(Space::S2.px()))
            .shadow(crate::tokens::panel_shadow())
    }
    /// Anchor measured content to the trigger, flipping with a 4px gap on either side.
    /// The host updates its `OverlayState` when opening/closing this surface.
    pub fn anchored(
        trigger: Bounds<Pixels>,
        side: PopoverSide,
        content: impl IntoElement,
    ) -> Deferred {
        OverlayLayer::Popover.defer(PlacedOverlay::new(trigger, side, content))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    #[test]
    fn popovers_flip_around_the_entire_trigger_and_keep_the_gap() {
        let view = size(px(300.), px(200.));
        let popup = size(px(100.), px(60.));
        let trigger = Bounds::new(point(px(20.), px(160.)), size(px(40.), px(28.)));
        assert_eq!(
            overlay_position(trigger, popup, view, PopoverSide::Below),
            point(px(20.), px(96.))
        );
        let trigger = Bounds::new(point(px(260.), px(20.)), size(px(28.), px(28.)));
        assert_eq!(
            overlay_position(trigger, popup, view, PopoverSide::Right),
            point(px(156.), px(20.))
        );
        assert_eq!(
            overlay_position(trigger, size(px(400.), px(300.)), view, PopoverSide::Below),
            Point::default()
        );
    }
}
