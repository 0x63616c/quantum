//! Built-in centered mirroring of pending tool commands.

use voxelite_plugin_api::{
    Command, CommandContribution, CommandTransform, CommandTransformContribution, Manifest, Mirror,
    Plugin, PluginId, Registrar, SettingContribution, SettingKind, SettingValue, TransformCx,
    Value, Version,
};

/// Plugin-owned axis settings in X/Y/Z order. The host retains their transient values.
pub const AXIS_SETTINGS: [&str; 3] = [
    "dev.voxelite.mirror.x_enabled",
    "dev.voxelite.mirror.y_enabled",
    "dev.voxelite.mirror.z_enabled",
];

/// Stable command transform identity.
pub const TRANSFORM_ID: &str = "dev.voxelite.mirror.transform";

/// Mirror command metadata, axis defaults and pending command behavior.
pub struct MirrorPlugin;

impl Plugin for MirrorPlugin {
    fn manifest(&self) -> Manifest {
        Manifest::builtin(
            PluginId::new("dev.voxelite.mirror").unwrap(),
            "plugins.mirror.name",
            "plugins.mirror.description",
            Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
        )
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
        for (key, label) in AXIS_SETTINGS
            .into_iter()
            .zip(["Mirror X", "Mirror Y", "Mirror Z"])
        {
            registrar.setting(SettingContribution {
                key: key.into(),
                label_key: label.into(),
                kind: SettingKind::Bool,
                default: SettingValue::Bool(false),
            });
        }
        registrar.command_transform(CommandTransformContribution {
            id: TRANSFORM_ID.into(),
            order: 0,
        });
    }

    fn command_transform(&self, id: &str) -> Option<&dyn CommandTransform> {
        (id == TRANSFORM_ID).then_some(self)
    }
}

impl CommandTransform for MirrorPlugin {
    fn transform(&self, cx: &dyn TransformCx, mut commands: Vec<Command>) -> Vec<Command> {
        let axes = AXIS_SETTINGS.map(|key| cx.setting(key) == Some(&SettingValue::Bool(true)));
        let mirror = Mirror::centered(cx.bounds(), axes);
        for command in &mut commands {
            mirror_command(command, mirror);
        }
        commands
    }
}

fn mirror_command(command: &mut Command, settings: Mirror) {
    match command {
        Command::SetVoxels { mirror, .. }
        | Command::FloodFill { mirror, .. }
        | Command::Primitive { mirror, .. } => *mirror = settings,
        Command::Batch(commands) => {
            for command in commands {
                mirror_command(command, settings);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod manifest_regression {
    use super::*;

    #[test]
    fn manifest_matches_original_literal() {
        let expected = Manifest {
            id: PluginId::new("dev.voxelite.mirror").unwrap(),
            name_key: "plugins.mirror.name".into(),
            description_key: "plugins.mirror.description".into(),
            version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            api_version: voxelite_plugin_api::API_VERSION,
            builtin: true,
            default_enabled: true,
            can_disable: true,
        };
        assert_eq!(MirrorPlugin.manifest(), expected);
    }
}
