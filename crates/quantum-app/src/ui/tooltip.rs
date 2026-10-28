//! Shared measured tooltip presentation; the app supplies its per-window overlay state.
use super::{HintText, Theme};
use gpui::*;
use quantum_ui::{OverlayState, PopoverSide, Tooltip};

#[derive(Clone, Copy)]
pub(super) enum TooltipSide {
    Right,
    Below,
    Above,
}

pub(super) trait AnchoredTooltip {
    fn anchored_hint(
        self,
        text: impl Into<HintText>,
        theme: Theme,
        side: TooltipSide,
        overlays: &Entity<OverlayState>,
    ) -> Self;
}
impl AnchoredTooltip for Stateful<Div> {
    fn anchored_hint(
        self,
        text: impl Into<HintText>,
        theme: Theme,
        side: TooltipSide,
        overlays: &Entity<OverlayState>,
    ) -> Self {
        let (label, shortcut) = text.into().tooltip_parts();
        let side = match side {
            TooltipSide::Right => PopoverSide::Right,
            TooltipSide::Below => PopoverSide::Below,
            TooltipSide::Above => PopoverSide::Above,
        };
        let mut hint = Tooltip::new(label, theme, overlays).side(side);
        if let Some(shortcut) = shortcut {
            hint = hint.shortcut(shortcut);
        }
        hint.decorate(self)
    }
}
