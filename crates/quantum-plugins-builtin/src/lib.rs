//! Composition roots for the app and standalone MCP server.
//!
//! This is infrastructure, not a plugin: concrete implementations stay outside
//! the portable API, host and MCP library.
use quantum_plugin_host::Host;

/// Append the editor's built-ins after its required core plugin, in shell order.
pub fn register_all(host: &mut Host) {
    host.register(vec![
        Box::new(quantum_plugin_tools::ToolsPlugin),
        Box::new(quantum_plugin_mirror::MirrorPlugin),
        Box::new(quantum_plugin_gizmo_grid::GizmoGridPlugin),
        Box::new(quantum_plugin_shadows_lighting::ShadowsLightingPlugin),
        Box::new(quantum_plugin_palette::PalettePlugin),
        Box::new(quantum_plugin_comments::CommentsPlugin),
        Box::new(quantum_plugin_agent::AgentPlugin),
        Box::new(quantum_plugin_exporters::ExportersPlugin),
    ]);
}

/// Construct the standalone MCP feature set in its existing format order.
/// Called on the request thread because plugin instances need not be `Send`.
pub fn mcp_host() -> Host {
    Host::new(vec![
        Box::new(quantum_plugin_exporters::ExportersPlugin),
        Box::new(quantum_plugin_palette::PalettePlugin),
        Box::new(quantum_plugin_comments::CommentsPlugin),
        Box::new(quantum_plugin_shadows_lighting::ShadowsLightingPlugin),
    ])
}

#[cfg(test)]
mod tests {
    use quantum_plugin_api::{API_VERSION, Plugin, Version};

    #[test]
    fn optional_builtin_manifests_keep_their_ids_keys_and_policy() {
        let cases: [(Box<dyn Plugin>, &str); 8] = [
            (Box::new(quantum_plugin_agent::AgentPlugin), "agent"),
            (
                Box::new(quantum_plugin_comments::CommentsPlugin),
                "comments",
            ),
            (
                Box::new(quantum_plugin_exporters::ExportersPlugin),
                "exporters",
            ),
            (
                Box::new(quantum_plugin_gizmo_grid::GizmoGridPlugin),
                "gizmo-grid",
            ),
            (Box::new(quantum_plugin_mirror::MirrorPlugin), "mirror"),
            (Box::new(quantum_plugin_palette::PalettePlugin), "palette"),
            (
                Box::new(quantum_plugin_shadows_lighting::ShadowsLightingPlugin),
                "shadows-lighting",
            ),
            (Box::new(quantum_plugin_tools::ToolsPlugin), "tools"),
        ];
        for (plugin, slug) in cases {
            let manifest = plugin.manifest();
            assert_eq!(manifest.id.as_str(), format!("dev.quantum.{slug}"));
            assert_eq!(manifest.name_key, format!("plugins.{slug}.name"));
            assert_eq!(
                manifest.description_key,
                format!("plugins.{slug}.description")
            );
            assert_eq!(
                manifest.version,
                Version::parse(env!("CARGO_PKG_VERSION")).unwrap()
            );
            assert_eq!(manifest.api_version, API_VERSION);
            assert!(
                manifest.builtin && manifest.default_enabled && manifest.can_disable,
                "{slug}"
            );
        }
    }
}
