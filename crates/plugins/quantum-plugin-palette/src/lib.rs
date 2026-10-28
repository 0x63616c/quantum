//! Built-in palettes, library management and portable panel behavior.
mod import;
mod interchange;
mod library;
mod manager;
mod panel;
mod presets;

pub use import::{from_gpl, from_hex, from_pal};
pub use interchange::{FORMATS, export, format, import, to_pal};
pub use library::{parse_color, step_swatch};
pub use presets::Preset;

pub use quantum_plugin_api::{
    PALETTE_PANEL_ID as PANEL_ID, PALETTE_PRESET_SETTING as PRESET_SETTING,
};
/// JASC-PAL export identity in the host format registry.
pub const PAL: &str = "dev.quantum.palette.pal";

use quantum_plugin_api::{
    CommandContribution, Manifest, PalettePanel, PanelContribution, Plugin, Registrar,
    SettingContribution, SettingKind, SettingValue, Value,
};

/// Standard palette plugin; all state and effects cross portable capabilities.
pub struct PalettePlugin;

impl Plugin for PalettePlugin {
    fn manifest(&self) -> Manifest {
        Manifest::builtin(
            "dev.quantum.palette",
            "plugins.palette",
            env!("CARGO_PKG_VERSION"),
        )
    }

    fn contribute(&self, registrar: &mut Registrar) {
        registrar.format(quantum_plugin_api::FormatContribution {
            id: PAL.into(),
            label_key: "JASC-PAL Palette (.pal)".into(),
            extensions: vec!["pal".into()],
            content: quantum_plugin_api::FormatContent::Palette,
            import: false,
            export: true,
        });
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
                "dev.quantum.palette.show_palette",
                "Show Palette",
                None::<&str>,
                "palette",
                "Palette",
                290,
            ),
            (
                "dev.quantum.palette.palette",
                "Choose palette…",
                None::<&str>,
                "palette",
                "Palette",
                300,
            ),
            (
                "dev.quantum.palette.manage_palettes",
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

    fn exporter(&self, id: &str) -> Option<quantum_plugin_api::ExportCallback> {
        (id == PAL).then_some(|cx| {
            to_pal(cx.palette())
                .map(String::into_bytes)
                .map_err(|error| quantum_plugin_api::ExportError::Invalid(error.to_string()))
        })
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

    quantum_plugin_host::plugin_conformance_test!(conforms_to_plugin_contract, || Box::new(
        PalettePlugin
    ));
}

#[cfg(test)]
mod characterization;
#[cfg(test)]
mod manager_tests;
#[cfg(test)]
mod panel_tests;
