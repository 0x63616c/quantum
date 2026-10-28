//! Controlled list, settings and empty-state compositions.
use crate::{
    Button, ColorToken as C, ControlHeight, ControlState, Radius, Space, TextSize, Theme,
    presentation, row, tokens::state,
};
use gpui::{prelude::*, *};

/// A single-line title with metadata and fixed leading/trailing slots.
#[derive(IntoElement)]
pub struct ListRow {
    id: ElementId,
    title: SharedString,
    meta: Option<SharedString>,
    theme: Theme,
    leading: Option<AnyElement>,
    leading_icon: Option<SharedString>,
    trailing: Option<AnyElement>,
    selected: bool,
    state: ControlState,
    navigation: bool,
    listener: Option<RowListener>,
}
type RowListener = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
impl ListRow {
    /// Shared interactive frame for rows whose rich content is composed by the host.
    /// Selection, hover, press, focus, typography and corners match ordinary rows.
    pub fn frame(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        theme: Theme,
        selected: bool,
    ) -> Stateful<Div> {
        Button::design(id, "", theme)
            .style(crate::ButtonStyle::Ghost)
            .selected(selected)
            .build()
            .aria_label(label)
            .h_auto()
            .min_h(px(ControlHeight::Default.px()))
            .justify_start()
    }
    /// Construct a row; long titles always ellipsize inside the flexible column.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>, theme: Theme) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            meta: None,
            theme,
            leading: None,
            leading_icon: None,
            trailing: None,
            selected: false,
            state: ControlState::Rest,
            navigation: false,
            listener: None,
        }
    }
    /// Leading icon or thumbnail. The frame clips thumbnails to the shared radius.
    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self.leading_icon = None;
        self
    }
    /// Leading asset icon, colored by the row's selected state.
    pub fn icon(mut self, name: impl AsRef<str>) -> Self {
        self.leading_icon = Some(name.as_ref().to_owned().into());
        self.leading = None;
        self
    }
    /// Secondary line, also truncated within the row.
    pub fn meta(mut self, meta: impl Into<SharedString>) -> Self {
        self.meta = Some(meta.into());
        self
    }
    /// Fixed trailing actions, which do not shrink with the title.
    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }
    /// Persistent selection uses accent fill, primary text and accent icon.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// Add the 2px leading selection bar for navigation rows.
    pub fn navigation(mut self, navigation: bool) -> Self {
        self.navigation = navigation;
        self
    }
    /// Controlled preview state; native interaction remains the default.
    pub fn state(mut self, state: ControlState) -> Self {
        self.state = state;
        self
    }
    /// Suppress activation and focus while retaining layout.
    pub fn disabled(self, disabled: bool) -> Self {
        self.state(if disabled {
            ControlState::Disabled
        } else {
            ControlState::Rest
        })
    }
    /// Guard activation using the row's disabled/loading state.
    pub fn on_click(
        mut self,
        listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.listener = Some(Box::new(listener));
        self
    }
    /// Build a row; prefer `on_click` above for guarded actions.
    pub fn build(self) -> Stateful<Div> {
        let t = self.theme;
        let leading = self.leading.or_else(|| {
            self.leading_icon.map(|name| {
                presentation::glyph(name)
                    .text_color(t.color(if self.selected {
                        C::AccentText
                    } else {
                        C::TextSecondary
                    }))
                    .into_any_element()
            })
        });
        let disabled = matches!(self.state, ControlState::Disabled | ControlState::Loading);
        let base = if self.selected {
            presentation::selected(t)
        } else {
            transparent_black()
        };
        let bg = |opacity| base.blend(t.color(C::OnAccent).opacity(opacity));
        let mut d = TextSize::Body
            .apply(row())
            .id(self.id)
            .relative()
            .role(Role::Button)
            .aria_label(self.title.clone())
            .aria_selected(self.selected)
            .min_h(px(ControlHeight::Default.px()))
            .px(px(Space::S4.px()))
            .py(px(Space::S2.px()))
            .gap(px(Space::S3.px()))
            .rounded(px(Radius::Sm.px()))
            .text_color(t.color(C::TextPrimary))
            .bg(match self.state {
                ControlState::Hovered => bg(state::HOVER),
                ControlState::Pressed => bg(state::PRESSED),
                _ => base,
            })
            .when(disabled, |d| {
                d.opacity(state::DISABLED)
                    .tab_stop(false)
                    .aria_description("Unavailable")
            })
            .when(!disabled, |d| {
                d.tab_index(0)
                    .cursor_pointer()
                    .hover(move |d| d.bg(base.blend(t.color(C::OnAccent).opacity(state::HOVER))))
                    .active(move |d| d.bg(base.blend(t.color(C::OnAccent).opacity(state::PRESSED))))
                    .focus_visible(move |d| d.shadow(presentation::ring(t, C::Surface)))
            })
            .when(self.state == ControlState::Focused, |d| {
                d.shadow(presentation::ring(t, C::Surface))
            })
            .when_some(leading, |d, leading| {
                d.child(
                    div()
                        .flex_shrink_0()
                        .rounded(px(Radius::Xs.px()))
                        .overflow_hidden()
                        .text_color(t.color(if self.selected {
                            C::AccentText
                        } else {
                            C::TextSecondary
                        }))
                        .child(leading),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w(px(Space::S0.px()))
                    .child(presentation::ellipsis(self.title))
                    .when_some(self.meta, |d, meta| {
                        d.child(
                            TextSize::Small
                                .apply(presentation::ellipsis(meta))
                                .text_color(t.color(C::TextSecondary)),
                        )
                    }),
            )
            .when_some(self.trailing, |d, trailing| {
                d.child(div().flex_shrink_0().child(trailing))
            })
            .when(self.selected && self.navigation, |d| {
                d.child(
                    div()
                        .absolute()
                        .left(px(Space::S0.px()))
                        .top(px(Space::S2.px()))
                        .bottom(px(Space::S2.px()))
                        .w(px(state::INDICATOR))
                        .bg(t.color(C::AccentText)),
                )
            });
        if let Some(listener) = self.listener {
            d = d.on_click(move |e, w, cx| {
                if !disabled {
                    listener(e, w, cx);
                }
            });
        }
        d
    }
}
impl RenderOnce for ListRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.build()
    }
}

/// Wrapping title/description alongside a nonshrinking control.
pub struct SettingsRow;
impl SettingsRow {
    /// Compose a settings row. Descriptions use proportional text and may wrap.
    pub fn row(
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
        control: impl IntoElement,
        theme: Theme,
    ) -> Div {
        let description = description.into();
        TextSize::Body
            .apply(row())
            .py(px(Space::S4.px()))
            .gap(px(Space::S5.px()))
            .text_color(theme.color(C::TextPrimary))
            .child(
                div()
                    .flex_1()
                    .min_w(px(Space::S0.px()))
                    .child(title.into())
                    .when(!description.is_empty(), |d| {
                        d.child(
                            TextSize::Small
                                .apply(div())
                                .mt(px(Space::S2.px()))
                                .whitespace_normal()
                                .text_color(theme.color(C::TextSecondary))
                                .child(description),
                        )
                    }),
            )
            .child(div().flex_shrink_0().child(control))
    }
}
/// A shared caption for settings groups, menu groups and sections.
pub struct SectionLabel;
impl SectionLabel {
    /// Caption 11/16 medium, never a custom heading size.
    pub fn label(label: impl Into<SharedString>, theme: Theme) -> Div {
        TextSize::Caption
            .apply(div())
            .text_color(theme.color(C::TextSecondary))
            .py(px(Space::S3.px()))
            .child(label.into())
    }
}
/// Empty-state composition with exactly two optional action slots.
#[derive(IntoElement)]
pub struct EmptyState {
    icon: SharedString,
    message: SharedString,
    theme: Theme,
    primary: Option<Button>,
    secondary: Option<Button>,
}
impl EmptyState {
    /// One glyph and a single explanatory line.
    pub fn new(
        icon: impl Into<SharedString>,
        message: impl Into<SharedString>,
        theme: Theme,
    ) -> Self {
        Self {
            icon: icon.into(),
            message: message.into(),
            theme,
            primary: None,
            secondary: None,
        }
    }
    /// Set or replace the first action; a third action is unrepresentable.
    pub fn primary(mut self, action: Button) -> Self {
        self.primary = Some(action);
        self
    }
    /// Set or replace the optional second action.
    pub fn secondary(mut self, action: Button) -> Self {
        self.secondary = Some(action);
        self
    }
    /// Build a centered state within the parent panel.
    pub fn build(self) -> Div {
        TextSize::Body
            .apply(div())
            .flex()
            .flex_col()
            .items_center()
            .gap(px(Space::S4.px()))
            .p(px(Space::S8.px()))
            .text_color(self.theme.color(C::TextSecondary))
            .child(presentation::glyph(self.icon).text_color(self.theme.color(C::TextSecondary)))
            .child(presentation::ellipsis(self.message))
            .when(self.primary.is_some() || self.secondary.is_some(), |d| {
                d.child(
                    row()
                        .gap(px(Space::S3.px()))
                        .children(self.primary)
                        .children(self.secondary),
                )
            })
    }
}
impl RenderOnce for EmptyState {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.build()
    }
}

impl crate::SidebarItem {
    /// Opt into the shared ListRow API; legacy `row` preserves existing screen pixels.
    pub fn design(
        id: impl Into<ElementId>,
        title: impl Into<SharedString>,
        theme: Theme,
    ) -> ListRow {
        ListRow::new(id, title, theme).navigation(true)
    }
}
impl crate::FormRow {
    /// New settings-row presentation; legacy `row` remains frozen until #134.
    pub fn design(
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
        control: impl IntoElement,
        theme: Theme,
    ) -> Div {
        SettingsRow::row(title, description, control, theme).px(px(Space::S5.px()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    #[test]
    fn selected_list_rows_keep_primary_text_and_disabled_opacity() {
        for theme in [Theme::Dark, Theme::Light] {
            let mut row = ListRow::new("row", "Title", theme)
                .selected(true)
                .disabled(true)
                .build();
            assert_eq!(row.style().opacity, Some(0.4));
            assert_eq!(row.style().text.color, Some(theme.color(C::TextPrimary)));
            assert_eq!(
                row.style().background,
                Some(theme.color(C::Accent).opacity(0.16).into())
            );
            assert_eq!(row.style().corner_radii.top_left, Some(px(6.).into()));
        }
    }
    #[test]
    fn empty_state_actions_replace_slots_instead_of_appending() {
        let t = Theme::Dark;
        let empty = EmptyState::new("folder", "No projects", t)
            .primary(Button::design("first", "First", t))
            .primary(Button::design("second", "Second", t))
            .secondary(Button::design("other", "Other", t));
        assert!(empty.primary.is_some());
        assert!(empty.secondary.is_some());
        let mut built = empty.build();
        assert_eq!(built.style().padding.top, Some(px(48.).into()));
    }
    #[test]
    fn settings_and_section_labels_use_distinct_type() {
        for theme in [Theme::Dark, Theme::Light] {
            let mut settings = SettingsRow::row("Title", "Description", div(), theme);
            let mut section = SectionLabel::label("Group", theme);
            assert_eq!(settings.style().text.font_size, Some(px(13.).into()));
            assert_eq!(settings.style().text.line_height, Some(px(20.).into()));
            assert_eq!(section.style().text.font_size, Some(px(11.).into()));
            assert_eq!(section.style().text.font_weight, Some(FontWeight::MEDIUM));
        }
    }
}
