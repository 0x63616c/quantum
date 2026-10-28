//! Minimal plugin template, not linked into the app.
//!
//! This example declares one command without implementing a feature or execution.
//! Export plugins use the API's `ExportCx`/`ExportResult` contracts and supply their
//! own encoders; the API has no encoding module or renderer dependency.

use voxelite_plugin_api::{
    API_VERSION, CommandContribution, Manifest, Plugin, PluginId, Registrar, Value, Version,
};

/// A stateless example using only the plugin API.
pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn manifest(&self) -> Manifest {
        Manifest {
            id: PluginId::new("dev.voxelite.example").unwrap(),
            name_key: "plugins.example.name".into(),
            description_key: "plugins.example.description".into(),
            version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            api_version: API_VERSION,
            builtin: false,
            default_enabled: false,
            can_disable: true,
        }
    }

    fn contribute(&self, registrar: &mut Registrar) {
        registrar.command(CommandContribution {
            id: "dev.voxelite.example.command".into(),
            label_key: "commands.example.label".into(),
            args_schema: Value::Object(
                [
                    ("type".into(), Value::String("object".into())),
                    ("properties".into(), Value::Object(Default::default())),
                    ("additionalProperties".into(), Value::Bool(false)),
                ]
                .into_iter()
                .collect(),
            ),
            default_keybinding: None,
            context: None,
            icon: None,
            menu: None,
            order: 0,
            application_section: None,
            agent_visible: false,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxelite_plugin_api::Contribution;

    voxelite_plugin_host::plugin_conformance_test!(conforms_to_plugin_contract, || Box::new(
        ExamplePlugin
    ));

    #[test]
    fn declares_one_example_command_without_builtin_enablement() {
        let plugin = ExamplePlugin;
        let manifest = plugin.manifest();
        assert_eq!(manifest.id.as_str(), "dev.voxelite.example");
        assert!(!manifest.builtin && !manifest.default_enabled && manifest.can_disable);

        let mut registrar = Registrar::default();
        plugin.contribute(&mut registrar);
        let contributions = registrar.finish().unwrap();
        assert_eq!(contributions.len(), 1);
        assert!(matches!(
            contributions.iter().next().unwrap(),
            Contribution::Command(command) if command.id == "dev.voxelite.example.command"
        ));
    }
}
