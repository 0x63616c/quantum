//! Tabs and segmented choices; selection and actions remain controlled by the app.
use crate::{
    Button, ButtonVariant, Theme, row,
    tokens::{self, ControlSize, metrics, radius, space, type_size},
};
use gpui::{prelude::*, *};

/// An underline tab with rounded hover, press and keyboard focus highlights.
#[derive(IntoElement)]
pub struct Tab {
    id: ElementId,
    label: SharedString,
    theme: Theme,
    selected: bool,
    tab_index: isize,
    design: bool,
    state: crate::ControlState,
}
impl Tab {
    /// Construct a tab. The selected underline does not participate in layout.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, theme: Theme) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            theme,
            selected: false,
            tab_index: 0,
            design: false,
            state: crate::ControlState::Rest,
        }
    }
    /// Mark the currently visible panel.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// Set keyboard traversal order.
    pub fn tab_index(mut self, index: isize) -> Self {
        self.tab_index = index;
        self
    }
    fn underline(theme: Theme) -> Div {
        div()
            .absolute()
            .left(px(metrics::TAB_UNDERLINE_INSET))
            .right(px(metrics::TAB_UNDERLINE_INSET))
            .bottom(px(space::NONE))
            .h(px(metrics::TAB_UNDERLINE))
            .bg(theme.text())
    }
    /// Build the accessible tab; attach the panel-selection callback to this element.
    pub fn build(self) -> Stateful<Div> {
        if self.design {
            return self.build_design();
        }
        let t = self.theme;
        row()
            .id(self.id)
            .relative()
            .role(Role::Tab)
            .aria_label(self.label.clone())
            .aria_selected(self.selected)
            .tab_index(self.tab_index)
            .h(px(metrics::TAB_HEIGHT))
            .min_w(px(metrics::TAB_HEIGHT))
            .flex_shrink_0()
            .justify_center()
            .px(px(space::LG))
            // Retain the old content baseline without making the underline a border.
            .pb(px(metrics::TAB_UNDERLINE))
            .rounded(px(radius::CONTROL))
            .font_family(type_size::UI)
            .text_size(px(type_size::BODY))
            .text_color(t.text())
            .cursor_pointer()
            .hover(move |d| d.bg(t.surface_raised()))
            .focus_visible(move |d| d.bg(t.surface_raised()).shadow(tokens::focus_ring(t)))
            .active(move |d| d.bg(t.selected()).text_color(t.accent()))
            .child(
                div()
                    .min_w(px(space::NONE))
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(self.label),
            )
            .when(self.selected, |d| d.child(Self::underline(t)))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
    }
}
impl RenderOnce for Tab {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.build()
    }
}
/// Sidebar tab strip. Each child is a `Tab` with its own callback.
pub struct TabBar;
impl TabBar {
    /// Shared inset, spacing and bottom separator for sidebar tabs.
    pub fn row(theme: Theme) -> Stateful<Div> {
        row()
            .id("tab-bar")
            .role(Role::TabList)
            .px(px(metrics::PANEL_GUTTER - space::MD))
            .gap(px(space::XS))
            .py(px(space::MD))
            .border_b_1()
            .border_color(theme.line())
    }
}

/// One choice in a segmented control.
#[derive(IntoElement)]
pub struct Segment {
    button: Button,
}
impl Segment {
    /// Construct a quiet choice; selection is controlled by the caller.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, theme: Theme) -> Self {
        Self {
            button: Button::new(id, label, theme).variant(ButtonVariant::Ghost),
        }
    }
    /// Set selection.
    pub fn selected(mut self, selected: bool) -> Self {
        self.button = self.button.selected(selected);
        self
    }
    /// Disable the choice while retaining its layout.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.button = self.button.disabled(disabled);
        self
    }
    /// Set the common control size.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.button = self.button.size(size);
        self
    }
    /// Add a leading glyph.
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.button = self.button.icon(icon);
        self
    }
    /// Build the segment with shared control focus, press and hover styles.
    pub fn build(self) -> Stateful<Div> {
        self.button.build()
    }
}
impl RenderOnce for Segment {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.build()
    }
}
/// A group of related, independently focusable choices.
pub struct SegmentedControl;
impl SegmentedControl {
    /// Bordered grouped choices, used for camera views and palette targets.
    pub fn row(theme: Theme) -> Div {
        Self::inline()
            .gap(px(space::XS))
            .p(px(space::XS))
            .rounded(px(radius::LG))
            .border_1()
            .border_color(theme.separator())
            .bg(theme.surface())
    }
    /// Unframed choices, used by compact sidebar filters.
    pub fn inline() -> Div {
        row().gap(px(space::XS))
    }
}

/// Audit name for the shared tab strip.
pub type Tabs = TabBar;
impl Tab {
    /// Token-only underline tab with the shared design-system states.
    pub fn design(id: impl Into<ElementId>, label: impl Into<SharedString>, theme: Theme) -> Self {
        Self {
            design: true,
            ..Self::new(id, label, theme)
        }
    }
    /// Controlled visual state; selection remains independent.
    pub fn state(mut self, state: crate::ControlState) -> Self {
        self.state = state;
        self
    }
    fn build_design(self) -> Stateful<Div> {
        use crate::{ButtonStyle, ColorToken as C, Space, tokens::state};
        Button::design(self.id, self.label, self.theme)
            .style(ButtonStyle::Ghost)
            .state(self.state)
            .selected(self.selected)
            .tab_index(self.tab_index)
            .build()
            .relative()
            .role(Role::Tab)
            .when(self.selected, |d| {
                d.child(
                    div()
                        .absolute()
                        .bottom(px(Space::S0.px()))
                        .left(px(Space::S1.px()))
                        .right(px(Space::S1.px()))
                        .h(px(state::INDICATOR))
                        .bg(self.theme.color(C::AccentText)),
                )
            })
    }
}
impl TabBar {
    /// Token-only tab strip. Place outside the scrolling body to keep it sticky.
    pub fn design(id: impl Into<ElementId>, theme: Theme) -> Stateful<Div> {
        use crate::{ColorToken as C, Space};
        row()
            .id(id)
            .role(Role::TabList)
            .flex_shrink_0()
            .gap(px(Space::S2.px()))
            .border_b_1()
            .border_color(theme.color(C::BorderSubtle))
    }
}
impl Segment {
    /// New fill-indicator choice; legacy `new` keeps its previous typography.
    pub fn design(id: impl Into<ElementId>, label: impl Into<SharedString>, theme: Theme) -> Self {
        Self {
            button: Button::design(id, label, theme)
                .style(crate::ButtonStyle::Ghost)
                .on_surface(crate::ColorToken::Raised),
        }
    }
    /// Shared design-system height.
    pub fn height(mut self, height: crate::ControlHeight) -> Self {
        self.button = self.button.height(height);
        self
    }
    /// Controlled preview state.
    pub fn state(mut self, state: crate::ControlState) -> Self {
        self.button = self.button.state(state);
        self
    }
    /// Guarded activation callback.
    pub fn on_click(
        mut self,
        listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.button = self.button.on_click(listener);
        self
    }
}
impl SegmentedControl {
    /// New grouped choices; the selected segment supplies the fill, with no underline.
    pub fn design(theme: Theme) -> Div {
        use crate::{ColorToken as C, Radius, Space, presentation};
        presentation::surface(theme, C::Raised, Radius::Sm)
            .flex()
            .items_center()
            .p(px(Space::S1.px()))
            .gap(px(Space::S1.px()))
    }
}

#[cfg(test)]
mod tests {
    use super::Tab;
    use crate::{
        Theme,
        tokens::{metrics, space},
    };
    use gpui::{Position, Styled, px};
    #[test]
    fn design_tabs_and_segments_share_selection_fill_and_height() {
        use crate::{ColorToken, Segment};
        let mut tab = Tab::design("tab", "Tab", Theme::Dark)
            .selected(true)
            .build();
        let mut segment = Segment::design("segment", "Segment", Theme::Dark)
            .selected(true)
            .build();
        assert_eq!(tab.style().size.height, Some(px(28.).into()));
        assert_eq!(segment.style().size.height, tab.style().size.height);
        assert_eq!(segment.style().background, tab.style().background);
        assert_eq!(
            tab.style().background,
            Some(Theme::Dark.color(ColorToken::Accent).opacity(0.16).into())
        );
        assert_eq!(segment.style().border_widths.bottom, None);
    }
    #[test]
    fn underline_is_inset_and_does_not_participate_in_layout() {
        let mut line = Tab::underline(Theme::Dark);
        let style = line.style();
        assert_eq!(style.position, Some(Position::Absolute));
        assert_eq!(
            style.inset.left,
            Some(px(metrics::TAB_UNDERLINE_INSET).into())
        );
        assert_eq!(style.inset.right, style.inset.left);
        assert_eq!(style.inset.bottom, Some(px(space::NONE).into()));
        assert_eq!(style.size.height, Some(px(metrics::TAB_UNDERLINE).into()));
        let mut tab = Tab::new("tab", "History", Theme::Dark)
            .selected(true)
            .build();
        assert_eq!(
            tab.style().size.height,
            Some(px(metrics::TAB_HEIGHT).into())
        );
        assert_eq!(tab.style().border_widths.bottom, None);
        assert_eq!(
            tab.style().padding.bottom,
            Some(px(metrics::TAB_UNDERLINE).into())
        );
    }
}
