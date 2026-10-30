use super::*;
use crate::{ColorToken as C, ControlHeight, Radius, Space, TextSize, presentation, tokens::state};

/// Explicit action hierarchy, independent of IDs, labels and icons.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonStyle {
    /// Accent fill with white text.
    Primary,
    /// Bordered raised fill.
    #[default]
    Secondary,
    /// Quiet transparent action.
    Ghost,
    /// Quiet destructive foreground.
    Danger,
}
impl From<ButtonStyle> for ButtonVariant {
    fn from(style: ButtonStyle) -> Self {
        match style {
            ButtonStyle::Primary => Self::Primary,
            ButtonStyle::Secondary => Self::Secondary,
            ButtonStyle::Ghost => Self::Ghost,
            ButtonStyle::Danger => Self::Danger,
        }
    }
}
/// Controlled state for previews and testing; ordinary controls use native interaction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlState {
    /// Native pointer and keyboard interaction.
    #[default]
    Rest,
    /// Pointer is hovering.
    Hovered,
    /// Pointer is held down.
    Pressed,
    /// Keyboard focus (never persistent selection).
    Focused,
    /// No input, 40% opacity.
    Disabled,
    /// Busy, retains label and geometry, suppresses activation.
    Loading,
}
#[derive(Clone, Copy)]
pub(super) struct Design {
    pub style: ButtonStyle,
    pub height: ControlHeight,
    pub state: ControlState,
    pub surface: C,
}
impl Default for Design {
    fn default() -> Self {
        Self {
            style: ButtonStyle::Secondary,
            height: ControlHeight::Default,
            state: ControlState::Rest,
            surface: C::Surface,
        }
    }
}
impl Button {
    /// New design-system action; legacy `new` retains existing screen pixels.
    pub fn design(id: impl Into<ElementId>, label: impl Into<SharedString>, theme: Theme) -> Self {
        Self::new(id, label, theme).style(ButtonStyle::Secondary)
    }
    /// Opt into the design system and choose an explicit hierarchy.
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.design.get_or_insert_default().style = style;
        self
    }
    /// Surrounding surface used by the 1px focus-ring offset (default: Surface).
    pub fn on_surface(mut self, surface: C) -> Self {
        self.design.get_or_insert_default().surface = surface;
        self
    }
    /// Typed hierarchy with frozen legacy appearance, for migration in #134.
    pub fn legacy_style(self, style: ButtonStyle) -> Self {
        self.variant(style.into())
    }
    /// Set a design-system height (24/28/32).
    pub fn height(mut self, height: ControlHeight) -> Self {
        self.design.get_or_insert_default().height = height;
        self
    }
    /// Preview an interaction state. Disabled and loading also suppress input.
    pub fn state(mut self, state: ControlState) -> Self {
        self.design.get_or_insert_default().state = state;
        self
    }
    /// Show progress while preserving the label and action's footprint.
    pub fn loading(self, loading: bool) -> Self {
        self.state(if loading {
            ControlState::Loading
        } else {
            ControlState::Rest
        })
    }
    /// Attach an action guarded by disabled/loading state. Prefer this to unguarded
    /// listeners on `build()` when the control's state can change.
    pub fn on_click(
        mut self,
        listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.listener = Some(Box::new(listener));
        self
    }
    /// Whether this builder accepts activation (including its loading state).
    pub fn is_interactive(&self) -> bool {
        !self.disabled
            && self
                .design
                .is_none_or(|d| !matches!(d.state, ControlState::Disabled | ControlState::Loading))
    }
    pub(super) fn build_design(self, design: Design) -> Stateful<Div> {
        let t = self.theme;
        let interaction = if self.disabled {
            ControlState::Disabled
        } else {
            design.state
        };
        let unavailable =
            self.disabled || matches!(interaction, ControlState::Disabled | ControlState::Loading);
        let disabled = self.disabled || interaction == ControlState::Disabled;
        let busy = interaction == ControlState::Loading;
        let icon_only = self.variant == ButtonVariant::Icon;
        let style = design.style;
        let fg = if self.selected {
            t.color(C::TextPrimary)
        } else {
            foreground(t, style)
        };
        let mut d = TextSize::Body
            .apply(Self::custom(self.id, self.label.clone()))
            .relative()
            .h(px(design.height.px()))
            .min_w(px(design.height.px()))
            .flex_shrink_0()
            .rounded(px(Radius::Sm.px()))
            .justify_center()
            .px(px(if design.height == ControlHeight::Dense {
                Space::S3.px()
            } else {
                Space::S4.px()
            }))
            .gap(px(Space::S3.px()))
            .text_color(fg)
            .aria_selected(self.selected)
            .bg(background(t, style, self.selected, interaction))
            .when(style == ButtonStyle::Secondary, |d| {
                d.border_1().border_color(t.color(C::Border))
            })
            .when(disabled, |d| d.opacity(state::DISABLED))
            .when(unavailable, |d| {
                d.tab_stop(false)
                    .cursor_default()
                    .aria_description(if busy { "Loading" } else { "Unavailable" })
            })
            .when(!unavailable, |d| {
                d.tab_index(self.tab_index)
                    .hover(move |d| {
                        d.bg(background(t, style, self.selected, ControlState::Hovered))
                    })
                    .active(move |d| {
                        d.bg(background(t, style, self.selected, ControlState::Pressed))
                    })
                    .focus_visible(move |d| d.shadow(presentation::ring(t, design.surface)))
            })
            .when(interaction == ControlState::Focused && !unavailable, |d| {
                d.shadow(presentation::ring(t, design.surface))
            })
            .when_some(self.icon, |d, name| {
                d.child(
                    presentation::glyph(name)
                        .text_color(if self.selected {
                            t.color(C::AccentText)
                        } else {
                            fg
                        })
                        .when(busy, |d| d.opacity(0.)),
                )
            })
            .when(!icon_only && !self.label.is_empty(), |d| {
                d.child(presentation::ellipsis(self.label).when(busy, |d| d.opacity(0.)))
            })
            .when(busy, |d| {
                d.child(
                    div()
                        .absolute()
                        .top(px(Space::S0.px()))
                        .left(px(Space::S0.px()))
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(crate::Spinner::new("button-spinner", t).color(
                            if style == ButtonStyle::Primary {
                                C::OnAccent
                            } else {
                                C::TextSecondary
                            },
                        )),
                )
            })
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());
        if let Some(listener) = self.listener {
            d = d.on_click(move |e, window, cx| {
                if !unavailable {
                    listener(e, window, cx);
                }
            });
        }
        d
    }
}
pub(super) fn foreground(t: Theme, style: ButtonStyle) -> Hsla {
    t.color(match style {
        ButtonStyle::Primary => C::OnAccent,
        ButtonStyle::Danger => C::Danger,
        _ => C::TextPrimary,
    })
}
pub(super) fn background(
    t: Theme,
    style: ButtonStyle,
    selected: bool,
    interaction: ControlState,
) -> Hsla {
    if style == ButtonStyle::Primary && !selected {
        return match interaction {
            ControlState::Hovered => presentation::neutral(t, C::Accent, state::PRIMARY_HOVER),
            ControlState::Pressed => presentation::neutral(t, C::Accent, state::PRESSED),
            _ => t.color(C::Accent),
        };
    }
    let base = if selected {
        presentation::selected(t)
    } else if style == ButtonStyle::Secondary {
        t.color(C::Raised)
    } else {
        transparent_black()
    };
    match interaction {
        ControlState::Hovered => base.blend(t.color(C::OnAccent).opacity(state::HOVER)),
        ControlState::Pressed => base.blend(t.color(C::OnAccent).opacity(state::PRESSED)),
        _ => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    #[test]
    fn all_styles_sizes_and_states_build_the_audit_geometry() {
        for theme in [Theme::Dark, Theme::Light] {
            for style in [
                ButtonStyle::Primary,
                ButtonStyle::Secondary,
                ButtonStyle::Ghost,
                ButtonStyle::Danger,
            ] {
                for (height, expected) in [
                    (ControlHeight::Dense, 24.),
                    (ControlHeight::Default, 28.),
                    (ControlHeight::Large, 32.),
                ] {
                    for interaction in [
                        ControlState::Rest,
                        ControlState::Hovered,
                        ControlState::Pressed,
                        ControlState::Focused,
                        ControlState::Disabled,
                        ControlState::Loading,
                    ] {
                        let mut button = Button::design("test", "Action", theme)
                            .style(style)
                            .height(height)
                            .state(interaction)
                            .build();
                        let built = button.style();
                        assert_eq!(built.size.height, Some(px(expected).into()));
                        assert_eq!(built.corner_radii.top_left, Some(px(6.).into()));
                        assert_eq!(built.text.font_size, Some(px(13.).into()));
                        assert_eq!(built.text.line_height, Some(px(20.).into()));
                        assert_eq!(
                            built.opacity,
                            if interaction == ControlState::Disabled {
                                Some(0.4)
                            } else {
                                None
                            }
                        );
                        assert_eq!(built.text.color, Some(foreground(theme, style)));
                        assert_eq!(
                            built.background,
                            Some(background(theme, style, false, interaction).into())
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn disabled_and_loading_actions_never_accept_activation() {
        let t = Theme::Dark;
        assert!(Button::design("a", "Action", t).is_interactive());
        for state in [ControlState::Disabled, ControlState::Loading] {
            assert!(
                !Button::design("a", "Action", t)
                    .state(state)
                    .is_interactive()
            );
        }
        let mut disabled = Button::design("a", "Action", t)
            .state(ControlState::Hovered)
            .disabled(true)
            .build();
        assert_eq!(disabled.style().background, Some(t.color(C::Raised).into()));
    }
    #[test]
    fn button_hierarchy_never_depends_on_label_id_or_icon_name() {
        for (id, label, icon) in [
            ("save", "Delete", "send"),
            ("delete", "Save", "trash"),
            ("other", "Other", "copy"),
        ] {
            let mut button = Button::design(id, label, Theme::Dark)
                .icon(icon)
                .style(ButtonStyle::Primary)
                .build();
            assert_eq!(
                button.style().background,
                Some(Theme::Dark.color(C::Accent).into())
            );
            assert_eq!(
                button.style().text.color,
                Some(Theme::Dark.color(C::OnAccent))
            );
        }
    }
    #[test]
    fn state_resolution_keeps_selection_separate_from_focus_and_disabled() {
        let t = Theme::Dark;
        assert_eq!(
            background(t, ButtonStyle::Ghost, true, ControlState::Focused),
            t.color(C::Accent).opacity(0.16)
        );
        assert_eq!(
            background(t, ButtonStyle::Ghost, false, ControlState::Focused),
            transparent_black()
        );
        let mut button = Button::design("test", "Test", t)
            .disabled(true)
            .state(ControlState::Focused)
            .build();
        assert_eq!(button.style().opacity, Some(0.4));
        assert!(
            button
                .style()
                .box_shadow
                .as_ref()
                .is_none_or(|shadows| shadows.is_empty())
        );
        let ring = presentation::ring(t, C::Surface);
        assert_eq!(ring.len(), 2);
        assert_eq!(ring[0].spread_radius, px(3.));
        assert_eq!(ring[1].spread_radius, px(1.));
    }
}
