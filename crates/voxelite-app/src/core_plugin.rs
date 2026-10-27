//! Built-in core command declarations; execution remains in the app.

use voxelite_plugin_api::{
    API_VERSION, ApplicationMenuSection, CommandContribution, Manifest, Plugin, PluginId,
    Registrar, Value, Version,
};

/// Metadata for the built-in core commands.
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn manifest(&self) -> Manifest {
        Manifest {
            id: PluginId::new("dev.voxelite.core").unwrap(),
            name_key: "plugins.core.name".into(),
            description_key: "plugins.core.description".into(),
            version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            api_version: API_VERSION,
            builtin: true,
            default_enabled: true,
            can_disable: false,
        }
    }

    fn contribute(&self, registrar: &mut Registrar) {
        for (id, label, key, context, icon, menu, order, application_section) in [
            (
                "dev.voxelite.core.new",
                "New project",
                Some("secondary-n"),
                Some("Editor"),
                "file-plus",
                "File",
                0,
                None,
            ),
            (
                "dev.voxelite.core.open",
                "Open project…",
                Some("secondary-o"),
                Some("Editor"),
                "folder-open",
                "File",
                10,
                None,
            ),
            (
                "dev.voxelite.core.save",
                "Save",
                Some("secondary-s"),
                Some("Editor"),
                "save",
                "File",
                20,
                None,
            ),
            (
                "dev.voxelite.core.save_as",
                "Save as…",
                Some("secondary-shift-s"),
                Some("Editor"),
                "file-output",
                "File",
                30,
                None,
            ),
            #[cfg(target_family = "wasm")]
            (
                "dev.voxelite.core.export_glb",
                "Export GLB…",
                None,
                None,
                "download",
                "File",
                40,
                None,
            ),
            #[cfg(target_family = "wasm")]
            (
                "dev.voxelite.core.export_png",
                "Export PNG…",
                None,
                None,
                "download",
                "File",
                50,
                None,
            ),
            (
                "dev.voxelite.core.quit",
                "Quit Voxelite",
                Some("secondary-q"),
                None,
                "log-out",
                "File",
                60,
                Some(ApplicationMenuSection::Quit),
            ),
            (
                "dev.voxelite.core.undo",
                "Undo",
                Some("secondary-z"),
                Some("Editor"),
                "undo",
                "Edit",
                70,
                None,
            ),
            (
                "dev.voxelite.core.redo",
                "Redo",
                Some("secondary-shift-z"),
                Some("Editor"),
                "redo",
                "Edit",
                80,
                None,
            ),
            (
                "dev.voxelite.core.commands",
                "Search commands…",
                Some("secondary-k"),
                Some("Editor"),
                "search",
                "Edit",
                90,
                None,
            ),
            (
                "dev.voxelite.core.settings",
                "Settings…",
                Some("secondary-,"),
                None,
                "settings",
                "Edit",
                100,
                Some(ApplicationMenuSection::Settings),
            ),
            (
                "dev.voxelite.core.frame",
                "Frame model",
                Some("f"),
                Some("Editor"),
                "cube",
                "View",
                110,
                None,
            ),
            (
                "dev.voxelite.core.front",
                "Front view",
                None,
                None,
                "panel-top",
                "View",
                120,
                None,
            ),
            (
                "dev.voxelite.core.right",
                "Right view",
                None,
                None,
                "panel-left",
                "View",
                130,
                None,
            ),
            (
                "dev.voxelite.core.top",
                "Top view",
                None,
                None,
                "square",
                "View",
                140,
                None,
            ),
            (
                "dev.voxelite.core.grid",
                "Toggle grid",
                None,
                None,
                "grid",
                "View",
                150,
                None,
            ),
            (
                "dev.voxelite.core.theme",
                "Appearance…",
                None,
                None,
                "sun",
                "View",
                160,
                None,
            ),
            (
                "dev.voxelite.core.add_layer",
                "Add layer",
                None,
                None,
                "layer",
                "Layers",
                280,
                None,
            ),
            (
                "dev.voxelite.core.minimize",
                "Minimize",
                Some("secondary-m"),
                Some("Editor"),
                "square",
                "Window",
                340,
                None,
            ),
            (
                "dev.voxelite.core.zoom_window",
                "Zoom",
                None,
                None,
                "square",
                "Window",
                350,
                None,
            ),
            (
                "dev.voxelite.core.help",
                "Keyboard & mouse guide",
                None,
                None,
                "keyboard",
                "Help",
                360,
                None,
            ),
            (
                "dev.voxelite.core.about",
                "About Voxelite",
                None,
                None,
                "cube",
                "Help",
                370,
                Some(ApplicationMenuSection::About),
            ),
            (
                "dev.voxelite.core.check_updates",
                "Check for Updates…",
                None,
                None,
                "download",
                "Voxelite",
                380,
                Some(ApplicationMenuSection::About),
            ),
            (
                "dev.voxelite.core.hide",
                "Hide Voxelite",
                Some("secondary-h"),
                None,
                "eye-off",
                "Voxelite",
                390,
                Some(ApplicationMenuSection::Visibility),
            ),
            (
                "dev.voxelite.core.hide_others",
                "Hide Others",
                Some("secondary-alt-h"),
                None,
                "eye-off",
                "Voxelite",
                400,
                Some(ApplicationMenuSection::Visibility),
            ),
            (
                "dev.voxelite.core.show_all",
                "Show All",
                None,
                None,
                "eye",
                "Voxelite",
                410,
                Some(ApplicationMenuSection::Visibility),
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
                context: context.map(str::to_owned),
                icon: Some(icon.into()),
                menu: Some(menu.into()),
                order,
                application_section,
                agent_visible: false,
            });
        }
    }
}

/// Construct the app's statically linked plugin host in a fixed registration order.
/// All built-ins start enabled; the shell does not expose enablement preferences yet.
pub fn app_host() -> voxelite_plugin_host::Host {
    voxelite_plugin_host::Host::new(vec![
        Box::new(CorePlugin),
        Box::new(voxelite_plugin_tools::ToolsPlugin),
        Box::new(voxelite_plugin_mirror::MirrorPlugin),
        Box::new(voxelite_plugin_palette::PalettePlugin),
        Box::new(voxelite_plugin_comments::CommentsPlugin),
        Box::new(voxelite_plugin_agent::AgentPlugin),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conforms_to_plugin_contract() {
        voxelite_plugin_host::conformance::check(|| Box::new(CorePlugin));
    }

    #[test]
    fn app_host_enables_builtins_in_order_and_keeps_core_required() {
        use voxelite_plugin_host::{HostError, PluginState};

        let mut host = app_host();
        assert_eq!(
            host.plugins()
                .map(|(manifest, state)| {
                    assert!(manifest.builtin && manifest.default_enabled);
                    assert_eq!(state, &PluginState::Enabled);
                    assert_eq!(
                        manifest.can_disable,
                        manifest.id.as_str() != "dev.voxelite.core"
                    );
                    manifest.id.as_str()
                })
                .collect::<Vec<_>>(),
            [
                "dev.voxelite.core",
                "dev.voxelite.tools",
                "dev.voxelite.mirror",
                "dev.voxelite.palette",
                "dev.voxelite.comments",
                "dev.voxelite.agent"
            ]
        );
        assert!(host.conflicts().is_empty());
        assert!(matches!(
            host.disable("dev.voxelite.core"),
            Err(HostError::CannotDisable { .. })
        ));
    }
}
