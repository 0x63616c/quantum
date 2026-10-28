//! Built-in discussions, anchor placement and pins. Core owns storage and undo.

use voxelite_plugin_api::*;
mod panel;
mod pins;

/// Built-in discussion owner.
pub const PLUGIN_ID: &str = "dev.voxelite.comments";
pub use voxelite_plugin_api::COMMENTS_PANEL_ID as PANEL_ID;
/// Registered placement tool.
pub const TOOL_ID: &str = "dev.voxelite.comments.tool";
/// Registered pins overlay.
pub const PINS_ID: &str = "dev.voxelite.comments.pins";
/// Existing shell command and shortcut identity.
pub const COMMAND_ID: &str = "dev.voxelite.comments.comment";

/// Standard, disableable discussion feature.
pub struct CommentsPlugin;

impl Plugin for CommentsPlugin {
    fn manifest(&self) -> Manifest {
        Manifest::builtin(
            PluginId::new(PLUGIN_ID).unwrap(),
            "plugins.comments.name",
            "plugins.comments.description",
            Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
        )
    }

    fn comments_panel(&self, id: &str) -> Option<&dyn CommentsPanel> {
        (id == PANEL_ID).then_some(self)
    }
    fn tool(&self, id: &str) -> Option<&dyn Tool> {
        (id == TOOL_ID).then_some(self)
    }
    fn overlay(&self, id: &str) -> Option<&dyn Overlay> {
        (id == PINS_ID).then_some(self)
    }
    fn contribute(&self, registrar: &mut Registrar) {
        registrar.panel(PanelContribution {
            id: PANEL_ID.into(),
            label_key: "Comments".into(),
            icon: Some("comment".into()),
        });
        registrar.tool(ToolContribution {
            id: TOOL_ID.into(),
            label_key: "Add comment pin".into(),
            icon: Some("comment".into()),
        });
        registrar.render_pass(RenderPassContribution {
            id: PINS_ID.into(),
            slot: RenderPassSlot::Overlay,
        });
        registrar.command(CommandContribution {
            id: COMMAND_ID.into(),
            label_key: "Add comment pin".into(),
            args_schema: Value::Object(
                [
                    ("type".into(), Value::String("object".into())),
                    ("properties".into(), Value::Object(Default::default())),
                    ("additionalProperties".into(), Value::Bool(false)),
                ]
                .into_iter()
                .collect(),
            ),
            default_keybinding: Some("c".into()),
            context: Some("Editor".into()),
            icon: Some("comment".into()),
            menu: Some("Tools".into()),
            order: 270,
            application_section: None,
            agent_visible: false,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    voxelite_plugin_host::plugin_conformance_test!(conforms_to_plugin_contract, || Box::new(
        CommentsPlugin
    ));

    #[test]
    fn declares_plugin_specific_capabilities() {
        let mut host = voxelite_plugin_host::Host::new(vec![Box::new(CommentsPlugin)]);
        assert!(host.has_comments_panel(PANEL_ID).unwrap());
        assert!(host.has_tool(TOOL_ID).unwrap());
        assert!(host.has_overlay(PINS_ID).unwrap());
        // Comments remain core content; no extension namespace is introduced.
        assert_eq!(host.registry().data_sections().count(), 0);
    }
}

#[cfg(test)]
mod manifest_regression {
    use super::*;

    #[test]
    fn manifest_matches_original_literal() {
        let expected = Manifest {
            id: PluginId::new("dev.voxelite.comments").unwrap(),
            name_key: "plugins.comments.name".into(),
            description_key: "plugins.comments.description".into(),
            version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            api_version: voxelite_plugin_api::API_VERSION,
            builtin: true,
            default_enabled: true,
            can_disable: true,
        };
        assert_eq!(CommentsPlugin.manifest(), expected);
    }
}
