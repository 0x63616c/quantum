//! Menu presentation. Open state, focus, dismissal and actions belong to the caller.
use crate::{
    Button, Theme, icon, row,
    tokens::{metrics, radius, space},
};
use gpui::{prelude::*, *};

/// Shared menu button and anchored panel styling.
pub struct Menu;
impl Menu {
    /// A secondary menu trigger with a trailing disclosure glyph.
    pub fn button(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        open: bool,
        theme: Theme,
    ) -> Stateful<Div> {
        Button::new(id, label, theme)
            .selected(open)
            .build()
            .aria_expanded(open)
            .child(
                icon("chevron")
                    .size(px(metrics::ICON_SM))
                    .text_color(theme.text()),
            )
    }
    /// Canvas menu with the editor's existing floating-panel dimensions and focus hooks.
    pub fn canvas(id: impl Into<ElementId>, theme: Theme, focus: &FocusHandle) -> Stateful<Div> {
        crate::Dialog::panel(id, "Menu", theme, focus)
            .role(Role::Menu)
            .bg(theme.background())
            .p(px(metrics::POPOVER_PADDING))
    }
    /// A menu surface; position it within a relative container or GPUI anchor.
    pub fn panel(id: impl Into<ElementId>, theme: Theme) -> Stateful<Div> {
        crate::Popover::menu(theme)
            .build()
            .id(id)
            .role(Role::Menu)
            .occlude()
            .p(px(metrics::MENU_PADDING))
            .w(px(metrics::MENU_WIDTH))
    }
    /// A full-width menu entry with a visible keyboard selection.
    pub fn item(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        selected: bool,
        theme: Theme,
    ) -> Stateful<Div> {
        let label = label.into();
        row()
            .id(id)
            .role(Role::MenuItem)
            .aria_label(label.clone())
            .tab_index(0)
            .h(px(crate::tokens::ControlSize::Md.height()))
            .px(px(space::LG))
            .rounded(px(radius::concentric(
                radius::LG,
                metrics::EDGE + space::XS,
            )))
            .cursor_pointer()
            .text_color(theme.text())
            .when(selected, |d| d.bg(theme.selected()))
            .hover(move |d| d.bg(theme.selected()))
            .focus_visible(move |d| d.bg(theme.selected()))
            .child(label)
    }
}

/// A separator inside a menu or grouped surface.
pub struct Separator;
impl Separator {
    /// Full-width subtle hairline with token spacing.
    pub fn line(theme: Theme) -> Div {
        use crate::{ColorToken, Space, tokens::geometry};
        div()
            .h(px(geometry::EDGE))
            .my(px(Space::S2.px()))
            .bg(theme.color(ColorToken::BorderSubtle))
    }
}
/// A 28px menu action with a trailing shortcut or submenu chevron.
#[derive(IntoElement)]
pub struct MenuItem {
    button: Button,
    shortcut: Option<SharedString>,
    submenu: Option<bool>,
    theme: Theme,
}
impl MenuItem {
    /// A normal menu entry. IDs do not determine styling.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, theme: Theme) -> Self {
        Self {
            button: Button::design(id, label, theme)
                .style(crate::ButtonStyle::Ghost)
                .on_surface(crate::ColorToken::Overlay),
            shortcut: None,
            submenu: None,
            theme,
        }
    }
    /// Right-aligned shortcut, rendered as one chip.
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
    /// Optional leading asset icon.
    pub fn icon(mut self, name: impl Into<SharedString>) -> Self {
        self.button = self.button.icon(name);
        self
    }
    /// Destructive foreground; group destructive items below `Separator`.
    pub fn danger(mut self) -> Self {
        self.button = self.button.style(crate::ButtonStyle::Danger);
        self
    }
    /// Controlled selected item during keyboard menu navigation.
    pub fn selected(mut self, selected: bool) -> Self {
        self.button = self.button.selected(selected);
        self
    }
    /// Disabled entries do not receive focus or activate callbacks.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.button = self.button.disabled(disabled);
        self
    }
    /// Deterministic state preview.
    pub fn state(mut self, state: crate::ControlState) -> Self {
        self.button = self.button.state(state);
        self
    }
    /// Guarded activation callback.
    pub fn on_click(
        mut self,
        callback: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.button = self.button.on_click(callback);
        self
    }
    /// Build a full-width row. The menu owner handles arrow navigation and dismissal.
    pub fn build(self) -> Stateful<Div> {
        use crate::{ColorToken as C, Kbd, Space, presentation};
        self.button
            .build()
            .role(Role::MenuItem)
            .w_full()
            .justify_start()
            .child(div().flex_1().min_w(px(Space::S0.px())))
            .when_some(self.shortcut, |d, s| d.child(Kbd::design(s, self.theme)))
            .when_some(self.submenu, |d, open| {
                d.aria_expanded(open).child(
                    presentation::glyph("chevron-right")
                        .text_color(self.theme.color(C::TextSecondary)),
                )
            })
    }
}
impl RenderOnce for MenuItem {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.build()
    }
}
/// Controlled nested menu. Its measured trigger anchors the content on either side.
#[derive(IntoElement)]
pub struct Submenu {
    item: MenuItem,
    open: bool,
    content: AnyElement,
}
impl Submenu {
    /// Construct a submenu trigger and its content. Host owns open state and updates
    /// `OverlayState` alongside it; attach the toggle callback to the `MenuItem`.
    pub fn new(mut item: MenuItem, open: bool, content: impl IntoElement) -> Self {
        item.submenu = Some(open);
        Self {
            item,
            open,
            content: content.into_any_element(),
        }
    }
    /// Build the trigger with a measured, flipping submenu when open.
    pub fn build(self) -> Stateful<Div> {
        use std::{cell::Cell, rc::Rc};
        let bounds = Rc::new(Cell::new(Bounds::default()));
        let measured = bounds.clone();
        self.item
            .build()
            .relative()
            .child(
                canvas(move |b, _, _| measured.set(b), |_, _, _, _| {})
                    .absolute()
                    .size_full(),
            )
            .when(self.open, |d| {
                d.child(
                    crate::OverlayLayer::Popover.defer(crate::PlacedOverlay::measured(
                        bounds,
                        crate::PopoverSide::Right,
                        self.content,
                    )),
                )
            })
    }
}
impl RenderOnce for Submenu {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.build()
    }
}
impl Menu {
    /// New menu surface with 28px `MenuItem` rows. Legacy `panel` is frozen for #134.
    pub fn design(id: impl Into<ElementId>, theme: Theme) -> Stateful<Div> {
        crate::Popover::design(theme)
            .id(id)
            .role(Role::Menu)
            .occlude()
            .w(px(crate::tokens::geometry::MENU_WIDTH))
    }
    /// A separated destructive group; keeps destructive actions out of normal groups.
    pub fn destructive_group(items: impl IntoIterator<Item = MenuItem>, theme: Theme) -> Div {
        div()
            .child(Separator::line(theme))
            .children(items.into_iter().map(MenuItem::danger))
    }
}

#[cfg(test)]
mod design_tests {
    use super::*;
    use core::prelude::v1::test;
    #[test]
    fn menu_rows_remain_28_pixels_with_shortcuts_or_submenus() {
        let t = Theme::Dark;
        let mut plain = MenuItem::new("plain", "Action", t).shortcut("⌘K").build();
        let mut submenu = Submenu::new(
            MenuItem::new("sub", "Recent", t),
            true,
            Menu::design("child", t),
        )
        .build();
        assert_eq!(plain.style().size.height, Some(px(28.).into()));
        assert_eq!(submenu.style().size.height, plain.style().size.height);
        assert_eq!(plain.style().corner_radii.top_left, Some(px(6.).into()));
    }
}
