//! Timeline slider and named version library.
use super::*;
use quantum_app::history::{HISTORY_STEP_INTERVAL, PreviewSource, display_label};
use quantum_core::{Changes, Origin, Uuid};
use quantum_ui::ColorToken;
use quantum_ui::Space;
use quantum_ui::TextSize;
use quantum_ui::TextStyle;

const RECENT_STEPS: usize = 5;

pub(super) fn change_summary(label: &str, changes: &Changes) -> String {
    if changes.voxel_count() == 0 {
        display_label(label).to_string()
    } else {
        format!(
            "+{} −{} · {} painted",
            changes.added, changes.removed, changes.recolored
        )
    }
}

/// Chronological indices to render; callers reverse them for newest-first rows.
fn visible_steps(count: usize, expanded: bool) -> std::ops::Range<usize> {
    (if expanded {
        0
    } else {
        count.saturating_sub(RECENT_STEPS)
    })..count
}

/// Selection changes reveal older steps and never automatically collapse them.
fn expanded_for_selection(count: usize, expanded: bool, selected: usize) -> bool {
    expanded || (selected < count && !visible_steps(count, false).contains(&selected))
}

/// Map a chronological step to a direct scroll child, skipping the toggle at 5.
fn step_child_index(count: usize, expanded: bool, selected: usize) -> Option<usize> {
    if !visible_steps(count, expanded).contains(&selected) {
        return None;
    }
    let reverse = count - selected - 1;
    Some(reverse + usize::from(reverse >= RECENT_STEPS))
}

pub(super) struct Timeline {
    pub name: Entity<crate::text_field::TextField>,
    pub dragging: bool,
    pub scroll: ScrollHandle,
    pub playback_task: Option<Task<()>>,
    pub expanded: bool,
}
impl Timeline {
    pub fn new(window: &mut Window, cx: &mut Context<EditorView>) -> Self {
        let name = crate::text_field::TextField::design("Name this version…", cx);
        cx.subscribe_in(
            &name,
            window,
            |this, _, event: &quantum_ui::TextFieldEvent, window, cx| {
                if matches!(event, quantum_ui::TextFieldEvent::Committed) {
                    this.save_named_version(window, cx);
                }
            },
        )
        .detach();
        Self {
            name,
            dragging: false,
            scroll: ScrollHandle::new(),
            playback_task: None,
            expanded: false,
        }
    }

    fn reveal_step(&mut self, count: usize, selected: usize) {
        self.expanded = expanded_for_selection(count, self.expanded, selected);
        if let Some(child) = step_child_index(count, self.expanded, selected) {
            self.scroll.scroll_to_item(child);
        }
    }
}
impl EditorView {
    fn save_named_version(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.timeline.name.read(cx).text().trim().to_string();
        let result = self.editor.save_version(name);
        if result.is_ok() {
            self.timeline
                .name
                .update(cx, |field, cx| field.set(String::new(), cx));
        }
        window.focus(&self.focus, cx);
        self.report(result, "Named version saved", cx);
    }
    pub(super) fn pause_history(&mut self, cx: &mut Context<Self>) {
        self.editor.pause_history();
        self.timeline.playback_task = None;
        cx.notify();
    }

    pub(super) fn toggle_history(&mut self, cx: &mut Context<Self>) {
        if self.editor.history_playing() {
            self.pause_history(cx);
            return;
        }
        // Dropping the previous task cancels its pending timer, including on a
        // rapid pause/resume. Only this playback session can advance the cursor.
        self.timeline.playback_task = None;
        match self.editor.play_history(std::time::Instant::now()) {
            Ok(true) => self.refresh_history_playback(cx),
            Ok(false) => return,
            Err(e) => {
                self.report(Err::<(), _>(e), "", cx);
                return;
            }
        }
        self.timeline.playback_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(HISTORY_STEP_INTERVAL).await;
                let playing = this.update(cx, |this, cx| {
                    if !this.editor.history_playing() {
                        return false;
                    }
                    // Slow renders must finish before advancing, or every
                    // intermediate frame could be discarded as stale.
                    if !this.viewport.is_current(&this.editor) {
                        return true;
                    }
                    match this.editor.tick_history(std::time::Instant::now()) {
                        Ok(true) => this.refresh_history_playback(cx),
                        Ok(false) => cx.notify(),
                        Err(e) => this.report(Err::<(), _>(e), "", cx),
                    }
                    this.editor.history_playing()
                });
                if !matches!(playing, Ok(true)) {
                    break;
                }
            }
        }));
    }

    fn refresh_history_playback(&mut self, cx: &mut Context<Self>) {
        self.timeline.reveal_step(
            self.editor.timeline_entries().count(),
            self.editor.timeline_index(),
        );
        self.status = "History preview · read-only".into();
        self.error = false;
        self.refresh_preview(cx);
    }

    pub(super) fn refresh_preview(&mut self, cx: &mut Context<Self>) {
        self.viewport.invalidate_preview();
        self.hover = None;
        self.drag = None;
        self.request_render(cx);
        cx.notify();
    }
    pub(super) fn scrub(&mut self, index: usize, cx: &mut Context<Self>) {
        // Even scrubbing to the already selected state must stop playback.
        self.pause_history(cx);
        let ids: Vec<Option<Uuid>> = self.editor.timeline_entries().collect();
        if let Some(id) = ids.get(index) {
            if self.editor.preview.as_ref().is_some_and(|p| {
                p.source == PreviewSource::History(*id)
                    && p.revision == self.editor.engine.revision()
            }) {
                // A manually collapsed list must also reveal an unchanged selection.
                self.timeline.reveal_step(ids.len(), index);
                return;
            }
            let r = self.editor.preview_history(*id);
            if r.is_ok() {
                self.timeline.reveal_step(ids.len(), index);
                self.refresh_preview(cx);
            }
            self.report(r, "History preview · read-only", cx);
        }
    }
    pub(super) fn timeline_panel(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let p = self.editor.preview.as_ref();
        let steps: Vec<_> = std::iter::once((None, "Initial state".to_string(), Origin::Human))
            .filter(|_| self.editor.engine.initial_retained())
            .chain(
                self.editor
                    .engine
                    .timeline()
                    .map(|e| (Some(e.id), e.label.clone(), e.actor.origin)),
            )
            .collect();
        let selected = self.editor.timeline_index();
        let count = steps.len();
        let recent = visible_steps(count, false);
        let earlier = visible_steps(count, self.timeline.expanded).start..recent.start;
        let event_row = |i: usize| {
            let (id, label, origin) = &steps[i];
            self.control(
                ("timeline-event", i),
                if *origin == Origin::Agent {
                    "diamond"
                } else {
                    "circle"
                },
                display_label(label).to_string(),
                cx,
                move |this, _, cx| this.scrub(i, cx),
            )
            .w_full()
            .border_0()
            .mb(px(Space::S2.px()))
            .when(
                p.is_some_and(|p| p.source == PreviewSource::History(*id)),
                |d| {
                    d.bg(t.selected())
                        .text_color(t.color(ColorToken::AccentText))
                },
            )
        };
        let playing = self.editor.history_playing();
        let playback_label = if playing {
            "Pause history"
        } else {
            "Play history"
        };
        let mut versions: Vec<_> = self.editor.engine.versions().collect();
        versions.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
        let bounds = Rc::new(Cell::new(Bounds::<Pixels>::default()));
        let paint = bounds.clone();
        let drag = bounds.clone();
        let scrub_at = move |this: &mut Self,
                             point: Point<Pixels>,
                             b: Bounds<Pixels>,
                             cx: &mut Context<Self>| {
            let value =
                (f32::from(point.x - b.left()) / f32::from(b.size.width).max(1.0)).clamp(0.0, 1.0);
            this.scrub(
                (value * count.saturating_sub(1) as f32).round() as usize,
                cx,
            );
        };
        Panel::section(t)
            .flex_1()
            .min_h(px(Space::S0.px()))
            .flex()
            .flex_col()
            .p(px(Space::S5.px()))
            .gap(px(Space::S4.px()))
            .child(
                row()
                    .justify_between()
                    .child(section_title("History & versions"))
                    .child(
                        row()
                            .gap(px(Space::S3.px()))
                            .child(
                                mono(format!("{} steps", steps.len()))
                                    .text_color(t.color(ColorToken::TextSecondary)),
                            )
                            .when(count > 1, |d| {
                                d.child(
                                    self.control(
                                        "history-playback",
                                        if playing { "pause" } else { "play" },
                                        "",
                                        cx,
                                        |this, _, cx| this.toggle_history(cx),
                                    )
                                    .w(px(metrics::PLAYBACK_WIDTH))
                                    .px(px(Space::S0.px()))
                                    .gap(px(Space::S0.px()))
                                    .justify_center()
                                    .aria_label(playback_label)
                                    .aria_keyshortcuts("Space")
                                    .anchored_hint(
                                        HintText::from(playback_label).key("space"),
                                        t,
                                        TooltipSide::Below,
                                        &self.overlays,
                                    ),
                                )
                            }),
                    ),
            )
            .child(
                div()
                    .text_color(t.color(ColorToken::TextSecondary))
                    .text_style(TextSize::Small)
                    .child(
                        p.map_or("Live document", |p| display_label(&p.label))
                            .to_string(),
                    ),
            )
            .child(
                div()
                    .id("timeline-slider")
                    .h(px(metrics::TIMELINE_HEIGHT))
                    .relative()
                    .cursor_pointer()
                    .anchored_hint(
                        "Drag to preview · Restore applies the selected state",
                        t,
                        TooltipSide::Below,
                        &self.overlays,
                    )
                    .child(
                        canvas(
                            move |b, _, _| paint.set(b),
                            move |b, _, window, _| {
                                window.paint_quad(fill(
                                    Bounds::new(
                                        point(b.left(), b.top() + px(Space::S4.px())),
                                        size(b.size.width, px(Space::S1.px())),
                                    ),
                                    t.color(ColorToken::BorderSubtle),
                                ));
                            },
                        )
                        .size_full(),
                    )
                    .children(
                        steps
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| {
                                count <= 32 || *i == selected || i % count.div_ceil(32) == 0
                            })
                            .map(|(i, (_, label, origin))| {
                                row()
                                    .id(("timeline-mark", i))
                                    .absolute()
                                    .left(relative(
                                        i as f32 / count.saturating_sub(1).max(1) as f32,
                                    ))
                                    .top(px(Space::S2.px()))
                                    .ml(px(-Space::S3.px()))
                                    .size(px(Space::S5.px()))
                                    .rounded_full()
                                    .justify_center()
                                    .bg(t.color(ColorToken::Canvas))
                                    .border_1()
                                    .border_color(if i == selected {
                                        t.color(ColorToken::AccentText)
                                    } else {
                                        t.color(ColorToken::Canvas)
                                    })
                                    .anchored_hint(
                                        format!("{} · {}", i, display_label(label)),
                                        t,
                                        TooltipSide::Below,
                                        &self.overlays,
                                    )
                                    .child(
                                        icon(if *origin == Origin::Agent {
                                            "diamond"
                                        } else {
                                            "circle"
                                        })
                                        .size(px(Space::S3.px()))
                                        .text_color(
                                            if i == selected {
                                                t.color(ColorToken::AccentText)
                                            } else {
                                                t.color(ColorToken::TextSecondary)
                                            },
                                        ),
                                    )
                            }),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, e: &MouseDownEvent, window, cx| {
                            window.focus(&this.focus, cx);
                            this.timeline.dragging = true;
                            scrub_at(this, e.position, bounds.get(), cx);
                            cx.stop_propagation();
                        }),
                    )
                    .on_mouse_move(cx.listener(move |this, e: &MouseMoveEvent, _, cx| {
                        if this.timeline.dragging && e.pressed_button == Some(MouseButton::Left) {
                            scrub_at(this, e.position, drag.get(), cx);
                        }
                    }))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _, _, _| this.timeline.dragging = false),
                    )
                    .on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(|this, _, _, _| this.timeline.dragging = false),
                    ),
            )
            .when(p.is_some(), |d| {
                d.child(
                    row()
                        .gap(px(Space::S3.px()))
                        .child(self.control(
                            "history-live",
                            "log-out",
                            "Return to live",
                            cx,
                            |this, _, cx| {
                                this.editor.return_live();
                                this.refresh_preview(cx);
                            },
                        ))
                        .child(
                            self.control(
                                "history-restore",
                                "rotate-ccw",
                                "Restore",
                                cx,
                                |this, _, cx| {
                                    let r = this.editor.restore_preview();
                                    if r.is_ok() {
                                        this.refresh_preview(cx);
                                    }
                                    this.report(r, "History restored as a new edit", cx);
                                },
                            )
                            .text_color(t.color(ColorToken::TextPrimary)),
                        ),
                )
            })
            .child(
                div()
                    .id("timeline-events")
                    .track_scroll(&self.timeline.scroll)
                    .flex_1()
                    .min_h(px(metrics::HISTORY_LIST_MIN_HEIGHT))
                    .overflow_y_scroll()
                    .children(recent.rev().map(event_row))
                    .when(count > RECENT_STEPS, |d| {
                        let expanded = self.timeline.expanded;
                        let label = if expanded {
                            "Hide earlier changes".to_owned()
                        } else {
                            format!("Show {} earlier changes", count - RECENT_STEPS)
                        };
                        d.child(
                            self.control(
                                "timeline-earlier-toggle",
                                if expanded { "chevron" } else { "chevron-right" },
                                label.clone(),
                                cx,
                                |this, _, cx| {
                                    this.timeline.expanded = !this.timeline.expanded;
                                    cx.notify();
                                },
                            )
                            .aria_label(label)
                            .aria_expanded(expanded)
                            .w_full()
                            .border_0()
                            .mb(px(Space::S2.px())),
                        )
                    })
                    .children(earlier.rev().map(event_row)),
            )
            .child(
                row()
                    .justify_between()
                    .child(section_title("Named versions"))
                    .child(
                        row()
                            .gap(px(Space::S2.px()))
                            .text_style(TextSize::Small)
                            .text_color(t.color(ColorToken::TextSecondary))
                            .child(
                                icon("circle")
                                    .size(px(Space::S4.px()))
                                    .text_color(t.color(ColorToken::TextSecondary)),
                            )
                            .child("Human")
                            .child(
                                icon("diamond")
                                    .size(px(Space::S4.px()))
                                    .text_color(t.color(ColorToken::AccentText)),
                            )
                            .child("Agent"),
                    ),
            )
            .child(self.named_versions(versions, cx))
            .when(p.is_none(), |d| {
                d.child(
                    row()
                        .gap(px(Space::S3.px()))
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(Space::S0.px()))
                                .child(self.timeline.name.clone()),
                        )
                        .child(self.control(
                            "save-version",
                            "save",
                            "Save",
                            cx,
                            Self::save_named_version,
                        )),
                )
            })
    }
}

impl EditorView {
    fn named_versions(
        &self,
        versions: Vec<&quantum_core::NamedVersion>,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let t = self.theme;
        let p = self.editor.preview.as_ref();
        div()
            .id("versions-scroll")
            .max_h(px(metrics::VERSIONS_MAX_HEIGHT))
            .overflow_y_scroll()
            .children(versions.into_iter().enumerate().map(|(i, v)| {
                let id = v.id;
                self.control(
                    ("named-version", i),
                    if v.origin == Origin::Agent {
                        "diamond"
                    } else {
                        "circle"
                    },
                    v.name.clone(),
                    cx,
                    move |this, _, cx| {
                        let r = this.editor.preview_version(id);
                        if r.is_ok() {
                            this.refresh_preview(cx);
                        }
                        this.report(r, "Version preview · read-only", cx);
                    },
                )
                .w_full()
                .border_0()
                .mb(px(Space::S2.px()))
                .when(
                    p.is_some_and(|p| p.source == PreviewSource::Version(id)),
                    |d| {
                        d.bg(t.selected())
                            .text_color(t.color(ColorToken::AccentText))
                    },
                )
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RECENT_STEPS, change_summary, expanded_for_selection, step_child_index, visible_steps,
    };
    use quantum_app::editor::Editor;
    use quantum_core::Changes;

    #[test]
    fn demo_non_voxel_changes_describe_the_edit() {
        let editor = Editor::example().unwrap();
        for label in [
            "Name the autumn garden layer",
            "Add a composition review note",
            "Create named version",
            "Warm the window and lantern glass",
        ] {
            let entry = editor.engine.history().find(|e| e.label == label).unwrap();
            assert_eq!(entry.changes.voxel_count(), 0);
            assert_eq!(change_summary(&entry.label, &entry.changes), label);
        }
    }

    #[test]
    fn non_voxel_changes_preserve_custom_labels_and_humanize_protocol_labels() {
        let metadata = Changes {
            metadata_changed: true,
            ..Changes::default()
        };
        assert_eq!(change_summary("Rename layer", &metadata), "Rename layer");
        assert_eq!(change_summary("comments add", &metadata), "Add comment");
        // Named versions change the document without setting snapshot change flags.
        assert_eq!(
            change_summary("versions save", &Changes::default()),
            "Save named version"
        );
    }

    #[test]
    fn voxel_changes_keep_counts_even_when_metadata_also_changes() {
        for (added, removed, recolored, expected) in [
            (1, 0, 0, "+1 −0 · 0 painted"),
            (0, 2, 0, "+0 −2 · 0 painted"),
            (0, 0, 3, "+0 −0 · 3 painted"),
            (4, 5, 6, "+4 −5 · 6 painted"),
        ] {
            for metadata_changed in [false, true] {
                let changes = Changes {
                    added,
                    removed,
                    recolored,
                    metadata_changed,
                    ..Changes::default()
                };
                assert_eq!(change_summary("Edit voxels", &changes), expected);
            }
        }
    }

    #[test]
    fn recent_window_handles_empty_short_and_long_timelines() {
        for count in 0..=RECENT_STEPS {
            assert_eq!(visible_steps(count, false), 0..count);
            assert_eq!(visible_steps(count, true), 0..count);
        }
        assert_eq!(
            visible_steps(6, false).rev().collect::<Vec<_>>(),
            [5, 4, 3, 2, 1]
        );
        assert_eq!(
            visible_steps(9, false).rev().collect::<Vec<_>>(),
            [8, 7, 6, 5, 4]
        );
        assert_eq!(visible_steps(9, true), 0..9);
    }

    #[test]
    fn scroll_indices_match_rows_on_both_sides_of_toggle() {
        for count in 0..20 {
            for expanded in [false, true] {
                let recent = visible_steps(count, false);
                let earlier = visible_steps(count, expanded).start..recent.start;
                let mut children: Vec<_> = recent.rev().map(Some).collect();
                if count > RECENT_STEPS {
                    assert_eq!(children.len(), RECENT_STEPS);
                    children.push(None); // Toggle is a direct child at index 5.
                }
                children.extend(earlier.rev().map(Some));
                for selected in 0..=count {
                    let expected = children.iter().position(|child| *child == Some(selected));
                    assert_eq!(step_child_index(count, expanded, selected), expected);
                }
            }
        }
        assert_eq!(step_child_index(9, true, 4), Some(4));
        assert_eq!(step_child_index(9, true, 3), Some(6));
        assert_eq!(step_child_index(9, true, 0), Some(9));
        assert_eq!(step_child_index(9, false, 3), None);
    }

    #[test]
    fn selection_reveals_older_steps_and_stays_expanded_when_moving_forward() {
        assert!(!expanded_for_selection(0, false, 0));
        assert!(!expanded_for_selection(5, false, 0));
        assert!(!expanded_for_selection(9, false, 4));
        assert!(expanded_for_selection(9, false, 3));
        assert!(expanded_for_selection(9, false, 0));
        assert!(expanded_for_selection(9, true, 8));
        assert!(!expanded_for_selection(9, false, 9));
        // Playback from oldest to newest keeps every selected row reachable.
        let mut expanded = false;
        for selected in 0..9 {
            expanded = expanded_for_selection(9, expanded, selected);
            assert!(expanded);
            assert!(step_child_index(9, expanded, selected).is_some());
        }
    }
}
