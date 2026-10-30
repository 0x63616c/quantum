//! MCP over a private Unix socket, with edits serialized by the editor's UI owner.
//!
//! The network worker only validates protocol framing and queues requests. `poll`
//! temporarily lends the live engine to the existing MCP session, so humans and
//! agents share one revision, history, locks and operation-receipt authority.
use crate::{catalog, error::Failure, session::Session, transport::JsonLines};
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt, model::*, service::RequestContext};
use serde_json::Value;
use std::{
    borrow::Cow,
    io,
    os::unix::{
        fs::{MetadataExt, PermissionsExt},
        net::UnixListener,
    },
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
};
use tokio::sync::oneshot;
use voxelite_core::{Changes, Engine, Snapshot, Uuid, compare};
use voxelite_format::Project;

struct Request {
    name: String,
    arguments: Value,
    reply: oneshot::Sender<Result<CallToolResult, ErrorData>>,
}
/// Actual completed MCP activity; failed calls never claim edits.
#[derive(Clone, Debug)]
pub struct SessionEvent {
    /// Public MCP tool name.
    pub tool: String,
    /// Committed history label, or the public tool name for a read.
    pub label: String,
    /// Whether domain execution succeeded.
    pub success: bool,
    /// Resulting shared revision.
    pub revision: u64,
    /// Committed entry, absent for reads, no-ops and failures.
    pub entry: Option<Uuid>,
    /// Actual live before/after differences.
    pub changes: Changes,
    /// Useful failure detail, when present.
    pub error: Option<String>,
    /// Immutable content inspected by a successful render; absent for other calls.
    pub inspection: Option<Arc<Snapshot>>,
    /// Requested preset views, validated by the renderer. Custom cameras omit presets.
    pub views: Vec<String>,
}
fn allowed(name: &str) -> bool {
    !matches!(
        name,
        "voxelite_document_create" | "voxelite_document_open" | "voxelite_document_save"
    )
}
#[derive(Clone)]
struct Bridge {
    sender: mpsc::SyncSender<Request>,
}
impl ServerHandler for Bridge {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build()).with_server_info(Implementation::new("voxelite-live",env!("CARGO_PKG_VERSION"))).with_instructions("Live native editor session. Start with voxelite_capabilities_get; use its open document and current epoch/revision. Human and agent edits share one history. Lifecycle and saving belong to the app. Changes are in memory until the user saves.")
    }
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Owned(vec![
            ProtocolVersion::V_2025_11_25,
            ProtocolVersion::V_2026_07_28,
        ])
    }
    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        if request.is_some_and(|r| r.cursor.is_some()) {
            return Err(ErrorData::invalid_params("Single page; omit cursor", None));
        }
        Ok(ListToolsResult {
            tools: catalog::CATALOG
                .iter()
                .filter(|s| allowed(&s.tool.name))
                .map(|s| s.tool.clone())
                .collect(),
            ..Default::default()
        })
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let (reply, receive) = oneshot::channel();
        self.sender
            .try_send(Request {
                name: request.name.to_string(),
                arguments: Value::Object(request.arguments.unwrap_or_default()),
                reply,
            })
            .map_err(|_| {
                ErrorData::internal_error(
                    "Editor request queue full or closed; retry after pending work",
                    None,
                )
            })?;
        tokio::select! {
            result=receive=>result.map_err(|_|ErrorData::internal_error("Editor session closed",None))?.map(Into::into),
            _=context.ct.cancelled()=>Err(ErrorData::internal_error("Request cancelled",None)),
        }
    }
}
/// Owner-thread half of a running MCP attachment. Dropping stops the socket worker.
pub struct AttachedSession {
    session: Session,
    requests: mpsc::Receiver<Request>,
    stop: Option<oneshot::Sender<()>>,
    worker: Option<std::thread::JoinHandle<()>>,
    path: PathBuf,
    clients: Arc<AtomicUsize>,
    render: Option<mpsc::Receiver<(Uuid, SessionEvent)>>,
    /// Human pause rejects new agent mutations; reads remain available.
    pub paused: bool,
}
impl AttachedSession {
    /// Bind a private socket. Existing paths are never removed or replaced.
    pub fn bind(root: &Path, project: Project, path: &Path) -> io::Result<Self> {
        let root = root.canonicalize()?;
        if !root.is_dir() {
            return Err(io::Error::other("Session root must be a directory"));
        }
        let mut session = Session::new(root);
        session
            .attach(project)
            .map_err(|e| io::Error::other(e.message.to_string()))?;
        let listener = UnixListener::bind(path)?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        listener.set_nonblocking(true)?;
        let metadata = std::fs::symlink_metadata(path)?;
        let identity = (metadata.dev(), metadata.ino());
        let path = path.to_path_buf();
        let worker_path = path.clone();
        let clients = Arc::new(AtomicUsize::new(0));
        let worker_clients = clients.clone();
        let (sender, requests) = mpsc::sync_channel(16);
        let (stop, stopped) = oneshot::channel();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()?;
        let worker = std::thread::Builder::new().name("voxelite-mcp".into()).spawn(move||{
            runtime.block_on(async move{
                let Ok(listener)=tokio::net::UnixListener::from_std(listener)else{return};let mut tasks=tokio::task::JoinSet::new();tokio::pin!(stopped);
                loop {tokio::select!{
                    _=&mut stopped=>break,
                    Some(_)=tasks.join_next(), if !tasks.is_empty()=>{},
                    accept=listener.accept()=>{
                        let Ok((stream,_))=accept else{break};
                        if worker_clients.load(Ordering::Relaxed)>=8{continue;}
                        let bridge=Bridge{sender:sender.clone()};let clients=worker_clients.clone();clients.fetch_add(1,Ordering::Relaxed);
                        tasks.spawn(async move{let (read,write)=stream.into_split();if let Ok(service)=bridge.serve(JsonLines::new(read,write)).await{let _=service.waiting().await;}clients.fetch_sub(1,Ordering::Relaxed);});
                    }
                }}
                tasks.abort_all();
            });
            if std::fs::symlink_metadata(&worker_path).is_ok_and(|m|(m.dev(),m.ino())==identity){let _=std::fs::remove_file(worker_path);}
        })?;
        Ok(Self {
            session,
            requests,
            stop: Some(stop),
            worker: Some(worker),
            path,
            clients,
            paused: false,
            render: None,
        })
    }
    /// Socket advertised in the app's connection panel.
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Number of currently connected transports (including initialization).
    pub fn clients(&self) -> usize {
        self.clients.load(Ordering::Relaxed)
    }
    /// Replace the attached document and epoch, invalidating previous receipts.
    pub fn replace(&mut self, project: Project) -> io::Result<()> {
        self.session
            .attach(project)
            .map_err(|e| io::Error::other(e.message.to_string()))
    }
    /// Process a bounded number of requests on the engine's owner thread.
    /// No request executes after its awaiting protocol caller has cancelled.
    pub fn poll(&mut self, engine: &mut Engine) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        if let Some(receiver) = &self.render {
            match receiver.try_recv() {
                Ok((epoch, event)) => {
                    if epoch == self.session.epoch {
                        events.push(event);
                    }
                    self.render = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => self.render = None,
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        let model = engine.document().id;
        if !self.session.documents.contains_key(&model) {
            return events;
        }
        // This swap is owner-thread exclusive. The worker never reads the shadow engine.
        std::mem::swap(
            engine,
            &mut self
                .session
                .documents
                .get_mut(&model)
                .expect("attached document")
                .engine,
        );
        for _ in 0..4 {
            let Ok(request) = self.requests.try_recv() else {
                break;
            };
            if request.reply.is_closed() {
                continue;
            }
            let before = self.session.documents[&model].engine.snapshot();
            let revision = self.session.documents[&model].engine.revision();
            if request.name == "voxelite_render" && self.render.is_none() {
                match self.session.prepare_render(request.arguments.clone()) {
                    Ok(task) => {
                        let snapshot = task.snapshot.clone();
                        let epoch = self.session.epoch;
                        let (send, receive) = mpsc::channel();
                        let views = if request.arguments.get("camera").is_some() {
                            vec![]
                        } else {
                            request
                                .arguments
                                .get("views")
                                .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
                                .unwrap_or_else(|| {
                                    vec![
                                        "iso".into(),
                                        "front".into(),
                                        "right".into(),
                                        "back".into(),
                                    ]
                                })
                        };
                        self.render = Some(receive);
                        // One bounded render worker per attachment. Mutations remain on the owner.
                        std::thread::spawn(move || {
                            if request.reply.is_closed() {
                                return;
                            }
                            let result = task.run();
                            let success = result.is_error != Some(true);
                            let error = result
                                .structured_content
                                .as_ref()
                                .and_then(|v| v.pointer("/error/message"))
                                .and_then(Value::as_str)
                                .map(str::to_string);
                            let event = SessionEvent {
                                tool: request.name,
                                label: "Inspect model views".into(),
                                success,
                                revision,
                                entry: None,
                                changes: Changes::default(),
                                error,
                                inspection: success.then_some(snapshot),
                                views,
                            };
                            let _ = send.send((epoch, event));
                            let _ = request.reply.send(Ok(result));
                        });
                        continue;
                    }
                    Err(error) => {
                        let _ = request.reply.send(Ok(self.session.error(error)));
                        continue;
                    }
                }
            }
            let spec = catalog::CATALOG
                .iter()
                .find(|s| s.tool.name == request.name);
            let result = if request.name == "voxelite_render" {
                Ok(self.session.error(Failure::new(
                    "render_busy",
                    "A render is in progress; retry after it completes",
                )))
            } else if !allowed(&request.name) {
                Ok(self.session.error(Failure::new(
                    "app_lifecycle_required",
                    "Open, create and save through the app while attached",
                )))
            } else if self.paused && spec.is_some_and(|s| s.mutation) {
                Ok(self.session.error(Failure::new(
                    "agent_paused",
                    "Human paused agent edits; retry after resuming",
                )))
            } else if let Some(spec) = spec {
                Ok(self.session.call(spec, request.arguments))
            } else {
                Err(ErrorData::invalid_params("Unknown tool", None))
            };
            let engine = &self.session.documents[&model].engine;
            let success = result.as_ref().is_ok_and(|r| r.is_error != Some(true));
            let error = result
                .as_ref()
                .ok()
                .and_then(|r| r.structured_content.as_ref())
                .and_then(|v| v.pointer("/error/message"))
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| result.as_ref().err().map(|e| e.message.to_string()));
            events.push(SessionEvent {
                label: if request.name == "voxelite_history_undo" {
                    "Undo edit".into()
                } else if request.name == "voxelite_history_redo" {
                    "Redo edit".into()
                } else if engine.revision() != revision {
                    engine
                        .history()
                        .next_back()
                        .map(|e| e.label.clone())
                        .unwrap_or_else(|| request.name.clone())
                } else {
                    request.name.clone()
                },
                tool: request.name.clone(),
                success,
                revision: engine.revision(),
                entry: if engine.revision() != revision
                    && !matches!(
                        request.name.as_str(),
                        "voxelite_history_undo" | "voxelite_history_redo"
                    ) {
                    result
                        .as_ref()
                        .ok()
                        .and_then(|r| r.structured_content.as_ref())
                        .and_then(|v| v.pointer("/data/undo_entry_id"))
                        .and_then(Value::as_str)
                        .and_then(|s| Uuid::parse_str(s).ok())
                } else {
                    None
                },
                changes: if engine.revision() != revision {
                    compare(&before.content, &engine.document().content)
                } else {
                    Changes::default()
                },
                error,
                inspection: None,
                views: vec![],
            });
            let _ = request.reply.send(result);
        }
        std::mem::swap(
            engine,
            &mut self
                .session
                .documents
                .get_mut(&model)
                .expect("attached document")
                .engine,
        );
        events
    }
}
impl Drop for AttachedSession {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
/// Forward stdio MCP frames to an already running app without another authority.
pub async fn proxy_stdio(path: &Path) -> io::Result<()> {
    use tokio::io::{AsyncWriteExt, copy};
    let stream = tokio::net::UnixStream::connect(path).await?;
    let (mut read, mut write) = stream.into_split();
    let input = async {
        copy(&mut tokio::io::stdin(), &mut write).await?;
        write.shutdown().await
    };
    let output = async { copy(&mut read, &mut tokio::io::stdout()).await.map(|_| ()) };
    tokio::try_join!(input, output)?;
    Ok(())
}
