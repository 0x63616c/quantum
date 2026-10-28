//! Shared construction helpers for Settings controls.
use super::*;

impl SettingsView {
    pub(super) fn chrome_button(
        &self,
        id: &'static str,
        glyph: &'static str,
        label: &'static str,
        enabled: bool,
        t: Theme,
    ) -> Stateful<Div> {
        IconButton::button(id, glyph, label, t)
            .disabled(!enabled)
            .build()
            .tooltip(super::super::hint(label, t))
    }

    pub(super) fn sidebar_row(
        &self,
        section: Section,
        setting: Option<&'static str>,
        index: usize,
        t: Theme,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let selected = setting.is_none() && section == self.history.current();
        let label = setting.unwrap_or(section.title());
        SidebarItem::row(
            ("settings-result", index),
            label,
            if setting.is_some() {
                "chevron-right"
            } else {
                section.icon()
            },
            selected,
            t,
        )
        .mb(px(space::XXS))
        .when(setting.is_some(), |d| {
            d.ml(px(space::XXL)).text_size(px(type_size::CAPTION))
        })
        .on_click(cx.listener(move |this, _, window, cx| {
            this.navigate(section, cx);
            window.focus(&this.focus, cx);
        }))
    }
    pub(super) fn form_row(
        &self,
        label: impl Into<SharedString>,
        detail: &'static str,
        control: impl IntoElement,
        t: Theme,
    ) -> Div {
        FormRow::row(label, detail, control, t)
    }
    pub(super) fn group(&self, rows: Vec<Div>, t: Theme) -> Div {
        Card::new(t).rows(rows)
    }
    pub(super) fn toggle(
        &self,
        id: &'static str,
        value: bool,
        edit: fn(&mut Preferences),
        t: Theme,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        Toggle::switch(id, id, value, t)
            .tooltip(super::super::hint(if value { "On" } else { "Off" }, t))
            .on_click(cx.listener(move |_, _, _, cx| change(cx, edit)))
    }
    pub(super) fn button(&self, id: &'static str, label: &'static str, t: Theme) -> Stateful<Div> {
        Button::new(id, label, t).build()
    }
}
