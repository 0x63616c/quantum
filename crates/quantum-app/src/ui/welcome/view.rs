//! Split Welcome presentation; callbacks stay on the launcher view.
use super::*;
use quantum_ui::ColorToken;
use quantum_ui::Radius;
use quantum_ui::Space;
use quantum_ui::TextSize;
use quantum_ui::TextStyle;

impl WelcomeView {
    fn sample_card(
        &self,
        index: usize,
        compact: bool,
        focused: bool,
        t: Theme,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let sample = Sample::ALL[index];
        let preview = self.samples.get(&sample);
        let selected = self.selected_item() == Selection::Sample(sample);
        quantum_ui::ListRow::frame(("sample", index), sample.name(), t, selected)
            .px(px(Space::S0.px()))
            .gap(px(Space::S0.px()))
            .tab_stop(false)
            .flex_1()
            .min_w(px(Space::S0.px()))
            .flex()
            .flex_col()
            .items_start()
            .text_color(t.color(ColorToken::TextPrimary))
            .role(Role::ListBoxOption)
            .aria_selected(selected)
            .when(selected, |d| d.aria_active_descendant())
            .when(!self.busy, |d| {
                d.on_click(cx.listener(move |this, _, window, cx| {
                    this.selection = Some(Selection::Sample(sample));
                    window.focus(&this.list_focus, cx);
                    this.open_sample(sample, cx);
                }))
            })
            .anchored_hint(sample.name(), t, TooltipSide::Below, &self.overlays)
            .child(
                div()
                    .w_full()
                    .h(px(if compact {
                        layout::SAMPLE_SMALL
                    } else {
                        layout::SAMPLE_LARGE
                    }))
                    .child(
                        layout::thumbnail(
                            preview.map(|p| &p.path),
                            t,
                            true,
                            if compact {
                                layout::SAMPLE_SMALL
                            } else {
                                layout::SAMPLE_LARGE
                            },
                        )
                        .when(selected && (!compact || focused), |d| {
                            d.border_color(t.color(ColorToken::AccentText))
                        })
                        .when(selected && focused, |d| {
                            d.shadow(quantum_ui::tokens::focus_ring(t))
                        }),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .mt(px(Space::S3.px()))
                    .text_style(if compact {
                        TextSize::Small
                    } else {
                        TextSize::Body
                    })
                    .h(px(type_size::LINE_SM))
                    .font_weight(if compact {
                        type_size::NORMAL
                    } else {
                        type_size::MEDIUM
                    })
                    .when(compact, |d| {
                        d.text_color(t.color(ColorToken::TextSecondary))
                    })
                    .truncate()
                    .child(sample.name()),
            )
            .when(!compact, |d| {
                d.child(
                    div()
                        .w_full()
                        .mt(px(Space::S1.px()))
                        .text_style(TextSize::Caption)
                        .h(px(type_size::LINE_SM))
                        .text_color(t.color(ColorToken::TextSecondary))
                        .truncate()
                        .child(preview.map(sample_summary).unwrap_or_default()),
                )
            })
    }
}

impl Render for WelcomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.evict_images(window, cx);
        let settings = cx.global::<settings::SettingsState>();
        let t = settings::theme(settings.values.appearance, window);
        let show_welcome = settings.values.show_welcome;
        let projects = cx.global::<Launcher>().projects.clone();
        let error = self
            .error
            .clone()
            .or_else(|| cx.global::<Launcher>().error.clone())
            .or(settings.error.clone());
        self.refresh_menus(window, cx);
        let home = std::env::var_os("HOME").map(PathBuf::from);
        #[cfg(feature = "ui-snapshots")]
        let home = super::snapshot::home(cx).or(home);
        let empty = projects.is_empty();
        let sidebar = self.sidebar(show_welcome, t, cx);
        let list_focused = self.list_focus.is_focused(window);
        let mut main = div()
            .id("welcome-projects")
            .track_focus(&self.list_focus)
            .tab_index(0)
            .role(Role::ListBox)
            .aria_label("Recent projects and samples. Use Up and Down to select, Enter to open.")
            .flex_1()
            .min_w(px(Space::S0.px()))
            .h_full()
            .flex()
            .flex_col()
            .pt(px(layout::MAIN_TOP))
            .px(px(Space::S5.px()))
            .pb(px(Space::S5.px()));
        if !empty {
            main = main.child(recent_header(t));
            // The list owns spare height; samples stay at the bottom for every
            // history length. Long histories scroll above the fixed strip.
            let mut list = div()
                .id("recent-projects")
                .flex_1()
                .min_h(px(Space::S0.px()))
                .flex_shrink(1.)
                .overflow_y_scroll()
                .track_scroll(&self.scroll);
            for (index, project) in projects.iter().enumerate() {
                list = list.child(self.recent_row(
                    index,
                    project,
                    home.as_deref(),
                    list_focused,
                    t,
                    cx,
                ));
            }
            main = main.child(list);
        }
        let samples = self.sample_list(empty, list_focused, t, cx);
        main = main
            .child(samples)
            .when(self.busy, |d| {
                d.child(
                    AsciiReveal::new("project-opening", "Opening project…", t)
                        .loading(true)
                        .reduced_motion(self.reduced_motion),
                )
            })
            .when_some(error, |d, error| {
                d.child(
                    div()
                        .mt(px(Space::S3.px()))
                        .text_color(t.color(ColorToken::Danger))
                        .child(error),
                )
            });
        div()
            .image_cache(self.images.clone())
            .id("welcome")
            .key_context("Editor")
            .track_focus(&self.focus)
            .tab_stop(false)
            .size_full()
            .flex()
            .overflow_hidden()
            .bg(t.color(ColorToken::Canvas))
            .text_color(t.color(ColorToken::TextPrimary))
            .font_family(type_size::UI)
            .text_style(TextSize::Small)
            .on_action(cx.listener(|this, op: &Op, window, cx| this.run(*op, None, window, cx)))
            .on_action(cx.listener(|this, op: &OpenRecentProject, window, cx| {
                this.run(Op::Open, Some(op.path.clone()), window, cx)
            }))
            .on_key_down(cx.listener(Self::key_down))
            .child(sidebar)
            .child(main)
    }
}

impl WelcomeView {
    fn sidebar(&self, show_welcome: bool, t: Theme, cx: &mut Context<Self>) -> Div {
        let sidebar = div()
            .relative()
            .w(px(layout::SIDEBAR))
            .h_full()
            .flex_shrink_0()
            .border_r_1()
            .border_color(t.color(ColorToken::BorderSubtle))
            .pt(px(layout::SIDEBAR_TOP))
            .px(px(Space::S5.px()))
            .pb(px(Space::S6.px()))
            .flex()
            .flex_col()
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h(px(layout::SIDEBAR_TOP))
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move()),
            )
            .child(
                img("welcome-mark.png")
                    .size(px(layout::MARK))
                    .ml(-px(Space::S2.px())),
            )
            .child(
                div()
                    .mt(px(Space::S5.px()))
                    .text_style(TextSize::Display)
                    .line_height(px(TextSize::Display.line_height()))
                    .font_weight(type_size::SEMIBOLD)
                    .child("Quantum"),
            )
            .child(
                div()
                    .mt(px(Space::S2.px()))
                    .text_style(TextSize::Mono)
                    .flex_shrink_0()
                    .text_color(t.color(ColorToken::TextSecondary))
                    .child(format!("Version {}", crate::ui::APP_VERSION)),
            );
        sidebar
            .child(self.sidebar_actions(t, cx))
            .child(div().flex_1())
            .child(self.launch_checkbox(show_welcome, t, cx))
    }
    fn sidebar_actions(&self, t: Theme, cx: &mut Context<Self>) -> Div {
        let mut actions = div()
            .mt(px(Space::S7.px()))
            .mx(-px(Space::S3.px()))
            .flex()
            .flex_col()
            .gap(px(Space::S1.px()));
        for (tab, (op, label)) in [
            (Op::New, "New Project"),
            (Op::Open, "Open…"),
            (Op::OpenSample, "Open Sample…"),
        ]
        .into_iter()
        .enumerate()
        {
            actions = actions.child(
                quantum_ui::ListRow::new(op.command_id(), label, t)
                    .icon(op.icon())
                    .trailing(layout::caps(op.key().unwrap_or(""), t))
                    .disabled(self.busy)
                    .build()
                    .track_focus(&self.sidebar_focus[tab])
                    .anchored_hint(label, t, TooltipSide::Below, &self.overlays)
                    .when(!self.busy, |d| {
                        d.on_click(
                            cx.listener(move |this, _, window, cx| this.run(op, None, window, cx)),
                        )
                    }),
            );
        }
        actions
    }
    fn launch_checkbox(
        &self,
        show_welcome: bool,
        t: Theme,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        quantum_ui::ListRow::frame("show-at-launch", "Show at launch", t, false)
            .track_focus(&self.sidebar_focus[3])
            .tab_index(0)
            .anchored_hint(
                "Show Welcome when Quantum launches",
                t,
                TooltipSide::Below,
                &self.overlays,
            )
            .role(Role::CheckBox)
            .aria_toggled(if show_welcome {
                Toggled::True
            } else {
                Toggled::False
            })
            .flex()
            .items_center()
            .gap(px(Space::S3.px()))
            .text_color(t.color(ColorToken::TextSecondary))
            .child(
                div()
                    .size(px(metrics::ICON_DISCLOSURE))
                    .rounded(px(Radius::Xs.px()))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(if show_welcome {
                        t.color(ColorToken::Accent)
                    } else {
                        t.hover()
                    })
                    .when(!show_welcome, |d| {
                        d.border_1().border_color(t.color(ColorToken::BorderSubtle))
                    })
                    .when(show_welcome, |d| {
                        d.child(
                            icon("check")
                                .size(px(metrics::ICON_TINY))
                                .text_color(t.color(ColorToken::OnAccent)),
                        )
                    }),
            )
            .child("Show at launch")
            .on_click(
                cx.listener(|_, _, _, cx| {
                    settings::change(cx, |p| p.show_welcome = !p.show_welcome)
                }),
            )
    }
}

impl WelcomeView {
    fn recent_row(
        &self,
        index: usize,
        project: &RecentProject,
        home: Option<&std::path::Path>,
        list_focused: bool,
        t: Theme,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let path = project.path.clone();
        let selected = self.selected_item() == Selection::Recent(path.clone());
        quantum_ui::ListRow::new(("recent", index), project.name.clone(), t)
            .meta(folder_label(&project.path, home))
            .selected(selected)
            .leading(
                div()
                    .w(px(layout::THUMB_WIDTH))
                    .h(px(layout::THUMB_HEIGHT))
                    .child(
                        layout::thumbnail(None, t, false, layout::THUMB_HEIGHT).child(
                            crate::ui::loading::recent_element(
                                self.previews.get(&path),
                                index,
                                project.name.clone(),
                                t,
                                self.reduced_motion,
                            ),
                        ),
                    ),
            )
            .trailing(
                div()
                    .text_style(TextSize::Small)
                    .text_color(t.color(ColorToken::TextSecondary))
                    .child(modified_label(project.modified, recent_clock(cx))),
            )
            .disabled(self.busy)
            .build()
            .tab_stop(false)
            .role(Role::ListBoxOption)
            .when(selected, |d| d.aria_active_descendant())
            .when(selected && list_focused, |d| {
                d.shadow(quantum_ui::tokens::focus_ring(t))
            })
            .h(px(layout::ROW))
            .anchored_hint(
                path.display().to_string(),
                t,
                TooltipSide::Below,
                &self.overlays,
            )
            .when(!self.busy, |d| {
                d.on_click(cx.listener(move |this, _, window, cx| {
                    this.selection = Some(Selection::Recent(path.clone()));
                    window.focus(&this.list_focus, cx);
                    this.run(Op::Open, Some(path.clone()), window, cx);
                }))
            })
    }
}

impl WelcomeView {
    fn sample_list(
        &self,
        empty: bool,
        list_focused: bool,
        t: Theme,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let mut samples = div()
            .id("samples")
            .flex_shrink_0()
            .when(!empty, |d| {
                d.mt(px(Space::S5.px()))
                    .pt(px(Space::S5.px()))
                    .border_t_1()
                    .border_color(t.color(ColorToken::BorderSubtle))
            })
            .child(layout::header("Samples", t).when(empty, |d| {
                d.child(div().flex_1()).child(
                    div()
                        .text_style(TextSize::Small)
                        .font_weight(type_size::NORMAL)
                        .child("No recent projects yet"),
                )
            }));
        let samples_focused = list_focused;
        if empty {
            for indices in [[0, 1], [2, 3]] {
                samples = samples.child(
                    div()
                        .flex()
                        .gap(px(Space::S4.px()))
                        .px(px(Space::S3.px()))
                        .when(indices[0] > 0, |d| d.mt(px(Space::S4.px())))
                        .children(
                            indices.map(|i| self.sample_card(i, false, samples_focused, t, cx)),
                        ),
                );
            }
        } else {
            samples = samples.child(
                div()
                    .flex()
                    .gap(px(Space::S3.px()))
                    .px(px(Space::S3.px()))
                    .children((0..4).map(|i| self.sample_card(i, true, samples_focused, t, cx))),
            );
        }
        samples
    }
}
fn sample_summary(preview: &SamplePreview) -> String {
    format!(
        "{} voxels · {} {}",
        grouped_count(preview.voxels),
        preview.layers,
        if preview.layers == 1 {
            "layer"
        } else {
            "layers"
        }
    )
}

impl WelcomeView {
    fn refresh_menus(&self, window: &Window, cx: &mut Context<Self>) {
        if crate::actions::sync_native_menus(window) {
            let native_menus = actions::menus_for_window(
                window.is_fullscreen(),
                |_| false,
                |op| {
                    !self.busy
                        && matches!(
                            op,
                            Op::New
                                | Op::Open
                                | Op::OpenSample
                                | Op::OpenRecent
                                | Op::ClearRecents
                                | Op::Welcome
                                | Op::Quit
                                | Op::Hide
                                | Op::HideOthers
                                | Op::ShowAll
                                | Op::Minimize
                                | Op::ZoomWindow
                        )
                },
            );
            cx.set_menus(menus(native_menus, !self.busy, cx));
        }
    }
}

impl WelcomeView {
    fn evict_images(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths: HashSet<_> = self.samples.values().map(|p| p.path.clone()).collect();
        for path in self.image_paths.difference(&paths) {
            self.images.update(cx, |images, cx| {
                images.remove(&Resource::Path(path.clone().into()), window, cx)
            });
        }
        self.image_paths = paths;
        for image in self.retired_previews.drain(..) {
            let _ = window.drop_image(image);
        }
    }
}

fn recent_clock(_cx: &App) -> std::time::SystemTime {
    #[cfg(feature = "ui-snapshots")]
    if let Some(clock) = super::snapshot::clock(_cx) {
        return clock;
    }
    std::time::SystemTime::now()
}

fn recent_header(t: Theme) -> Div {
    layout::header("Recent", t).child(div().flex_1()).child(
        row()
            .gap(px(Space::S2.px()))
            .child(layout::caps("up", t))
            .child(layout::caps("down", t))
            .child(layout::caps("enter", t)),
    )
}
