//! Voxelite's versioned, headless MCP adapter.
//!
//! The server owns a local editing session. All content edits use the shared
//! core engine; no display, GPU, plugin host, or application runtime is needed.
use rmcp::{
    ServerHandler,
    model::{Implementation, ServerCapabilities, ServerConfig},
};

/// Version of the application tool contract, independent of MCP and `.voxl`.
pub const API_VERSION: &str = "voxelite.v1";

/// Headless stdio MCP service.
#[derive(Clone, Default)]
pub struct VoxeliteServer;

impl ServerHandler for VoxeliteServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("voxelite-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions("Voxelite API voxelite.v1. Coordinates are integer, Y up; region maxima are exclusive. Call voxelite_capabilities_get first.")
    }
}
