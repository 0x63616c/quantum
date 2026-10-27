//! Built-in palette command declarations; execution remains in the app.

use voxelite_plugin_api::{
    API_VERSION, CommandContribution, Manifest, Plugin, PluginId, Registrar, Value, Version,
};

/// Metadata for the built-in palette commands.
pub struct PalettePlugin;

impl Plugin for PalettePlugin {
    fn manifest(&self) -> Manifest {
        Manifest {
            id: PluginId::new("dev.voxelite.palette").unwrap(),
            name_key: "plugins.palette.name".into(),
            description_key: "plugins.palette.description".into(),
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
                "dev.voxelite.palette.show_palette",
                "Show Palette",
                None::<&str>,
                "palette",
                "Palette",
                290,
            ),
            (
                "dev.voxelite.palette.palette",
                "Choose palette…",
                None::<&str>,
                "palette",
                "Palette",
                300,
            ),
            (
                "dev.voxelite.palette.manage_palettes",
                "Manage palettes…",
                None::<&str>,
                "sliders-horizontal",
                "Palette",
                310,
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
        voxelite_plugin_host::conformance::check(|| Box::new(PalettePlugin));
    }
}
