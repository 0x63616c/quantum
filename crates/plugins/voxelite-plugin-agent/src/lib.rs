//! Built-in agent command declarations; execution remains in the app.

use voxelite_plugin_api::{
    API_VERSION, CommandContribution, Manifest, Plugin, PluginId, Registrar, Value, Version,
};

/// Metadata for the built-in agent commands.
pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn manifest(&self) -> Manifest {
        Manifest {
            id: PluginId::new("dev.voxelite.agent").unwrap(),
            name_key: "plugins.agent.name".into(),
            description_key: "plugins.agent.description".into(),
            version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            api_version: API_VERSION,
            builtin: true,
            default_enabled: true,
            can_disable: true,
        }
    }

    fn contribute(&self, registrar: &mut Registrar) {
        for (id, label, key, icon, menu, order) in [
            (
                "dev.voxelite.agent.agent",
                "Agent connection",
                None::<&str>,
                "spark",
                "Agent",
                320,
            ),
            (
                "dev.voxelite.agent.extensions",
                "Extensions",
                None::<&str>,
                "puzzle",
                "Extensions",
                330,
            ),
        ] {
            registrar.command(CommandContribution {
                id: id.into(),
                label_key: label.into(),
                args_schema: Value::Object(
                    [
                        ("type".into(), Value::String("object".into())),
                        ("properties".into(), Value::Object(Default::default())),
                        ("additionalProperties".into(), Value::Bool(false)),
                    ]
                    .into_iter()
                    .collect(),
                ),
                default_keybinding: key.map(str::to_owned),
                context: key.map(|_| "Editor".into()),
                icon: Some(icon.into()),
                menu: Some(menu.into()),
                order,
                application_section: None,
                // These open UI panels; neither is an existing MCP operation.
                agent_visible: false,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conforms_to_plugin_contract() {
        voxelite_plugin_host::conformance::check(|| Box::new(AgentPlugin));
    }
}
