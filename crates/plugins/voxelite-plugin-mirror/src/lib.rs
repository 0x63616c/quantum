//! Built-in mirror command declarations; execution remains in the app.

use voxelite_plugin_api::{
    API_VERSION, CommandContribution, Manifest, Plugin, PluginId, Registrar, Value, Version,
};

/// Metadata for the built-in mirror commands.
pub struct MirrorPlugin;

impl Plugin for MirrorPlugin {
    fn manifest(&self) -> Manifest {
        Manifest {
            id: PluginId::new("dev.voxelite.mirror").unwrap(),
            name_key: "plugins.mirror.name".into(),
            description_key: "plugins.mirror.description".into(),
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
                "dev.voxelite.mirror.mirror_x",
                "Mirror X",
                Some("shift-x"),
                "flip-horizontal-2",
                "Tools",
                230,
            ),
            (
                "dev.voxelite.mirror.mirror_y",
                "Mirror Y",
                Some("shift-y"),
                "flip-vertical-2",
                "Tools",
                240,
            ),
            (
                "dev.voxelite.mirror.mirror_z",
                "Mirror Z",
                Some("shift-z"),
                "flip-horizontal-2",
                "Tools",
                250,
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
        voxelite_plugin_host::conformance::check(|| Box::new(MirrorPlugin));
    }
}
