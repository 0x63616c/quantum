use super::{OverlayLayer, OverlayState, PlacedOverlay, PopoverSide};
use crate::{
    ColorToken as C, Kbd, Radius, Space, TextSize, Theme, presentation, row, tokens::state,
};
use gpui::{prelude::*, *};
use std::{cell::Cell, rc::Rc, time::Duration};

/// Pure tooltip lifecycle. A dismissal invalidates any pending timer.
#[derive(Default)]
struct Dwell {
    hovered: bool,
    visible: bool,
    ticket: u64,
}
impl Dwell {
    fn hover(&mut self, hovered: bool, allowed: bool) -> Option<u64> {
        self.dismiss();
        self.hovered = hovered && allowed;
        self.hovered.then_some(self.ticket)
    }
    fn elapsed(&mut self, ticket: u64, allowed: bool) {
        self.visible = self.hovered && self.ticket == ticket && allowed;
    }
    fn dismiss(&mut self) {
        self.hovered = false;
        self.visible = false;
        self.ticket = self.ticket.wrapping_add(1);
    }
}
struct TooltipRuntime {
    dwell: Dwell,
    timer: Option<Task<()>>,
    _subscription: Subscription,
}

fn position_trigger(mut trigger: Stateful<Div>, cover: bool) -> Stateful<Div> {
    if cover {
        return trigger.absolute().inset_0();
    }
    if trigger.style().position.is_none() {
        trigger = trigger.relative();
    }
    trigger
}
impl TooltipRuntime {
    fn dismiss(&mut self, cx: &mut Context<Self>) {
        self.dwell.dismiss();
        self.timer = None;
        cx.notify();
    }
}
/// Delayed, noninteractive tooltip with an optional single shortcut chip.
/// Uses the library's deferred tooltip layer, not GPUI's topmost native tooltip slot.
#[derive(Clone)]
pub struct Tooltip {
    label: SharedString,
    shortcut: Option<SharedString>,
    theme: Theme,
    overlays: Entity<OverlayState>,
    delay: Duration,
    side: PopoverSide,
}
impl Tooltip {
    /// The required label also names icon-only triggers for accessibility.
    pub fn new(
        label: impl Into<SharedString>,
        theme: Theme,
        overlays: &Entity<OverlayState>,
    ) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            theme,
            overlays: overlays.clone(),
            delay: state::TOOLTIP_DELAY,
            side: PopoverSide::Below,
        }
    }
    /// Add one shortcut chip, without nesting keycap borders.
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
    /// Override the default 500ms hover dwell.
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }
    /// Preferred side; measured placement flips and clamps to the viewport.
    pub fn side(mut self, side: PopoverSide) -> Self {
        self.side = side;
        self
    }
    /// Decorate a host-composed control while retaining its event and style hooks.
    /// The transparent child follows the control's bounds without changing layout.
    pub fn decorate(self, trigger: Stateful<Div>) -> Stateful<Div> {
        let id = Element::id(&trigger).unwrap_or_else(|| self.label.clone().into());
        let mut hint = self.attach(id, div().id("tooltip-hitbox").size_full());
        hint.cover = true;
        position_trigger(trigger, false).child(hint)
    }
    /// Build tooltip content for a static gallery or the delayed trigger.
    pub fn build(&self) -> Div {
        Self::content(self.label.clone(), self.shortcut.clone(), self.theme)
    }
    /// Static content using the same single-border keycap as the live tooltip.
    pub fn content(
        label: impl Into<SharedString>,
        shortcut: Option<SharedString>,
        theme: Theme,
    ) -> Div {
        TextSize::Small
            .apply(presentation::surface(theme, C::Overlay, Radius::Sm))
            .flex()
            .items_center()
            .gap(px(Space::S3.px()))
            .px(px(Space::S4.px()))
            .py(px(Space::S3.px()))
            .child(label.into())
            .when_some(shortcut, |d, s| d.child(Kbd::design(s, theme)))
    }
    /// Attach lifecycle listeners to a trigger. Give each trigger a unique stable ID.
    pub fn attach(self, id: impl Into<ElementId>, trigger: Stateful<Div>) -> TooltipTrigger {
        TooltipTrigger {
            id: id.into(),
            tooltip: self,
            trigger,
            cover: false,
        }
    }
}
/// A trigger with a delayed tooltip. Press and overlay opening cancel pending shows too.
#[derive(IntoElement)]
pub struct TooltipTrigger {
    id: ElementId,
    tooltip: Tooltip,
    trigger: Stateful<Div>,
    cover: bool,
}
impl RenderOnce for TooltipTrigger {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let overlays = self.tooltip.overlays.clone();
        let state = window.use_keyed_state(self.id, cx, |_, cx| {
            let sub = cx.observe(&overlays, |this: &mut TooltipRuntime, _, cx| {
                this.dismiss(cx)
            });
            TooltipRuntime {
                dwell: Dwell::default(),
                timer: None,
                _subscription: sub,
            }
        });
        let bounds = Rc::new(Cell::new(Bounds::default()));
        let measured = bounds.clone();
        let hover = state.clone();
        let press = state.clone();
        let press_out = state.clone();
        let show = state.read(cx).dwell.visible && overlays.read(cx).allows_tooltip();
        let tooltip = self.tooltip;
        position_trigger(self.trigger, self.cover)
            .aria_label(tooltip.label.clone())
            .on_hover(move |inside, window, cx| {
                let allowed = overlays.read(cx).allows_tooltip();
                hover.update(cx, |this, cx| {
                    this.timer = None;
                    if let Some(ticket) = this.dwell.hover(*inside, allowed) {
                        let overlays = overlays.clone();
                        this.timer = Some(cx.spawn_in(window, async move |entity, cx| {
                            cx.background_executor().timer(tooltip.delay).await;
                            let _ = entity.update_in(cx, |this, _, cx| {
                                this.timer = None;
                                this.dwell
                                    .elapsed(ticket, overlays.read(cx).allows_tooltip());
                                cx.notify();
                            });
                        }));
                    }
                    cx.notify();
                });
            })
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                press.update(cx, |this, cx| this.dismiss(cx))
            })
            .on_mouse_down_out(move |_, _, cx| press_out.update(cx, |this, cx| this.dismiss(cx)))
            .child(
                canvas(
                    move |b, _, _| measured.set(b),
                    move |_, _, window, _cx| {
                        // Register a capture listener so all mouse buttons dismiss even if a
                        // child action consumes the press during bubble dispatch.
                        let state = state.clone();
                        window.on_mouse_event(move |_: &MouseDownEvent, phase, _, cx| {
                            if phase == DispatchPhase::Capture {
                                state.update(cx, |this, cx| this.dismiss(cx));
                            }
                        });
                    },
                )
                .absolute()
                .size_full(),
            )
            .when(show, |d| {
                d.child(OverlayLayer::Tooltip.defer(PlacedOverlay::measured(
                    bounds,
                    tooltip.side,
                    tooltip.build(),
                )))
            })
    }
}
impl Kbd {
    /// One bordered caption chip, with no enclosing shortcut well.
    pub fn design(label: impl Into<SharedString>, theme: Theme) -> Div {
        TextSize::Caption
            .apply(row())
            .px(px(Space::S2.px()))
            .rounded(px(Radius::Xs.px()))
            .border_1()
            .border_color(theme.color(C::Border))
            .bg(theme.color(C::Raised))
            .text_color(theme.color(C::TextSecondary))
            .child(label.into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    #[test]
    fn decorated_tooltip_hitboxes_never_consume_control_layout_space() {
        let mut overlay = position_trigger(div().id("cover").size_full(), true);
        assert_eq!(overlay.style().position, Some(Position::Absolute));
        assert_eq!(overlay.style().inset.top, Some(px(0.).into()));
        assert_eq!(overlay.style().inset.left, Some(px(0.).into()));
        let mut ordinary = position_trigger(div().id("ordinary"), false);
        assert_eq!(ordinary.style().position, Some(Position::Relative));
        let mut marker = position_trigger(
            div().id("marker").absolute().top(px(12.)).left(px(4.)),
            false,
        );
        assert_eq!(marker.style().position, Some(Position::Absolute));
        assert_eq!(marker.style().inset.top, Some(px(12.).into()));
        assert_eq!(marker.style().inset.left, Some(px(4.).into()));
    }
    #[test]
    fn shortcut_uses_caption_type_and_tooltip_uses_small_type() {
        for theme in [Theme::Dark, Theme::Light] {
            let mut chip = Kbd::design("⇧⌘S", theme);
            assert_eq!(chip.style().text.font_size, Some(px(11.).into()));
            assert_eq!(chip.style().border_widths.top, Some(px(1.).into()));
            let mut tooltip = Tooltip::content("Save as", Some("⇧⌘S".into()), theme);
            assert_eq!(tooltip.style().text.font_size, Some(px(12.).into()));
        }
    }
    #[test]
    fn press_leave_and_overlay_open_cancel_pending_and_visible_tooltips() {
        let mut dwell = Dwell::default();
        let ticket = dwell.hover(true, true).unwrap();
        assert!(!dwell.visible);
        dwell.dismiss();
        dwell.elapsed(ticket, true);
        assert!(!dwell.visible);
        let ticket = dwell.hover(true, true).unwrap();
        dwell.elapsed(ticket, true);
        assert!(dwell.visible);
        dwell.hover(false, true);
        assert!(!dwell.visible);
        let ticket = dwell.hover(true, true).unwrap();
        dwell.elapsed(ticket, false);
        assert!(!dwell.visible);
        dwell.dismiss();
        dwell.elapsed(ticket, true);
        assert!(!dwell.visible);
        assert_eq!(dwell.hover(true, false), None);
    }
}
