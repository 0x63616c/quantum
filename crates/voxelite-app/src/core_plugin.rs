//! Built-in core command declarations; execution remains in the app.

use voxelite_plugin_api::{
    ApplicationMenuSection, CommandContribution, Manifest, Plugin, Registrar, Value,
};

/// Metadata for the built-in core commands.
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn manifest(&self) -> Manifest {
        Manifest {
            can_disable: false,
            ..Manifest::builtin(
                "dev.voxelite.core",
                "plugins.core",
                env!("CARGO_PKG_VERSION"),
            )
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
                "dev.voxelite.core.open_sample",
                "Open Sample…",
                Some("secondary-shift-o"),
                Some("Editor"),
                "layer",
                "File",
                11,
                None,
            ),
            (
                "dev.voxelite.core.open_recent",
                "Open Recent",
                None,
                None,
                "folder-open",
                "File",
                12,
                None,
            ),
            (
                "dev.voxelite.core.clear_recents",
                "Clear Recent Projects",
                None,
                None,
                "trash-2",
                "File",
                14,
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
            (
                "dev.voxelite.core.quit",
                "Quit Quantum",
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
                "dev.voxelite.core.debug_overlay",
                "Toggle debug overlay",
                Some("secondary-shift-d"),
                Some("Editor"),
                "settings",
                "View",
                162,
                None,
            ),
            (
                "dev.voxelite.core.fullscreen",
                "Toggle Full Screen",
                Some("ctrl-cmd-f"),
                None,
                "fit",
                "View",
                165,
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
                "dev.voxelite.core.welcome",
                "Welcome",
                Some("secondary-shift-1"),
                None,
                "cube",
                "Window",
                335,
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
                "About Quantum",
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
                "Quantum",
                380,
                Some(ApplicationMenuSection::About),
            ),
            (
                "dev.voxelite.core.hide",
                "Hide Quantum",
                Some("secondary-h"),
                None,
                "eye-off",
                "Quantum",
                390,
                Some(ApplicationMenuSection::Visibility),
            ),
            (
                "dev.voxelite.core.hide_others",
                "Hide Others",
                Some("secondary-alt-h"),
                None,
                "eye-off",
                "Quantum",
                400,
                Some(ApplicationMenuSection::Visibility),
            ),
            (
                "dev.voxelite.core.show_all",
                "Show All",
                None,
                None,
                "eye",
                "Quantum",
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
    let mut host = voxelite_plugin_host::Host::new(vec![Box::new(CorePlugin)]);
    voxelite_plugins_builtin::register_all(&mut host);
    host
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_manifest_policy_and_keys() {
        let manifest = CorePlugin.manifest();
        assert!(!manifest.can_disable);
        assert_eq!(manifest.name_key, "plugins.core.name");
        assert_eq!(manifest.description_key, "plugins.core.description");
    }

    voxelite_plugin_host::plugin_conformance_test!(conforms_to_plugin_contract, || Box::new(
        CorePlugin
    ));

    #[test]
    fn welcome_commands_are_declared_without_gpui() {
        let host = app_host();
        for (id, menu, key) in [
            ("dev.voxelite.core.open_recent", "File", None),
            ("dev.voxelite.core.clear_recents", "File", None),
            (
                "dev.voxelite.core.welcome",
                "Window",
                Some("secondary-shift-1"),
            ),
        ] {
            let (_, command) = host
                .registry()
                .commands()
                .find(|(_, c)| c.id == id)
                .unwrap();
            assert_eq!(command.menu.as_deref(), Some(menu));
            assert_eq!(command.default_keybinding.as_deref(), key);
        }
    }

    #[test]
    fn grid_and_camera_command_ids_and_shortcuts_are_stable() {
        let host = app_host();
        for (id, key, order) in [
            ("dev.voxelite.core.frame", Some("f"), 110),
            ("dev.voxelite.core.front", None, 120),
            ("dev.voxelite.core.right", None, 130),
            ("dev.voxelite.core.top", None, 140),
            ("dev.voxelite.core.grid", None, 150),
        ] {
            let command = host
                .registry()
                .commands()
                .find(|(_, c)| c.id == id)
                .unwrap()
                .1;
            assert_eq!(command.default_keybinding.as_deref(), key);
            assert_eq!(command.context.as_deref(), key.map(|_| "Editor"));
            assert_eq!(command.order, order);
            assert_eq!(command.menu.as_deref(), Some("View"));
        }
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
                "dev.voxelite.gizmo-grid",
                "dev.voxelite.shadows-lighting",
                "dev.voxelite.palette",
                "dev.voxelite.comments",
                "dev.voxelite.agent",
                "dev.voxelite.exporters"
            ]
        );
        assert!(host.conflicts().is_empty());
        assert!(matches!(
            host.disable("dev.voxelite.core"),
            Err(HostError::CannotDisable { .. })
        ));
    }
}
