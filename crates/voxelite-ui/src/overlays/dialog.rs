use super::OverlayLayer;
use crate::{
    Button, ButtonStyle, ColorToken as C, ControlHeight, Dialog, Radius, Space, TextSize, Theme,
    presentation, row,
    tokens::{geometry, state},
};
use gpui::{prelude::*, *};
use std::rc::Rc;

type Cancel = Rc<dyn Fn(&mut Window, &mut App)>;
/// A modal composition that always supplies Cancel and Escape dismissal.
/// The host owns open state, focus restoration and its `OverlayState` entry.
#[derive(IntoElement)]
pub struct ModalDialog {
    id: ElementId,
    title: SharedString,
    theme: Theme,
    focus: FocusHandle,
    cancel: Cancel,
    body: Option<AnyElement>,
    primary: Option<Button>,
    danger: Option<Button>,
}
impl Dialog {
    /// Compose a design-system dialog. A cancel handler is required by the API.
    pub fn design(
        id: impl Into<ElementId>,
        title: impl Into<SharedString>,
        theme: Theme,
        focus: &FocusHandle,
        cancel: impl Fn(&mut Window, &mut App) + 'static,
    ) -> ModalDialog {
        ModalDialog {
            id: id.into(),
            title: title.into(),
            theme,
            focus: focus.clone(),
            cancel: Rc::new(cancel),
            body: None,
            primary: None,
            danger: None,
        }
    }
}
impl ModalDialog {
    /// Wrapping body content, with the body type scale inherited.
    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }
    /// Primary action at the right edge, after Cancel.
    pub fn primary(mut self, action: Button) -> Self {
        self.primary = Some(
            action
                .style(ButtonStyle::Primary)
                .on_surface(C::Overlay)
                .height(ControlHeight::Large),
        );
        self
    }
    /// Destructive secondary action at the left edge.
    pub fn danger(mut self, action: Button) -> Self {
        self.danger = Some(
            action
                .style(ButtonStyle::Danger)
                .on_surface(C::Overlay)
                .height(ControlHeight::Large),
        );
        self
    }
    /// The modal panel, useful for inline gallery previews. Production uses `overlay`.
    pub fn build(self) -> Stateful<Div> {
        let cancel_key = self.cancel.clone();
        let cancel_button = self.cancel;
        let focus = self.focus.clone();
        let t = self.theme;
        presentation::surface(t, C::Overlay, Radius::Lg)
            .id(self.id)
            .role(Role::Dialog)
            .aria_label(self.title.clone())
            .track_focus(&self.focus)
            .tab_group()
            .tab_stop(false)
            .occlude()
            .w(px(geometry::DIALOG_WIDTH))
            .max_w(relative(0.95))
            .max_h(relative(0.95))
            .flex()
            .flex_col()
            .p(px(Space::S6.px()))
            .gap(px(Space::S5.px()))
            .child(TextSize::Title.apply(div()).child(self.title))
            .when_some(self.body, |d, body| {
                d.child(
                    div()
                        .id("dialog-body")
                        .min_h(px(Space::S0.px()))
                        .overflow_y_scroll()
                        .whitespace_normal()
                        .child(body),
                )
            })
            .child(
                row()
                    .gap(px(Space::S3.px()))
                    .children(self.danger)
                    .child(div().flex_1())
                    .child(
                        Button::design("dialog-cancel", "Cancel", t)
                            .on_surface(C::Overlay)
                            .height(ControlHeight::Large)
                            .on_click(move |_, w, cx| cancel_button(w, cx)),
                    )
                    .children(self.primary),
            )
            .capture_key_down(move |e, window, cx| match e.keystroke.key.as_str() {
                "escape" => {
                    cancel_key(window, cx);
                    cx.stop_propagation();
                }
                "tab" => {
                    Dialog::trap_tab(&focus, e.keystroke.modifiers.shift, window, cx);
                    cx.stop_propagation();
                }
                _ => {}
            })
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
    }
    /// Center the dialog over a 50% black scrim in the top overlay layer.
    pub fn overlay(self) -> Deferred {
        OverlayLayer::Dialog.defer(
            div()
                .id("dialog-scrim")
                .occlude()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .absolute()
                .top(px(Space::S0.px()))
                .left(px(Space::S0.px()))
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(state::scrim())
                .child(self.build()),
        )
    }
}
impl RenderOnce for ModalDialog {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.overlay()
    }
}
