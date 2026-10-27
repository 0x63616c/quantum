use std::{cell::Cell, collections::BTreeSet};

use super::*;

struct TestPlugin {
    manifest: Manifest,
    contributions: Vec<Contribution>,
    registrations: Cell<usize>,
}

impl Plugin for TestPlugin {
    fn manifest(&self) -> Manifest {
        self.manifest.clone()
    }

    fn register(&self, r: &mut Registrar) {
        self.registrations.set(self.registrations.get() + 1);
        for contribution in self.contributions.clone() {
            match contribution {
                Contribution::Command(spec) => r.command(spec),
                Contribution::Setting(spec) => r.setting(spec),
                Contribution::Format(spec) => r.format(spec),
            }
        }
    }
}

fn plugin(id: &str, contributions: Vec<Contribution>) -> TestPlugin {
    TestPlugin {
        manifest: Manifest {
            id: PluginId::new(id).unwrap(),
            name: "Test plugin".into(),
            description: "A built-in test plugin".into(),
            version: "0.2.0".into(),
            builtin: true,
            default_enabled: true,
            can_disable: true,
        },
        contributions,
        registrations: Cell::new(0),
    }
}

fn command(id: &str, shortcut: Option<&str>, context: Option<&str>) -> CommandSpec {
    CommandSpec {
        id: id.into(),
        label: "Paint".into(),
        shortcut: shortcut.map(str::to_owned),
        context: context.map(str::to_owned),
        icon: Some("paint-brush".into()),
        menu: Some("Tools".into()),
        agent_visible: true,
    }
}

fn setting(key: &str) -> SettingSpec {
    SettingSpec {
        key: key.into(),
        label: "Enabled".into(),
        kind: SettingKind::Bool,
        default: SettingValue::Bool(true),
    }
}

fn format(id: &str) -> FormatSpec {
    FormatSpec {
        id: id.into(),
        label: "Image".into(),
        extensions: vec!["png".into(), "apng".into()],
        import: false,
        export: true,
    }
}

fn declarations(id: &str) -> [Contribution; 3] {
    [
        Contribution::Command(command(id, None, None)),
        Contribution::Setting(setting(id)),
        Contribution::Format(format(id)),
    ]
}

fn enabled(ids: &[&str]) -> BTreeSet<PluginId> {
    ids.iter().map(|id| PluginId::new(*id).unwrap()).collect()
}

fn build(plugins: &[&dyn Plugin]) -> Result<Registry, RegistryError> {
    let enabled = plugins.iter().map(|plugin| plugin.manifest().id).collect();
    Registry::new(plugins, &enabled)
}

#[test]
fn plugin_ids_accept_lowercase_namespaces_without_changing_them() {
    for id in [
        "voxelite.tools",
        "a.b",
        "org.voxelite.tools",
        "v2.paint_tool.box-fill",
    ] {
        let plugin_id = PluginId::new(id).unwrap();
        assert_eq!(plugin_id.as_str(), id);
        assert_eq!(plugin_id.as_ref(), id);
        assert_eq!(plugin_id.to_string(), id);
        assert_eq!(id.parse::<PluginId>().unwrap(), plugin_id);
        assert_eq!(PluginId::try_from(id.to_owned()).unwrap(), plugin_id);
    }
}

#[test]
fn plugin_ids_reject_malformed_namespaces_on_every_construction_path() {
    for id in [
        "",
        "tools",
        ".tools",
        "voxelite.",
        "voxelite..tools",
        "Voxelite.tools",
        "voxelite.Tools",
        "voxelite.2tools",
        "2voxelite.tools",
        "voxelite._tools",
        "voxelite.-tools",
        " voxelite.tools",
        "voxelite.tools ",
        "voxelite.tool box",
        "voxelite/tools",
        "voxelite:tools",
        "vöxelite.tools",
        "voxelite.工具",
        "voxelite.tools\n",
        "voxelite.\0tools",
    ] {
        let expected = InvalidPluginId { id: id.into() };
        assert_eq!(PluginId::new(id).unwrap_err(), expected);
        assert_eq!(id.parse::<PluginId>().unwrap_err(), expected);
        assert_eq!(PluginId::try_from(id.to_owned()).unwrap_err(), expected);
        let message = expected.to_string();
        assert!(message.contains(&format!("{id:?}")));
        assert!(message.contains("dot-separated"));
    }
}

#[test]
fn plugin_ids_have_value_equality_and_ordering() {
    let ids = enabled(&["voxelite.z", "voxelite.a", "voxelite.z"]);
    assert_eq!(ids.len(), 2);
    assert_eq!(
        ids.iter().map(PluginId::as_str).collect::<Vec<_>>(),
        ["voxelite.a", "voxelite.z"]
    );
}

#[test]
fn empty_registry_and_unknown_enabled_ids_expose_nothing() {
    for enabled in [BTreeSet::new(), enabled(&["voxelite.missing"])] {
        let registry = Registry::new(&[], &enabled).unwrap();
        assert_eq!(registry.plugins().len(), 0);
        assert_eq!(registry.contributions().len(), 0);
        assert_eq!(registry.commands().count(), 0);
        assert_eq!(registry.settings().count(), 0);
        assert_eq!(registry.formats().count(), 0);
        assert!(registry.command("missing.command").is_none());
        assert!(registry.setting("missing.setting").is_none());
        assert!(registry.format("missing.format").is_none());
    }
}

#[test]
fn manifests_and_contribution_metadata_are_preserved() {
    let command = command("voxelite.tools.paint", Some("p"), Some("Editor"));
    let mut setting = setting("voxelite.tools.mode");
    setting.kind = SettingKind::Choice(vec!["solid".into(), "hollow".into()]);
    setting.default = SettingValue::Choice("hollow".into());
    let format = format("voxelite.export.png");
    let plugin = plugin(
        "voxelite.tools",
        vec![
            Contribution::Command(command.clone()),
            Contribution::Setting(setting.clone()),
            Contribution::Format(format.clone()),
        ],
    );
    let registry = build(&[&plugin]).unwrap();
    assert_eq!(registry.plugins().collect::<Vec<_>>(), [&plugin.manifest]);
    assert_eq!(registry.command(&command.id), Some(&command));
    assert_eq!(registry.setting(&setting.key), Some(&setting));
    assert_eq!(registry.format(&format.id), Some(&format));
    assert!(registry.command(&setting.key).is_none());
    assert!(registry.setting(&format.id).is_none());
    assert!(registry.format(&command.id).is_none());
    for entry in registry.contributions() {
        assert_eq!(entry.plugin_id, plugin.manifest.id);
    }
    assert_eq!(plugin.registrations.get(), 1);
}

#[test]
fn iteration_preserves_plugin_and_interleaved_registration_order() {
    let z = plugin(
        "voxelite.z",
        vec![
            Contribution::Format(format("z.zformat")),
            Contribution::Command(command("z.zcommand", None, None)),
            Contribution::Setting(setting("z.zsetting")),
            Contribution::Command(command("z.acommand", None, None)),
            Contribution::Format(format("z.aformat")),
            Contribution::Setting(setting("z.asetting")),
        ],
    );
    let a = plugin(
        "voxelite.a",
        vec![
            Contribution::Setting(setting("a.setting")),
            Contribution::Format(format("a.format")),
            Contribution::Command(command("a.command", None, None)),
        ],
    );
    let disabled = plugin(
        "voxelite.disabled",
        vec![Contribution::Setting(setting("disabled.setting"))],
    );
    let registry = Registry::new(
        &[&z, &disabled, &a],
        &enabled(&["voxelite.a", "voxelite.z"]),
    )
    .unwrap();
    assert_eq!(
        registry
            .plugins()
            .map(|m| m.id.as_str())
            .collect::<Vec<_>>(),
        ["voxelite.z", "voxelite.disabled", "voxelite.a"]
    );
    assert_eq!(
        registry
            .contributions()
            .map(|entry| entry.contribution.id())
            .collect::<Vec<_>>(),
        [
            "z.zformat",
            "z.zcommand",
            "z.zsetting",
            "z.acommand",
            "z.aformat",
            "z.asetting",
            "a.setting",
            "a.format",
            "a.command",
        ]
    );
    assert_eq!(
        registry
            .commands()
            .map(|spec| spec.id.as_str())
            .collect::<Vec<_>>(),
        ["z.zcommand", "z.acommand", "a.command"]
    );
    assert_eq!(
        registry
            .settings()
            .map(|spec| spec.key.as_str())
            .collect::<Vec<_>>(),
        ["z.zsetting", "z.asetting", "a.setting"]
    );
    assert_eq!(
        registry
            .formats()
            .map(|spec| spec.id.as_str())
            .collect::<Vec<_>>(),
        ["z.zformat", "z.aformat", "a.format"]
    );
    assert_eq!(
        registry
            .contributions()
            .map(|entry| entry.plugin_id.as_str())
            .collect::<Vec<_>>(),
        [
            "voxelite.z",
            "voxelite.z",
            "voxelite.z",
            "voxelite.z",
            "voxelite.z",
            "voxelite.z",
            "voxelite.a",
            "voxelite.a",
            "voxelite.a",
        ]
    );
}

#[test]
fn disabled_plugins_keep_manifests_but_never_register_or_expose_contributions() {
    let plugin = plugin(
        "voxelite.tools",
        vec![
            Contribution::Command(command("tools.paint", Some("p"), None)),
            Contribution::Setting(setting("tools.enabled")),
            Contribution::Format(format("tools.png")),
        ],
    );
    let registry = Registry::new(&[&plugin], &BTreeSet::new()).unwrap();
    assert_eq!(registry.plugins().collect::<Vec<_>>(), [&plugin.manifest]);
    assert_eq!(registry.contributions().len(), 0);
    assert_eq!(registry.commands().count(), 0);
    assert_eq!(registry.settings().count(), 0);
    assert_eq!(registry.formats().count(), 0);
    assert!(registry.command("tools.paint").is_none());
    assert!(registry.setting("tools.enabled").is_none());
    assert!(registry.format("tools.png").is_none());
    assert_eq!(plugin.registrations.get(), 0);
}

#[test]
fn enabled_set_is_authoritative_over_manifest_policy_flags() {
    let mut plugin = plugin(
        "voxelite.tools",
        vec![Contribution::Setting(setting("tools.enabled"))],
    );
    plugin.manifest.default_enabled = false;
    plugin.manifest.builtin = false;
    plugin.manifest.can_disable = false;
    let registry = build(&[&plugin]).unwrap();
    assert!(registry.setting("tools.enabled").is_some());
    assert_eq!(registry.plugins().next(), Some(&plugin.manifest));
    plugin.manifest.default_enabled = true;
    let registry = Registry::new(&[&plugin], &BTreeSet::new()).unwrap();
    assert!(registry.setting("tools.enabled").is_none());
}

#[test]
fn duplicate_plugin_ids_fail_even_when_disabled_and_before_registration() {
    let first = plugin("voxelite.tools", vec![]);
    let second = plugin("voxelite.tools", vec![]);
    for enabled in [BTreeSet::new(), enabled(&["voxelite.tools"])] {
        let error = Registry::new(&[&first, &second], &enabled).unwrap_err();
        assert_eq!(
            error,
            RegistryError::DuplicatePlugin {
                id: first.manifest.id.clone()
            }
        );
        assert!(error.to_string().contains("voxelite.tools"));
    }
    assert_eq!(first.registrations.get(), 0);
    assert_eq!(second.registrations.get(), 0);
}

#[test]
fn duplicate_contribution_ids_fail_across_every_pair_of_kinds_and_plugins() {
    for first in declarations("shared.entry") {
        for second in declarations("shared.entry") {
            let a = plugin("voxelite.a", vec![first.clone()]);
            let b = plugin("voxelite.b", vec![second]);
            let error = build(&[&a, &b]).unwrap_err();
            assert_eq!(
                error,
                RegistryError::DuplicateContribution {
                    id: "shared.entry".into(),
                    first_plugin: a.manifest.id.clone(),
                    second_plugin: b.manifest.id.clone(),
                }
            );
            let message = error.to_string();
            for text in ["shared.entry", "voxelite.a", "voxelite.b"] {
                assert!(message.contains(text), "{message}");
            }
        }
    }
}

#[test]
fn duplicate_contribution_ids_fail_within_one_plugin() {
    for first in declarations("tools.entry") {
        for second in declarations("tools.entry") {
            let plugin = plugin("voxelite.tools", vec![first.clone(), second]);
            assert_eq!(
                build(&[&plugin]).unwrap_err(),
                RegistryError::DuplicateContribution {
                    id: "tools.entry".into(),
                    first_plugin: plugin.manifest.id.clone(),
                    second_plugin: plugin.manifest.id.clone(),
                }
            );
        }
    }
}

#[test]
fn disabled_plugins_do_not_reserve_contribution_ids_or_shortcuts() {
    let a = plugin(
        "voxelite.a",
        vec![
            Contribution::Command(command("shared.paint", Some("p"), Some("Editor"))),
            Contribution::Setting(setting("shared.setting")),
            Contribution::Format(format("shared.format")),
        ],
    );
    let mut b = plugin("voxelite.b", a.contributions.clone());
    b.contributions.push(Contribution::Command(command(
        "b.paint",
        Some("p"),
        Some("Editor"),
    )));
    for plugins in [[&a as &dyn Plugin, &b], [&b as &dyn Plugin, &a]] {
        let registry = Registry::new(&plugins, &enabled(&["voxelite.a"])).unwrap();
        assert_eq!(registry.contributions().len(), 3);
        assert_eq!(registry.commands().count(), 1);
        assert_eq!(registry.settings().count(), 1);
        assert_eq!(registry.formats().count(), 1);
    }
    assert_eq!(b.registrations.get(), 0);
}

#[test]
fn duplicate_shortcuts_fail_in_global_and_named_contexts_across_plugins() {
    for context in [None, Some("Editor")] {
        let a = plugin(
            "voxelite.a",
            vec![Contribution::Command(command(
                "a.paint",
                Some("p"),
                context,
            ))],
        );
        let b = plugin(
            "voxelite.b",
            vec![Contribution::Command(command(
                "b.paint",
                Some("p"),
                context,
            ))],
        );
        let error = build(&[&a, &b]).unwrap_err();
        assert_eq!(
            error,
            RegistryError::DuplicateShortcut {
                shortcut: "p".into(),
                context: context.map(str::to_owned),
                first_command: "a.paint".into(),
                second_command: "b.paint".into(),
            }
        );
        let message = error.to_string();
        for text in [
            "duplicate shortcut",
            "\"p\"",
            context.unwrap_or("global"),
            "a.paint",
            "b.paint",
        ] {
            assert!(message.contains(text), "{message}");
        }
    }
}

#[test]
fn duplicate_shortcuts_fail_within_one_plugin() {
    let plugin = plugin(
        "voxelite.tools",
        vec![
            Contribution::Command(command("tools.paint", Some("p"), Some("Editor"))),
            Contribution::Command(command("tools.pick", Some("p"), Some("Editor"))),
        ],
    );
    assert_eq!(
        build(&[&plugin]).unwrap_err(),
        RegistryError::DuplicateShortcut {
            shortcut: "p".into(),
            context: Some("Editor".into()),
            first_command: "tools.paint".into(),
            second_command: "tools.pick".into(),
        }
    );
}

#[test]
fn shortcuts_can_be_reused_in_distinct_contexts_and_can_be_absent() {
    let plugin = plugin(
        "voxelite.tools",
        vec![
            Contribution::Command(command("tools.global", Some("p"), None)),
            Contribution::Command(command("tools.editor", Some("p"), Some("Editor"))),
            Contribution::Command(command("tools.text", Some("p"), Some("TextField"))),
            Contribution::Command(command("tools.erase", Some("e"), Some("Editor"))),
            Contribution::Command(command("tools.unbound", None, Some("Editor"))),
            Contribution::Command(command("tools.other", None, Some("Editor"))),
        ],
    );
    assert_eq!(build(&[&plugin]).unwrap().commands().count(), 6);
}

#[test]
fn shortcut_and_context_strings_are_opaque_and_compared_exactly() {
    let plugin = plugin(
        "voxelite.tools",
        vec![
            Contribution::Command(command("tools.first", Some("ctrl-shift-p"), Some("Editor"))),
            Contribution::Command(command(
                "tools.second",
                Some("shift-ctrl-p"),
                Some("Editor"),
            )),
            Contribution::Command(command("tools.third", Some("ctrl-shift-p"), Some("editor"))),
        ],
    );
    assert_eq!(build(&[&plugin]).unwrap().commands().count(), 3);
}

#[test]
fn all_contribution_kinds_require_namespaced_ids() {
    for id in [
        "",
        "paint",
        "tools.Paint",
        "tools..paint",
        "tools.paint ",
        "tools.绘画",
    ] {
        for contribution in declarations(id) {
            let plugin = plugin("voxelite.tools", vec![contribution]);
            let RegistryError::InvalidContribution {
                plugin_id,
                id: rejected,
                reason,
            } = build(&[&plugin]).unwrap_err()
            else {
                panic!("expected an invalid contribution");
            };
            assert_eq!(plugin_id, plugin.manifest.id);
            assert_eq!(rejected, id);
            assert!(reason.contains("lowercase dotted ID"));
        }
    }
}

#[test]
fn boolean_settings_accept_both_default_values() {
    for default in [false, true] {
        let mut spec = setting("tools.enabled");
        spec.default = SettingValue::Bool(default);
        let plugin = plugin("voxelite.tools", vec![Contribution::Setting(spec.clone())]);
        assert_eq!(build(&[&plugin]).unwrap().setting(&spec.key), Some(&spec));
    }
}

#[test]
fn invalid_setting_defaults_and_choice_lists_are_rejected() {
    let cases = [
        (
            SettingKind::Bool,
            SettingValue::Choice("yes".into()),
            "match the setting kind",
        ),
        (
            SettingKind::Choice(vec!["yes".into()]),
            SettingValue::Bool(true),
            "match the setting kind",
        ),
        (
            SettingKind::Choice(vec![]),
            SettingValue::Choice("yes".into()),
            "nonempty",
        ),
        (
            SettingKind::Choice(vec!["yes".into(), "yes".into()]),
            SettingValue::Choice("yes".into()),
            "distinct",
        ),
        (
            SettingKind::Choice(vec!["".into(), "yes".into()]),
            SettingValue::Choice("yes".into()),
            "nonempty",
        ),
        (
            SettingKind::Choice(vec!["yes".into()]),
            SettingValue::Choice("no".into()),
            "one of the declared choices",
        ),
    ];
    for (kind, default, reason) in cases {
        let mut spec = setting("tools.mode");
        spec.kind = kind;
        spec.default = default;
        let plugin = plugin("voxelite.tools", vec![Contribution::Setting(spec)]);
        let error = build(&[&plugin]).unwrap_err();
        assert!(
            matches!(&error, RegistryError::InvalidContribution { plugin_id, id, reason: message }
            if plugin_id == &plugin.manifest.id && id == "tools.mode" && message.contains(reason))
        );
        let message = error.to_string();
        assert!(message.contains("tools.mode"));
        assert!(message.contains("voxelite.tools"));
        assert!(message.contains(reason));
    }
}

#[test]
fn format_capabilities_and_extension_order_are_preserved() {
    for (import, export) in [(true, false), (false, true), (true, true), (false, false)] {
        let mut spec = format("formats.image");
        spec.import = import;
        spec.export = export;
        let plugin = plugin("voxelite.formats", vec![Contribution::Format(spec.clone())]);
        assert_eq!(build(&[&plugin]).unwrap().format(&spec.id), Some(&spec));
    }
}

#[test]
fn command_presentation_and_agent_visibility_are_optional() {
    let mut spec = command("tools.paint", None, None);
    spec.icon = None;
    spec.menu = None;
    spec.agent_visible = false;
    let plugin = plugin("voxelite.tools", vec![Contribution::Command(spec.clone())]);
    assert_eq!(build(&[&plugin]).unwrap().command(&spec.id), Some(&spec));
}

#[test]
fn rebuilding_with_a_new_enabled_set_leaves_previous_registry_unchanged() {
    let a = plugin(
        "voxelite.a",
        vec![Contribution::Command(command("a.paint", Some("p"), None))],
    );
    let b = plugin(
        "voxelite.b",
        vec![Contribution::Command(command("b.paint", Some("p"), None))],
    );
    let first = Registry::new(&[&a, &b], &enabled(&["voxelite.a"])).unwrap();
    assert!(build(&[&a, &b]).is_err());
    let second = Registry::new(&[&a, &b], &enabled(&["voxelite.b"])).unwrap();
    assert!(first.command("a.paint").is_some());
    assert!(first.command("b.paint").is_none());
    assert!(second.command("a.paint").is_none());
    assert!(second.command("b.paint").is_some());
}
