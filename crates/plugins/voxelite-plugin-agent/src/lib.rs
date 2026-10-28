//! Built-in agent command declarations; execution remains in the app.

use voxelite_plugin_api::{CommandContribution, Manifest, Plugin, Registrar, Value};

/// Metadata for the built-in agent commands.
pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn manifest(&self) -> Manifest {
        Manifest::builtin(
            "dev.voxelite.agent",
            "plugins.agent",
            env!("CARGO_PKG_VERSION"),
        )
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

    voxelite_plugin_host::plugin_conformance_test!(conforms_to_plugin_contract, || Box::new(
        AgentPlugin
    ));
}
