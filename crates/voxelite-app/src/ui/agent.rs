//! Live agent connection and real MCP event feed.
use super::*;
use std::collections::VecDeque;
use voxelite_mcp::attached::{AttachedSession, SessionEvent};
pub(super) struct CapturedViews {
    pub revision: u64,
    pub views: Vec<(String, PresetView, Arc<RenderImage>)>,
    pub selected: usize,
}
#[derive(Default)]
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
                let dir = voxelite_app::local_settings::data_dir();
                std::fs::create_dir_all(&dir)?;
                dir.join(format!("session-{}.sock", std::process::id()))
            }
        };
        let session = AttachedSession::bind(
            &editor.attachment_root()?,
            editor.project(),
            &path,
            voxelite_plugins_builtin::mcp_host,
        )?;
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
        if self.save_flow.dialog_open() || self.save_flow.needs_confirmation() {
            return;
        }
        let Some(session) = &mut self.agent.session else {
            return;
        };
        let update = match self.editor.poll_attachment(session) {
            Ok(update) => update,
            Err(error) => {
                self.status = format!("Agent file root unavailable: {error}").into();
                self.error = true;
                cx.notify();
                return;
            }
        };
        let notify = update.clients != self.agent.clients || !update.events.is_empty();
        self.agent.clients = update.clients;
        for mut event in update.events {
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
        if update.changed {
            self.request_render(cx);
            self.status = format!(
                "Agent updated document · r{}",
                self.editor.engine.revision()
            )
            .into();
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
        div().absolute().top(px(metrics::AGENT_POPOVER_TOP)).right(px(space::XXL)).child(self.floating().w(px(metrics::AGENT_POPOVER_WIDTH)).p(px(space::PANEL)).flex().flex_col().gap(px(space::XL))
            .child(row().justify_between().child(section_title("Connect your agent")).child(mono("LOCAL MCP").text_color(t.muted())))
            .child(div().text_color(t.muted()).child("Use the MCP server as a stdio bridge to this window. Edits appear live and share your undo history."))
            .child(mono("voxelite-mcp --connect")).child(mono(path).text_size(px(type_size::BODY)).overflow_hidden())
            .child(self.control("copy-mcp-config","copy","Copy MCP configuration",cx,|this,_,cx|{
                if let Some(s)=&this.agent.session{
                    let command = std::env::current_exe().ok()
                        .filter(|exe| exe.parent().is_some_and(|dir| {
                            dir.ends_with("Contents/MacOS") && dir.parent().and_then(|contents| contents.parent())
                                .is_some_and(|bundle| bundle.extension().is_some_and(|ext| ext == "app"))
                        }))
                        .map(|exe| exe.with_file_name("voxelite-mcp"))
                        .filter(|path| path.is_file())
                        .unwrap_or_else(|| PathBuf::from("voxelite-mcp"));
                    let config=serde_json::json!({"mcpServers":{"voxelite":{"command":command,"args":["--connect",s.path()]}}});
                    cx.write_to_clipboard(ClipboardItem::new_string(serde_json::to_string_pretty(&config).unwrap()));this.status="MCP configuration copied".into();cx.notify();
                }
            }))
            .child(div().text_size(px(type_size::BODY)).text_color(t.muted()).child("Uses the bundled MCP server, or voxelite-mcp on PATH for source builds. This socket accepts local processes running as you. Changes stay in memory until saved in the app.")))
            .into_any_element()
    }
    pub(super) fn agent_feed(&self, compact: bool, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let paused = self.agent.session.as_ref().is_some_and(|s| s.paused);
        if self.agent.events.is_empty() {
            let connected = self.agent.clients > 0;
            let has_history = self.editor.engine.history().next().is_some();
            let invitation = Panel::section(t)
                .px(px(PANEL_GUTTER))
                .py(px(if compact { space::XL } else { space::PANEL }))
                .when(connected, |d| {
                    d.child(section_title("Ready for agent edits")).child(
                        div()
                            .mt(px(space::SM))
                            .text_color(t.muted())
                            .child("Edits will appear here as they happen."),
                    )
                })
                .when(!connected, |d| {
                    d.child(self.control(
                        "connect-agent",
                        "plug",
                        "Connect agent",
                        cx,
                        |this, _, cx| {
                            this.agent.open = true;
                            cx.notify();
                        },
                    ))
                });
            return div()
                .flex_shrink_0()
                .flex()
                .flex_col()
                .child(invitation)
                .when(has_history, |d| {
                    d.child(
                        self.history_panel(true, cx)
                            .h(px(metrics::AGENT_HISTORY_HEIGHT)),
                    )
                });
        }
        Panel::section(t)
            .flex_1()
            .min_h(px(metrics::AGENT_FEED_MIN_HEIGHT))
            .flex()
            .flex_col()
            .px(px(PANEL_GUTTER))
            .py(px(space::XL))
            .gap(px(space::LG))
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
            .child(self.control("follow-agent", "camera", if self.agent.follow { "Follow agent camera · On" } else { "Follow agent camera · Off" }, cx, |this,_,cx| {
                this.agent.follow = !this.agent.follow;
                if this.agent.follow && let Some(view) = this.agent.capture.as_ref().and_then(|c|c.views.first()).map(|v|v.1) {
                    this.set_agent_view(view,cx);
                }
                cx.notify();
            }).h(px(space::XXXL)).border_0().tooltip(hint("Follow the agent’s requested preset views. Manual navigation turns following off.",t)))
            .child(
                div()
                    .id("agent-feed")
                    .flex_1()
                    .min_h(px(metrics::AGENT_EVENT_MIN_HEIGHT))
                    .overflow_y_scroll()
                    .children(self.agent.events.iter().enumerate().rev().filter(|(_,e)| e.entry.is_some() || e.changes.voxel_count()>0 || !e.success).map(|(i, e)| {
                        let entry = e.entry;
                        let expanded = self.agent.expanded == Some(i);
                        voxelite_ui::SidebarItem::disclosure(("agent-event", i), e.tool.clone())
                            .py(px(space::INSET))
                            .border_b_1()
                            .border_color(t.line())
                            .cursor_pointer()
                            .child(
                                row()
                                    .gap(px(space::MD))
                                    .child(
                                        icon(if e.success { "check" } else { "close" }).text_color(
                                            if e.success { t.accent() } else { t.danger() },
                                        ),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(space::NONE))
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
                                    .child(icon(if expanded {"chevron"} else {"chevron-right"}).size(px(space::FOOTER)).text_color(t.muted())),
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
                                .mt(px(space::SM))
                                .text_color(t.muted()),
                            )
                            .when(expanded, |d| {
                                d.child(
                                    div()
                                        .mt(px(space::MD))
                                        .text_size(px(type_size::BODY))
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
                                        .mt(px(space::MD)),
                                    )
                                })
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.agent.expanded = if expanded { None } else { Some(i) };
                                cx.notify();
                            }))
                    })),
            )
            .when_some(self.agent.capture.as_ref(), |d, capture| {
                d.child(row().justify_between()
                    .child(div().font_weight(FontWeight::MEDIUM).child("Captured views"))
                    .child(mono(format!("r{:03}", capture.revision)).text_color(t.muted())))
                .child(row().gap(px(space::SM)).children(capture.views.iter().enumerate().map(|(i, (name, view, image))| {
                    let view = *view;
                    Button::tile(("capture-view", i), name.clone()).flex_1().min_w(px(space::NONE)).h(px(metrics::CAPTURE_HEIGHT))
                        .flex().flex_col().items_center().rounded(px(radius::CONTROL)).border_1()
                        .border_color(if i == capture.selected {t.accent()} else {t.line()})
                        .cursor_pointer().hover(move |d|d.bg(t.hover()))
                        .tooltip(hint(format!("{} view · captured at r{} · click to orient the live camera", name, capture.revision),t))
                        .child(img(image.clone()).w(px(metrics::APP_ICON)).h(px(metrics::CAPTURE_IMAGE_HEIGHT)).object_fit(ObjectFit::Contain))
                        .child(row().gap(px(space::XS)).text_size(px(type_size::BODY)).text_color(t.muted())
                            .child(icon("camera").size(px(space::LG)).text_color(t.muted()))
                            .child(name[..1].to_uppercase() + &name[1..]))
                        .on_click(cx.listener(move |this,_,_,cx| {
                            if let Some(c) = &mut this.agent.capture {c.selected=i;}
                            this.agent.follow=false;
                            this.set_agent_view(view,cx);
                            cx.notify();
                        }))
                })))
            })
            .when(self.agent.capturing, |d|d.child(div().text_size(px(type_size::BODY)).text_color(t.muted()).child("Capturing views…")))
    }

    fn set_agent_view(&mut self, view: PresetView, cx: &mut Context<Self>) {
        let b = self.viewport.bounds();
        let aspect = f64::from(f32::from(b.size.width) / f32::from(b.size.height).max(1.0));
        let destination = OrbitCamera::fit_view(self.editor.display_snapshot(), aspect, view);
        self.camera_view = match view {
            PresetView::Front => Some(Op::Front),
            PresetView::Right => Some(Op::Right),
            PresetView::Top => Some(Op::Top),
            PresetView::Iso => Some(Op::Frame),
            _ => None,
        };
        self.camera_fitted = false;
        self.animate_camera_to(destination, cx);
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
                            width: 180,
                            height: 160,
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
                        this.status = format!("Could not capture views: {e}").into();
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
