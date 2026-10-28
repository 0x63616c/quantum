//! Action controls. Legacy constructors preserve #134 screen pixels.
use crate::tokens::{self, ControlSize, Theme, metrics, radius, space, type_size};
use crate::{icon, row};
use gpui::{prelude::*, *};
mod design;
pub use design::{ButtonStyle, ControlState};

type ClickListener = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// Legacy button hierarchy; migrate to `ButtonStyle` in #134.
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
    design: Option<design::Design>,
    listener: Option<ClickListener>,
}
impl Button {
    /// Custom-content action frame. Inherits typography and layout from its parent;
    /// callers add their content, dimensions and existing event/focus behavior.
    /// This is useful for menu triggers, rich choices and document rows.
    pub fn custom(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Stateful<Div> {
        row()
            .id(id)
            .role(Role::Button)
            .aria_label(label)
            .rounded(px(radius::CONTROL))
            .cursor_pointer()
    }
    /// Block-content action for rich tiles. Existing card corner geometry is retained.
    pub fn tile(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Stateful<Div> {
        div()
            .id(id)
            .role(Role::Button)
            .aria_label(label)
            .rounded(px(radius::CONTROL))
            .cursor_pointer()
    }

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
            design: None,
            listener: None,
        }
    }
    /// Set the action hierarchy.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        if let Some(design) = &mut self.design {
            design.style = match variant {
                ButtonVariant::Primary => ButtonStyle::Primary,
                ButtonVariant::Secondary => ButtonStyle::Secondary,
                ButtonVariant::Danger => ButtonStyle::Danger,
                ButtonVariant::Ghost | ButtonVariant::Icon => ButtonStyle::Ghost,
            };
        }
        self
    }
    /// Set the shared height.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        if let Some(design) = &mut self.design {
            design.height = match size {
                ControlSize::Sm => crate::ControlHeight::Dense,
                ControlSize::Md => crate::ControlHeight::Default,
                ControlSize::Lg => crate::ControlHeight::Large,
            };
        }
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
        if let Some(design) = self.design {
            return self.build_design(design);
        }
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
        Self::custom(self.id, self.label.clone())
            .aria_selected(self.selected)
            .h(px(self.size.height()))
            .min_w(px(self.size.height()))
            .flex_shrink_0()
            .justify_center()
            .px(px(if icon_only { space::SM } else { space::LG }))
            .gap(px(space::MD))
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
                        .min_w(px(space::NONE))
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(self.label),
                )
            })
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .when_some(self.listener, |d, listener| {
                d.on_click(move |e, w, cx| {
                    if !self.disabled {
                        listener(e, w, cx);
                    }
                })
            })
    }
}
impl RenderOnce for Button {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.build()
    }
}
/// A square button with an accessible label and an optional overlaid corner badge.
#[derive(IntoElement)]
pub struct IconButton {
    button: Button,
    side: f32,
    rail: bool,
    badge: Option<SharedString>,
    overlays: Option<Entity<crate::OverlayState>>,
}
impl IconButton {
    /// Square frame for caller-provided glyph content (for example independently colored icons).
    pub fn custom(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        size: ControlSize,
    ) -> Stateful<Div> {
        Button::custom(id, label)
            .size(px(size.height()))
            .justify_center()
    }

    /// Construct a square icon action. The label is accessible but not painted.
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        label: impl Into<SharedString>,
        theme: Theme,
    ) -> Self {
        Self {
            button: Button::design(id, label, theme)
                .variant(ButtonVariant::Icon)
                .icon(name),
            side: crate::ControlHeight::Default.px(),
            rail: false,
            badge: None,
            overlays: None,
        }
    }
    /// Compatibility constructor, now returning an actual square `IconButton`.
    pub fn button(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        label: impl Into<SharedString>,
        theme: Theme,
    ) -> Self {
        Self::new(id, name, label, theme)
    }
    /// A tool rail action with the same square geometry and border in every state.
    pub fn rail(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        label: impl Into<SharedString>,
        theme: Theme,
    ) -> Self {
        Self {
            side: metrics::RAIL_BUTTON,
            rail: true,
            ..Self::new(id, name, label, theme)
        }
    }
    /// Set both dimensions together. Rail buttons always keep the rail metric.
    pub fn size(mut self, size: ControlSize) -> Self {
        if !self.rail {
            self.side = size.height();
        }
        self
    }
    /// Overlay a small monospace badge without moving the centered glyph.
    pub fn badge(mut self, text: impl Into<SharedString>) -> Self {
        self.badge = Some(text.into());
        self
    }
    /// Show persistent selection.
    pub fn selected(mut self, selected: bool) -> Self {
        self.button = self.button.selected(selected);
        self
    }
    /// Disable keyboard traversal and action colors.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.button = self.button.disabled(disabled);
        self
    }
    /// Set keyboard traversal order.
    pub fn tab_index(mut self, index: isize) -> Self {
        self.button = self.button.tab_index(index);
        self
    }
    /// Build a square frame; badges are absolutely positioned inside its border.
    pub fn build(self) -> Stateful<Div> {
        let t = self.button.theme;
        let selected = self.button.selected;
        self.button
            .build()
            .relative()
            .size(px(self.side))
            .min_w(px(self.side))
            .max_w(px(self.side))
            .min_h(px(self.side))
            .max_h(px(self.side))
            .px(px(space::NONE))
            .gap(px(space::NONE))
            .when(self.rail, |d| {
                d.border_1().border_color(if selected {
                    t.accent().opacity(0.4)
                } else {
                    t.background()
                })
            })
            .when_some(self.badge, |d, badge| {
                d.child(
                    div()
                        .absolute()
                        .bottom(px(space::XXS))
                        .right(px(space::XXS))
                        .font_family(type_size::MONO)
                        .text_size(px(type_size::BADGE))
                        .line_height(px(type_size::BADGE))
                        .child(badge),
                )
            })
    }
}
impl RenderOnce for IconButton {
    fn render(mut self, _: &mut Window, _: &mut App) -> impl IntoElement {
        if let Some(overlays) = self.overlays.take() {
            let id = self.button.id.clone();
            let tooltip =
                crate::Tooltip::new(self.button.label.clone(), self.button.theme, &overlays);
            tooltip.attach(id, self.build()).into_any_element()
        } else {
            self.build().into_any_element()
        }
    }
}

impl IconButton {
    /// Design-system ghost icon with a required tooltip/accessibility label.
    /// Render the builder itself to include its tooltip; `build` exposes the raw frame.
    pub fn design(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        label: impl Into<SharedString>,
        theme: Theme,
        overlays: &Entity<crate::OverlayState>,
    ) -> Self {
        let mut button = Self::new(id, name, label, theme);
        button.button = button.button.style(ButtonStyle::Ghost);
        button.side = crate::ControlHeight::Default.px();
        button.overlays = Some(overlays.clone());
        button
    }
    /// Design-system dense/default size. Rail controls keep their fixed 36px square.
    pub fn height(mut self, height: crate::ControlHeight) -> Self {
        if !self.rail {
            self.side = height.px();
        }
        self.button = self.button.height(height);
        self
    }
    /// Controlled preview state.
    pub fn state(mut self, state: ControlState) -> Self {
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
