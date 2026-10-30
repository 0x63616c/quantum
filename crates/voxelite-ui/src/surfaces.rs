//! Shared surface chrome. Layout, placement and dismissal remain with the caller.
use crate::{
    Theme, row,
    tokens::{self, metrics, radius, space, type_size},
};
use gpui::{prelude::*, *};

/// Sidebar or rail section surface with optional heading.
pub struct Panel {
    theme: Theme,
    header: Option<SharedString>,
}
impl Panel {
    /// A bordered, padded panel on the canvas background.
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            header: None,
        }
    }
    /// Add a title row before the panel's content.
    pub fn header(mut self, title: impl Into<SharedString>) -> Self {
        self.header = Some(title.into());
        self
    }
    /// Build a panel; append content with `child`/`children`.
    pub fn build(self) -> Div {
        Self::section(self.theme)
            .p(px(metrics::PANEL_PADDING))
            .border_1()
            .border_color(self.theme.line())
            .bg(self.theme.background())
            .when_some(self.header, |d, title| d.child(Self::title(title)))
    }
    /// Unframed section within an existing surface. Insets belong to the section's layout.
    pub fn section(theme: Theme) -> Div {
        div().border_color(theme.line())
    }
    /// Grouped form card, retaining its quiet separator opacity and clipping.
    pub fn card(theme: Theme) -> Div {
        div()
            .rounded(px(radius::XL))
            .border_1()
            .border_color(theme.separator().opacity(0.55))
            .overflow_hidden()
            .bg(theme.card())
    }
    /// Section heading; append trailing actions to the returned row.
    pub fn title(title: impl Into<SharedString>) -> Div {
        row()
            .text_size(px(type_size::TITLE))
            .font_weight(type_size::SEMIBOLD)
            .child(title.into())
    }
    /// Full-height tool rail, sharing the editor's canvas surface.
    pub fn rail(theme: Theme) -> Div {
        Self::section(theme)
            .w(px(metrics::RAIL_WIDTH))
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .py(px(space::LG))
            .gap(px(space::SM))
            .border_r_1()
            .border_color(theme.line())
    }
    /// Editor sidebar surface; add scrolling and content at the call site.
    pub fn sidebar(theme: Theme) -> Div {
        Self::section(theme)
            .w(px(metrics::SIDEBAR_WIDTH))
            .flex_shrink_0()
            .h_full()
            .border_l_1()
            .border_color(theme.line())
            .flex()
            .flex_col()
    }
}

/// Floating surface shared by dialogs, menus, palettes and tooltips.
/// GPUI clips rectangular bounds only: flush child backgrounds must use `inner`
/// (or the equivalent concentric corner on the edge they touch).
pub struct Popover {
    theme: Theme,
    corner: f32,
    background: Hsla,
}
impl Popover {
    /// Standard raised floating surface with XL corners and panel elevation.
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            corner: radius::XL,
            background: theme.surface(),
        }
    }
    /// Compact menu chrome, retaining the existing menu geometry.
    pub fn menu(theme: Theme) -> Self {
        Self {
            theme,
            corner: radius::LG,
            background: theme.card(),
        }
    }
    /// Compact tooltip chrome, retaining the existing tooltip geometry.
    pub fn tooltip(theme: Theme) -> Self {
        Self {
            theme,
            corner: radius::CONTROL,
            background: theme.card(),
        }
    }
    /// Canvas-colored floating content such as the palette chooser.
    pub fn canvas(theme: Theme) -> Self {
        Self {
            background: theme.background(),
            ..Self::new(theme)
        }
    }
    /// Shared border, background, clipping and elevation, without layout constraints.
    pub fn build(self) -> Div {
        div()
            .bg(self.background)
            .border_1()
            .border_color(self.theme.separator())
            .rounded(px(self.corner))
            .shadow(tokens::panel_shadow())
            .overflow_hidden()
    }
    /// Clip a flush inner background around the same corner center as the shell.
    /// `inset` includes the outer border and padding.
    pub fn inner(inset: f32) -> Div {
        div()
            .rounded(px(radius::concentric(radius::XL, inset)))
            .overflow_hidden()
    }
}

#[cfg(test)]
mod tests {
    use super::Popover;
    use crate::{
        Theme,
        tokens::{metrics, radius, space},
    };
    use gpui::{Styled, px};
    #[test]
    fn floating_surface_and_flush_child_have_concentric_corners() {
        let mut outer = Popover::new(Theme::Dark).build();
        let mut inner = Popover::inner(metrics::EDGE + space::SM);
        assert_eq!(
            outer.style().corner_radii.top_left,
            Some(px(radius::XL).into())
        );
        assert_eq!(
            inner.style().corner_radii.top_left,
            Some(px(radius::concentric(radius::XL, metrics::EDGE + space::SM)).into())
        );
        let style = outer.style();
        assert_eq!(style.corner_radii.bottom_right, style.corner_radii.top_left);
    }
}
