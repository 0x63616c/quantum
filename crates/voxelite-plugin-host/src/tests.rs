use std::{cell::RefCell, collections::BTreeSet, panic::AssertUnwindSafe, rc::Rc, time::Duration};

use voxelite_plugin_api::*;

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hook {
    Manifest,
    Contribute,
    Activate,
    Deactivate,
}

#[derive(Default, Debug, PartialEq, Eq)]
struct Calls {
    manifest: usize,
    contribute: usize,
    activate: usize,
    deactivate: usize,
    active: bool,
}

struct TestPlugin {
    manifest: Manifest,
    declarations: Vec<Contribution>,
    calls: Rc<RefCell<Calls>>,
    panic_at: Option<Hook>,
    leak_metadata: bool,
}

impl TestPlugin {
    fn new(id: &str) -> Self {
        Self {
            manifest: Manifest {
                id: PluginId::new(id).unwrap(),
                name_key: "plugin.name".into(),
                description_key: "plugin.description".into(),
                version: Version::new(9, 0, 0),
                api_version: API_VERSION,
                builtin: true,
                default_enabled: true,
                can_disable: true,
            },
            declarations: vec![command(&format!("{id}.run"), None, None)],
            calls: Rc::default(),
            panic_at: None,
            leak_metadata: false,
        }
    }

    fn panic_if(&self, hook: Hook) {
        if self.panic_at == Some(hook) {
            match hook {
                Hook::Manifest => panic!("manifest panic"),
                Hook::Contribute => panic!("contribute panic"),
                Hook::Activate => std::panic::panic_any(String::from("activate panic")),
                Hook::Deactivate => std::panic::panic_any(42_u32),
            }
        }
    }
}

impl Plugin for TestPlugin {
    fn manifest(&self) -> Manifest {
        self.calls.borrow_mut().manifest += 1;
        self.panic_if(Hook::Manifest);
        self.manifest.clone()
    }

    fn contribute(&self, registrar: &mut Registrar) {
        self.calls.borrow_mut().contribute += 1;
        for contribution in &self.declarations {
            registrar.contribute(contribution.clone());
        }
        // Partial declarations must not escape a panicking contribution hook.
        self.panic_if(Hook::Contribute);
    }

    fn activate(&mut self, _: &mut dyn ActivateCx) {
        {
            let mut calls = self.calls.borrow_mut();
            assert!(!calls.active, "activated twice without cleanup");
            calls.activate += 1;
            calls.active = true;
        }
        self.panic_if(Hook::Activate);
    }

    fn deactivate(&mut self) {
        {
            let mut calls = self.calls.borrow_mut();
            assert!(calls.active, "deactivated without activation");
            calls.deactivate += 1;
            calls.active = false;
        }
        if self.leak_metadata {
            self.declarations.push(tool("dev.example.leaked"));
        }
        self.panic_if(Hook::Deactivate);
    }
}

fn command(id: &str, key: Option<&str>, context: Option<&str>) -> Contribution {
    Contribution::Command(CommandContribution {
        id: id.into(),
        label_key: "command.label".into(),
        args_schema: Value::Object(Default::default()),
        default_keybinding: key.map(str::to_owned),
        context: context.map(str::to_owned),
        icon: None,
        menu: None,
        order: 0,
        application_section: None,
        agent_visible: true,
    })
}

fn tool(id: &str) -> Contribution {
    Contribution::Tool(ToolContribution {
        id: id.into(),
        label_key: "tool.label".into(),
        icon: None,
    })
}

fn all_kinds() -> Vec<Contribution> {
    vec![
        tool("dev.example.tool"),
        command("dev.example.command", Some("secondary-p"), Some("Editor")),
        Contribution::Panel(PanelContribution {
            id: "dev.example.panel".into(),
            label_key: "panel.label".into(),
            icon: None,
        }),
        Contribution::Format(FormatContribution {
            id: "dev.example.format".into(),
            label_key: "format.label".into(),
            extensions: vec!["png".into()],
            import: false,
            export: true,
        }),
        Contribution::RenderPass(RenderPassContribution {
            id: "dev.example.render".into(),
            slot: RenderPassSlot::Overlay,
        }),
        Contribution::Setting(SettingContribution {
            key: "dev.example.setting".into(),
            label_key: "setting.label".into(),
            kind: SettingKind::Bool,
            default: SettingValue::Bool(true),
        }),
        Contribution::DataSection(DataSectionContribution {
            section_name: "comments".into(),
            schema_version: 1,
        }),
    ]
}

fn snapshot(host: &Host) -> Vec<(String, Contribution)> {
    host.registry()
        .iter()
        .map(|(id, contribution)| (id.to_string(), contribution.clone()))
        .collect()
}

#[test]
fn compatibility_follows_pre_one_minor_and_stable_major_boundaries() {
    for (required, available, compatible) in [
        ("0.1.0", "0.1.0", true),
        ("0.1.0", "0.1.2", true),
        ("0.1.3", "0.1.2", false),
        ("0.0.1", "0.1.0", false),
        ("0.2.0", "0.1.0", false),
        ("0.1.0", "1.0.0", false),
        ("1.0.0", "1.5.4", true),
        ("1.5.4", "1.5.4", true),
        ("1.5.5", "1.5.4", false),
        ("1.6.0", "1.5.4", false),
        ("1.5.4", "2.0.0", false),
        ("2.0.0", "1.5.4", false),
        ("0.1.0+build.z", "0.1.0+build.a", true),
        ("1.2.3-alpha.1", "1.2.3-alpha.1+build", true),
        ("1.2.3-alpha.1", "1.2.3-alpha.2", false),
        ("0.1.0-alpha", "0.1.0", false),
        ("0.1.0", "0.1.0-alpha", false),
    ] {
        assert_eq!(
            api_compatible(
                &Version::parse(required).unwrap(),
                &Version::parse(available).unwrap()
            ),
            compatible,
            "required {required}, available {available}"
        );
    }
}

#[test]
fn manifest_validation_rejects_blank_localization_and_incompatible_api_only() {
    let mut manifest = TestPlugin::new("dev.example.plugin").manifest;
    assert!(validate_manifest(&manifest).is_ok());
    for blank in ["", " \n\t"] {
        manifest.name_key = blank.into();
        assert_eq!(
            validate_manifest(&manifest),
            Err(ManifestError::EmptyLocalizationKey { field: "name_key" })
        );
    }
    manifest.name_key = "plugin.name".into();
    manifest.description_key.clear();
    assert_eq!(
        validate_manifest(&manifest),
        Err(ManifestError::EmptyLocalizationKey {
            field: "description_key"
        })
    );
    manifest.description_key = "plugin.description".into();
    manifest.api_version.patch += 1;
    assert!(matches!(
        validate_manifest(&manifest),
        Err(ManifestError::IncompatibleApi { .. })
    ));
}

#[test]
fn invalid_registrations_do_not_stop_other_plugins_or_publish_partial_declarations() {
    let mut incompatible = TestPlugin::new("dev.incompatible");
    incompatible.manifest.api_version.minor += 1;
    let incompatible_calls = incompatible.calls.clone();
    let mut malformed = TestPlugin::new("dev.malformed");
    malformed.declarations.push(tool("invalid"));
    let malformed_calls = malformed.calls.clone();
    let mut host = Host::new(vec![
        Box::new(incompatible),
        Box::new(malformed),
        Box::new(TestPlugin::new("dev.good")),
    ]);
    assert!(host.state("dev.incompatible").is_none());
    assert!(host.state("dev.malformed").is_none());
    assert_eq!(host.state("dev.good"), Some(&PluginState::Enabled));
    assert_eq!(host.registry().iter().count(), 1);
    assert_eq!(incompatible_calls.borrow().contribute, 0);
    assert_eq!(malformed_calls.borrow().activate, 0);
    assert!(matches!(
        host.drain_events().as_slice(),
        [
            HostEvent::RegistrationRejected(RegistrationError::InvalidManifest { .. }),
            HostEvent::RegistrationRejected(RegistrationError::InvalidContributions { .. }),
            HostEvent::Enabled(_),
        ]
    ));
    assert!(host.drain_events().is_empty());
}

#[test]
fn local_duplicate_ids_and_invalid_setting_defaults_are_rejected() {
    for declarations in [
        vec![tool("dev.shared"), command("dev.shared", None, None)],
        vec![Contribution::Setting(SettingContribution {
            key: "dev.setting".into(),
            label_key: "setting.label".into(),
            kind: SettingKind::Bool,
            default: SettingValue::Number(1.0),
        })],
    ] {
        let mut plugin = TestPlugin::new("dev.bad");
        plugin.declarations = declarations;
        let mut host = Host::new(vec![Box::new(plugin)]);
        assert!(host.plugins().next().is_none());
        assert!(host.registry().iter().next().is_none());
        assert!(matches!(
            host.drain_events().as_slice(),
            [HostEvent::RegistrationRejected(
                RegistrationError::InvalidContributions { .. }
            )]
        ));
    }
}

#[test]
fn duplicate_plugin_ids_never_replace_enabled_disabled_or_faulted_instances() {
    for (enabled, panic_at) in [(true, None), (false, None), (true, Some(Hook::Contribute))] {
        let mut first = TestPlugin::new("dev.same");
        first.manifest.default_enabled = enabled;
        first.panic_at = panic_at;
        let mut host = Host::new(vec![Box::new(first)]);
        let before = host.state("dev.same").cloned();
        host.drain_events();
        let duplicate = TestPlugin::new("dev.same");
        let calls = duplicate.calls.clone();
        host.register(vec![Box::new(duplicate)]);
        assert_eq!(host.plugins().count(), 1);
        assert_eq!(host.state("dev.same").cloned(), before);
        assert_eq!(calls.borrow().contribute, 0);
        assert!(
            matches!(host.drain_events().as_slice(), [HostEvent::RegistrationRejected(RegistrationError::DuplicatePluginId { plugin_id })] if plugin_id.as_str() == "dev.same")
        );
    }
}

#[test]
fn all_queries_filter_enabled_plugins_and_preserve_both_registration_orders() {
    let mut first = TestPlugin::new("dev.zeta");
    first.declarations = all_kinds();
    let declarations = first.declarations.clone();
    let calls = first.calls.clone();
    let second = TestPlugin::new("dev.alpha");
    let mut host = Host::new(vec![Box::new(first), Box::new(second)]);
    let before = snapshot(&host);
    assert_eq!(
        before
            .iter()
            .take(7)
            .map(|(_, value)| value.clone())
            .collect::<Vec<_>>(),
        declarations
    );
    assert_eq!(before[0].0, "dev.zeta");
    assert_eq!(before[7].0, "dev.alpha");
    assert_eq!(
        host.plugins()
            .map(|(manifest, _)| manifest.id.as_str())
            .collect::<Vec<_>>(),
        ["dev.zeta", "dev.alpha"]
    );
    {
        let registry = host.registry();
        assert_eq!(registry.commands().count(), 2);
        assert_eq!(registry.tools().next().unwrap().1.id, "dev.example.tool");
        assert_eq!(registry.panels().next().unwrap().1.id, "dev.example.panel");
        assert_eq!(registry.formats().next().unwrap().1.extensions, ["png"]);
        assert_eq!(
            registry.render_passes().next().unwrap().1.slot,
            RenderPassSlot::Overlay
        );
        assert_eq!(
            registry.settings().next().unwrap().1.key,
            "dev.example.setting"
        );
        assert_eq!(
            registry.data_sections().next().unwrap().1.section_name,
            "comments"
        );
        for declaration in &declarations {
            assert_eq!(registry.get(declaration.id()).unwrap().1, declaration);
        }
        assert!(registry.get("dev.missing").is_none());
    }
    host.drain_events();
    for _ in 0..3 {
        host.enable("dev.zeta").unwrap();
        host.disable("dev.zeta").unwrap();
        host.disable("dev.zeta").unwrap();
        let registry = host.registry();
        assert_eq!(registry.iter().count(), 1);
        assert_eq!(registry.commands().count(), 1);
        assert_eq!(registry.tools().count(), 0);
        assert_eq!(registry.panels().count(), 0);
        assert_eq!(registry.formats().count(), 0);
        assert_eq!(registry.render_passes().count(), 0);
        assert_eq!(registry.settings().count(), 0);
        assert_eq!(registry.data_sections().count(), 0);
        assert!(registry.for_plugin("dev.zeta").is_none());
        assert!(registry.get("comments").is_none());
        assert!(!host.enabled_set().contains("dev.zeta"));
        host.enable("dev.zeta").unwrap();
        assert_eq!(snapshot(&host), before);
    }
    assert_eq!(
        *calls.borrow(),
        Calls {
            manifest: 1,
            contribute: 1,
            activate: 4,
            deactivate: 3,
            active: true
        }
    );
    assert_eq!(host.drain_events().len(), 6);
    assert!(host.drain_events().is_empty());
}

#[test]
fn enabled_sets_round_trip_and_override_defaults_but_not_required_plugins() {
    let saved = EnabledSet::from(vec![
        "dev.optional".into(),
        "dev.unknown".into(),
        "dev.optional".into(),
    ]);
    assert_eq!(saved.to_vec(), ["dev.optional", "dev.unknown"]);
    let set: BTreeSet<String> = saved.clone().into();
    assert_eq!(EnabledSet::from(set), saved);
    assert_eq!(EnabledSet::from(Vec::<String>::from(saved.clone())), saved);
    let mut optional = TestPlugin::new("dev.optional");
    optional.manifest.default_enabled = false;
    let default = TestPlugin::new("dev.default");
    let mut required = TestPlugin::new("dev.required");
    required.manifest.default_enabled = false;
    required.manifest.can_disable = false;
    let calls = required.calls.clone();
    let mut host = Host::with_enabled(
        vec![Box::new(default), Box::new(required), Box::new(optional)],
        saved,
    );
    assert_eq!(
        host.enabled_set().to_vec(),
        ["dev.optional", "dev.required"]
    );
    assert_eq!(host.state("dev.default"), Some(&PluginState::Disabled));
    assert!(matches!(
        host.disable("dev.required"),
        Err(HostError::CannotDisable { .. })
    ));
    assert_eq!(calls.borrow().deactivate, 0);
    assert_eq!(host.state("dev.required"), Some(&PluginState::Enabled));
    host.disable("dev.optional").unwrap();
    host.enable("dev.default").unwrap();
    let saved = host.enabled_set();
    let restored = Host::with_enabled(
        vec![
            Box::new(TestPlugin::new("dev.default")),
            Box::new(TestPlugin::new("dev.required")),
            Box::new(TestPlugin::new("dev.optional")),
        ],
        saved.clone(),
    );
    assert_eq!(restored.enabled_set(), saved);
    let empty = Host::with_enabled(
        vec![Box::new(TestPlugin::new("dev.default"))],
        EnabledSet::default(),
    );
    assert!(empty.enabled_set().as_set().is_empty());
}

#[test]
fn unknown_requests_and_empty_plugins_are_well_defined() {
    let mut host = Host::default();
    assert!(host.conflicts().is_empty());
    assert_eq!(
        host.enable("missing"),
        Err(HostError::UnknownPlugin {
            id: "missing".into()
        })
    );
    assert_eq!(
        host.disable("missing"),
        Err(HostError::UnknownPlugin {
            id: "missing".into()
        })
    );
    let mut empty = TestPlugin::new("dev.empty");
    empty.declarations.clear();
    host.register(vec![Box::new(empty)]);
    assert!(host.registry().for_plugin("dev.empty").unwrap().is_empty());
    assert!(host.enabled_set().contains("dev.empty"));
    host.disable("dev.empty").unwrap();
    assert!(host.registry().for_plugin("dev.empty").is_none());
}

#[test]
fn cross_plugin_conflicts_are_structured_and_hot_enable_can_recover() {
    let mut first = TestPlugin::new("dev.first");
    first.declarations = vec![
        tool("dev.shared"),
        command("dev.first.command", Some("p"), Some("Editor")),
    ];
    let mut second = TestPlugin::new("dev.second");
    second.declarations = vec![command("dev.shared", Some("p"), Some("Editor"))];
    let second_calls = second.calls.clone();
    let mut host = Host::new(vec![Box::new(first), Box::new(second)]);
    let owner = |plugin, contribution: &str| ContributionOwner {
        plugin_id: PluginId::new(plugin).unwrap(),
        contribution_id: contribution.into(),
    };
    let expected = vec![
        Conflict::DuplicateContributionId {
            id: "dev.shared".into(),
            first: owner("dev.first", "dev.shared"),
            second: owner("dev.second", "dev.shared"),
        },
        Conflict::DuplicateDefaultKeybinding {
            keybinding: "p".into(),
            context: Some("Editor".into()),
            first: owner("dev.first", "dev.first.command"),
            second: owner("dev.second", "dev.shared"),
        },
    ];
    assert_eq!(host.conflicts(), expected);
    assert_eq!(host.state("dev.second"), Some(&PluginState::Disabled));
    assert_eq!(second_calls.borrow().activate, 0);
    assert_eq!(host.registry().iter().count(), 2);
    assert!(
        matches!(host.drain_events().as_slice(), [HostEvent::Enabled(_), HostEvent::EnableRejected { conflicts, .. }] if conflicts == &expected)
    );
    assert_eq!(
        host.enable("dev.second"),
        Err(HostError::Conflicts {
            plugin_id: PluginId::new("dev.second").unwrap(),
            conflicts: expected.clone()
        })
    );
    host.disable("dev.first").unwrap();
    host.enable("dev.second").unwrap();
    assert_eq!(
        host.registry().get("dev.shared").unwrap().0.as_str(),
        "dev.second"
    );
    assert_eq!(host.registry().iter().count(), 1);
    assert_eq!(
        host.enable("dev.first"),
        Err(HostError::Conflicts {
            plugin_id: PluginId::new("dev.first").unwrap(),
            conflicts: expected
        })
    );
    host.disable("dev.second").unwrap();
    host.enable("dev.first").unwrap();
    assert_eq!(
        host.registry().get("dev.shared").unwrap().0.as_str(),
        "dev.first"
    );
}

#[test]
fn every_contribution_kind_participates_in_cross_plugin_id_conflicts() {
    for declaration in all_kinds() {
        let mut first = TestPlugin::new("dev.first");
        first.declarations = vec![declaration.clone()];
        let mut second = TestPlugin::new("dev.second");
        second.declarations = vec![declaration];
        let host = Host::new(vec![Box::new(first), Box::new(second)]);
        assert!(matches!(
            host.conflicts().first(),
            Some(Conflict::DuplicateContributionId { .. })
        ));
        assert_eq!(host.state("dev.second"), Some(&PluginState::Disabled));
    }
}

#[test]
fn default_keybindings_are_compared_only_within_identical_contexts() {
    let mut plugin = TestPlugin::new("dev.keys");
    plugin.declarations = vec![
        command("dev.global", Some("p"), None),
        command("dev.editor", Some("p"), Some("Editor")),
        command("dev.palette", Some("p"), Some("Palette")),
        command("dev.other", Some("e"), Some("Editor")),
        command("dev.unbound", None, Some("Editor")),
        command("dev.unbound_again", None, Some("Editor")),
    ];
    let mut host = Host::new(vec![Box::new(plugin)]);
    assert!(host.conflicts().is_empty());
    assert_eq!(host.registry().commands().count(), 6);
    let mut duplicates = TestPlugin::new("dev.duplicates");
    duplicates.manifest.default_enabled = false;
    duplicates.declarations = vec![
        command("dev.duplicate_one", Some("x"), None),
        command("dev.duplicate_two", Some("x"), None),
    ];
    let calls = duplicates.calls.clone();
    host.register(vec![Box::new(duplicates)]);
    assert!(matches!(
        host.conflicts().as_slice(),
        [Conflict::DuplicateDefaultKeybinding { context: None, .. }]
    ));
    assert!(matches!(
        host.enable("dev.duplicates"),
        Err(HostError::Conflicts { .. })
    ));
    assert_eq!(calls.borrow().activate, 0);
}

#[test]
fn all_hook_panics_are_isolated_and_faulted_plugins_are_never_called_again() {
    for (hook, phase, message) in [
        (Hook::Contribute, Phase::Contribute, "contribute panic"),
        (Hook::Activate, Phase::Activate, "activate panic"),
        (
            Hook::Deactivate,
            Phase::Deactivate,
            "non-string panic payload",
        ),
    ] {
        let mut bad = TestPlugin::new("dev.bad");
        bad.panic_at = Some(hook);
        let bad_calls = bad.calls.clone();
        let good = TestPlugin::new("dev.good");
        let good_calls = good.calls.clone();
        let mut host = Host::new(vec![Box::new(bad), Box::new(good)]);
        if hook == Hook::Deactivate {
            assert!(matches!(
                host.disable("dev.bad"),
                Err(HostError::Faulted { .. })
            ));
        }
        assert_eq!(
            host.state("dev.bad"),
            Some(&PluginState::Faulted {
                phase,
                message: message.into()
            })
        );
        assert!(host.registry().for_plugin("dev.bad").is_none());
        assert_eq!(host.registry().commands().count(), 1);
        assert_eq!(host.enabled_set().to_vec(), ["dev.good"]);
        assert!(host.conflicts().is_empty());
        let events = host.drain_events();
        assert_eq!(events.iter().filter(|event| matches!(event, HostEvent::Faulted { plugin_id, phase: failed, message: text } if plugin_id.as_str() == "dev.bad" && failed == &phase && text == message)).count(), 1);
        let before = format!("{:?}", bad_calls.borrow());
        assert!(matches!(
            host.enable("dev.bad"),
            Err(HostError::Faulted { .. })
        ));
        assert!(matches!(
            host.disable("dev.bad"),
            Err(HostError::Faulted { .. })
        ));
        assert_eq!(format!("{:?}", bad_calls.borrow()), before);
        assert!(host.drain_events().is_empty());
        host.disable("dev.good").unwrap();
        host.enable("dev.good").unwrap();
        assert_eq!(good_calls.borrow().activate, 2);
        assert_eq!(good_calls.borrow().deactivate, 1);
        assert_eq!(
            host.timings()
                .get(
                    "dev.bad",
                    match phase {
                        Phase::Contribute => "contribute",
                        Phase::Activate => "activate",
                        Phase::Deactivate => "deactivate",
                    }
                )
                .unwrap()
                .count,
            1
        );
        // Discarding faulted declarations also frees their IDs for other plugins.
        let mut replacement = TestPlugin::new("dev.replacement");
        replacement.declarations = vec![command("dev.bad.run", None, None)];
        host.register(vec![Box::new(replacement)]);
        assert!(host.conflicts().is_empty());
        assert_eq!(host.state("dev.replacement"), Some(&PluginState::Enabled));
    }
}

#[test]
fn manifest_panics_are_rejected_without_inventing_a_plugin_identity() {
    let mut bad = TestPlugin::new("dev.bad");
    bad.panic_at = Some(Hook::Manifest);
    let mut host = Host::new(vec![Box::new(bad), Box::new(TestPlugin::new("dev.good"))]);
    assert!(host.state("dev.bad").is_none());
    assert_eq!(host.plugins().count(), 1);
    assert!(
        matches!(host.drain_events().as_slice(), [HostEvent::RegistrationRejected(RegistrationError::ManifestPanicked { message }), HostEvent::Enabled(_)] if message == "manifest panic")
    );
}

#[test]
fn timings_and_budget_notices_cover_lifecycle_without_affecting_enablement() {
    let mut host = Host::default();
    host.timings_mut().set_budget(Some(Duration::ZERO));
    host.register(vec![Box::new(TestPlugin::new("dev.timed"))]);
    host.disable("dev.timed").unwrap();
    host.enable("dev.timed").unwrap();
    for (label, count) in [("contribute", 1), ("activate", 2), ("deactivate", 1)] {
        let timing = host.timings().get("dev.timed", label).unwrap();
        assert_eq!(timing.count, count);
        assert!(timing.total >= timing.max);
    }
    // Ensure a measurable duration even on platforms with coarse clocks.
    host.timings_mut().time("dev.timed", "render", || {
        std::thread::sleep(Duration::from_millis(2))
    });
    assert_eq!(host.timings().budget(), Some(Duration::ZERO));
    assert_eq!(host.state("dev.timed"), Some(&PluginState::Enabled));
    let events = host.drain_events();
    assert!(events.iter().any(|event| matches!(event, HostEvent::BudgetExceeded(notice) if notice.plugin_id == "dev.timed" && notice.label == "render" && notice.budget == Duration::ZERO && notice.elapsed > Duration::ZERO)));
    assert!(host.drain_events().is_empty());
    assert!(host.timings_mut().drain_notices().is_empty());
}

#[test]
fn conformance_accepts_optional_required_and_empty_plugins() {
    for required in [false, true] {
        for empty in [false, true] {
            let mut plugin = TestPlugin::new("dev.conforming");
            plugin.manifest.can_disable = !required;
            if empty {
                plugin.declarations.clear();
            }
            let calls = plugin.calls.clone();
            conformance::check(|| Box::new(plugin));
            assert_eq!(calls.borrow().activate, 2);
            assert_eq!(calls.borrow().deactivate, 2);
            assert!(!calls.borrow().active);
        }
    }
}

#[test]
fn conformance_rejects_panics_invalid_metadata_conflicts_and_residue() {
    let mut invalid_manifest = TestPlugin::new("dev.invalid");
    invalid_manifest.manifest.name_key.clear();
    let mut invalid_contributions = TestPlugin::new("dev.invalid");
    invalid_contributions.declarations.push(tool("invalid"));
    let mut conflicting = TestPlugin::new("dev.conflicting");
    conflicting.declarations = vec![
        command("dev.one", Some("x"), None),
        command("dev.two", Some("x"), None),
    ];
    let mut residue = TestPlugin::new("dev.residue");
    residue.leak_metadata = true;
    let mut bad_plugins = vec![
        invalid_manifest,
        invalid_contributions,
        conflicting,
        residue,
    ];
    for hook in [
        Hook::Manifest,
        Hook::Contribute,
        Hook::Activate,
        Hook::Deactivate,
    ] {
        let mut plugin = TestPlugin::new("dev.panicking");
        plugin.panic_at = Some(hook);
        bad_plugins.push(plugin);
    }
    for plugin in bad_plugins {
        assert!(
            std::panic::catch_unwind(AssertUnwindSafe(|| conformance::check(|| Box::new(plugin))))
                .is_err()
        );
    }
}
