//! Timeline slider and named version library.
use super::*;
use voxelite_app::history::{HISTORY_STEP_INTERVAL, PreviewSource, display_label};
use voxelite_core::{Origin, Uuid};
pub(super) struct Timeline {
    pub name: Entity<crate::text_field::TextField>,
    pub dragging: bool,
    pub scroll: ScrollHandle,
    pub playback_task: Option<Task<()>>,
}
impl Timeline {
    pub fn new(cx: &mut App) -> Self {
        Self {
            name: crate::text_field::TextField::new("Name this version…", cx),
            dragging: false,
            scroll: ScrollHandle::new(),
            playback_task: None,
        }
    }
}
impl EditorView {
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
        match self.editor.play_history(web_time::Instant::now()) {
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
                    if this.displayed != Some(this.frame_identity()) {
                        return true;
                    }
                    match this.editor.tick_history(web_time::Instant::now()) {
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
        self.timeline.scroll.scroll_to_item(
            self.editor.timeline_entries().count() - self.editor.timeline_index() - 1,
        );
        self.status = "History preview · read-only".into();
        self.error = false;
        self.refresh_preview(cx);
    }

    pub(super) fn refresh_preview(&mut self, cx: &mut Context<Self>) {
        self.document_generation = self.document_generation.wrapping_add(1);
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
                return;
            }
            let r = self.editor.preview_history(*id);
            if r.is_ok() {
                self.timeline
                    .scroll
                    .scroll_to_item(ids.len().saturating_sub(index + 1));
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
        div()
            .flex_1()
            .min_h(px(0.0))
            .flex()
            .flex_col()
            .p(px(20.0))
            .gap(px(12.0))
            .child(
                row()
                    .justify_between()
                    .child(section_title("History & versions"))
                    .child(
                        row()
                            .gap(px(8.0))
                            .child(mono(format!("{} steps", steps.len())).text_color(t.muted()))
                            .when(count > 1, |d| {
                                d.child(
                                    self.control(
                                        "history-playback",
                                        if playing { "pause" } else { "play" },
                                        "",
                                        cx,
                                        |this, _, cx| this.toggle_history(cx),
                                    )
                                    .w(px(34.0))
                                    .px(px(0.0))
                                    .gap(px(0.0))
                                    .justify_center()
                                    .aria_label(playback_label)
                                    .aria_keyshortcuts("Space")
                                    .anchored_hint(
                                        HintText::from(playback_label).key("space"),
                                        t,
                                        TooltipSide::Below,
                                    ),
                                )
                            }),
                    ),
            )
            .child(
                div().text_color(t.muted()).text_size(px(12.0)).child(
                    p.map_or("Live document", |p| display_label(&p.label))
                        .to_string(),
                ),
            )
            .child(
                div()
                    .id("timeline-slider")
                    .h(px(26.0))
                    .relative()
                    .cursor_pointer()
                    .tooltip(hint(
                        "Drag to preview · Restore applies the selected state",
                        t,
                    ))
                    .child(
                        canvas(
                            move |b, _, _| paint.set(b),
                            move |b, _, window, _| {
                                window.paint_quad(fill(
                                    Bounds::new(
                                        point(b.left(), b.top() + px(12.0)),
                                        size(b.size.width, px(2.0)),
                                    ),
                                    t.line(),
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
                                    .top(px(5.0))
                                    .ml(px(-8.0))
                                    .size(px(16.0))
                                    .rounded_full()
                                    .justify_center()
                                    .bg(t.background())
                                    .border_1()
                                    .border_color(if i == selected {
                                        t.accent()
                                    } else {
                                        t.background()
                                    })
                                    .tooltip(hint(format!("{} · {}", i, display_label(label)), t))
                                    .child(
                                        icon(if *origin == Origin::Agent {
                                            "diamond"
                                        } else {
                                            "circle"
                                        })
                                        .size(px(8.0))
                                        .text_color(
                                            if i == selected { t.accent() } else { t.muted() },
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
                        .gap(px(8.0))
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
                            .text_color(t.text()),
                        ),
                )
            })
            .child(
                div()
                    .id("timeline-events")
                    .track_scroll(&self.timeline.scroll)
                    .flex_1()
                    .min_h(px(70.0))
                    .overflow_y_scroll()
                    .children(
                        steps
                            .iter()
                            .enumerate()
                            .rev()
                            .map(|(i, (id, label, origin))| {
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
                                .mb(px(3.0))
                                .when(
                                    p.is_some_and(|p| p.source == PreviewSource::History(*id)),
                                    |d| d.bg(t.selected()).text_color(t.accent()),
                                )
                            }),
                    ),
            )
            .child(
                row()
                    .justify_between()
                    .child(section_title("Named versions"))
                    .child(
                        row()
                            .gap(px(5.0))
                            .text_size(px(12.0))
                            .text_color(t.muted())
                            .child(icon("circle").size(px(10.0)).text_color(t.muted()))
                            .child("Human")
                            .child(icon("diamond").size(px(10.0)).text_color(t.accent()))
                            .child("Agent"),
                    ),
            )
            .child(
                div()
                    .id("versions-scroll")
                    .max_h(px(110.0))
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
                        .mb(px(3.0))
                        .when(
                            p.is_some_and(|p| p.source == PreviewSource::Version(id)),
                            |d| d.bg(t.selected()).text_color(t.accent()),
                        )
                    })),
            )
            .when(p.is_none(), |d| {
                d.child(
                    row()
                        .gap(px(8.0))
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.0))
                                .child(self.timeline.name.clone()),
                        )
                        .child(self.control(
                            "save-version",
                            "save",
                            "Save",
                            cx,
                            |this, window, cx| {
                                let name = this.timeline.name.read(cx).text.trim().to_string();
                                let r = this.editor.save_version(name);
                                if r.is_ok() {
                                    this.timeline
                                        .name
                                        .update(cx, |f, cx| f.set(String::new(), cx));
                                }
                                window.focus(&this.focus, cx);
                                this.report(r, "Named version saved", cx);
                            },
                        )),
                )
            })
    }
}
