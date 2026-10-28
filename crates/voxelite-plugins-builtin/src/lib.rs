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
        Box::new(voxelite_plugin_shadows_lighting::ShadowsLightingPlugin),
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
        Box::new(voxelite_plugin_shadows_lighting::ShadowsLightingPlugin),
    ])
}

#[cfg(test)]
mod tests {
    use voxelite_plugin_api::Plugin;

    #[test]
    fn manifests_match_original_literals() {
        // Original literals: id, name, description, version, API, built-in, enabled, disableable.
        #[rustfmt::skip]
        let cases = [
            (voxelite_plugin_agent::AgentPlugin.manifest(), "dev.voxelite.agent", "plugins.agent.name", "plugins.agent.description", env!("CARGO_PKG_VERSION"), "0.9.0", true, true, true),
            (voxelite_plugin_comments::CommentsPlugin.manifest(), "dev.voxelite.comments", "plugins.comments.name", "plugins.comments.description", env!("CARGO_PKG_VERSION"), "0.9.0", true, true, true),
            (voxelite_plugin_exporters::ExportersPlugin.manifest(), "dev.voxelite.exporters", "plugins.exporters.name", "plugins.exporters.description", env!("CARGO_PKG_VERSION"), "0.9.0", true, true, true),
            (voxelite_plugin_gizmo_grid::GizmoGridPlugin.manifest(), "dev.voxelite.gizmo-grid", "plugins.gizmo-grid.name", "plugins.gizmo-grid.description", env!("CARGO_PKG_VERSION"), "0.9.0", true, true, true),
            (voxelite_plugin_mirror::MirrorPlugin.manifest(), "dev.voxelite.mirror", "plugins.mirror.name", "plugins.mirror.description", env!("CARGO_PKG_VERSION"), "0.9.0", true, true, true),
            (voxelite_plugin_palette::PalettePlugin.manifest(), "dev.voxelite.palette", "plugins.palette.name", "plugins.palette.description", env!("CARGO_PKG_VERSION"), "0.9.0", true, true, true),
            (voxelite_plugin_shadows_lighting::ShadowsLightingPlugin.manifest(), "dev.voxelite.shadows-lighting", "plugins.shadows-lighting.name", "plugins.shadows-lighting.description", env!("CARGO_PKG_VERSION"), "0.9.0", true, true, true),
            (voxelite_plugin_tools::ToolsPlugin.manifest(), "dev.voxelite.tools", "plugins.tools.name", "plugins.tools.description", env!("CARGO_PKG_VERSION"), "0.9.0", true, true, true),
        ];
        for (manifest, id, name, description, version, api, builtin, enabled, disable) in cases {
            assert_eq!(
                manifest,
                voxelite_plugin_api::Manifest {
                    id: voxelite_plugin_api::PluginId::new(id).unwrap(),
                    name_key: name.into(),
                    description_key: description.into(),
                    version: voxelite_plugin_api::Version::parse(version).unwrap(),
                    api_version: voxelite_plugin_api::Version::parse(api).unwrap(),
                    builtin,
                    default_enabled: enabled,
                    can_disable: disable,
                },
                "{id}"
            );
        }
    }
}
