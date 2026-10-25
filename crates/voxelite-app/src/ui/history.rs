//! Timeline slider and named version library.
use super::*;
use voxelite_app::history::PreviewSource;
use voxelite_core::{Origin, Uuid};
pub(super) struct Timeline {
    pub name: Entity<crate::text_field::TextField>,
    pub dragging: bool,
}
impl Timeline {
    pub fn new(cx: &mut App) -> Self {
        Self {
            name: crate::text_field::TextField::new("Name this version…", cx),
            dragging: false,
        }
    }
}
impl EditorView {
    pub(super) fn refresh_preview(&mut self, cx: &mut Context<Self>) {
        self.document_generation = self.document_generation.wrapping_add(1);
        self.hover = None;
        self.drag = None;
        self.request_render(cx);
        cx.notify();
    }
    pub(super) fn scrub(&mut self, index: usize, cx: &mut Context<Self>) {
        let ids: Vec<Option<Uuid>> = std::iter::once(None)
            .filter(|_| self.editor.engine.initial_retained())
            .chain(self.editor.engine.timeline().map(|e| Some(e.id)))
            .collect();
        if let Some(id) = ids.get(index) {
            if self.editor.preview.as_ref().is_some_and(|p| {
                p.source == PreviewSource::History(*id)
                    && p.revision == self.editor.engine.revision()
            }) {
                return;
            }
            let r = self.editor.preview_history(*id);
            if r.is_ok() {
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
        let selected = p
            .and_then(|p| {
                steps
                    .iter()
                    .position(|(id, _, _)| p.source == PreviewSource::History(*id))
            })
            .unwrap_or(steps.len().saturating_sub(1));
        let count = steps.len();
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
            .min_h(px(140.0))
            .flex()
            .flex_col()
            .p(px(20.0))
            .gap(px(12.0))
            .child(
                row()
                    .justify_between()
                    .child(section_title("History & versions"))
                    .child(mono(format!("{} steps", steps.len())).text_color(t.muted())),
            )
            .child(
                div()
                    .text_color(t.muted())
                    .text_size(px(12.0))
                    .child(p.map_or("Live document", |p| p.label.as_str()).to_string()),
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
                                let x = b.left()
                                    + b.size.width
                                        * (selected as f32 / count.saturating_sub(1).max(1) as f32);
                                window.paint_quad(fill(
                                    Bounds::new(
                                        point(x - px(4.0), b.top() + px(7.0)),
                                        size(px(8.0), px(12.0)),
                                    ),
                                    t.accent(),
                                ));
                            },
                        )
                        .size_full(),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, e: &MouseDownEvent, _, cx| {
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
                            "play",
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
                            .bg(t.text())
                            .text_color(t.background()),
                        ),
                )
            })
            .child(
                div()
                    .id("timeline-events")
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
                                    label.clone(),
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
            .child(section_title("Named versions"))
            .child(
                div()
                    .id("versions-scroll")
                    .max_h(px(110.0))
                    .overflow_y_scroll()
                    .children(self.editor.engine.versions().enumerate().map(|(i, v)| {
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
                    })),
            )
            .child(
                row()
                    .gap(px(8.0))
                    .child(div().flex_1().child(self.timeline.name.clone()))
                    .child(
                        self.control("save-version", "save", "Save", cx, |this, window, cx| {
                            let name = this.timeline.name.read(cx).text.trim().to_string();
                            let r = this.editor.save_version(name);
                            if r.is_ok() {
                                this.timeline
                                    .name
                                    .update(cx, |f, cx| f.set(String::new(), cx));
                            }
                            window.focus(&this.focus, cx);
                            this.report(r, "Named version saved", cx);
                        }),
                    ),
            )
    }
}
