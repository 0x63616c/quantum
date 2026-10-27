//! Built-in palettes, library management and portable panel behavior.
mod import;
mod library;
mod panel;
mod presets;

pub use import::{from_gpl, from_hex};
pub use library::{parse_color, step_swatch};
pub use presets::Preset;

/// Registered palette panel identity.
pub const PANEL_ID: &str = "dev.voxelite.palette.panel";
/// Transient project/preset choice, retained across disable/enable in one editor.
pub const PRESET_SETTING: &str = "dev.voxelite.palette.preset";

use voxelite_plugin_api::{
    API_VERSION, CommandContribution, Manifest, PalettePanel, PanelContribution, Plugin, PluginId,
    Registrar, SettingContribution, SettingKind, SettingValue, Value, Version,
};

/// Standard palette plugin; all state and effects cross portable capabilities.
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
        registrar.panel(PanelContribution {
            id: PANEL_ID.into(),
            label_key: "Palette".into(),
            icon: Some("palette".into()),
        });
        registrar.setting(SettingContribution {
            key: PRESET_SETTING.into(),
            label_key: "Palette".into(),
            kind: SettingKind::Choice(panel::choices()),
            default: SettingValue::Choice("project".into()),
        });
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

    fn palette_panel(&self, id: &str) -> Option<&dyn PalettePanel> {
        (id == PANEL_ID).then_some(self)
    }
}

#[cfg(test)]
mod interchange_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conforms_to_plugin_contract() {
        voxelite_plugin_host::conformance::check(|| Box::new(PalettePlugin));
    }
}

#[cfg(test)]
mod characterization;
#[cfg(test)]
mod panel_tests;
