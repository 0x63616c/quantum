//! Built-in comments command declarations; execution remains in the app.

use voxelite_plugin_api::{
    API_VERSION, CommandContribution, Manifest, Plugin, PluginId, Registrar, Value, Version,
};

/// Metadata for the built-in comments commands.
pub struct CommentsPlugin;

impl Plugin for CommentsPlugin {
    fn manifest(&self) -> Manifest {
        Manifest {
            id: PluginId::new("dev.voxelite.comments").unwrap(),
            name_key: "plugins.comments.name".into(),
            description_key: "plugins.comments.description".into(),
            version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            api_version: API_VERSION,
            builtin: true,
            default_enabled: true,
            can_disable: true,
        }
    }

    fn contribute(&self, registrar: &mut Registrar) {
        registrar.command(CommandContribution {
            id: "dev.voxelite.comments.comment".into(),
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

    #[test]
    fn conforms_to_plugin_contract() {
        voxelite_plugin_host::conformance::check(|| Box::new(CommentsPlugin));
    }
}
