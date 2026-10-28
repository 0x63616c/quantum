//! Composable presentation builders; `build` exposes GPUI event and tooltip hooks.
use crate::tokens::{self, ControlSize, Theme, metrics, radius, space, type_size};
use crate::{ColorToken, ControlHeight, Radius, Space, TextSize, TextStyle};
use gpui::{prelude::*, *};

/// A horizontal row aligned on the center line.
pub fn row() -> Div {
    div().flex().items_center()
}
/// Fixed-advance text using the bundled Geist Mono family.
pub fn mono(text: impl Into<SharedString>) -> Div {
    TextSize::Mono.apply(div()).child(text.into())
}
/// An icon from the shell's asset source. The library owns no icon assets.
pub fn icon(name: impl AsRef<str>) -> Svg {
    svg()
        .path(format!("icons/{}.svg", name.as_ref()))
        .size(px(metrics::ICON))
        .flex_shrink_0()
}
/// Compatibility name for callers grouping buttons. Prefer `SegmentedControl` for choices.
pub type ButtonGroup = crate::SegmentedControl;
/// Compatibility name; segments now have their own component.
pub type Segmented = crate::SegmentedControl;

/// Presentation wrapper for a native editor entity or custom input content.
/// The editor retains clipboard, IME, selection and focus ownership.
pub struct FieldFrame {
    pub(crate) theme: Theme,
    pub(crate) focused: bool,
    pub(crate) embedded: bool,
    pub(crate) cells: Option<usize>,
    pub(crate) design: bool,
    pub(crate) invalid: bool,
}
impl FieldFrame {
    /// Standard field frame.
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            focused: false,
            embedded: false,
            cells: None,
            design: false,
            invalid: false,
        }
    }
    /// Show the editor's current focus state.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
    /// Remove the frame for a field inside a search row.
    pub fn embedded(mut self, embedded: bool) -> Self {
        self.embedded = embedded;
        self
    }
    /// Fixed-advance numeric/hex presentation. Input validation remains with the editor.
    pub fn numeric(mut self, cells: usize) -> Self {
        self.cells = Some(cells);
        self
    }
    /// Build the frame; insert the existing editor's canvas or entity as its child.
    pub fn build(self) -> Div {
        if self.design {
            return self.build_design();
        }
        let t = self.theme;
        div()
            .h(px(ControlSize::Lg.height()))
            .w_full()
            .px(px(space::LG))
            .py(px(space::MD))
            .rounded(px(radius::CONTROL))
            .border_1()
            .border_color(if self.focused {
                t.accent()
            } else {
                t.separator()
            })
            .bg(t.background())
            .font_family(type_size::UI)
            .text_style(TextSize::Body)
            .overflow_hidden()
            .cursor_text()
            .when(self.embedded, |d| {
                d.h(px(ControlSize::Md.height()))
                    .px(px(space::NONE))
                    .py(px(space::SM))
                    .border_0()
                    .bg(transparent_black())
            })
            .when_some(self.cells, |d, cells| {
                d.w(px(cells as f32
                    * type_size::BODY_LG
                    * type_size::MONO_ADVANCE
                    + space::LG * 2.
                    // Border plus the caret edge, so an exactly full field does not scroll.
                    + 4.))
                    .flex_shrink_0()
                    .font_family(type_size::MONO)
            })
    }
    /// A search row with icon/editor space and the embedded editor’s keyboard focus ring.
    /// Pass true only when that editor is focused and the last input was keyboard input.
    pub fn search_row(theme: Theme, focus_visible: bool) -> Div {
        TextSize::Body
            .apply(row())
            .h(px(ControlHeight::Large.px()))
            .px(px(Space::S4.px()))
            .gap(px(Space::S3.px()))
            .rounded(px(Radius::Sm.px()))
            .border_1()
            .border_color(theme.color(ColorToken::Border))
            .bg(theme.color(ColorToken::Raised))
            .when(focus_visible, |d| {
                d.shadow(crate::presentation::ring(theme, ColorToken::Surface))
            })
    }
}
/// Selectable square color with an inset edge and optional read-only badge.
pub struct Swatch {
    pub(crate) theme: Theme,
    pub(crate) color: Hsla,
    pub(crate) selected: bool,
    pub(crate) locked: bool,
    pub(crate) size: ControlSize,
    pub(crate) design: Option<crate::ControlHeight>,
}
impl Swatch {
    /// Left half of a previous/current color comparison. Only the exposed edge is rounded.
    pub fn previous(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Stateful<Div> {
        div()
            .id(id)
            .role(Role::Button)
            .aria_label(label)
            .rounded_l(px(radius::CONTROL))
            .cursor_pointer()
    }

    /// Canvas palette swatch, with an inset edge and the established two-pixel selection.
    /// The manager's swatches keep their compact compatibility geometry in `build`.
    pub fn canvas(
        id: impl Into<ElementId>,
        theme: Theme,
        color: impl Into<Hsla>,
        selected: bool,
    ) -> Stateful<Div> {
        Self::design(theme, color)
            .selected(selected)
            .build()
            .id(id)
            .cursor_pointer()
            .hover(move |d| d.border_color(theme.color(ColorToken::TextPrimary)))
    }

    /// Construct a medium swatch. Add IDs, labels and listeners to the returned div.
    pub fn new(theme: Theme, color: impl Into<Hsla>) -> Self {
        Self {
            theme,
            color: color.into(),
            selected: false,
            locked: false,
            size: ControlSize::Md,
            design: None,
        }
    }
    /// Show the selected ring.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// Show a read-only lock badge.
    pub fn locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }
    /// Set the square size.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        if self.design.is_some() {
            self.design = Some(match size {
                ControlSize::Sm => ControlHeight::Dense,
                ControlSize::Md => ControlHeight::Default,
                ControlSize::Lg => ControlHeight::Large,
            });
        }
        self
    }
    /// Build the square; interactive callers should add a label and tab stop.
    pub fn build(self) -> Div {
        if let Some(height) = self.design {
            return self.build_design(height);
        }
        let t = self.theme;
        row()
            .relative()
            .size(px(self.size.height()))
            .flex_shrink_0()
            .justify_center()
            .rounded(px(radius::SM))
            .bg(self.color)
            .border_1()
            .border_color(t.swatch_edge())
            .when(self.selected, |d| d.shadow(tokens::focus_ring(t)))
            .hover(move |d| d.border_color(t.accent()))
            .when(self.locked, |d| {
                d.child(
                    row()
                        .absolute()
                        .bottom(px(space::NONE))
                        .right(px(space::NONE))
                        .rounded(px(radius::SM))
                        .bg(t.card())
                        .child(
                            icon("lock")
                                .size(px(metrics::ICON_SM))
                                .text_color(t.muted()),
                        ),
                )
            })
    }
}
/// Grouped card with inset dividers and an optional header.
pub struct Card {
    theme: Theme,
    header: Option<SharedString>,
}
impl Card {
    /// Construct a grouped card.
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            header: None,
        }
    }
    /// Add a section header above the rows.
    pub fn header(mut self, header: impl Into<SharedString>) -> Self {
        self.header = Some(header.into());
        self
    }
    /// Build rows with separators between them.
    pub fn rows(self, rows: impl IntoIterator<Item = impl IntoElement>) -> Div {
        let t = self.theme;
        crate::Panel::card(t)
            .when_some(self.header, |d, h| {
                d.child(
                    div()
                        .px(px(space::XL))
                        .py(px(space::LG))
                        .font_weight(type_size::SEMIBOLD)
                        .child(h),
                )
            })
            .children(rows.into_iter().enumerate().map(|(i, item)| {
                div()
                    .when(i > 0, |d| {
                        d.child(
                            div()
                                .mx(px(space::XL))
                                .h(px(metrics::EDGE))
                                .bg(t.color(ColorToken::BorderSubtle)),
                        )
                    })
                    .child(item)
            }))
    }
}
/// Synonym for a grouped settings card.
pub type Group = Card;
/// A wrapping label/description beside a non-shrinking control.
pub struct FormRow;
impl FormRow {
    /// Build a form row. Descriptions use the UI font and wrap within the label column.
    pub fn row(
        label: impl Into<SharedString>,
        detail: impl Into<SharedString>,
        control: impl IntoElement,
        theme: Theme,
    ) -> Div {
        let detail = detail.into();
        row()
            .min_h(px(metrics::FORM_HEIGHT))
            .px(px(space::XL))
            .py(px(space::LG))
            .gap(px(space::XL))
            .font_family(type_size::UI)
            .text_style(TextSize::Body)
            .text_color(theme.text())
            .child(
                div()
                    .flex_1()
                    .min_w(px(space::NONE))
                    .child(label.into())
                    .when(!detail.is_empty(), |d| {
                        d.child(
                            div()
                                .mt(px(space::XS))
                                .whitespace_normal()
                                .text_style(TextSize::Small)
                                .line_height(px(type_size::LINE_SM))
                                .text_color(theme.muted())
                                .child(detail),
                        )
                    }),
            )
            .child(div().flex_shrink_0().child(control))
    }
}
/// A sidebar navigation row, also suitable for command results.
pub struct SidebarItem;
impl SidebarItem {
    /// Rich document disclosure with an unrounded separator, preserving the list's
    /// existing geometry. Content and expansion affordances belong to the caller.
    pub fn disclosure(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Stateful<Div> {
        div()
            .id(id)
            .role(Role::Button)
            .aria_label(label)
            .cursor_pointer()
    }

    /// Build a row with native focus styling; append a trailing element with `child`.
    pub fn row(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        glyph: &str,
        selected: bool,
        theme: Theme,
    ) -> Stateful<Div> {
        crate::ListRow::new(id, label, theme)
            .icon(glyph)
            .selected(selected)
            .navigation(true)
            .build()
    }
    /// Build the navigation frame for custom content such as a palette thumbnail.
    /// The label names the complete row for assistive technology.
    pub fn custom(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        selected: bool,
        theme: Theme,
    ) -> Stateful<Div> {
        let label = label.into();
        row()
            .id(id)
            .tab_index(0)
            .role(Role::Button)
            .aria_label(label.clone())
            .aria_selected(selected)
            .min_h(px(ControlSize::Lg.height()))
            .px(px(space::LG))
            .gap(px(space::MD))
            .rounded(px(radius::LG))
            .font_family(type_size::UI)
            .text_color(if selected {
                theme.accent_fg()
            } else {
                theme.text()
            })
            .when(selected, |d| d.bg(theme.tint()))
            .when(!selected, |d| {
                d.hover(move |d| d.bg(theme.surface_raised()))
            })
            .cursor_pointer()
            .focus_visible(move |d| {
                d.bg(if selected {
                    theme.tint()
                } else {
                    theme.surface_raised()
                })
                .shadow(tokens::focus_ring(theme))
            })
    }
}
/// Quiet sidebar group label.
pub struct SidebarSection;
impl SidebarSection {
    /// Build a section header.
    pub fn header(label: impl Into<SharedString>, theme: Theme) -> Div {
        div()
            .px(px(space::LG))
            .pt(px(space::XL))
            .pb(px(space::SM))
            .text_style(TextSize::Caption)
            .font_weight(type_size::SEMIBOLD)
            .text_color(theme.muted())
            .child(label.into())
    }
}
/// Horizontal editor toolbar.
pub struct Toolbar;
impl Toolbar {
    /// Build a fixed-height toolbar; children own actions and layout constraints.
    pub fn row(theme: Theme) -> Div {
        row()
            .h(px(metrics::TOOLBAR_HEIGHT))
            .flex_shrink_0()
            .px(px(space::XXL))
            .gap(px(space::XL))
            .border_b_1()
            .border_color(theme.color(ColorToken::BorderSubtle))
            .bg(theme.color(ColorToken::Surface))
    }
}
/// A non-shrinking toolbar/status divider.
pub struct ToolbarSeparator;
impl ToolbarSeparator {
    /// Build a short vertical separator.
    pub fn line(theme: Theme) -> Div {
        div()
            .w(px(metrics::EDGE))
            .h(px(space::LG))
            .flex_shrink_0()
            .bg(theme.line())
    }
}
/// Modal presentation and focus hooks. The caller owns open/close state and focus restoration.
pub struct Dialog;
impl Dialog {
    /// Center a panel over the owning window. `quiet` is appropriate for About.
    pub fn scrim(theme: Theme, quiet: bool) -> Div {
        div()
            .absolute()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(if quiet {
                theme.quiet_scrim()
            } else {
                theme.scrim()
            })
    }
    /// Build the focusable shell. Attach Escape with `on_key_down` or let it bubble
    /// to the existing modal owner. Call `trap_tab` from the owner's key handler.
    pub fn panel(
        id: impl Into<ElementId>,
        title: impl Into<SharedString>,
        theme: Theme,
        focus: &FocusHandle,
    ) -> Stateful<Div> {
        crate::Popover::design(theme)
            .rounded(px(Radius::Lg.px()))
            .id(id)
            .role(Role::Dialog)
            .aria_label(title)
            .track_focus(focus)
            .tab_group()
            .tab_stop(false)
            .occlude()
            .w(px(metrics::DIALOG_WIDTH))
            .max_w(relative(0.95))
            .max_h(relative(0.95))
            .flex()
            .flex_col()
            .text_color(theme.text())
            .font_family(type_size::UI)
            .text_style(TextSize::Body)
            // GPUI 1.25 only masks a rectangle, inside the border. Inset
            // controls (including their focus rings) and round flush child
            // backgrounds explicitly. The panel shadow paints before this mask;
            // native tooltips paint separately at the window root.
            .overflow_hidden()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
    }
    /// Title bar; append a close button.
    pub fn title(title: impl Into<SharedString>, theme: Theme) -> Div {
        row()
            .px(px(space::XXL))
            .py(px(space::XL))
            .gap(px(space::LG))
            .border_b_1()
            .border_color(theme.separator())
            .child(
                div()
                    .flex_1()
                    .text_style(TextSize::Title)
                    .font_weight(type_size::SEMIBOLD)
                    .child(title.into()),
            )
    }
    /// Optional right-aligned footer actions.
    pub fn footer(theme: Theme) -> Div {
        row()
            .justify_end()
            .gap(px(space::MD))
            .px(px(space::XXL))
            .py(px(space::XL))
            .border_t_1()
            .border_color(theme.separator())
    }
    /// Keep forward/reverse tab traversal inside this modal, including wraparound.
    pub fn trap_tab(focus: &FocusHandle, backwards: bool, window: &mut Window, cx: &mut App) {
        for _ in 0..256 {
            if backwards {
                window.focus_prev(cx);
            } else {
                window.focus_next(cx);
            }
            if focus.contains_focused(window, cx) {
                return;
            }
        }
        window.focus(focus, cx);
    }
}
/// Small semantic label, distinct from an action.
pub struct Badge;
impl Badge {
    /// Build a quiet badge.
    pub fn label(label: impl Into<SharedString>, theme: Theme) -> Div {
        row()
            .px(px(space::SM))
            .py(px(space::XXS))
            .rounded(px(radius::SM))
            .bg(theme.surface_raised())
            .text_color(theme.muted())
            .text_style(TextSize::Caption)
            .child(label.into())
    }
}
/// Synonym for a compact informational badge.
pub type Pill = Badge;
/// A keyboard key cap. Use semantic shortcut tokens for platform-specific glyphs.
pub struct Kbd;
impl Kbd {
    /// Inline keycap around already shaped shortcut glyphs, inheriting text color.
    pub fn frame(color: Hsla) -> Div {
        row()
            .justify_center()
            .flex_shrink_0()
            .min_w(px(metrics::KEYCAP))
            .h(px(metrics::KEYCAP))
            .px(px(space::GLYPH))
            .rounded(px(radius::SM))
            .border_1()
            .border_color(color.opacity(0.28))
    }
    /// Wider read-only shortcut well used in settings.
    pub fn well(theme: Theme) -> Div {
        row()
            .min_w(px(metrics::SHORTCUT_WIDTH))
            .h(px(metrics::SHORTCUT_HEIGHT))
            .px(px(space::MD))
            .justify_center()
            .rounded(px(radius::CONTROL))
            .bg(theme.control())
    }

    /// Build a single key cap.
    pub fn key(label: impl Into<SharedString>, theme: Theme) -> Div {
        mono(label)
            .text_style(TextSize::Caption)
            .px(px(space::XS))
            .rounded(px(radius::SM))
            .border_1()
            .border_color(theme.separator())
            .bg(theme.control())
    }
}
/// Boolean form control; attach the state change callback to `build`'s result.
pub struct Toggle;
impl Toggle {
    /// Build a focusable switch with an accessible current value.
    pub fn switch(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        value: bool,
        theme: Theme,
    ) -> Stateful<Div> {
        row()
            .id(id)
            .tab_index(0)
            .role(Role::Switch)
            .aria_toggled(if value { Toggled::True } else { Toggled::False })
            .aria_label(label)
            .aria_value(if value { "On" } else { "Off" })
            .w(px(metrics::SWITCH))
            .h(px(ControlSize::Sm.height()))
            .flex_shrink_0()
            .p(px(space::XS))
            .rounded_full()
            .bg(theme.color(if value {
                ColorToken::Accent
            } else {
                ColorToken::Border
            }))
            .when(value, |d| d.justify_end())
            .cursor_pointer()
            .focus_visible(move |d| d.shadow(tokens::focus_ring(theme)))
            .child(
                div()
                    .size(px(metrics::THUMB))
                    .rounded_full()
                    .bg(theme.accent_fg())
                    .shadow(tokens::panel_shadow()),
            )
    }
}
/// Shared tooltip appearance; the shell chooses placement and content.
pub fn tooltip(theme: Theme) -> Div {
    crate::Popover::tooltip(theme)
        .build()
        .flex()
        .items_center()
        .flex_none()
        .whitespace_nowrap()
        .px(px(space::LG))
        .py(px(space::SM))
        .text_color(theme.text())
        .font_family(type_size::UI)
        .text_style(TextSize::Small)
        .line_height(px(type_size::LINE_SM))
}

#[cfg(test)]
mod geometry_tests {
    use crate::{Button, IconButton};
    use crate::{
        Segment, Tab, Theme,
        tokens::{ControlSize, metrics, radius},
    };
    use gpui::{Styled, px};

    fn assert_control_corners(element: &mut impl Styled) {
        let corners = &element.style().corner_radii;
        let expected = Some(px(radius::CONTROL).into());
        assert_eq!(corners.top_left, expected);
        assert_eq!(corners.top_right, expected);
        assert_eq!(corners.bottom_left, expected);
        assert_eq!(corners.bottom_right, expected);
    }

    #[test]
    fn actions_tabs_segments_and_rail_use_all_four_control_corners() {
        for theme in [Theme::Dark, Theme::Light] {
            for selected in [false, true] {
                assert_control_corners(
                    &mut Button::new("button", "Action", theme)
                        .selected(selected)
                        .build(),
                );
                assert_control_corners(
                    &mut IconButton::rail("rail", "mirror", "Mirror X", theme)
                        .selected(selected)
                        .build(),
                );
                assert_control_corners(
                    &mut Tab::new("tab", "History", theme).selected(selected).build(),
                );
                assert_control_corners(
                    &mut Segment::new("segment", "Open", theme)
                        .selected(selected)
                        .build(),
                );
            }
        }
    }

    #[test]
    fn rail_geometry_is_identical_with_or_without_axis_badges() {
        for badge in [None, Some("X"), Some("Y"), Some("Z")] {
            for selected in [false, true] {
                let mut button =
                    IconButton::rail("rail", "mirror", "Mirror", Theme::Dark).selected(selected);
                if let Some(badge) = badge {
                    button = button.badge(badge);
                }
                let mut element = button.size(ControlSize::Lg).build();
                let style = element.style();
                let side = Some(px(metrics::RAIL_BUTTON).into());
                assert_eq!(style.size.width, side);
                assert_eq!(style.size.height, side);
                assert_eq!(style.min_size.width, side);
                assert_eq!(style.max_size.width, side);
                assert_eq!(style.min_size.height, side);
                assert_eq!(style.max_size.height, side);
                assert_eq!(style.border_widths.left, Some(px(metrics::EDGE).into()));
                assert_eq!(style.border_widths.right, style.border_widths.left);
            }
        }
    }

    #[test]
    fn icon_control_sizes_remain_square() {
        for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
            let mut element = IconButton::new("icon", "close", "Close", Theme::Dark)
                .size(size)
                .build();
            assert_eq!(element.style().size.width, Some(px(size.height()).into()));
            let style = element.style();
            assert_eq!(style.size.height, style.size.width);
        }
    }
}
