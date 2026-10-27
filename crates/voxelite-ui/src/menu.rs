//! Menu presentation. Open state, focus, dismissal and actions belong to the caller.
use crate::{
    Button, Theme, icon, row,
    tokens::{self, radius, space},
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
            .child(icon("chevron").size(px(12.)).text_color(theme.text()))
    }
    /// A menu surface; position it within a relative container or GPUI anchor.
    pub fn panel(id: impl Into<ElementId>, theme: Theme) -> Stateful<Div> {
        div()
            .id(id)
            .role(Role::Menu)
            .occlude()
            .p(px(space::XS))
            .w(px(224.))
            .rounded(px(radius::LG))
            .border_1()
            .border_color(theme.separator())
            .bg(theme.card())
            .shadow(tokens::panel_shadow())
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
            .h(px(32.))
            .px(px(space::LG))
            .rounded(px(radius::SM))
            .cursor_pointer()
            .text_color(theme.text())
            .when(selected, |d| d.bg(theme.selected()))
            .hover(move |d| d.bg(theme.selected()))
            .focus_visible(move |d| d.bg(theme.selected()))
            .child(label)
    }
}
