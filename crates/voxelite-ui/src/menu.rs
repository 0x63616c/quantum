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
