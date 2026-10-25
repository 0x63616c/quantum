//! Live agent connection and real MCP event feed.
use super::*;
use std::collections::VecDeque;
use voxelite_mcp::attached::{AttachedSession, SessionEvent};
pub(super) struct Agent {
    pub session: Option<AttachedSession>,
    pub events: VecDeque<SessionEvent>,
    pub open: bool,
    pub clients: usize,
    pub expanded: Option<usize>,
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
        for event in events {
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
            .child(self.control("copy-mcp-config","Copy MCP configuration",cx,|this,_,cx|{
                if let Some(s)=&this.agent.session{
                    let config=serde_json::json!({"mcpServers":{"voxelite":{"command":"voxelite-mcp","args":["--connect",s.path()]}}});
                    cx.write_to_clipboard(ClipboardItem::new_string(serde_json::to_string_pretty(&config).unwrap()));this.status="MCP configuration copied".into();cx.notify();
                }
            }))
            .child(div().text_size(px(12.0)).text_color(t.muted()).child("Requires voxelite-mcp on PATH. This socket accepts local processes running as you. Changes stay in memory until saved in the app.")))
            .into_any_element()
    }
    pub(super) fn agent_feed(&self, _compact: bool, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let paused = self.agent.session.as_ref().is_some_and(|s| s.paused);
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
            .min_h(px(150.0))
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
                        "Live MCP calls · shared document"
                    }),
            )
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
                                                e.tool
                                                    .trim_start_matches("voxelite_")
                                                    .replace('_', " "),
                                            ),
                                    )
                                    .child(mono(format!("r{}", e.revision)).text_color(t.muted())),
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
                                            "Revert this edit",
                                            cx,
                                            move |this, _, cx| {
                                                let r = this.editor.ensure_live_revert(id);
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
    }
}
