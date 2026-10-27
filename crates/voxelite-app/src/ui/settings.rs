//! A shared settings view: a native utility window or an in-page web modal.
use super::{EditorView, icon, mono, row};
use crate::{
    actions::{self, Op},
    shortcuts::ShortcutPlatform,
    text_field::TextField,
    theme::Theme,
};
use gpui::{prelude::*, *};
use voxelite_app::preferences::{Appearance, Preferences};

#[derive(Clone)]
pub(super) struct SettingsState {
    pub values: Preferences,
    pub error: Option<String>,
}
impl Global for SettingsState {}

fn path() -> std::path::PathBuf {
    voxelite_app::palettes::data_dir().join("settings.json")
}
pub(super) fn init(override_theme: Option<Theme>, cx: &mut App) {
    if cx.has_global::<SettingsState>() {
        return;
    }
    let (mut values, error) = match Preferences::load(&path()) {
        Ok(values) => (values, None),
        Err(error) => (
            Preferences::default(),
            Some(format!("Could not load settings: {error}")),
        ),
    };
    if let Some(theme) = override_theme {
        values.appearance = if theme == Theme::Light {
            Appearance::Light
        } else {
            Appearance::Dark
        };
    }
    apply_appearance(values.appearance, cx);
    cx.set_global(SettingsState { values, error });
}
fn apply_appearance(appearance: Appearance, cx: &App) {
    cx.set_window_appearance(match appearance {
        Appearance::System => None,
        Appearance::Light => Some(WindowAppearance::Light),
        Appearance::Dark => Some(WindowAppearance::Dark),
    });
}
pub(super) fn theme(appearance: Appearance, window: &Window) -> Theme {
    match appearance {
        Appearance::Light => Theme::Light,
        Appearance::Dark => Theme::Dark,
        Appearance::System => match window.appearance() {
            WindowAppearance::Light | WindowAppearance::VibrantLight => Theme::Light,
            _ => Theme::Dark,
        },
    }
}
pub(super) fn change(cx: &mut App, edit: impl FnOnce(&mut Preferences)) {
    let mut state = cx.global::<SettingsState>().clone();
    let mut next = state.values.clone();
    edit(&mut next);
    match next.save(&path()) {
        Ok(()) => {
            apply_appearance(next.appearance, cx);
            state.values = next;
            state.error = None;
        }
        Err(error) => state.error = Some(format!("Settings were not saved: {error}")),
    }
    cx.set_global(state);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Section {
    General,
    Appearance,
    Viewport,
    Shortcuts,
    Agent,
    Updates,
    Advanced,
}
impl Section {
    const ALL: [Self; 7] = [
        Self::General,
        Self::Appearance,
        Self::Viewport,
        Self::Shortcuts,
        Self::Agent,
        Self::Updates,
        Self::Advanced,
    ];
    fn title(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Appearance => "Appearance",
            Self::Viewport => "Viewport / Rendering",
            Self::Shortcuts => "Shortcuts",
            Self::Agent => "Agent",
            Self::Updates => "Updates",
            Self::Advanced => "Advanced",
        }
    }
    fn icon(self) -> &'static str {
        match self {
            Self::General => "settings",
            Self::Appearance => "sun",
            Self::Viewport => "cube",
            Self::Shortcuts => "keyboard",
            Self::Agent => "spark",
            Self::Updates => "download",
            Self::Advanced => "wrench",
        }
    }
    fn group(self) -> &'static str {
        match self {
            Self::General | Self::Appearance => "Essentials",
            Self::Viewport | Self::Shortcuts => "Editor",
            Self::Agent | Self::Updates => "Connect",
            Self::Advanced => "Other",
        }
    }
    fn settings(self) -> Vec<(&'static str, String)> {
        let entries: &[(&str, &str)] = match self {
            Self::General => &[
                (
                    "Navigation help",
                    "keyboard mouse guide viewport navigation hints",
                ),
                ("Voxelite", "about installed version workspace voxels"),
            ],
            Self::Appearance => &[("Appearance", "theme system device light dark color")],
            Self::Viewport => &[
                ("Ground grid", "spatial reference beneath model"),
                ("Antialiasing", "smooth edges camera quality rendering"),
                (
                    "Render performance",
                    "show viewport fps average frame time renderer statistics",
                ),
            ],
            Self::Shortcuts => {
                return actions::shortcut_sheet(ShortcutPlatform::current())
                    .map(|op| {
                        (
                            op.label(),
                            format!(
                                "{} {} {}",
                                op.menu(cfg!(target_os = "macos")),
                                op.shortcut(),
                                op.key().unwrap_or("")
                            ),
                        )
                    })
                    .collect();
            }
            Self::Agent => &[(
                "Project connection",
                "open connection manage assistant attached project mcp session",
            )],
            Self::Updates => &[
                ("Installed version", "Voxelite version release"),
                #[cfg(not(target_family = "wasm"))]
                (
                    "Software updates",
                    "check for updates latest version automatic updates",
                ),
                #[cfg(target_family = "wasm")]
                ("Browser edition", "reload page latest published version"),
            ],
            Self::Advanced => &[
                ("Preferences", "saved locally device storage"),
                (
                    "Restore defaults",
                    "reset settings appearance viewport preferences",
                ),
            ],
        };
        entries
            .iter()
            .map(|(title, terms)| (*title, (*terms).to_owned()))
            .collect()
    }
    fn keywords(self) -> &'static str {
        match self {
            Self::General => "navigation hints mouse startup version about",
            Self::Appearance => "theme light dark system color",
            Self::Viewport => "grid antialiasing quality performance fps frame",
            Self::Shortcuts => "keyboard commands bindings undo redo save",
            Self::Agent => "connection assistant mcp session",
            Self::Updates => "check version release",
            Self::Advanced => "reset defaults storage local preferences",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SearchResult {
    section: Section,
    settings: Vec<&'static str>,
}

fn matches_query(text: &str, query: &str) -> bool {
    let text = text.to_lowercase();
    query.split_whitespace().all(|word| text.contains(word))
}

fn search_sections(query: &str) -> Vec<SearchResult> {
    let query = query.trim().to_lowercase();
    Section::ALL
        .into_iter()
        .filter_map(|section| {
            let settings: Vec<_> = if query.is_empty() {
                Vec::new()
            } else {
                section
                    .settings()
                    .into_iter()
                    .filter(|(title, terms)| {
                        matches_query(&format!("{} {title} {terms}", section.title()), &query)
                    })
                    .map(|(title, _)| title)
                    .collect()
            };
            (matches_query(
                &format!("{} {}", section.title(), section.keywords()),
                &query,
            ) || !settings.is_empty())
            .then_some(SearchResult { section, settings })
        })
        .collect()
}

/// Browser-style history: selecting a new section after going back drops the forward branch.
struct SectionHistory {
    entries: Vec<Section>,
    cursor: usize,
}
impl SectionHistory {
    fn new(section: Section) -> Self {
        Self {
            entries: vec![section],
            cursor: 0,
        }
    }
    fn current(&self) -> Section {
        self.entries[self.cursor]
    }
    fn can_back(&self) -> bool {
        self.cursor > 0
    }
    fn can_forward(&self) -> bool {
        self.cursor + 1 < self.entries.len()
    }
    fn visit(&mut self, section: Section) {
        if self.current() != section {
            self.entries.truncate(self.cursor + 1);
            self.entries.push(section);
            self.cursor += 1;
        }
    }
    fn back(&mut self) {
        if self.can_back() {
            self.cursor -= 1;
        }
    }
    fn forward(&mut self) {
        if self.can_forward() {
            self.cursor += 1;
        }
    }
}

fn settings_color(t: Theme, dark: u32, light: u32) -> Hsla {
    rgb(if t == Theme::Dark { dark } else { light }).into()
}
fn blue() -> Hsla {
    rgb(0x007aff).into()
}
fn hairline(t: Theme) -> Hsla {
    settings_color(t, 0x393939, 0xd7d7db)
}
fn control_bg(t: Theme) -> Hsla {
    settings_color(t, 0x353535, 0xe6e6e9)
}

#[cfg(target_family = "wasm")]
pub(super) struct Dismiss;
pub(super) struct SettingsView {
    history: SectionHistory,
    sidebar_visible: bool,
    pane_scroll: ScrollHandle,
    search: Entity<TextField>,
    focus: FocusHandle,
    owner: WeakEntity<EditorView>,
    owner_window: AnyWindowHandle,
    message: Option<String>,
}
#[cfg(target_family = "wasm")]
impl EventEmitter<Dismiss> for SettingsView {}
impl SettingsView {
    fn new(
        section: Section,
        owner: WeakEntity<EditorView>,
        owner_window: AnyWindowHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = TextField::new("Search settings", cx);
        search.update(cx, |field, _| field.embed());
        cx.observe(&search, |_, _, cx| cx.notify()).detach();
        cx.observe_global::<SettingsState>(|_, cx| cx.notify())
            .detach();
        cx.observe_window_activation(window, |_, _, cx| cx.notify())
            .detach();
        cx.observe_window_appearance(window, |_, _, cx| cx.notify())
            .detach();
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self {
            history: SectionHistory::new(section),
            sidebar_visible: true,
            pane_scroll: ScrollHandle::new(),
            search,
            focus,
            owner,
            owner_window,
            message: None,
        }
    }
    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        #[cfg(not(target_family = "wasm"))]
        window.remove_window();
        #[cfg(target_family = "wasm")]
        {
            let _ = window;
            cx.emit(Dismiss);
        }
        cx.notify();
    }
    fn editor_action(&self, op: Op, cx: &mut Context<Self>) {
        let owner = self.owner.clone();
        let owner_window = self.owner_window;
        // On the web, Settings and the editor occupy the window currently
        // handling this click. Wait until it is available for update again.
        cx.defer(move |cx| {
            if owner_window
                .update(cx, move |_, window, cx| {
                    window.activate_window();
                    let _ = owner.update(cx, |this, cx| {
                        #[cfg(target_family = "wasm")]
                        {
                            this.settings_modal = None;
                        }
                        this.invoke(op, window, cx);
                    });
                })
                .is_err()
                && op == Op::Quit
            {
                cx.quit();
            }
        });
    }
    fn navigate(&mut self, section: Section, cx: &mut Context<Self>) {
        self.history.visit(section);
        self.message = None;
        self.pane_scroll.set_offset(point(px(0.0), px(0.0)));
        cx.notify();
    }
    fn history_step(&mut self, forward: bool, cx: &mut Context<Self>) {
        if forward {
            self.history.forward();
        } else {
            self.history.back();
        }
        self.message = None;
        self.pane_scroll.set_offset(point(px(0.0), px(0.0)));
        cx.notify();
    }
    fn chrome_button(
        &self,
        id: &'static str,
        glyph: &'static str,
        label: &'static str,
        enabled: bool,
        t: Theme,
    ) -> Stateful<Div> {
        row()
            .id(id)
            .role(Role::Button)
            .aria_label(label)
            .size(px(30.0))
            .justify_center()
            .rounded_full()
            .text_color(if enabled {
                t.text()
            } else {
                t.muted().opacity(0.35)
            })
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .when(enabled, |d| {
                d.tab_index(0)
                    .cursor_pointer()
                    .focus_visible(move |d| d.shadow(super::focus_ring(t)))
                    .hover(move |d| d.bg(t.text().opacity(0.08)))
                    .active(move |d| d.bg(t.text().opacity(0.14)))
            })
            .when(!enabled, |d| d.tab_stop(false))
            .tooltip(super::hint(label, t))
            .child(icon(glyph).size(px(18.0)).text_color(if enabled {
                t.text()
            } else {
                t.muted().opacity(0.35)
            }))
    }
    fn sidebar_row(
        &self,
        section: Section,
        setting: Option<&'static str>,
        index: usize,
        t: Theme,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let selected = setting.is_none() && section == self.history.current();
        let label = setting.unwrap_or(section.title());
        row()
            .id(("settings-result", index))
            .tab_index(0)
            .role(Role::Button)
            .aria_label(label)
            .aria_selected(selected)
            .min_h(px(if setting.is_some() { 30.0 } else { 36.0 }))
            .px(px(10.0))
            .gap(px(10.0))
            .mb(px(2.0))
            .rounded(px(8.0))
            .when(setting.is_some(), |d| d.ml(px(22.0)).text_size(px(11.0)))
            .when(selected, |d| d.bg(blue()).text_color(rgb(0xffffff)))
            .when(!selected, |d| {
                d.hover(move |d| d.bg(t.text().opacity(0.06)))
            })
            .cursor_pointer()
            .focus_visible(move |d| d.shadow(super::focus_ring(t)))
            .active(move |d| d.bg(blue().opacity(0.6)))
            .child(
                icon(if setting.is_some() {
                    "chevron-right"
                } else {
                    section.icon()
                })
                .size(px(if setting.is_some() { 12.0 } else { 17.0 }))
                .text_color(if selected {
                    rgb(0xffffff).into()
                } else {
                    t.muted()
                })
                .flex_shrink_0(),
            )
            .child(div().min_w(px(0.0)).child(label))
            .on_click(cx.listener(move |this, _, window, cx| {
                this.navigate(section, cx);
                window.focus(&this.focus, cx);
            }))
    }
    fn form_row(
        &self,
        label: impl Into<SharedString>,
        detail: &'static str,
        control: impl IntoElement,
        t: Theme,
    ) -> Div {
        row()
            .min_h(px(68.0))
            .px(px(16.0))
            .py(px(12.0))
            .gap(px(20.0))
            .child(div().flex_1().min_w(px(0.0)).child(label.into()).when(
                !detail.is_empty(),
                |d| {
                    d.child(
                        div()
                            .mt(px(3.0))
                            .text_size(px(12.0))
                            .text_color(t.muted())
                            .child(detail),
                    )
                },
            ))
            .child(control)
    }
    fn group(&self, rows: Vec<Div>, t: Theme) -> Div {
        let count = rows.len();
        div()
            .rounded(px(12.0))
            .border_1()
            .border_color(hairline(t).opacity(0.55))
            .overflow_hidden()
            .bg(settings_color(t, 0x262626, 0xffffff))
            .children(rows.into_iter().enumerate().map(|(i, row)| {
                div().child(row).when(i + 1 < count, |d| {
                    d.child(div().mx(px(16.0)).h(px(1.0)).bg(hairline(t).opacity(0.7)))
                })
            }))
    }
    fn toggle(
        &self,
        id: &'static str,
        value: bool,
        edit: fn(&mut Preferences),
        t: Theme,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        row()
            .id(id)
            .tab_index(0)
            .focus_visible(move |d| d.shadow(super::focus_ring(t)))
            .active(move |d| d.bg(t.selected()))
            .aria_label(id)
            .focus(move |d| d.border_2().border_color(t.accent()))
            .w(px(38.0))
            .h(px(23.0))
            .flex_shrink_0()
            .p(px(3.0))
            .rounded(px(20.0))
            .bg(if value {
                blue()
            } else {
                t.muted().opacity(0.28)
            })
            .when(value, |d| d.justify_end())
            .cursor_pointer()
            .tooltip(super::hint(if value { "On" } else { "Off" }, t))
            .child(
                div()
                    .size(px(17.0))
                    .rounded_full()
                    .bg(rgb(0xffffff))
                    .shadow_sm(),
            )
            .on_click(cx.listener(move |_, _, _, cx| change(cx, edit)))
    }
    fn button(&self, id: &'static str, label: &'static str, t: Theme) -> Stateful<Div> {
        row()
            .id(id)
            .tab_index(0)
            .focus_visible(move |d| d.shadow(super::focus_ring(t)))
            .active(move |d| d.bg(t.selected()))
            .aria_label(label)
            .focus(move |d| d.border_color(t.accent()))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .h(px(30.0))
            .px(px(12.0))
            .justify_center()
            .flex_shrink_0()
            .rounded(px(7.0))
            .border_1()
            .border_color(hairline(t))
            .bg(control_bg(t))
            .text_size(px(12.0))
            .cursor_pointer()
            .hover(move |d| d.bg(t.text().opacity(0.12)))
            .child(label)
    }
    fn appearance_preview(appearance: Appearance) -> Div {
        let fill: Background = match appearance {
            Appearance::System => linear_gradient(
                150.0,
                linear_color_stop(rgb(0xfafafa), 0.5),
                linear_color_stop(rgb(0x303030), 0.5),
            ),
            Appearance::Light => rgb(0xfafafa).into(),
            Appearance::Dark => rgb(0x303030).into(),
        };
        div()
            .h(px(90.0))
            .rounded(px(8.0))
            .overflow_hidden()
            .bg(fill)
            .child(
                row()
                    .h(px(20.0))
                    .px(px(8.0))
                    .gap(px(4.0))
                    .bg(rgba(0x88888822))
                    .children(
                        [0xff6058, 0xffbd2e, 0x28c840]
                            .map(|color| div().size(px(5.0)).rounded_full().bg(rgb(color))),
                    ),
            )
            .child(
                row()
                    .h(px(70.0))
                    .items_stretch()
                    .child(
                        div()
                            .w(relative(0.28))
                            .p(px(7.0))
                            .bg(rgba(0x88888822))
                            .children([1.0, 0.8, 0.6].map(|width| {
                                div()
                                    .mt(px(6.0))
                                    .h(px(4.0))
                                    .w(relative(width))
                                    .rounded_full()
                                    .bg(rgba(0x99999988))
                            })),
                    )
                    .child(
                        div()
                            .flex_1()
                            .p(px(10.0))
                            .children([0.9, 0.65, 0.75].map(|width| {
                                div()
                                    .mt(px(4.0))
                                    .mb(px(6.0))
                                    .h(px(4.0))
                                    .w(relative(width))
                                    .rounded_full()
                                    .bg(rgba(0x99999988))
                            })),
                    ),
            )
    }
    fn grid_setting(&self, cx: &Context<Self>) -> Option<voxelite_plugin_api::SettingContribution> {
        self.owner.upgrade().and_then(|owner| {
            owner
                .read(cx)
                .editor
                .plugin_host()
                .registry()
                .settings()
                .find(|(_, setting)| setting.key == voxelite_plugin_gizmo_grid::GRID_SETTING)
                .map(|(_, setting)| setting.clone())
        })
    }
    fn content(&self, section: Section, t: Theme, cx: &Context<Self>) -> Div {
        let p = &cx.global::<SettingsState>().values;
        match section {
            Section::General => self.group(
                vec![
                    self.form_row(
                        "Navigation help",
                        "Show the keyboard and mouse guide button in the viewport.",
                        self.toggle(
                            "navigation-hints",
                            p.navigation_hints,
                            |p| p.navigation_hints = !p.navigation_hints,
                            t,
                            cx,
                        ),
                        t,
                    ),
                    self.form_row(
                        "Voxelite",
                        "A small workspace for building in voxels.",
                        mono(format!("v{}", env!("CARGO_PKG_VERSION"))),
                        t,
                    ),
                ],
                t,
            ),
            Section::Appearance => self.group(
                vec![
                    div()
                        .p(px(18.0))
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .mb(px(16.0))
                                .child("Window appearance"),
                        )
                        .child(
                            row().gap(px(12.0)).children(
                                [
                                    (Appearance::System, "System"),
                                    (Appearance::Light, "Light"),
                                    (Appearance::Dark, "Dark"),
                                ]
                                .into_iter()
                                .enumerate()
                                .map(|(i, (appearance, label))| {
                                    let selected = p.appearance == appearance;
                                    div()
                                        .id(("appearance", i))
                                        .tab_index(0)
                                        .role(Role::Button)
                                        .aria_label(label)
                                        .aria_selected(selected)
                                        .flex_1()
                                        .min_w(px(0.0))
                                        .p(px(7.0))
                                        .rounded(px(10.0))
                                        .when(selected, |d| d.bg(blue().opacity(0.08)))
                                        .cursor_pointer()
                                        .focus_visible(move |d| d.shadow(super::focus_ring(t)))
                                        .hover(move |d| d.bg(blue().opacity(0.1)))
                                        .child(
                                            Self::appearance_preview(appearance)
                                                .border_2()
                                                .border_color(if selected {
                                                    blue()
                                                } else {
                                                    hairline(t)
                                                }),
                                        )
                                        .child(
                                            div()
                                                .mt(px(9.0))
                                                .text_center()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(if selected {
                                                    blue()
                                                } else {
                                                    t.text()
                                                })
                                                .child(label),
                                        )
                                        .on_click(cx.listener(move |_, _, _, cx| {
                                            change(cx, |p| p.appearance = appearance)
                                        }))
                                }),
                            ),
                        )
                        .child(
                            div()
                                .mt(px(18.0))
                                .text_size(px(12.0))
                                .text_color(t.muted())
                                .child("System follows your device’s light and dark appearance."),
                        ),
                ],
                t,
            ),
            Section::Viewport => self.group(
                self.grid_setting(cx)
                    .map(|setting| {
                        self.form_row(
                            setting.label_key,
                            "Keep a spatial reference beneath your model.",
                            self.toggle("ground-grid", p.grid, |p| p.grid = !p.grid, t, cx),
                            t,
                        )
                    })
                    .into_iter()
                    .chain([
                        self.form_row(
                            "Antialiasing",
                            "Smooth edges when the camera comes to rest.",
                            self.toggle(
                                "antialiasing",
                                p.antialiasing,
                                |p| p.antialiasing = !p.antialiasing,
                                t,
                                cx,
                            ),
                            t,
                        ),
                        self.form_row(
                            "Render performance",
                            "Show viewport FPS, average frame time and renderer.",
                            self.toggle(
                                "performance",
                                p.show_performance,
                                |p| p.show_performance = !p.show_performance,
                                t,
                                cx,
                            ),
                            t,
                        ),
                    ])
                    .collect(),
                t,
            ),
            Section::Shortcuts => self.group(
                actions::shortcut_sheet(ShortcutPlatform::current())
                    .enumerate()
                    .map(|(i, op)| {
                        self.form_row(
                            op.label(),
                            op.menu(cfg!(target_os = "macos")),
                            row()
                                .gap(px(8.0))
                                .child(
                                    row()
                                        .min_w(px(96.0))
                                        .h(px(30.0))
                                        .px(px(8.0))
                                        .justify_center()
                                        .rounded(px(6.0))
                                        .bg(control_bg(t))
                                        .child(super::shortcut_hint::sheet_shortcut(op)),
                                )
                                // The registry exposes fixed bindings, all already at their defaults.
                                .child(
                                    row()
                                        .id(("reset-shortcut", i))
                                        .tab_stop(false)
                                        .aria_label("Reset shortcut")
                                        .h(px(30.0))
                                        .px(px(10.0))
                                        .rounded(px(6.0))
                                        .bg(control_bg(t).opacity(0.5))
                                        .text_color(t.muted().opacity(0.5))
                                        .tooltip(super::hint(
                                            "This shortcut already uses its built-in default.",
                                            t,
                                        ))
                                        .child("Reset"),
                                ),
                            t,
                        )
                    })
                    .collect(),
                t,
            ),
            Section::Agent => self.group(
                vec![
                    self.form_row(
                        "Project connection",
                        "Manage the assistant attached to this project.",
                        self.button("agent-settings", "Open connection…", t)
                            .on_click(
                                cx.listener(|this, _, _, cx| this.editor_action(Op::Agent, cx)),
                            ),
                        t,
                    ),
                ],
                t,
            ),
            Section::Updates => {
                let mut rows = vec![self.form_row(
                    "Installed version",
                    "Voxelite",
                    mono(env!("CARGO_PKG_VERSION")),
                    t,
                )];
                #[cfg(not(target_family = "wasm"))]
                let updates_available = super::updates_available();
                #[cfg(not(target_family = "wasm"))]
                rows.push(self.form_row(
                    "Software updates",
                    if updates_available {
                        "Check for the latest version of Voxelite."
                    } else {
                        "Automatic updates are not available in this build."
                    },
                    div().when(updates_available, |d| {
                        d.child(
                            self.button("check-updates", "Check for Updates…", t)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.message = super::check_for_updates().map(str::to_owned);
                                    cx.notify();
                                })),
                        )
                    }),
                    t,
                ));
                #[cfg(target_family = "wasm")]
                rows.push(self.form_row(
                    "Browser edition",
                    "Reload the page to use the latest published version.",
                    div(),
                    t,
                ));
                self.group(rows, t)
            }
            Section::Advanced => self.group(
                vec![
                    self.form_row(
                        "Preferences",
                        "Saved locally on this device.",
                        icon("lock").text_color(t.muted()),
                        t,
                    ),
                    self.form_row(
                        "Restore defaults",
                        "Reset appearance and viewport preferences.",
                        self.button("reset-settings", "Reset settings", t).on_click(
                            cx.listener(|_, _, _, cx| change(cx, |p| *p = Preferences::default())),
                        ),
                        t,
                    ),
                ],
                t,
            ),
        }
    }
}
impl Render for SettingsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<SettingsState>().clone();
        let t = theme(state.values.appearance, window);
        self.search.update(cx, |field, cx| field.appearance(t, cx));
        let query = self.search.read(cx).text.clone();
        let mut results = search_sections(&query);
        if self.grid_setting(cx).is_none() {
            for result in &mut results {
                result.settings.retain(|label| *label != "Ground grid");
            }
            if !query.trim().is_empty() {
                results.retain(|result| {
                    !result.settings.is_empty()
                        || result.section != Section::Viewport
                        || matches_query(
                            &format!(
                                "{} {}",
                                result.section.title(),
                                result.section.keywords().replace("grid ", "")
                            ),
                            &query.trim().to_lowercase(),
                        )
                });
            }
        }
        let section = self.history.current();
        let mut sidebar_rows = div()
            .id("settings-sections")
            .flex_1()
            .min_h(px(0.0))
            .overflow_y_scroll();
        let mut previous_group = "";
        let mut index = 0;
        for result in &results {
            if query.trim().is_empty() && result.section.group() != previous_group {
                previous_group = result.section.group();
                sidebar_rows = sidebar_rows.child(
                    div()
                        .px(px(10.0))
                        .pt(px(17.0))
                        .pb(px(7.0))
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(t.muted())
                        .child(previous_group.to_uppercase()),
                );
            }
            sidebar_rows = sidebar_rows.child(self.sidebar_row(result.section, None, index, t, cx));
            index += 1;
            for setting in &result.settings {
                sidebar_rows = sidebar_rows.child(self.sidebar_row(
                    result.section,
                    Some(setting),
                    index,
                    t,
                    cx,
                ));
                index += 1;
            }
        }
        if results.is_empty() {
            sidebar_rows = sidebar_rows.child(
                div()
                    .p(px(12.0))
                    .text_size(px(12.0))
                    .text_color(t.muted())
                    .child("No matching settings"),
            );
        }
        if window.is_window_active() && cfg!(target_os = "macos") {
            cx.set_menus(actions::menus(
                |_| false,
                |op| {
                    op.availability(actions::CommandContext::Settings {
                        updates_available: super::updates_available(),
                    })
                    .is_ok()
                },
            ));
        }
        div()
            .flex()
            .flex_col()
            .id("settings-root")
            .tab_group()
            .tab_stop(false)
            .key_context("Settings")
            .track_focus(&self.focus)
            .size_full()
            .bg(settings_color(t, 0x1e1e1e, 0xf5f5f7))
            .font(gpui::font(if cfg!(target_os = "macos") {
                ".AppleSystemUIFont"
            } else {
                "Geist"
            }))
            .text_size(px(13.0))
            .text_color(t.text())
            .rounded(px(14.0))
            .overflow_hidden()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_action(cx.listener(|this, op: &Op, window, cx| {
                if op
                    .availability(actions::CommandContext::Settings {
                        updates_available: super::updates_available(),
                    })
                    .is_err()
                {
                    return;
                }
                match op {
                    Op::Settings => window.activate_window(),
                    Op::About => this.navigate(Section::General, cx),
                    Op::Help => this.navigate(Section::Shortcuts, cx),
                    Op::CheckUpdates => {
                        this.navigate(Section::Updates, cx);
                        this.message = super::check_for_updates().map(str::to_owned);
                    }
                    Op::Hide => cx.hide(),
                    Op::HideOthers => cx.hide_other_apps(),
                    Op::ShowAll => cx.unhide_other_apps(),
                    Op::Minimize => window.minimize_window(),
                    Op::ZoomWindow => window.zoom_window(),
                    Op::Quit => this.editor_action(Op::Quit, cx),
                    _ => {}
                }
                cx.notify();
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "escape"
                    || (key == "w"
                        && ShortcutPlatform::current().is_primary(event.keystroke.modifiers))
                {
                    this.close(window, cx);
                    cx.stop_propagation();
                } else if key == "tab" {
                    if event.keystroke.modifiers.shift {
                        window.focus_prev(cx);
                    } else {
                        window.focus_next(cx);
                    }
                    cx.stop_propagation();
                }
            }))
            .child(
                row()
                    .id("settings-titlebar")
                    .h(px(62.0))
                    .flex_shrink_0()
                    .px(px(16.0))
                    .gap(px(16.0))
                    .when(cfg!(target_os = "macos"), |d| d.pl(px(92.0)))
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move())
                    .child(
                        row()
                            .p(px(3.0))
                            .gap(px(2.0))
                            .rounded_full()
                            .border_1()
                            .border_color(hairline(t))
                            .bg(settings_color(t, 0x282828, 0xebebed))
                            .child(
                                self.chrome_button(
                                    "settings-sidebar-toggle",
                                    "panel-left",
                                    "Toggle sidebar",
                                    true,
                                    t,
                                )
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.sidebar_visible = !this.sidebar_visible;
                                        cx.notify();
                                    },
                                )),
                            )
                            .child(
                                self.chrome_button(
                                    "settings-back",
                                    "chevron-left",
                                    "Back",
                                    self.history.can_back(),
                                    t,
                                )
                                .when(
                                    self.history.can_back(),
                                    |d| {
                                        d.on_click(cx.listener(|this, _, _, cx| {
                                            this.history_step(false, cx)
                                        }))
                                    },
                                ),
                            )
                            .child(
                                self.chrome_button(
                                    "settings-forward",
                                    "chevron-right",
                                    "Forward",
                                    self.history.can_forward(),
                                    t,
                                )
                                .when(
                                    self.history.can_forward(),
                                    |d| {
                                        d.on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.history_step(true, cx)
                                            }),
                                        )
                                    },
                                ),
                            ),
                    )
                    .child(
                        div()
                            .min_w(px(0.0))
                            .text_size(px(16.0))
                            .font_weight(FontWeight::BOLD)
                            .child("Voxelite Settings"),
                    )
                    .child(div().flex_1())
                    .child(
                        self.button("close-settings", "Done", t)
                            .on_click(cx.listener(|this, _, window, cx| this.close(window, cx))),
                    ),
            )
            .child(
                row()
                    .flex_1()
                    .min_h(px(0.0))
                    .items_stretch()
                    .when(self.sidebar_visible, |d| {
                        d.child(
                            div()
                                .w(px(224.0))
                                .flex_shrink_0()
                                .m(px(10.0))
                                .mt(px(0.0))
                                .mr(px(0.0))
                                .p(px(10.0))
                                .flex()
                                .flex_col()
                                .rounded(px(12.0))
                                .border_1()
                                .border_color(hairline(t))
                                .bg(settings_color(t, 0x191919, 0xebebed))
                                .child(
                                    row()
                                        .h(px(34.0))
                                        .flex_shrink_0()
                                        .px(px(9.0))
                                        .gap(px(6.0))
                                        .mb(px(4.0))
                                        .rounded_full()
                                        .bg(settings_color(t, 0x2b2b2b, 0xe0e0e3))
                                        .border_1()
                                        .border_color(if self.search.read(cx).focused(window) {
                                            blue()
                                        } else {
                                            transparent_black()
                                        })
                                        .child(
                                            icon("search")
                                                .size(px(15.0))
                                                .flex_shrink_0()
                                                .text_color(t.muted()),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_w(px(0.0))
                                                .child(self.search.clone()),
                                        )
                                        .when(!query.is_empty(), |d| {
                                            d.child(
                                                self.chrome_button(
                                                    "clear-settings-search",
                                                    "close",
                                                    "Clear search",
                                                    true,
                                                    t,
                                                )
                                                .size(px(20.0))
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.search.update(cx, |field, cx| {
                                                        field.set(String::new(), cx)
                                                    });
                                                    window.focus(&this.search.focus_handle(cx), cx);
                                                })),
                                            )
                                        }),
                                )
                                .child(sidebar_rows),
                        )
                    })
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .id("settings-pane")
                                    .flex_1()
                                    .min_h(px(0.0))
                                    .overflow_y_scroll()
                                    .track_scroll(&self.pane_scroll)
                                    .px(px(28.0))
                                    .pt(px(18.0))
                                    .pb(px(28.0))
                                    .child(
                                        div()
                                            .mb(px(24.0))
                                            .text_size(px(24.0))
                                            .font_weight(FontWeight::BOLD)
                                            .child(section.title()),
                                    )
                                    .child(self.content(section, t, cx)),
                            )
                            .when_some(state.error.or(self.message.clone()), |d, message| {
                                d.child(
                                    div()
                                        .px(px(28.0))
                                        .pb(px(18.0))
                                        .text_size(px(12.0))
                                        .text_color(t.danger())
                                        .child(message),
                                )
                            }),
                    ),
            )
    }
}

impl EditorView {
    pub(super) fn open_settings(
        &mut self,
        section: Section,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let owner = cx.weak_entity();
        let owner_window = window.window_handle();
        #[cfg(not(target_family = "wasm"))]
        {
            if let Some(handle) = self.settings_window
                && handle
                    .update(cx, |view, window, cx| {
                        view.navigate(section, cx);
                        cx.notify();
                        window.activate_window();
                    })
                    .is_ok()
            {
                return;
            }
            let bounds = Bounds::centered(None, size(px(880.0), px(640.0)), cx);
            match cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    app_owns_titlebar_drag: cfg!(target_os = "macos"),
                    window_min_size: Some(size(px(760.0), px(520.0))),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Voxelite Settings".into()),
                        appears_transparent: true,
                        traffic_light_position: Some(point(px(18.0), px(23.0))),
                    }),
                    window_background: if cfg!(target_os = "macos") {
                        WindowBackgroundAppearance::Blurred
                    } else {
                        WindowBackgroundAppearance::Opaque
                    },
                    app_id: Some("dev.voxelite.editor".into()),
                    ..Default::default()
                },
                |window, cx| {
                    cx.new(|cx| SettingsView::new(section, owner, owner_window, window, cx))
                },
            ) {
                Ok(handle) => self.settings_window = Some(handle),
                Err(error) => {
                    self.status = format!("Could not open settings: {error}").into();
                    self.error = true;
                }
            }
        }
        #[cfg(target_family = "wasm")]
        {
            let view = cx.new(|cx| SettingsView::new(section, owner, owner_window, window, cx));
            cx.subscribe_in(&view, window, |this, _, _: &Dismiss, window, cx| {
                this.settings_modal = None;
                window.focus(&this.focus, cx);
                cx.notify();
            })
            .detach();
            self.settings_modal = Some(view);
        }
        cx.notify();
    }
    pub(super) fn sync_preferences(&mut self, window: &Window, cx: &mut Context<Self>) {
        let next = cx.global::<SettingsState>().values.clone();
        let next_theme = theme(next.appearance, window);
        let rerender = self.preferences.antialiasing != next.antialiasing
            || (self.viewport.gpu_enabled()
                && (self.theme != next_theme || self.preferences.grid != next.grid));
        self.theme = next_theme;
        self.editor
            .set_grid(next.grid)
            .expect("declared boolean grid setting");
        self.preferences = next;
        if rerender {
            self.request_render(cx);
        }
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::{SearchResult, Section, SectionHistory, search_sections};
    use crate::actions::Op;

    #[test]
    fn empty_search_keeps_all_sections_without_setting_rows() {
        let results = search_sections("  ");
        assert_eq!(
            results.iter().map(|r| r.section).collect::<Vec<_>>(),
            Section::ALL
        );
        assert!(results.iter().all(|r| r.settings.is_empty()));
    }

    #[test]
    fn search_finds_individual_settings_by_label_and_description() {
        for (query, section, title) in [
            (" ground GRID ", Section::Viewport, "Ground grid"),
            (
                "average frame time",
                Section::Viewport,
                "Render performance",
            ),
            ("smooth edges", Section::Viewport, "Antialiasing"),
            ("navigation hints", Section::General, "Navigation help"),
            ("assistant", Section::Agent, "Project connection"),
            ("reset settings", Section::Advanced, "Restore defaults"),
            ("system dark", Section::Appearance, "Appearance"),
        ] {
            assert_eq!(
                search_sections(query),
                vec![SearchResult {
                    section,
                    settings: vec![title]
                }],
                "{query}"
            );
        }
    }

    #[test]
    fn search_includes_parent_sections_and_individual_shortcuts() {
        let results = search_sections("save");
        let shortcuts = results
            .iter()
            .find(|r| r.section == Section::Shortcuts)
            .unwrap();
        assert!(shortcuts.settings.contains(&Op::Save.label()));
        assert!(shortcuts.settings.contains(&Op::SaveAs.label()));
        assert!(!shortcuts.settings.contains(&Op::Undo.label()));
        assert!(search_sections("unmatched-xyz").is_empty());
        assert_eq!(
            search_sections("viewport ground")[0].settings,
            vec!["Ground grid"]
        );
    }

    #[test]
    fn history_moves_back_and_forward_and_stops_at_boundaries() {
        let mut history = SectionHistory::new(Section::General);
        assert!(!history.can_back());
        assert!(!history.can_forward());
        history.back();
        history.forward();
        assert_eq!(history.current(), Section::General);
        history.visit(Section::Appearance);
        history.visit(Section::Viewport);
        history.back();
        assert_eq!(history.current(), Section::Appearance);
        assert!(history.can_back());
        assert!(history.can_forward());
        history.back();
        history.back();
        assert_eq!(history.current(), Section::General);
        history.forward();
        history.forward();
        history.forward();
        assert_eq!(history.current(), Section::Viewport);
        assert!(!history.can_forward());
    }

    #[test]
    fn history_preserves_forward_on_reselection_and_truncates_on_new_navigation() {
        let mut history = SectionHistory::new(Section::General);
        history.visit(Section::Appearance);
        history.visit(Section::Viewport);
        history.back();
        history.visit(Section::Appearance);
        assert!(history.can_forward());
        assert_eq!(history.entries.len(), 3);
        history.visit(Section::Agent);
        assert!(!history.can_forward());
        assert_eq!(
            history.entries,
            vec![Section::General, Section::Appearance, Section::Agent]
        );
        history.visit(Section::Agent);
        history.back();
        assert_eq!(history.current(), Section::Appearance);
    }
}
