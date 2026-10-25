//! Live agent connection and real MCP event feed.
use super::*;
use std::collections::VecDeque;
use voxelite_mcp::attached::{AttachedSession, SessionEvent};
pub(super) struct CapturedViews {
    pub revision: u64,
    pub views: Vec<(String, PresetView, Arc<RenderImage>)>,
    pub selected: usize,
}
pub(super) struct Agent {
    pub session: Option<AttachedSession>,
    pub events: VecDeque<SessionEvent>,
    pub open: bool,
    pub clients: usize,
    pub expanded: Option<usize>,
    pub capture: Option<CapturedViews>,
    pub capture_generation: u64,
    pub capturing: bool,
    pub follow: bool,
    pub retired: Vec<Arc<RenderImage>>,
}
impl Agent {
    pub fn new(editor: &Editor, path: Option<PathBuf>) -> std::io::Result<Self> {
        let path = match path {
            Some(p) => p,
            None => {
                let dir = voxelite_app::palettes::data_dir();
                std::fs::create_dir_all(&dir)?;
                dir.join(format!("session-{}.sock", std::process::id()))
            }
        };
        let session = AttachedSession::bind(&std::env::current_dir()?, editor.project(), &path)?;
        Ok(Self {
            session: Some(session),
            events: VecDeque::new(),
            open: false,
            clients: 0,
            expanded: None,
            capture: None,
            capture_generation: 0,
            capturing: false,
            follow: false,
            retired: vec![],
        })
    }
}
impl EditorView {
    pub(super) fn poll_agent(&mut self, cx: &mut Context<Self>) {
        if self.dialog_open || self.pending.is_some() {
            return;
        }
        let Some(session) = &mut self.agent.session else {
            return;
        };
        let revision = self.editor.engine.revision();
        let clients = session.clients();
        let events = session.poll(&mut self.editor.engine);
        let notify = clients != self.agent.clients || !events.is_empty();
        self.agent.clients = clients;
        for mut event in events {
            if let Some(snapshot) = event.inspection.take() {
                if self.agent.follow
                    && self.editor.preview.is_none()
                    && let Some(view) = event.views.first().and_then(|v| preset(v))
                {
                    self.set_agent_view(view, cx);
                }
                self.capture_agent_views(snapshot, event.views.clone(), event.revision, cx);
            }
            self.agent.events.push_back(event);
            if self.agent.events.len() > 200 {
                self.agent.events.pop_front();
                self.agent.expanded = None;
            }
        }
        if self.editor.engine.revision() != revision {
            self.editor.agent_updated();
            self.request_render(cx);
            self.status = format!(
                "Agent updated document · r{}",
                self.editor.engine.revision()
            );
            self.error = false;
        }
        if notify {
            cx.notify();
        }
    }
    pub(super) fn agent_popover(&self, cx: &Context<Self>) -> AnyElement {
        let t = self.theme;
        let path = self
            .agent
            .session
            .as_ref()
            .map(|s| s.path().display().to_string())
            .unwrap_or_else(|| "Unavailable".into());
        div().absolute().top(px(100.0)).right(px(24.0)).child(self.floating().w(px(460.0)).p(px(20.0)).flex().flex_col().gap(px(16.0))
            .child(row().justify_between().child(section_title("Connect your agent")).child(mono("LOCAL MCP").text_color(t.muted())))
            .child(div().text_color(t.muted()).child("Use the MCP server as a stdio bridge to this window. Edits appear live and share your undo history."))
            .child(mono("voxelite-mcp --connect")).child(mono(path).text_size(px(12.0)).overflow_hidden())
            .child(self.control("copy-mcp-config","copy","Copy MCP configuration",cx,|this,_,cx|{
                if let Some(s)=&this.agent.session{
                    let config=serde_json::json!({"mcpServers":{"voxelite":{"command":"voxelite-mcp","args":["--connect",s.path()]}}});
                    cx.write_to_clipboard(ClipboardItem::new_string(serde_json::to_string_pretty(&config).unwrap()));this.status="MCP configuration copied".into();cx.notify();
                }
            }))
            .child(div().text_size(px(12.0)).text_color(t.muted()).child("Requires voxelite-mcp on PATH. This socket accepts local processes running as you. Changes stay in memory until saved in the app.")))
            .into_any_element()
    }
    pub(super) fn agent_feed(&self, compact: bool, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let paused = self.agent.session.as_ref().is_some_and(|s| s.paused);
        if compact && self.agent.events.is_empty() {
            return row()
                .h(px(72.0))
                .flex_shrink_0()
                .px(px(20.0))
                .justify_between()
                .child(div().text_color(t.muted()).child("No edits yet"))
                .child(self.control(
                    "connect-agent-compact",
                    "plug",
                    "Connect agent",
                    cx,
                    |this, _, cx| {
                        this.agent.open = true;
                        cx.notify();
                    },
                ));
        }
        if self.agent.events.is_empty() {
            return div()
                .flex_1()
                .min_h(px(160.0))
                .flex()
                .flex_col()
                .child(
                    row()
                        .px(px(20.0))
                        .pt(px(12.0))
                        .gap(px(8.0))
                        .child(self.control(
                            "connect-agent",
                            "plug",
                            "Connect agent",
                            cx,
                            |this, _, cx| {
                                this.agent.open = true;
                                cx.notify();
                            },
                        )),
                )
                .child(self.history_panel(true, cx));
        }
        div()
            .flex_1()
            .min_h(px(if compact { 220.0 } else { 350.0 }))
            .flex()
            .flex_col()
            .p(px(20.0))
            .gap(px(12.0))
            .child(
                row()
                    .justify_between()
                    .child(section_title("Agent activity"))
                    .child(self.control(
                        "pause-agent",
                        if paused { "play" } else { "pause" },
                        if paused { "Resume" } else { "Pause" },
                        cx,
                        |this, _, cx| {
                            if let Some(s) = &mut this.agent.session {
                                s.paused = !s.paused;
                            }
                            cx.notify();
                        },
                    )),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(t.muted())
                    .child(if paused {
                        "Agent edits paused · reads available"
                    } else {
                        if self.agent.clients > 0 { "Connected · edits appear live" } else { "Agent disconnected · changes retained" }
                    }),
            )
            .child(self.control("follow-agent", "camera", if self.agent.follow { "Follow agent camera · On" } else { "Follow agent camera · Off" }, cx, |this,_,cx| {
                this.agent.follow = !this.agent.follow;
                if this.agent.follow && let Some(view) = this.agent.capture.as_ref().and_then(|c|c.views.first()).map(|v|v.1) {
                    this.set_agent_view(view,cx);
                }
                cx.notify();
            }).border_0().tooltip(hint("Follow the agent’s requested preset views. Manual navigation turns following off.",t)))
            .child(
                div()
                    .id("agent-feed")
                    .flex_1()
                    .min_h(px(0.0))
                    .overflow_y_scroll()
                    .children(self.agent.events.iter().enumerate().rev().map(|(i, e)| {
                        let entry = e.entry;
                        let expanded = self.agent.expanded == Some(i);
                        div()
                            .id(("agent-event", i))
                            .py(px(10.0))
                            .border_b_1()
                            .border_color(t.line())
                            .cursor_pointer()
                            .child(
                                row()
                                    .gap(px(8.0))
                                    .child(
                                        icon(if e.success { "check" } else { "close" }).text_color(
                                            if e.success { t.accent() } else { t.danger() },
                                        ),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(0.0))
                                            .text_ellipsis()
                                            .overflow_hidden()
                                            .font_weight(FontWeight::MEDIUM)
                                            .child(
                                                e.label
                                                    .trim_start_matches("voxelite_")
                                                    .replace('_', " "),
                                            ),
                                    )
                                    .child(mono(format!("r{}", e.revision)).text_color(t.muted()))
                                    .child(icon(if expanded {"chevron"} else {"chevron-right"}).size(px(14.0)).text_color(t.muted())),
                            )
                            .child(
                                mono(if e.changes.voxel_count() > 0 {
                                    format!(
                                        "+{}  −{}  {} painted",
                                        e.changes.added, e.changes.removed, e.changes.recolored
                                    )
                                } else if e.entry.is_some() {
                                    "Metadata updated".into()
                                } else if e.success {
                                    "Completed".into()
                                } else {
                                    "Rejected · no edit".into()
                                })
                                .mt(px(6.0))
                                .text_color(t.muted()),
                            )
                            .when(expanded, |d| {
                                d.child(
                                    div()
                                        .mt(px(8.0))
                                        .text_size(px(12.0))
                                        .text_color(t.muted())
                                        .child(e.error.clone().unwrap_or_else(|| {
                                            format!("{}\nBounds: {:?}", e.tool, e.changes.bounds)
                                        })),
                                )
                                .when_some(entry, |d, id| {
                                    d.child(
                                        self.control(
                                            ("revert-agent", i),
                                            "undo",
                                            "Revert this edit",
                                            cx,
                                            move |this, _, cx| {
                                                let r = this.editor.revert_step(id);
                                                this.report(r, "Agent edit reverted", cx);
                                            },
                                        )
                                        .mt(px(8.0)),
                                    )
                                })
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.agent.expanded = if expanded { None } else { Some(i) };
                                cx.notify();
                            }))
                    })),
            )
            .when_some(self.agent.capture.as_ref(), |d,capture| {
                d.child(row().justify_between().child(div().font_weight(FontWeight::MEDIUM).child("Captured views"))
                    .child(mono(format!("r{:03}",capture.revision)).text_color(t.muted())))
                .child(row().gap(px(4.0)).children(capture.views.iter().enumerate().map(|(i,(name,view,_))| {
                    let view=*view;
                    self.control(("capture-view",i),"camera",name.clone(),cx,move |this,_,cx| {
                        if let Some(c)=&mut this.agent.capture {c.selected=i;}
                        this.agent.follow=false; this.set_agent_view(view,cx); cx.notify();
                    }).flex_1().px(px(6.0)).justify_center().border_0()
                    .when(i == capture.selected,|d|d.bg(t.selected()))
                })))
                .when_some(capture.views.get(capture.selected),|d,(_,_,image)|d.child(
                    div().h(px(116.0)).flex_shrink_0().rounded(px(8.0)).border_1().border_color(t.line()).overflow_hidden()
                        .child(img(image.clone()).size_full().object_fit(ObjectFit::Contain))
                ))
            })
            .when(self.agent.capturing, |d|d.child(div().text_size(px(12.0)).text_color(t.muted()).child("Capturing views…")))
    }

    fn set_agent_view(&mut self, view: PresetView, cx: &mut Context<Self>) {
        let b = self.viewport.get();
        let aspect = f64::from(f32::from(b.size.width) / f32::from(b.size.height).max(1.0));
        self.editor.camera = OrbitCamera::fit_view(self.editor.display_snapshot(), aspect, view);
        self.camera_view = match view {
            PresetView::Front => Some(Op::Front),
            PresetView::Right => Some(Op::Right),
            PresetView::Top => Some(Op::Top),
            PresetView::Iso => Some(Op::Frame),
            _ => None,
        };
        self.camera_fitted = false;
        self.request_render(cx);
    }

    /// Render thumbnails from the exact content inspected by MCP, not newer live edits.
    fn capture_agent_views(
        &mut self,
        snapshot: Arc<voxelite_core::Snapshot>,
        names: Vec<String>,
        revision: u64,
        cx: &mut Context<Self>,
    ) {
        let views: Vec<_> = names
            .into_iter()
            .filter_map(|name| preset(&name).map(|view| (name, view)))
            .take(3)
            .collect();
        if views.is_empty() {
            return;
        }
        self.agent.capture_generation = self.agent.capture_generation.wrapping_add(1);
        let generation = self.agent.capture_generation;
        self.agent.capturing = true;
        let work = cx.background_executor().spawn(async move {
            views
                .into_iter()
                .map(|(name, view)| {
                    let mut frame = voxelite_render::render(
                        &snapshot,
                        &RenderOptions {
                            width: 300,
                            height: 116,
                            supersampling: 2,
                            background: [0; 4],
                            view: View::Preset {
                                direction: view,
                                perspective: false,
                            },
                            ..Default::default()
                        },
                    )?;
                    for pixel in frame.rgba.as_chunks_mut::<4>().0 {
                        pixel.swap(0, 2);
                    }
                    let buffer = image::RgbaImage::from_raw(frame.width, frame.height, frame.rgba)
                        .expect("thumbnail dimensions");
                    Ok((
                        name,
                        view,
                        Arc::new(RenderImage::new(vec![image::Frame::new(buffer)])),
                    ))
                })
                .collect::<Result<Vec<_>, voxelite_render::RenderError>>()
        });
        cx.spawn(async move |this, cx| {
            let result = work.await;
            let _ = this.update(cx, |this, cx| {
                if this.agent.capture_generation != generation {
                    return;
                }
                this.agent.capturing = false;
                match result {
                    Ok(views) => {
                        if let Some(old) = this.agent.capture.take() {
                            this.agent
                                .retired
                                .extend(old.views.into_iter().map(|v| v.2));
                        }
                        this.agent.capture = Some(CapturedViews {
                            revision,
                            views,
                            selected: 0,
                        });
                    }
                    Err(e) => {
                        this.status = format!("Could not capture views: {e}");
                        this.error = true;
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}
fn preset(name: &str) -> Option<PresetView> {
    Some(match name {
        "iso" => PresetView::Iso,
        "front" => PresetView::Front,
        "right" => PresetView::Right,
        "back" => PresetView::Back,
        "left" => PresetView::Left,
        "top" => PresetView::Top,
        _ => return None,
    })
}
