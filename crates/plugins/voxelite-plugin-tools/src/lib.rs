//! Built-in tools command declarations; execution remains in the app.

use voxelite_plugin_api::{
    API_VERSION, CommandContribution, Manifest, Plugin, PluginId, Registrar, Value, Version,
};

/// Metadata for the built-in tools commands.
pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn manifest(&self) -> Manifest {
        Manifest {
            id: PluginId::new("dev.voxelite.tools").unwrap(),
            name_key: "plugins.tools.name".into(),
            description_key: "plugins.tools.description".into(),
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
                "dev.voxelite.tools.place",
                "Place",
                Some("b"),
                "place",
                "Tools",
                170,
            ),
            (
                "dev.voxelite.tools.erase",
                "Erase",
                Some("e"),
                "erase",
                "Tools",
                180,
            ),
            (
                "dev.voxelite.tools.paint",
                "Paint",
                Some("p"),
                "paint",
                "Tools",
                190,
            ),
            (
                "dev.voxelite.tools.fill",
                "Fill",
                Some("g"),
                "fill",
                "Tools",
                200,
            ),
            (
                "dev.voxelite.tools.box",
                "Box",
                Some("u"),
                "box",
                "Tools",
                210,
            ),
            (
                "dev.voxelite.tools.picker",
                "Color picker",
                Some("i"),
                "picker",
                "Tools",
                220,
            ),
            (
                "dev.voxelite.tools.tool_picker",
                "Choose tool…",
                None,
                "wrench",
                "Tools",
                260,
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
        voxelite_plugin_host::conformance::check(|| Box::new(ToolsPlugin));
    }
}
