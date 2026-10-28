//! Overlay ordering and shared tooltip cancellation, owned by each window.
mod dialog;
mod placement;
mod tooltip;
pub use dialog::*;
use gpui::*;
pub use placement::*;
use std::collections::BTreeMap;
pub use tooltip::*;

/// Explicit deferred paint order; higher layers occlude lower ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OverlayLayer {
    /// Noninteractive hints, below all popovers.
    Tooltip,
    /// Anchored menus and panels.
    Popover,
    /// Modal scrim and panel.
    Dialog,
}
impl OverlayLayer {
    /// GPUI deferred paint priority.
    pub const fn priority(self) -> usize {
        match self {
            Self::Tooltip => 10,
            Self::Popover => 20,
            Self::Dialog => 30,
        }
    }
    /// Wrap an element in the shared deferred layer.
    pub fn defer(self, element: impl IntoElement) -> Deferred {
        deferred(element).with_priority(self.priority())
    }
}
/// One entity per window, shared by tooltips and the host's overlay state.
/// Call `set_open` in the same update that opens/closes a popover or dialog.
/// Closing a layer never resurrects a tooltip; a fresh hover is required.
#[derive(Default)]
pub struct OverlayState {
    open: BTreeMap<SharedString, OverlayLayer>,
    generation: u64,
}
impl OverlayState {
    /// Construct shared state for a window.
    pub fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|_| Self::default())
    }
    /// Update one independently owned overlay and notify all tooltip triggers.
    pub fn set_open(
        &mut self,
        id: impl Into<SharedString>,
        layer: OverlayLayer,
        open: bool,
        cx: &mut Context<Self>,
    ) {
        if self.transition(id.into(), layer, open) {
            cx.notify();
        }
    }
    fn transition(&mut self, id: SharedString, layer: OverlayLayer, open: bool) -> bool {
        let changed = if open {
            self.open.insert(id, layer) != Some(layer)
        } else {
            self.open.remove(&id).is_some()
        };
        if changed {
            self.generation = self.generation.wrapping_add(1);
        }
        changed
    }
    /// Tooltips are suppressed while a popover or dialog is open.
    pub fn allows_tooltip(&self) -> bool {
        self.open.values().all(|l| *l == OverlayLayer::Tooltip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    #[test]
    fn layers_and_nested_overlay_lifetimes_are_ordered() {
        assert_eq!(
            [
                OverlayLayer::Tooltip,
                OverlayLayer::Popover,
                OverlayLayer::Dialog
            ]
            .map(OverlayLayer::priority),
            [10, 20, 30]
        );
        let mut state = OverlayState::default();
        assert!(state.allows_tooltip());
        assert!(state.transition("menu".into(), OverlayLayer::Popover, true));
        assert!(!state.transition("menu".into(), OverlayLayer::Popover, true));
        assert!(!state.allows_tooltip());
        state.transition("confirm".into(), OverlayLayer::Dialog, true);
        state.transition("confirm".into(), OverlayLayer::Dialog, false);
        assert!(!state.allows_tooltip());
        state.transition("menu".into(), OverlayLayer::Popover, false);
        assert!(state.allows_tooltip());
        assert_eq!(state.generation, 4);
    }
}
