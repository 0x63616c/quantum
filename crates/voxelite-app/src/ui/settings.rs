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

#[derive(Clone, Copy, PartialEq, Eq)]
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
            Self::General => "sliders-horizontal",
            Self::Appearance => "sun",
            Self::Viewport => "cube",
            Self::Shortcuts => "keyboard",
            Self::Agent => "spark",
            Self::Updates => "download",
            Self::Advanced => "wrench",
        }
    }
    fn color(self) -> u32 {
        match self {
            Self::General => 0x858b96,
            Self::Appearance => 0xa56af6,
            Self::Viewport => 0x3894f5,
            Self::Shortcuts => 0xf28b39,
            Self::Agent => 0xd566a6,
            Self::Updates => 0x48aa78,
            Self::Advanced => 0x657a9b,
        }
    }
    fn description(self) -> &'static str {
        match self {
            Self::General => "Make room for your next idea.",
            Self::Appearance => "A familiar workspace, in any light.",
            Self::Viewport => "Find the balance between detail and responsiveness.",
            Self::Shortcuts => "Keep your hands on the keyboard and your ideas moving.",
            Self::Agent => "Connect an assistant to your current project.",
            Self::Updates => "Keep Voxelite up to date.",
            Self::Advanced => "Local preferences and workspace defaults.",
        }
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
    fn matches(self, query: &str) -> bool {
        format!("{} {}", self.title(), self.keywords())
            .to_lowercase()
            .contains(query)
    }
}

#[cfg(target_family = "wasm")]
pub(super) struct Dismiss;
pub(super) struct SettingsView {
    section: Section,
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
            section,
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
        if self
            .owner_window
            .update(cx, move |_, window, cx| {
                window.activate_window();
                let _ = owner.update(cx, |this, cx| this.invoke(op, window, cx));
            })
            .is_err()
            && op == Op::Quit
        {
            cx.quit();
        }
    }
    fn tile(section: Section, size: f32) -> Div {
        row()
            .size(px(size))
            .flex_shrink_0()
            .justify_center()
            .rounded(px(size * 0.25))
            .bg(rgb(section.color()))
            .border_1()
            .border_color(rgba(0xffffff30))
            .child(
                icon(section.icon())
                    .size(px(size * 0.57))
                    .text_color(rgb(0xffffff)),
            )
    }
    fn form_row(
        &self,
        label: &'static str,
        detail: &'static str,
        control: impl IntoElement,
        t: Theme,
    ) -> Div {
        row()
            .min_h(px(62.0))
            .px(px(16.0))
            .py(px(12.0))
            .gap(px(20.0))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .child(label)
                    .when(!detail.is_empty(), |d| {
                        d.child(
                            div()
                                .mt(px(3.0))
                                .text_size(px(11.0))
                                .text_color(t.muted())
                                .child(detail),
                        )
                    }),
            )
            .child(control)
    }
    fn group(&self, rows: Vec<Div>, t: Theme) -> Div {
        let count = rows.len();
        div()
            .rounded(px(12.0))
            .border_1()
            .border_color(t.line())
            .overflow_hidden()
            .bg(rgb(if t == Theme::Dark { 0x2b2c32 } else { 0xffffff }).opacity(0.8))
            .children(rows.into_iter().enumerate().map(|(i, row)| {
                div().child(row).when(i + 1 < count, |d| {
                    d.child(div().mx(px(16.0)).h(px(1.0)).bg(t.line().opacity(0.65)))
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
                rgb(0x3485ef).into()
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
            .border_color(t.line())
            .bg(t.background())
            .text_size(px(12.0))
            .cursor_pointer()
            .hover(move |d| d.bg(t.hover()))
            .child(label)
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
            Section::Appearance => div()
                .child(self.group(
                    vec![self.form_row(
                        "Appearance",
                        "System follows your device’s light and dark appearance.",
                        div(),
                        t,
                    )],
                    t,
                ))
                .child(
                    row().mt(px(18.0)).gap(px(12.0)).children(
                        [
                            (Appearance::Light, "Light"),
                            (Appearance::Dark, "Dark"),
                            (Appearance::System, "System"),
                        ]
                        .into_iter()
                        .enumerate()
                        .map(|(i, (appearance, label))| {
                            let active = p.appearance == appearance;
                            div()
                                .id(("appearance", i))
                                .tab_index(0)
                                .focus_visible(move |d| d.shadow(super::focus_ring(t)))
                                .active(move |d| d.bg(t.selected()))
                                .aria_label(label)
                                .focus(move |d| d.border_color(t.accent()))
                                .flex_1()
                                .p(px(8.0))
                                .rounded(px(12.0))
                                .border_2()
                                .border_color(if active { t.accent() } else { t.line() })
                                .cursor_pointer()
                                .bg(if active {
                                    t.selected()
                                } else {
                                    t.background().opacity(0.65)
                                })
                                .child(
                                    row()
                                        .h(px(78.0))
                                        .rounded(px(6.0))
                                        .overflow_hidden()
                                        .bg(rgb(if appearance == Appearance::Dark {
                                            0x27292f
                                        } else {
                                            0xebeef4
                                        }))
                                        .child(
                                            div()
                                                .w(px(25.0))
                                                .h_full()
                                                .p(px(5.0))
                                                .bg(rgb(if appearance == Appearance::Light {
                                                    0xd9dfea
                                                } else {
                                                    0x414653
                                                }))
                                                .child(
                                                    div()
                                                        .h(px(5.0))
                                                        .rounded(px(2.0))
                                                        .bg(rgb(0x73a9ef)),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .p(px(8.0))
                                                .child(
                                                    div()
                                                        .h(px(8.0))
                                                        .w(px(34.0))
                                                        .rounded(px(2.0))
                                                        .bg(rgb(0x8a98b0)),
                                                )
                                                .child(
                                                    div()
                                                        .mt(px(8.0))
                                                        .h(px(24.0))
                                                        .rounded(px(3.0))
                                                        .bg(rgb(
                                                            if appearance == Appearance::Light {
                                                                0xffffff
                                                            } else {
                                                                0x575f70
                                                            },
                                                        )),
                                                ),
                                        ),
                                )
                                .child(
                                    row()
                                        .mt(px(10.0))
                                        .justify_center()
                                        .gap(px(6.0))
                                        .child(label)
                                        .when(active, |d| {
                                            d.child(
                                                icon("check").size(px(13.0)).text_color(t.accent()),
                                            )
                                        }),
                                )
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    change(cx, |p| p.appearance = appearance)
                                }))
                        }),
                    ),
                ),
            Section::Viewport => self.group(
                vec![
                    self.form_row(
                        "Ground grid",
                        "Keep a spatial reference beneath your model.",
                        self.toggle("ground-grid", p.grid, |p| p.grid = !p.grid, t, cx),
                        t,
                    ),
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
                        "Show CPU render FPS and milliseconds per frame.",
                        self.toggle(
                            "performance",
                            p.show_performance,
                            |p| p.show_performance = !p.show_performance,
                            t,
                            cx,
                        ),
                        t,
                    ),
                ],
                t,
            ),
            Section::Shortcuts => self.group(
                [
                    Op::New,
                    Op::Open,
                    Op::Save,
                    Op::SaveAs,
                    Op::Undo,
                    Op::Redo,
                    Op::Commands,
                    Op::Settings,
                    Op::Frame,
                    Op::Place,
                    Op::Erase,
                    Op::Paint,
                    Op::Box,
                ]
                .into_iter()
                .map(|op| {
                    self.form_row(op.label(), "", super::shortcut_hint::action_shortcut(op), t)
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
                let group = self.group(
                    vec![self.form_row(
                        "Installed version",
                        "Voxelite",
                        mono(env!("CARGO_PKG_VERSION")),
                        t,
                    )],
                    t,
                );
                #[cfg(not(target_family = "wasm"))]
                let updates_available = super::updates_available();
                #[cfg(not(target_family = "wasm"))]
                let group = group.child(self.form_row(
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
                let group = group.child(self.form_row(
                    "Browser edition",
                    "Reload the page to use the latest published version.",
                    div(),
                    t,
                ));
                group
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
        let query = self.search.read(cx).text.trim().to_lowercase();
        let sections: Vec<_> = Section::ALL
            .into_iter()
            .filter(|section| section.matches(&query))
            .collect();
        let section = if sections.contains(&self.section) {
            Some(self.section)
        } else {
            sections.first().copied()
        };
        if window.is_window_active() && cfg!(target_os = "macos") {
            cx.set_menus(actions::menus(
                |_| false,
                |op| {
                    matches!(
                        op,
                        Op::About
                            | Op::Settings
                            | Op::CheckUpdates
                            | Op::Quit
                            | Op::Hide
                            | Op::HideOthers
                            | Op::ShowAll
                            | Op::Minimize
                            | Op::ZoomWindow
                            | Op::Help
                    ) && (op != Op::CheckUpdates || super::updates_available())
                },
            ));
        }
        let sidebar_bg = rgb(if t == Theme::Dark { 0x24252b } else { 0xe6e8ef });
        let pane_bg = rgb(if t == Theme::Dark { 0x1b1c21 } else { 0xf4f5f8 });
        row()
            .id("settings-root")
            .tab_group()
            .tab_stop(false)
            .key_context("Settings")
            .track_focus(&self.focus)
            .size_full()
            .items_stretch()
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
                match op {
                    Op::Settings => window.activate_window(),
                    Op::About => this.section = Section::General,
                    Op::Help => this.section = Section::Shortcuts,
                    Op::CheckUpdates => {
                        this.section = Section::Updates;
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
                div()
                    .w(px(224.0))
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .bg(sidebar_bg.opacity(0.85))
                    .border_r_1()
                    .border_color(t.line())
                    .child(
                        row()
                            .id("settings-titlebar")
                            .h(px(62.0))
                            .flex_shrink_0()
                            .px(px(18.0))
                            .when(cfg!(target_os = "macos"), |d| d.pl(px(92.0)))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Settings")
                            .on_mouse_down(MouseButton::Left, |_, window, _| {
                                window.start_window_move()
                            }),
                    )
                    .child(div().px(px(12.0)).mb(px(18.0)).child(self.search.clone()))
                    .child(
                        div()
                            .id("settings-sections")
                            .flex_1()
                            .min_h(px(0.0))
                            .overflow_y_scroll()
                            .px(px(10.0))
                            .children(sections.into_iter().enumerate().map(|(i, entry)| {
                                row()
                                    .id(("settings-section", i))
                                    .tab_index(0)
                                    .focus_visible(move |d| d.shadow(super::focus_ring(t)))
                                    .active(move |d| d.bg(t.selected()))
                                    .aria_label(entry.title())
                                    .focus(move |d| d.bg(t.accent().opacity(0.22)))
                                    .h(px(42.0))
                                    .px(px(9.0))
                                    .gap(px(10.0))
                                    .mb(px(3.0))
                                    .rounded(px(9.0))
                                    .cursor_pointer()
                                    .when(section == Some(entry), |d| {
                                        d.bg(t.accent().opacity(0.16))
                                    })
                                    .hover(move |d| d.bg(t.accent().opacity(0.10)))
                                    .child(Self::tile(entry, 27.0))
                                    .child(div().text_size(px(12.0)).child(entry.title()))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.section = entry;
                                        this.message = None;
                                        window.focus(&this.focus, cx);
                                        cx.notify();
                                    }))
                            })),
                    )
                    .child(
                        div()
                            .p(px(20.0))
                            .text_size(px(11.0))
                            .text_color(t.muted())
                            .child("Voxelite")
                            .child(div().mt(px(3.0)).child("Your space to create.")),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .flex()
                    .flex_col()
                    .bg(pane_bg.opacity(0.96))
                    .child(
                        row()
                            .id("settings-pane-titlebar")
                            .on_mouse_down(MouseButton::Left, |_, window, _| {
                                window.start_window_move()
                            })
                            .h(px(62.0))
                            .flex_shrink_0()
                            .px(px(24.0))
                            .justify_end()
                            .child(self.button("close-settings", "Done", t).on_click(
                                cx.listener(|this, _, window, cx| this.close(window, cx)),
                            )),
                    )
                    .child(
                        div()
                            .id("settings-pane")
                            .flex_1()
                            .min_h(px(0.0))
                            .overflow_y_scroll()
                            .px(px(28.0))
                            .pb(px(28.0))
                            .when_some(section, |d, section| {
                                d.child(
                                    row()
                                        .gap(px(14.0))
                                        .mb(px(24.0))
                                        .child(Self::tile(section, 44.0))
                                        .child(
                                            div()
                                                .flex_1()
                                                .child(
                                                    div()
                                                        .text_size(px(23.0))
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .child(section.title()),
                                                )
                                                .child(
                                                    div()
                                                        .mt(px(5.0))
                                                        .text_size(px(12.0))
                                                        .text_color(t.muted())
                                                        .child(section.description()),
                                                ),
                                        ),
                                )
                                .child(self.content(section, t, cx))
                            })
                            .when(section.is_none(), |d| {
                                d.child(
                                    div()
                                        .mt(px(80.0))
                                        .text_center()
                                        .text_color(t.muted())
                                        .child("No matching settings"),
                                )
                            }),
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
                        view.section = section;
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
        let rerender = self.preferences.antialiasing != next.antialiasing;
        self.theme = theme(next.appearance, window);
        self.grid = next.grid;
        self.preferences = next;
        if rerender {
            self.request_render(cx);
        }
        cx.notify();
    }
}
