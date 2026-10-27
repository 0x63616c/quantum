//! Composition roots for the app and standalone MCP server.
//!
//! This is infrastructure, not a plugin: concrete implementations stay outside
//! the portable API, host and MCP library.
use voxelite_plugin_host::Host;

/// Append the editor's built-ins after its required core plugin, in shell order.
pub fn register_all(host: &mut Host) {
    host.register(vec![
        Box::new(voxelite_plugin_tools::ToolsPlugin),
        Box::new(voxelite_plugin_mirror::MirrorPlugin),
        Box::new(voxelite_plugin_gizmo_grid::GizmoGridPlugin),
        Box::new(voxelite_plugin_palette::PalettePlugin),
        Box::new(voxelite_plugin_comments::CommentsPlugin),
        Box::new(voxelite_plugin_agent::AgentPlugin),
        Box::new(voxelite_plugin_exporters::ExportersPlugin),
    ]);
}

/// Construct the standalone MCP feature set in its existing format order.
/// Called on the request thread because plugin instances need not be `Send`.
pub fn mcp_host() -> Host {
    Host::new(vec![
        Box::new(voxelite_plugin_exporters::ExportersPlugin),
        Box::new(voxelite_plugin_palette::PalettePlugin),
        Box::new(voxelite_plugin_comments::CommentsPlugin),
    ])
}
