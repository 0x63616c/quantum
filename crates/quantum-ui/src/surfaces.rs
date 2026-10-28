//! Shared surface chrome. Layout, placement and dismissal remain with the caller.
use crate::{ColorToken, Radius, TextSize};
use crate::{
    Theme, row,
    tokens::{self, metrics, radius, space},
};
use gpui::{prelude::*, *};

/// Sidebar or rail section surface with optional heading.
pub struct Panel {
    theme: Theme,
    header: Option<SharedString>,
    design: bool,
    actions: Option<AnyElement>,
    body: Option<AnyElement>,
    footer: Option<AnyElement>,
}
impl Panel {
    /// A bordered, padded panel on the canvas background.
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            header: None,
            design: false,
            actions: None,
            body: None,
            footer: None,
        }
    }
    /// Add a title row before the panel's content.
    pub fn header(mut self, title: impl Into<SharedString>) -> Self {
        self.header = Some(title.into());
        self
    }
    /// Build a panel; append content with `child`/`children`.
    pub fn build(self) -> Div {
        if self.design {
            return self.build_design();
        }
        Self::section(self.theme)
            .p(px(metrics::PANEL_PADDING))
            .border_1()
            .border_color(self.theme.line())
            .bg(self.theme.background())
            .when_some(self.header, |d, title| d.child(Self::title(title)))
    }
    /// Unframed section within an existing surface. Insets belong to the section's layout.
    pub fn section(theme: Theme) -> Div {
        TextSize::Body
            .apply(div())
            .border_color(theme.color(ColorToken::BorderSubtle))
    }
    /// Grouped form card, retaining its quiet separator opacity and clipping.
    pub fn card(theme: Theme) -> Div {
        div()
            .rounded(px(Radius::Md.px()))
            .border_1()
            .border_color(theme.color(ColorToken::BorderSubtle))
            .overflow_hidden()
            .bg(theme.color(ColorToken::Raised))
    }
    /// Section heading; append trailing actions to the returned row.
    pub fn title(title: impl Into<SharedString>) -> Div {
        TextSize::Title.apply(row()).child(title.into())
    }
    /// Full-height tool rail, sharing the editor's canvas surface.
    pub fn rail(theme: Theme) -> Div {
        Self::section(theme)
            .bg(theme.color(ColorToken::Surface))
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
            .bg(theme.color(ColorToken::Surface))
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

impl Panel {
    /// Design-system panel with 16px padding and header/body/footer slots.
    pub fn design(theme: Theme) -> Self {
        Self {
            design: true,
            ..Self::new(theme)
        }
    }
    /// Trailing header actions; pair with `header`.
    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions = Some(actions.into_any_element());
        self
    }
    /// Independently scrollable body. Give the panel a constrained height to scroll.
    pub fn body(mut self, id: impl Into<ElementId>, body: impl IntoElement) -> Self {
        use crate::Space;
        self.body = Some(
            div()
                .id(id)
                .flex_1()
                .min_h(px(Space::S0.px()))
                // Reserve room for the 3px outer focus extent inside GPUI's rectangular
                // scroll mask, without changing the panel's 16px content gutter.
                .m(px(-Space::S2.px()))
                .p(px(Space::S2.px()))
                .overflow_y_scroll()
                .child(body)
                .into_any_element(),
        );
        self
    }
    /// Optional fixed footer content.
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }
    fn build_design(self) -> Div {
        use crate::{ColorToken as C, Radius, Space, TextSize, presentation};
        let t = self.theme;
        presentation::surface(t, C::Surface, Radius::Md)
            .flex()
            .flex_col()
            .min_h(px(Space::S0.px()))
            .p(px(Space::S5.px()))
            .gap(px(Space::S5.px()))
            .when_some(self.header, |d, title| {
                d.child(
                    row()
                        .gap(px(Space::S3.px()))
                        .child(
                            TextSize::Title
                                .apply(presentation::ellipsis(title))
                                .flex_1(),
                        )
                        .when_some(self.actions, |d, actions| {
                            d.child(div().flex_shrink_0().child(actions))
                        }),
                )
            })
            .children(self.body)
            .when_some(self.footer, |d, footer| {
                d.child(
                    div()
                        .flex_shrink_0()
                        .pt(px(Space::S4.px()))
                        .border_t_1()
                        .border_color(t.color(C::BorderSubtle))
                        .child(footer),
                )
            })
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
    fn design_panel_has_sixteen_pixel_padding_and_scroll_slot() {
        use crate::{ColorToken, Panel};
        let mut panel = Panel::design(Theme::Dark)
            .header("Title")
            .body("scroll", gpui::div())
            .footer(gpui::div())
            .build();
        assert_eq!(panel.style().padding.top, Some(px(16.).into()));
        assert_eq!(panel.style().padding.left, Some(px(16.).into()));
        assert_eq!(panel.style().corner_radii.top_left, Some(px(10.).into()));
        assert_eq!(
            panel.style().background,
            Some(Theme::Dark.color(ColorToken::Surface).into())
        );
    }
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
