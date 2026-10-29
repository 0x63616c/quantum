//! Settings presented in a native utility window.
use super::{AnchoredTooltip, EditorView, TooltipSide, icon, mono, row};
use crate::{
    actions::{self, Op},
    shortcuts::ShortcutPlatform,
    text_field::TextField,
    theme::Theme,
};
use gpui::{prelude::*, *};
use quantum_app::preferences::{Appearance, Preferences};
use quantum_render::AmbientOcclusionStrength;
use quantum_ui::ColorToken;
use quantum_ui::Radius;
use quantum_ui::Space;
use quantum_ui::TextSize;
use quantum_ui::TextStyle;
use quantum_ui::tokens::geometry;
use quantum_ui::tokens::metrics;
use quantum_ui::tokens::state;
use quantum_ui::tokens::type_size;
use quantum_ui::{
    Button, Card, FormRow, IconButton, Panel, SidebarItem, SidebarSection, Slider, SliderRange,
    Toggle,
};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(super) struct SettingsState {
    pub values: Preferences,
    pub error: Option<String>,
}
impl Global for SettingsState {}

mod history;
use history::SectionHistory;

mod controls;
mod menu_actions;
mod persistence;
mod window;
#[cfg(all(test, feature = "test-support"))]
pub(super) use persistence::init_at;
#[cfg(test)]
use persistence::{WriteQueue, drain_writes};
pub(super) use persistence::{change, init, init_loaded};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Section {
    General,
    Appearance,
    Viewport,
    Shortcuts,
    Agent,
    Updates,
    Advanced,
}
impl Section {
    pub(super) const ALL: [Self; 7] = [
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
                (
                    "Animate camera",
                    "preset programmatic view transitions reduce motion",
                ),
                ("Quantum", "about installed version workspace voxels"),
            ],
            Self::Appearance => &[("Appearance", "theme system device light dark color")],
            Self::Viewport => &[
                ("Ground grid", "spatial reference beneath model"),
                ("Antialiasing", "smooth edges camera quality rendering"),
                (
                    "Ambient occlusion",
                    "ao voxel contact shading shadows strength",
                ),
                (
                    "Render performance",
                    "show viewport fps average frame time renderer statistics",
                ),
                ("Debug overlay", "cpu gpu timings chunks memory diagnostics"),
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
                ("Installed version", "Quantum version release"),
                (
                    "Software updates",
                    "check for updates latest version automatic updates",
                ),
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
            Self::Viewport => {
                "grid antialiasing ambient occlusion ao strength quality performance fps frame"
            }
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

pub(super) struct SettingsView {
    overlays: Entity<quantum_ui::OverlayState>,
    history: SectionHistory,
    sidebar_visible: bool,
    pane_scroll: ScrollHandle,
    search: Entity<TextField>,
    focus: FocusHandle,
    owner: WeakEntity<EditorView>,
    owner_window: AnyWindowHandle,
}
impl SettingsView {
    fn new(
        section: Section,
        owner: WeakEntity<EditorView>,
        owner_window: AnyWindowHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = TextField::design("Search settings", cx);
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
            overlays: quantum_ui::OverlayState::new(cx),
            sidebar_visible: true,
            pane_scroll: ScrollHandle::new(),
            search,
            focus,
            owner,
            owner_window,
        }
    }
    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.remove_window();
        cx.notify();
    }
    fn editor_action(&self, op: Op, cx: &mut Context<Self>) {
        let owner = self.owner.clone();
        let owner_window = self.owner_window;
        // Dispatch after the current window update finishes.
        cx.defer(move |cx| {
            if owner_window
                .update(cx, move |_, window, cx| {
                    window.activate_window();
                    let _ = owner.update(cx, |this, cx| {
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
        self.pane_scroll
            .set_offset(point(px(Space::S0.px()), px(Space::S0.px())));
        cx.notify();
    }
    fn history_step(&mut self, forward: bool, cx: &mut Context<Self>) {
        if forward {
            self.history.forward();
        } else {
            self.history.back();
        }
        self.pane_scroll
            .set_offset(point(px(Space::S0.px()), px(Space::S0.px())));
        cx.notify();
    }
    fn appearance_preview(appearance: Appearance) -> Div {
        let fill: Background = match appearance {
            Appearance::System => linear_gradient(
                150.0,
                linear_color_stop(Theme::Light.preview_background(), 0.5),
                linear_color_stop(Theme::Dark.preview_background(), 0.5),
            ),
            Appearance::Light => Theme::Light.preview_background().into(),
            Appearance::Dark => Theme::Dark.preview_background().into(),
        };
        div()
            .h(px(metrics::APPEARANCE_PREVIEW_HEIGHT))
            .rounded(px(Radius::Md.px()))
            .overflow_hidden()
            .bg(fill)
            .child(
                row()
                    .h(px(Space::S5.px()))
                    .px(px(Space::S3.px()))
                    .gap(px(Space::S2.px()))
                    .bg(Theme::Dark.preview_chrome())
                    .children(
                        Theme::preview_lights()
                            .map(|color| div().size(px(Space::S2.px())).rounded_full().bg(color)),
                    ),
            )
            .child(
                row()
                    .h(px(metrics::APPEARANCE_BODY_HEIGHT))
                    .items_stretch()
                    .child(
                        div()
                            .w(relative(geometry::PREVIEW_SIDEBAR_FRACTION))
                            .p(px(Space::S3.px()))
                            .bg(Theme::Dark.preview_chrome())
                            .children([1.0, 0.8, 0.6].map(|width| {
                                div()
                                    .mt(px(Space::S3.px()))
                                    .h(px(Space::S2.px()))
                                    .w(relative(width))
                                    .rounded_full()
                                    .bg(Theme::Dark.preview_content())
                            })),
                    )
                    .child(
                        div()
                            .flex_1()
                            .p(px(Space::S4.px()))
                            .children([0.9, 0.65, 0.75].map(|width| {
                                div()
                                    .mt(px(Space::S2.px()))
                                    .mb(px(Space::S3.px()))
                                    .h(px(Space::S2.px()))
                                    .w(relative(width))
                                    .rounded_full()
                                    .bg(Theme::Dark.preview_content())
                            })),
                    ),
            )
    }
    fn grid_setting(&self, cx: &Context<Self>) -> Option<quantum_plugin_api::SettingContribution> {
        self.owner.upgrade().and_then(|owner| {
            owner
                .read(cx)
                .editor
                .plugin_host()
                .registry()
                .settings()
                .find(|(_, setting)| setting.key == quantum_plugin_gizmo_grid::GRID_SETTING)
                .map(|(_, setting)| setting.clone())
        })
    }
    #[expect(clippy::expect_used, reason = "internal invariant: bounded AO slider")]
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
                        "Animate camera",
                        "Ease between preset and programmatic view changes. Always off when macOS Reduce Motion is on.",
                        self.toggle(
                            "animate-camera",
                            p.animate_camera,
                            |p| p.animate_camera = !p.animate_camera,
                            t,
                            cx,
                        ),
                        t,
                    ),
                    self.form_row(
                        "Quantum",
                        "A voxel workspace by Calum",
                        mono(format!("v{}", crate::ui::APP_VERSION)),
                        t,
                    ),
                ],
                t,
            ),
            Section::Appearance => self.group(
                vec![
                    div()
                        .p(px(Space::S5.px()))
                        .child(
                            div()
                                .font_weight(type_size::SEMIBOLD)
                                .mb(px(Space::S5.px()))
                                .child("Window appearance"),
                        )
                        .child(
                            row().gap(px(Space::S4.px())).children(
                                [
                                    (Appearance::System, "System"),
                                    (Appearance::Light, "Light"),
                                    (Appearance::Dark, "Dark"),
                                ]
                                .into_iter()
                                .enumerate()
                                .map(|(i, (appearance, label))| {
                                    let selected = p.appearance == appearance;
                                    quantum_ui::ListRow::frame(("appearance", i), label, t, selected).flex_col().items_stretch()
                                        .tab_index(0)
                                        .role(Role::Button)
                                        .aria_label(label)
                                        .aria_selected(selected)
                                        .flex_1()
                                        .min_w(px(Space::S0.px()))
                                        .p(px(Space::S3.px()))
                                        .rounded(px(Radius::Md.px()))
                                        .when(selected, |d| d.bg(t.color(ColorToken::Accent).opacity(state::PREVIEW_SELECTED)))
                                        .cursor_pointer()


                                        .child(
                                            Self::appearance_preview(appearance)
                                                .border_2()
                                                .border_color(if selected {
                                                    t.color(ColorToken::Accent)
                                                } else {
                                                    t.color(ColorToken::Border)
                                                }),
                                        )
                                        .child(
                                            div()
                                                .mt(px(Space::S3.px()))
                                                .text_center()
                                                .font_weight(type_size::SEMIBOLD)
                                                .text_color(if selected {
                                                    t.color(ColorToken::Accent)
                                                } else {
                                                    t.color(ColorToken::TextPrimary)
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
                                .mt(px(Space::S5.px()))
                                .text_style(TextSize::Small)
                                .text_color(t.color(ColorToken::TextSecondary))
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
                            "Ambient occlusion",
                            "Shade contacts between nearby voxels.",
                            self.toggle(
                                "ambient-occlusion",
                                p.ambient_occlusion,
                                |p| {
                                    p.ambient_occlusion = !p.ambient_occlusion;
                                    if p.ambient_occlusion
                                        && p.ambient_occlusion_strength
                                            == AmbientOcclusionStrength::OFF
                                    {
                                        p.ambient_occlusion_strength =
                                            AmbientOcclusionStrength::default();
                                    }
                                },
                                t,
                                cx,
                            ),
                            t,
                        ),
                        self.form_row(
                            "AO strength",
                            "0 is off; 1 is the default.",
                            Slider::labelled(
                                Slider::new(
                                    "ambient-occlusion-strength",
                                    "Ambient occlusion strength",
                                    t,
                                    SliderRange::new(0.0, AmbientOcclusionStrength::MAX, 0.05),
                                    p.ambient_occlusion_strength.get(),
                                )
                                .on_change(|value, _, cx| {
                                    let strength = AmbientOcclusionStrength::new(*value)
                                        .expect("bounded AO slider");
                                    change(cx, |p| {
                                        p.ambient_occlusion_strength = strength;
                                        p.ambient_occlusion =
                                            strength != AmbientOcclusionStrength::OFF;
                                    });
                                }),
                                mono(format!("{:.2}", p.ambient_occlusion_strength.get())),
                            )
                            .w(px(geometry::SETTINGS_SLIDER)),
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
                        self.form_row(
                            "Debug overlay",
                            "Show renderer timings, chunks and memory over the viewport.",
                            self.toggle(
                                "debug-overlay",
                                p.show_debug_overlay,
                                |p| p.show_debug_overlay = !p.show_debug_overlay,
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
                                .gap(px(Space::S3.px()))
                                .child(
                                    quantum_ui::Kbd::well(t)
                                        .child(super::shortcut_hint::sheet_shortcut(op)),
                                )
                                // The registry exposes fixed bindings, all already at their defaults.
                                .child(
                                    Button::design(("reset-shortcut", i), "Reset", t)
                                        .disabled(true)
                                        .build()
                                        .aria_label("Reset shortcut")
                                        .anchored_hint(
                                            "This shortcut already uses its built-in default.",
                                            t, TooltipSide::Below, &self.overlays),
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
                    "Quantum",
                    mono(crate::ui::APP_VERSION),
                    t,
                )];
                let updates_available = super::updates_available();
                rows.push(
                    self.form_row(
                        "Software updates",
                        if updates_available {
                            "Check for the latest version of Quantum."
                        } else {
                            "Updates are unavailable in this build."
                        },
                        Button::design("check-updates", "Check for Updates…", t)
                            .disabled(!updates_available)
                            .build()
                            .aria_description(if updates_available {
                                "Check for a new version"
                            } else {
                                "Updates are unavailable in this build."
                            })
                            .when(updates_available, |d| {
                                d.on_click(cx.listener(|_, _, _, _| super::check_for_updates()))
                            }),
                        t,
                    ),
                );
                self.group(rows, t)
            }
            Section::Advanced => self.group(
                vec![
                    self.form_row(
                        "Preferences",
                        "Saved locally on this device.",
                        icon("lock").text_color(t.color(ColorToken::TextSecondary)),
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
        self.search
            .update(cx, |field, _| field.focus_policy(None, self.focus.clone()));
        let query = self.search.read(cx).text().to_owned();
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
            .min_h(px(Space::S0.px()))
            .overflow_y_scroll();
        let mut previous_group = "";
        let mut index = 0;
        for result in &results {
            if query.trim().is_empty() && result.section.group() != previous_group {
                previous_group = result.section.group();
                sidebar_rows = sidebar_rows.child(SidebarSection::header(previous_group, t));
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
                    .p(px(Space::S4.px()))
                    .text_style(TextSize::Small)
                    .text_color(t.color(ColorToken::TextSecondary))
                    .child("No matching settings"),
            );
        }
        if crate::actions::sync_native_menus(window) {
            cx.set_menus(actions::menus_for_window(
                window.is_fullscreen(),
                |_| false,
                |op| {
                    (matches!(op, Op::Undo | Op::Redo) && crate::text_field::focused(window, cx))
                        || op
                            .availability(actions::CommandContext::Settings {
                                updates_available: super::updates_available(),
                            })
                            .is_ok()
                },
            ));
        }
        Panel::section(t)
            .flex()
            .flex_col()
            .id("settings-root")
            .tab_group()
            .tab_stop(false)
            .key_context("Settings")
            .track_focus(&self.focus)
            .size_full()
            .bg(t.color(ColorToken::Surface))
            .font(gpui::font(if cfg!(target_os = "macos") {
                ".AppleSystemUIFont"
            } else {
                "Geist"
            }))
            .text_style(TextSize::Body)
            .text_color(t.color(ColorToken::TextPrimary))
            .rounded(px(Radius::Lg.px()))
            .overflow_hidden()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_action(cx.listener(|this, op: &Op, window, cx| {
                this.action(op, window, cx);
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
                    .h(px(metrics::SETTINGS_TITLEBAR))
                    .flex_shrink_0()
                    .px(px(Space::S5.px()))
                    .gap(px(Space::S5.px()))
                    .when(cfg!(target_os = "macos"), |d| {
                        d.pl(px(metrics::SETTINGS_TRAFFIC_INSET))
                    })
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move())
                    .child(
                        row()
                            .p(px(Space::S2.px()))
                            .gap(px(Space::S1.px()))
                            .rounded_full()
                            .border_1()
                            .border_color(t.color(ColorToken::Border))
                            .bg(t.color(ColorToken::Raised))
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
                            .min_w(px(Space::S0.px()))
                            .text_style(TextSize::Title)
                            .font_weight(type_size::BOLD)
                            .child("Quantum Settings"),
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
                    .min_h(px(Space::S0.px()))
                    .items_stretch()
                    .when(self.sidebar_visible, |d| {
                        d.child(
                            Panel::new(t)
                                .build()
                                .w(px(metrics::MENU_WIDTH))
                                .flex_shrink_0()
                                .m(px(Space::S4.px()))
                                .mt(px(Space::S0.px()))
                                .mr(px(Space::S0.px()))
                                .p(px(Space::S4.px()))
                                .flex()
                                .flex_col()
                                .rounded(px(Radius::Lg.px()))
                                .border_1()
                                .border_color(t.color(ColorToken::Border))
                                .bg(t.color(ColorToken::Surface))
                                .child(
                                    quantum_ui::FieldFrame::search_row(
                                        t,
                                        self.search.read(cx).focus_visible(window),
                                    )
                                    .flex_shrink_0()
                                    .mb(px(Space::S2.px()))
                                    .child(
                                        icon("search")
                                            .size(px(metrics::ICON_DENSE))
                                            .flex_shrink_0()
                                            .text_color(t.color(ColorToken::TextSecondary)),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(Space::S0.px()))
                                            .child(self.search.clone()),
                                    )
                                    .when(
                                        !query.is_empty(),
                                        |d| {
                                            d.child(
                                                self.chrome_button(
                                                    "clear-settings-search",
                                                    "close",
                                                    "Clear search",
                                                    true,
                                                    t,
                                                )
                                                .size(px(Space::S5.px()))
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.search.update(cx, |field, cx| {
                                                        field.set(String::new(), cx)
                                                    });
                                                    window.focus(&this.search.focus_handle(cx), cx);
                                                })),
                                            )
                                        },
                                    ),
                                )
                                .child(sidebar_rows),
                        )
                    })
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(Space::S0.px()))
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .id("settings-pane")
                                    .flex_1()
                                    .min_h(px(Space::S0.px()))
                                    .overflow_y_scroll()
                                    .track_scroll(&self.pane_scroll)
                                    .px(px(Space::S7.px()))
                                    .pt(px(Space::S5.px()))
                                    .pb(px(Space::S7.px()))
                                    .child(
                                        div()
                                            .mb(px(Space::S6.px()))
                                            .text_style(TextSize::Display)
                                            .font_weight(type_size::BOLD)
                                            .child(section.title()),
                                    )
                                    .child(self.content(section, t, cx)),
                            )
                            .when_some(state.error, |d, message| {
                                d.child(
                                    div()
                                        .px(px(Space::S7.px()))
                                        .pb(px(Space::S5.px()))
                                        .text_style(TextSize::Small)
                                        .text_color(t.color(ColorToken::Danger))
                                        .child(message),
                                )
                            }),
                    ),
            )
    }
}

#[cfg(test)]
mod tests;
