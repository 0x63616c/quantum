//! Composable presentation builders; `build` exposes GPUI event and tooltip hooks.
use crate::tokens::{self, ControlSize, Theme, metrics, radius, space, type_size};
use gpui::{prelude::*, *};

/// A horizontal row aligned on the center line.
pub fn row() -> Div {
    div().flex().items_center()
}
/// Fixed-advance text using the bundled Geist Mono family.
pub fn mono(text: impl Into<SharedString>) -> Div {
    div()
        .font_family(type_size::MONO)
        .text_size(px(type_size::BODY))
        .child(text.into())
}
/// An icon from the shell's asset source. The library owns no icon assets.
pub fn icon(name: impl AsRef<str>) -> Svg {
    svg()
        .path(format!("icons/{}.svg", name.as_ref()))
        .size(px(metrics::ICON))
        .flex_shrink_0()
}
/// Button visual hierarchy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Strong primary action.
    Primary,
    /// Bordered secondary action.
    #[default]
    Secondary,
    /// Quiet text action.
    Ghost,
    /// Square icon action.
    Icon,
    /// Destructive action.
    Danger,
}
/// A labelled action with keyboard focus, selected and disabled states.
/// Attach listeners and tooltips to `build()`; attach listeners only when enabled.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    theme: Theme,
    variant: ButtonVariant,
    size: ControlSize,
    icon: Option<SharedString>,
    disabled: bool,
    selected: bool,
    tab_index: isize,
}
impl Button {
    /// Construct a secondary button. Labels also name icon-only buttons for accessibility.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, theme: Theme) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            theme,
            variant: ButtonVariant::Secondary,
            size: ControlSize::Md,
            icon: None,
            disabled: false,
            selected: false,
            tab_index: 0,
        }
    }
    /// Set the action hierarchy.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Set the shared height.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// Add a leading icon from `icons/<name>.svg`.
    pub fn icon(mut self, name: impl Into<SharedString>) -> Self {
        self.icon = Some(name.into());
        self
    }
    /// Remove the tab stop and use disabled colors.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Show persistent selection.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// Set the tab order within the surrounding focus group.
    pub fn tab_index(mut self, index: isize) -> Self {
        self.tab_index = index;
        self
    }
    /// Build a GPUI element, exposing native event, accessibility and tooltip hooks.
    pub fn build(self) -> Stateful<Div> {
        let t = self.theme;
        let primary = self.variant == ButtonVariant::Primary;
        let icon_only = self.variant == ButtonVariant::Icon;
        let fg = if self.disabled {
            t.disabled()
        } else if primary {
            t.background()
        } else if self.variant == ButtonVariant::Danger {
            t.danger()
        } else if self.selected {
            t.accent()
        } else {
            t.text()
        };
        row()
            .id(self.id)
            .role(Role::Button)
            .aria_label(self.label.clone())
            .aria_selected(self.selected)
            .h(px(self.size.height()))
            .min_w(px(self.size.height()))
            .flex_shrink_0()
            .justify_center()
            .px(px(if icon_only { space::SM } else { space::LG }))
            .gap(px(space::MD))
            .rounded(px(radius::MD))
            .font_family(type_size::UI)
            .text_size(px(type_size::BODY))
            .text_color(fg)
            .when(
                self.variant == ButtonVariant::Secondary || self.variant == ButtonVariant::Danger,
                |d| d.border_1().border_color(t.separator()).bg(t.control()),
            )
            .when(primary, |d| {
                d.bg(if self.disabled { t.control() } else { t.text() })
            })
            .when(self.selected && !self.disabled, |d| d.bg(t.selected()))
            .when(!self.disabled, |d| {
                d.tab_index(self.tab_index)
                    .cursor_pointer()
                    .focus_visible(move |d| {
                        d.bg(if primary {
                            t.text()
                        } else if self.selected {
                            t.selected()
                        } else {
                            t.surface_raised()
                        })
                        .shadow(tokens::focus_ring(t))
                    })
                    .hover(move |d| {
                        d.bg(if primary {
                            t.text().opacity(0.88)
                        } else {
                            t.surface_raised()
                        })
                    })
                    .active(move |d| d.bg(t.selected()).text_color(t.accent()))
            })
            .when(self.disabled, |d| {
                d.tab_stop(false).aria_description("Unavailable")
            })
            .when_some(self.icon, |d, name| d.child(icon(name).text_color(fg)))
            .when(!icon_only && !self.label.is_empty(), |d| {
                d.child(
                    div()
                        .min_w(px(0.))
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(self.label),
                )
            })
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
    }
}
impl RenderOnce for Button {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.build()
    }
}
/// A square button with an accessible label.
pub struct IconButton;
impl IconButton {
    /// Construct an icon-only button; further configure it as a `Button`.
    pub fn button(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        label: impl Into<SharedString>,
        theme: Theme,
    ) -> Button {
        Button::new(id, label, theme)
            .variant(ButtonVariant::Icon)
            .icon(name)
    }
}
/// A compact group of related actions; child buttons own selection and callbacks.
pub struct ButtonGroup;
impl ButtonGroup {
    /// Build a group that can contain any number of buttons.
    pub fn row(theme: Theme) -> Div {
        row()
            .gap(px(space::XS))
            .p(px(space::XS))
            .rounded(px(radius::LG))
            .border_1()
            .border_color(theme.separator())
            .bg(theme.surface())
    }
}
/// Segmented choice group. Mark one child button as selected.
pub type Segmented = ButtonGroup;

/// Presentation wrapper for a native editor entity or custom input content.
/// The editor retains clipboard, IME, selection and focus ownership.
pub struct TextField {
    theme: Theme,
    focused: bool,
    embedded: bool,
    cells: Option<usize>,
}
impl TextField {
    /// Standard field frame.
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            focused: false,
            embedded: false,
            cells: None,
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
        let t = self.theme;
        div()
            .h(px(ControlSize::Lg.height()))
            .w_full()
            .px(px(space::LG))
            .py(px(space::MD))
            .rounded(px(radius::MD))
            .border_1()
            .border_color(if self.focused {
                t.accent()
            } else {
                t.separator()
            })
            .bg(t.background())
            .font_family(type_size::UI)
            .text_size(px(type_size::BODY_LG))
            .overflow_hidden()
            .cursor_text()
            .when(self.embedded, |d| {
                d.h(px(ControlSize::Md.height()))
                    .px(px(0.))
                    .py(px(space::SM))
                    .border_0()
                    .bg(transparent_black())
            })
            .when_some(self.cells, |d, cells| {
                d.w(px(cells as f32
                    * type_size::BODY_LG
                    * type_size::MONO_ADVANCE
                    + space::LG * 2.))
                    .flex_shrink_0()
                    .font_family(type_size::MONO)
            })
    }
    /// A search field row with space for an icon, editor, and clear button.
    pub fn search_row(theme: Theme, focused: bool) -> Div {
        row()
            .h(px(ControlSize::Lg.height()))
            .px(px(space::LG))
            .gap(px(space::SM))
            .rounded(px(radius::LG))
            .border_1()
            .border_color(if focused {
                theme.accent()
            } else {
                theme.separator()
            })
            .bg(theme.surface_raised())
            .font_family(type_size::UI)
            .text_size(px(type_size::BODY_LG))
    }
}
/// Selectable square color with an inset edge and optional read-only badge.
pub struct Swatch {
    theme: Theme,
    color: Hsla,
    selected: bool,
    locked: bool,
    size: ControlSize,
}
impl Swatch {
    /// Construct a medium swatch. Add IDs, labels and listeners to the returned div.
    pub fn new(theme: Theme, color: impl Into<Hsla>) -> Self {
        Self {
            theme,
            color: color.into(),
            selected: false,
            locked: false,
            size: ControlSize::Md,
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
        self
    }
    /// Build the square; interactive callers should add a label and tab stop.
    pub fn build(self) -> Div {
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
                        .bottom(px(0.))
                        .right(px(0.))
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
        div()
            .rounded(px(radius::XL))
            .border_1()
            .border_color(t.separator().opacity(0.55))
            .overflow_hidden()
            .bg(t.card())
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
                                .bg(t.separator().opacity(0.7)),
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
            .text_size(px(type_size::BODY_LG))
            .text_color(theme.text())
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .child(label.into())
                    .when(!detail.is_empty(), |d| {
                        d.child(
                            div()
                                .mt(px(space::XS))
                                .whitespace_normal()
                                .text_size(px(type_size::BODY))
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
    /// Build a row with native focus styling; append a trailing element with `child`.
    pub fn row(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        glyph: &str,
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
            .child(icon(glyph).text_color(if selected {
                theme.accent_fg()
            } else {
                theme.muted()
            }))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(label),
            )
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
            .text_size(px(type_size::CAPTION))
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
            .border_color(theme.line())
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
        div()
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
            .bg(theme.surface())
            .text_color(theme.text())
            .font_family(type_size::UI)
            .text_size(px(type_size::BODY_LG))
            .border_1()
            .border_color(theme.separator())
            .rounded(px(radius::XL))
            .shadow(tokens::panel_shadow())
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
                    .text_size(px(type_size::HEADLINE))
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
            .text_size(px(type_size::CAPTION))
            .child(label.into())
    }
}
/// Synonym for a compact informational badge.
pub type Pill = Badge;
/// A keyboard key cap. Use semantic shortcut tokens for platform-specific glyphs.
pub struct Kbd;
impl Kbd {
    /// Build a single key cap.
    pub fn key(label: impl Into<SharedString>, theme: Theme) -> Div {
        mono(label)
            .text_size(px(type_size::CAPTION))
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
            .bg(if value { theme.tint() } else { theme.control() })
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
    row()
        .flex_none()
        .whitespace_nowrap()
        .px(px(space::LG))
        .py(px(space::SM))
        .rounded(px(radius::MD))
        .border_1()
        .border_color(theme.separator())
        .bg(theme.card())
        .text_color(theme.text())
        .font_family(type_size::UI)
        .text_size(px(type_size::BODY))
        .line_height(px(type_size::LINE_SM))
        .shadow(tokens::panel_shadow())
}
