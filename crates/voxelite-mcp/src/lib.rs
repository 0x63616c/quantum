//! Voxelite's versioned, headless MCP adapter.
//!
//! The server owns a bounded local editing session. Content edits go through the
//! shared core engine with agent attribution. No display, GPU, or
//! application runtime is needed. See `docs/mcp.md` for the wire contract.
/// Local socket attachment to a running native editor.
#[cfg(unix)]
pub mod attached;
mod catalog;
mod comments;
mod edit;
mod error;
mod exports;
mod files;
mod images;
mod palettes;
mod session;
mod transport;

use rmcp::{ErrorData, RoleServer, ServerHandler, model::*, service::RequestContext};
use serde_json::Value;
use std::{
    borrow::Cow,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::sync::Semaphore;

/// Application tool contract version, independent of MCP and `.voxl` versions.
pub const API_VERSION: &str = "voxelite.v1";

/// One serialized document authority shared by all requests on this service.
///
/// Expensive work runs off the async executor. A bounded admission semaphore
/// prevents unbounded blocking jobs; each admitted edit executes atomically.
#[derive(Clone)]
pub struct VoxeliteServer {
    state: Arc<Mutex<session::Session>>,
    host_factory: fn() -> voxelite_plugin_host::Host,
    admission: Arc<Semaphore>,
    identity: (voxelite_core::Uuid, voxelite_core::Uuid),
}
impl VoxeliteServer {
    /// Create an empty session scoped to an existing project directory.
    /// The composition root supplies plugins on each request's worker thread.
    pub fn new(
        root: impl AsRef<Path>,
        host_factory: fn() -> voxelite_plugin_host::Host,
    ) -> std::io::Result<Self> {
        let root = root.as_ref().canonicalize()?;
        if !root.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "project root must be a directory",
            ));
        }
        // Validate/compile the complete catalog before accepting protocol input.
        let _ = catalog::CATALOG.len();
        let state = session::Session::new(root, host_factory);
        let identity = (state.id, state.epoch);
        Ok(Self {
            state: Arc::new(Mutex::new(state)),
            host_factory,
            identity,
            admission: Arc::new(Semaphore::new(16)),
        })
    }
    /// Serve bounded newline-delimited stdio using rmcp lifecycle negotiation.
    /// Malformed JSON receives a parse error; oversized lines receive an invalid
    /// request error. EOF shuts down cleanly after the client closes stdin.
    pub async fn serve_stdio(self) -> Result<(), Box<dyn std::error::Error>> {
        use rmcp::ServiceExt;
        let (read, write) = rmcp::transport::stdio_pipes();
        self.serve(transport::JsonLines::new(read, write))
            .await?
            .waiting()
            .await?;
        Ok(())
    }

    /// Execute a tool directly with the same validation and receipts as MCP.
    ///
    /// Unknown tool names are protocol errors. Domain/argument failures are
    /// `CallToolResult` values with `isError=true` and a structured error envelope.
    pub fn call(&self, name: &str, arguments: Value) -> Result<CallToolResult, ErrorData> {
        let spec = catalog::CATALOG
            .iter()
            .find(|s| s.tool.name == name)
            .ok_or_else(|| ErrorData::invalid_params("Unknown tool", None))?;
        let mut state = self.state.lock().map_err(|_| {
            ErrorData::internal_error("session lock poisoned; restart the server", None)
        })?;
        Ok(state.call(spec, arguments))
    }
    /// Return the complete, stable, name-sorted catalog with JSON Schema inputs.
    pub fn tools() -> Vec<Tool> {
        catalog::CATALOG
            .iter()
            .map(|spec| spec.tool.clone())
            .collect()
    }
}
impl ServerHandler for VoxeliteServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("voxelite-mcp",env!("CARGO_PKG_VERSION")))
            .with_instructions("Voxelite API voxelite.v1. Start with voxelite_capabilities_get. Use explicit model/layer IDs and returned epoch/revision on edits; keep operation_id unchanged on exact retries. Coordinates are integers, Y up; region maxima are exclusive. Edits are in memory until document_save. Render CPU PNG feedback to inspect your work.")
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
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        if request.is_some_and(|r| r.cursor.is_some()) {
            return Err(ErrorData::invalid_params(
                "tools/list has one page; omit cursor",
                None,
            ));
        }
        Ok(ListToolsResult {
            tools: exports::tools(&(self.host_factory)()),
            ..Default::default()
        })
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let spec = catalog::CATALOG
            .iter()
            .find(|s| s.tool.name == request.name)
            .ok_or_else(|| ErrorData::invalid_params("Unknown tool", None))?;
        let permit = match self.admission.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                return Ok(session::Session::error_for(
                    self.identity.0,
                    self.identity.1,
                    error::Failure::new(
                        "model_busy",
                        "request queue is full; retry after pending requests finish",
                    ),
                )
                .into());
            }
        };
        let server = self.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let mut state = server
                .state
                .lock()
                .map_err(|_| ErrorData::internal_error("session unavailable", None))?;
            // Cancellation must be rechecked after waiting for the document
            // authority. Once synchronous preparation starts, it runs atomically.
            if context.ct.is_cancelled() {
                return Ok(state.error(error::Failure::new(
                    "cancelled",
                    "request cancelled before execution",
                )));
            }
            Ok(state.call(spec, Value::Object(request.arguments.unwrap_or_default())))
        })
        .await
        .map_err(|_| ErrorData::internal_error("tool worker failed", None))?
        .map(Into::into)
    }
}
