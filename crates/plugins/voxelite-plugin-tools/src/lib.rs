//! Built-in editing tools. All document mutations are proposed as core commands.

mod behavior;
use behavior::BuiltinTool;

use voxelite_plugin_api::{
    CommandContribution, Manifest, Plugin, Registrar, Tool, ToolContribution, Value,
};

/// Declarations and behavior for the standard editing tools.
pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn tool(&self, id: &str) -> Option<&dyn Tool> {
        BuiltinTool::ALL
            .iter()
            .find(|tool| tool.id() == id)
            .map(|tool| tool as &dyn Tool)
    }

    fn manifest(&self) -> Manifest {
        Manifest::builtin(
            "dev.voxelite.tools",
            "plugins.tools",
            env!("CARGO_PKG_VERSION"),
        )
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
            if id != "dev.voxelite.tools.tool_picker" {
                registrar.tool(ToolContribution {
                    id: format!("{id}.tool"),
                    label_key: label.into(),
                    icon: Some(icon.into()),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    voxelite_plugin_host::plugin_conformance_test!(conforms_to_plugin_contract, || Box::new(
        ToolsPlugin
    ));
}
