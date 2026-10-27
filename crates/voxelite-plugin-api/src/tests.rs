use super::*;
use serde_json::json;

fn manifest() -> Manifest {
    Manifest {
        id: PluginId::new("dev.voxelite.tools").unwrap(),
        name_key: "plugins.tools.name".into(),
        description_key: "plugins.tools.description".into(),
        version: Version::parse("2.3.4-beta.1+build.7").unwrap(),
        api_version: API_VERSION,
        builtin: true,
        default_enabled: true,
        can_disable: true,
    }
}

fn command(id: &str) -> CommandContribution {
    CommandContribution {
        id: id.into(),
        label_key: "commands.paint.label".into(),
        args_schema: json!({
            "type": "object",
            "properties": {"size": {"type": "integer", "minimum": 1}},
            "required": ["size"],
            "additionalProperties": false
        }),
        default_keybinding: Some("p".into()),
        context: Some("Editor".into()),
        icon: Some("paint".into()),
        menu: Some("tools".into()),
        order: 0,
        application_section: None,
        agent_visible: true,
    }
}

fn setting(id: &str) -> SettingContribution {
    SettingContribution {
        key: id.into(),
        label_key: "settings.enabled.label".into(),
        kind: SettingKind::Bool,
        default: SettingValue::Bool(true),
    }
}

fn declarations(id: &str) -> Vec<Contribution> {
    vec![
        Contribution::Command(command(id)),
        Contribution::Tool(ToolContribution {
            id: id.into(),
            label_key: "tools.paint.label".into(),
            icon: Some("paint".into()),
        }),
        Contribution::Panel(PanelContribution {
            id: id.into(),
            label_key: "panels.palette.label".into(),
            icon: None,
        }),
        Contribution::Format(FormatContribution {
            content: FormatContent::Document,
            id: id.into(),
            label_key: "formats.image.label".into(),
            extensions: vec!["png".into(), "apng".into()],
            import: false,
            export: true,
        }),
        Contribution::RenderPass(RenderPassContribution {
            id: id.into(),
            slot: RenderPassSlot::Overlay,
        }),
        Contribution::Setting(setting(id)),
        Contribution::DataSection(DataSectionContribution {
            section_name: id.into(),
            schema_version: 2,
        }),
        Contribution::CommandTransform(CommandTransformContribution {
            id: id.into(),
            order: -10,
        }),
    ]
}

fn collect(contributions: Vec<Contribution>) -> Result<Contributions, ValidationError> {
    let mut registrar = Registrar::default();
    for contribution in contributions {
        registrar.contribute(contribution);
    }
    registrar.finish()
}

#[test]
fn plugin_ids_preserve_reverse_dns_identity() {
    for id in [
        "dev.voxelite.tools",
        "org.example",
        "com.example-2.tool3",
        "a.b",
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
fn malformed_plugin_ids_are_rejected_without_normalization() {
    for id in [
        "",
        "tools",
        "dev..tools",
        ".dev.tools",
        "dev.tools.",
        "Dev.tools",
        "dev.Tools",
        " dev.tools",
        "dev.tools ",
        "dev.tools\n",
        "dev.工具",
        "dev.tool_name",
        "dev.-tools",
        "dev.tools-",
        "dev.2tools",
        "dev/tools",
        "dev.tool:paint",
    ] {
        let expected = InvalidPluginId { id: id.into() };
        assert_eq!(PluginId::new(id).unwrap_err(), expected);
        assert_eq!(id.parse::<PluginId>().unwrap_err(), expected);
        assert_eq!(PluginId::try_from(id.to_owned()).unwrap_err(), expected);
        assert!(expected.to_string().contains("reverse-DNS"));
    }
}

#[test]
fn plugin_id_dns_length_limits_are_checked() {
    assert!(PluginId::new(format!("dev.{}", "a".repeat(63))).is_ok());
    assert!(PluginId::new(format!("dev.{}", "a".repeat(64))).is_err());
    let prefix = format!("{0}.{0}.{0}", "a".repeat(63));
    let longest = format!("{prefix}.{}", "b".repeat(61));
    assert_eq!(longest.len(), 253);
    assert!(PluginId::new(longest).is_ok());
    assert!(PluginId::new(format!("{prefix}.{}", "b".repeat(62))).is_err());
}

#[test]
fn manifest_uses_independent_semver_versions_and_localization_keys() {
    let manifest = manifest();
    assert_eq!(manifest.version.to_string(), "2.3.4-beta.1+build.7");
    assert_eq!(manifest.api_version, Version::new(0, 7, 0));
    assert_ne!(manifest.version, manifest.api_version);
    assert_eq!(manifest.name_key, "plugins.tools.name");
    assert_eq!(manifest.description_key, "plugins.tools.description");
    assert!(manifest.builtin && manifest.default_enabled && manifest.can_disable);
}

#[test]
fn empty_registrar_produces_an_empty_collection() {
    let contributions = Registrar::default().finish().unwrap();
    assert!(contributions.is_empty());
    assert_eq!(contributions.len(), 0);
    assert_eq!(contributions.iter().len(), 0);
    assert_eq!(contributions, Contributions::default());
}

#[test]
fn all_contribution_kinds_and_metadata_preserve_registration_order() {
    let ids = [
        "dev.voxelite.tools.paint",
        "dev.voxelite.tools.brush",
        "dev.voxelite.palette.panel",
        "dev.voxelite.export.png",
        "dev.voxelite.grid.render",
        "dev.voxelite.tools.brush_size",
        "dev.voxelite.comments.pins",
        "dev.voxelite.mirror.transform",
    ];
    let expected: Vec<_> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| declarations(id).remove(i))
        .collect();
    let mut registrar = Registrar::default();
    for contribution in expected.clone() {
        match contribution {
            Contribution::Command(value) => registrar.command(value),
            Contribution::Tool(value) => registrar.tool(value),
            Contribution::CommandTransform(value) => registrar.command_transform(value),
            Contribution::Panel(value) => registrar.panel(value),
            Contribution::Format(value) => registrar.format(value),
            Contribution::RenderPass(value) => registrar.render_pass(value),
            Contribution::Setting(value) => registrar.setting(value),
            Contribution::DataSection(value) => registrar.data_section(value),
        }
    }
    let contributions = registrar.finish().unwrap();
    assert!(!contributions.is_empty());
    assert_eq!(contributions.len(), expected.len());
    assert_eq!(contributions.iter().cloned().collect::<Vec<_>>(), expected);
    assert_eq!(
        contributions
            .iter()
            .map(Contribution::id)
            .collect::<Vec<_>>(),
        ids
    );
}

#[test]
fn duplicate_ids_fail_within_one_plugin_across_every_pair_of_kinds() {
    let id = "dev.voxelite.tools.shared";
    for first in declarations(id) {
        for second in declarations(id) {
            let contributions = vec![first.clone(), second];
            let expected = ValidationError::DuplicateContribution { id: id.into() };
            assert_eq!(
                validate_contributions(&contributions).unwrap_err(),
                expected
            );
            assert_eq!(collect(contributions).unwrap_err(), expected);
            assert!(expected.to_string().contains(id));
        }
    }
}

#[test]
fn registrars_do_not_check_cross_plugin_or_keybinding_conflicts() {
    let commands = vec![
        Contribution::Command(command("dev.voxelite.tools.paint")),
        Contribution::Command(command("dev.voxelite.tools.erase")),
    ];
    // Both commands use the same keybinding and context; only the host resolves it.
    let first = collect(commands.clone()).unwrap();
    let second = collect(commands).unwrap();
    assert_eq!(first.len(), 2);
    assert_eq!(first, second);
}

#[test]
fn malformed_identifiers_fail_for_every_contribution_kind() {
    for id in [
        "",
        "tools.Paint",
        "tools..paint",
        ".tools.paint",
        "tools.paint.",
        "tools.paint ",
        "tools.绘画",
        "tools/paint",
        "tools.2paint",
    ] {
        for contribution in declarations(id) {
            let error = collect(vec![contribution]).unwrap_err();
            assert!(
                matches!(&error, ValidationError::InvalidContribution { id: rejected, reason }
                if rejected == id && reason.contains("segments"))
            );
            assert!(error.to_string().contains(id));
        }
    }
}

#[test]
fn ids_require_namespaces_but_data_sections_allow_plugin_local_names() {
    for contribution in declarations("comments") {
        let is_section = matches!(contribution, Contribution::DataSection(_));
        assert_eq!(collect(vec![contribution]).is_ok(), is_section);
    }
    assert!(
        collect(vec![Contribution::DataSection(DataSectionContribution {
            section_name: "build_history".into(),
            schema_version: 1,
        })])
        .is_ok()
    );
}

#[test]
fn data_sections_require_positive_versions_and_bounded_names() {
    for (section_name, schema_version) in [("state".into(), 0), ("a".repeat(254), 1)] {
        let mut registrar = Registrar::default();
        registrar.data_section(DataSectionContribution {
            section_name,
            schema_version,
        });
        assert!(registrar.finish().is_err());
    }
}

#[test]
fn valid_setting_defaults_cover_all_supported_kinds() {
    for (kind, default) in [
        (SettingKind::Bool, SettingValue::Bool(false)),
        (SettingKind::Bool, SettingValue::Bool(true)),
        (
            SettingKind::Choice(vec!["flat".into(), "lit".into()]),
            SettingValue::Choice("lit".into()),
        ),
        (SettingKind::Number, SettingValue::Number(-1.5)),
        (SettingKind::Number, SettingValue::Number(0.0)),
        (SettingKind::Number, SettingValue::Number(f64::MAX)),
    ] {
        let declaration = Contribution::Setting(SettingContribution {
            kind,
            default,
            ..setting("dev.voxelite.lighting.mode")
        });
        assert_eq!(
            collect(vec![declaration.clone()]).unwrap().iter().next(),
            Some(&declaration)
        );
    }
}

#[test]
fn setting_defaults_must_match_the_declared_kind() {
    let kinds = [
        SettingKind::Bool,
        SettingKind::Choice(vec!["lit".into()]),
        SettingKind::Number,
    ];
    let defaults = [
        SettingValue::Bool(true),
        SettingValue::Choice("lit".into()),
        SettingValue::Number(1.0),
    ];
    for (i, kind) in kinds.iter().enumerate() {
        for (j, default) in defaults.iter().enumerate() {
            if i == j {
                continue;
            }
            let declaration = Contribution::Setting(SettingContribution {
                kind: kind.clone(),
                default: default.clone(),
                ..setting("dev.voxelite.lighting.mode")
            });
            assert!(
                collect(vec![declaration])
                    .unwrap_err()
                    .to_string()
                    .contains("match the setting kind")
            );
        }
    }
}

#[test]
fn invalid_choice_lists_and_defaults_are_rejected() {
    for (choices, default, reason) in [
        (vec![], "lit", "nonempty"),
        (vec!["lit", "lit"], "lit", "distinct"),
        (vec!["", "lit"], "lit", "nonempty"),
        (
            vec!["flat", "lit"],
            "missing",
            "one of the declared choices",
        ),
    ] {
        let declaration = Contribution::Setting(SettingContribution {
            kind: SettingKind::Choice(choices.into_iter().map(str::to_owned).collect()),
            default: SettingValue::Choice(default.into()),
            ..setting("dev.voxelite.lighting.mode")
        });
        assert!(
            collect(vec![declaration])
                .unwrap_err()
                .to_string()
                .contains(reason)
        );
    }
}

#[test]
fn numeric_defaults_must_be_finite() {
    for default in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let declaration = Contribution::Setting(SettingContribution {
            kind: SettingKind::Number,
            default: SettingValue::Number(default),
            ..setting("dev.voxelite.lighting.intensity")
        });
        assert!(
            collect(vec![declaration])
                .unwrap_err()
                .to_string()
                .contains("finite")
        );
    }
}

#[test]
fn command_schema_and_optional_presentation_are_preserved() {
    let mut command = command("dev.voxelite.tools.paint");
    command.default_keybinding = None;
    command.context = None;
    command.icon = None;
    command.menu = None;
    command.agent_visible = false;
    let declaration = Contribution::Command(command);
    assert_eq!(
        collect(vec![declaration.clone()]).unwrap().iter().next(),
        Some(&declaration)
    );
}

#[test]
fn format_capabilities_and_extension_order_are_preserved() {
    for (import, export) in [(true, false), (false, true), (true, true), (false, false)] {
        let declaration = Contribution::Format(FormatContribution {
            content: FormatContent::Document,
            id: "dev.voxelite.formats.image".into(),
            label_key: "formats.image.label".into(),
            extensions: vec!["png".into(), "apng".into()],
            import,
            export,
        });
        assert_eq!(
            collect(vec![declaration.clone()]).unwrap().iter().next(),
            Some(&declaration)
        );
    }
}

#[test]
fn render_pass_slots_are_preserved() {
    for slot in [
        RenderPassSlot::BeforeScene,
        RenderPassSlot::Geometry,
        RenderPassSlot::Lighting,
        RenderPassSlot::AfterScene,
        RenderPassSlot::Overlay,
        RenderPassSlot::PostProcess,
    ] {
        let declaration = Contribution::RenderPass(RenderPassContribution {
            id: "dev.voxelite.lighting.render".into(),
            slot,
        });
        assert_eq!(
            collect(vec![declaration.clone()]).unwrap().iter().next(),
            Some(&declaration)
        );
    }
}

struct TestPlugin;

impl Plugin for TestPlugin {
    fn manifest(&self) -> Manifest {
        manifest()
    }
    fn contribute(&self, registrar: &mut Registrar) {
        registrar.command(command("dev.voxelite.tools.paint"));
    }
}

#[derive(Default)]
struct TestHost {
    palette: Palette,
}
impl ActivateCx for TestHost {}
impl CommandCx for TestHost {}
impl ToolCx for TestHost {
    fn hovered_hit(&self) -> Option<ToolHit> {
        None
    }
    fn ground_target(&self) -> Option<Pos> {
        None
    }
    fn sample(&self, _: Pos) -> Option<(u16, Uuid)> {
        None
    }
    fn voxel(&self, _: Uuid, _: Pos) -> Result<u16, CoreError> {
        Ok(0)
    }
    fn palette(&self) -> &Palette {
        &self.palette
    }
    fn active_color(&self) -> &Swatch {
        self.palette.get(1).unwrap()
    }
    fn active_layer(&self) -> Uuid {
        Uuid::nil()
    }
    fn layer(&self, _: Uuid) -> Result<ToolLayer, CoreError> {
        Ok(ToolLayer {
            visible: true,
            locked: false,
        })
    }
    fn bounds(&self) -> Region {
        Region::new([0; 3], [32; 3]).unwrap()
    }
}
impl ExportCx for TestHost {
    fn snapshot(&self) -> &Snapshot {
        unreachable!("type-only capability test")
    }
    fn render_view(&self) -> ExportResult<ExportImage> {
        unreachable!("type-only capability test")
    }
}

#[test]
fn plugins_and_host_contexts_are_dyn_compatible_with_noop_lifecycle_defaults() {
    let mut host = TestHost::default();
    let _: &mut dyn CommandCx = &mut host;
    let _: &mut dyn ToolCx = &mut host;
    let _: &mut dyn ExportCx = &mut host;
    let mut plugin = TestPlugin;
    let plugin: &mut dyn Plugin = &mut plugin;
    assert!(plugin.tool("dev.voxelite.tools.paint").is_none());
    let before = plugin.manifest();
    plugin.activate(&mut host);
    let mut registrar = Registrar::default();
    plugin.contribute(&mut registrar);
    assert_eq!(registrar.finish().unwrap().len(), 1);
    plugin.deactivate();
    assert_eq!(plugin.manifest(), before);
}

#[test]
fn plugins_can_override_lifecycle_hooks() {
    struct StatefulPlugin {
        active: bool,
    }
    impl Plugin for StatefulPlugin {
        fn manifest(&self) -> Manifest {
            manifest()
        }
        fn contribute(&self, _registrar: &mut Registrar) {}
        fn activate(&mut self, _cx: &mut dyn ActivateCx) {
            self.active = true;
        }
        fn deactivate(&mut self) {
            self.active = false;
        }
    }
    let mut plugin = StatefulPlugin { active: false };
    let mut host = TestHost::default();
    let active: &mut dyn Plugin = &mut plugin;
    active.activate(&mut host);
    assert!(plugin.active);
    let active: &mut dyn Plugin = &mut plugin;
    active.deactivate();
    assert!(!plugin.active);
}
