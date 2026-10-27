use super::*;

struct TransformPlugin {
    metadata: TestPlugin,
    failure: ToolFailure,
    calls: Rc<RefCell<Vec<String>>>,
}

impl TransformPlugin {
    fn new(id: &str, orders: &[i32], failure: ToolFailure) -> Self {
        let mut metadata = TestPlugin::new(id);
        for (index, order) in orders.iter().enumerate() {
            metadata.declarations.push(Contribution::CommandTransform(
                CommandTransformContribution {
                    id: format!("{id}.transform_{index}"),
                    order: *order,
                },
            ));
        }
        metadata
            .declarations
            .push(Contribution::Setting(SettingContribution {
                key: format!("{id}.setting"),
                label_key: "Enabled".into(),
                kind: SettingKind::Bool,
                default: SettingValue::Bool(true),
            }));
        Self {
            metadata,
            failure,
            calls: Rc::default(),
        }
    }
}

impl Plugin for TransformPlugin {
    fn manifest(&self) -> Manifest {
        self.metadata.manifest()
    }
    fn contribute(&self, registrar: &mut Registrar) {
        self.metadata.contribute(registrar);
    }
    fn command_transform(&self, id: &str) -> Option<&dyn CommandTransform> {
        self.calls.borrow_mut().push(id.into());
        assert!(
            self.failure != ToolFailure::BindPanic,
            "transform binding panic"
        );
        // Intentionally bind even undeclared IDs; admission belongs to the host.
        (self.failure != ToolFailure::Unbound).then_some(self)
    }
}

fn named(name: &str) -> Command {
    Command::EditDocument {
        name: Some(name.into()),
        bounds: None,
    }
}

impl CommandTransform for TransformPlugin {
    fn transform(&self, cx: &dyn TransformCx, mut commands: Vec<Command>) -> Vec<Command> {
        assert!(
            self.failure != ToolFailure::InteractPanic,
            "transform panic"
        );
        assert_eq!(cx.bounds(), BoundsOnly.bounds());
        let id = self.metadata.manifest.id.as_str();
        assert_eq!(
            cx.setting(&format!("{id}.setting")),
            Some(&SettingValue::Bool(true))
        );
        assert_eq!(cx.setting("dev.secret.setting"), None);
        if !commands.is_empty() {
            commands.push(named(id));
        }
        commands
    }
}

#[test]
fn transforms_compose_in_stable_order_with_only_their_own_settings() {
    let first = TransformPlugin::new("dev.first", &[20, -10, 20], ToolFailure::None);
    let first_calls = first.calls.clone();
    let second = TransformPlugin::new("dev.second", &[20, 0], ToolFailure::None);
    let second_calls = second.calls.clone();
    let mut secret = TestPlugin::new("dev.secret");
    secret.declarations = vec![Contribution::Setting(SettingContribution {
        key: "dev.secret.setting".into(),
        label_key: "Secret".into(),
        kind: SettingKind::Bool,
        default: SettingValue::Bool(true),
    })];
    let mut host = Host::new(vec![Box::new(first), Box::new(second), Box::new(secret)]);
    for id in ["dev.unknown", "dev.first.run", "dev.first.setting"] {
        assert!(!host.has_command_transform(id).unwrap());
    }
    assert!(first_calls.borrow().is_empty());
    assert_eq!(
        host.transform_commands(BoundsOnly.bounds(), vec![named("input")])
            .unwrap(),
        [
            "input",
            "dev.first",
            "dev.second",
            "dev.first",
            "dev.first",
            "dev.second"
        ]
        .map(named)
    );
    assert_eq!(
        *first_calls.borrow(),
        [
            "dev.first.transform_1",
            "dev.first.transform_0",
            "dev.first.transform_2"
        ]
    );
    assert_eq!(
        *second_calls.borrow(),
        ["dev.second.transform_1", "dev.second.transform_0"]
    );
    // Execution sorting must not reorder the registry's declarations.
    assert_eq!(
        host.registry()
            .command_transforms()
            .map(|(_, t)| t.order)
            .collect::<Vec<_>>(),
        [20, -10, 20, 20, 0]
    );
    host.disable("dev.first").unwrap();
    assert!(!host.has_command_transform("dev.first.transform_0").unwrap());
    assert_eq!(
        host.transform_commands(BoundsOnly.bounds(), vec![named("input")])
            .unwrap(),
        ["input", "dev.second", "dev.second"].map(named)
    );
    assert_eq!(first_calls.borrow().len(), 3);
    host.enable("dev.first").unwrap();
    assert!(host.has_command_transform("dev.first.transform_0").unwrap());
}

#[test]
fn dispatch_runs_transforms_on_empty_outcomes_without_changing_feedback() {
    let transform = TransformPlugin::new("dev.transform", &[0], ToolFailure::None);
    let calls = transform.calls.clone();
    let mut host = Host::new(vec![
        Box::new(ToolPlugin::new(ToolFailure::None)),
        Box::new(transform),
    ]);
    assert_eq!(
        host.dispatch_tool("dev.tool.brush", &BoundsOnly, tool_input())
            .unwrap(),
        Some(ToolOutcome {
            label: "Test".into(),
            anchor: tool_input().anchor,
            ..Default::default()
        })
    );
    assert_eq!(*calls.borrow(), ["dev.transform.transform_0"]);
}

#[test]
fn transform_panics_abort_the_pipeline_and_quarantine_only_the_owner() {
    for failure in [ToolFailure::BindPanic, ToolFailure::InteractPanic] {
        let bad = TransformPlugin::new("dev.bad", &[0], failure);
        let calls = bad.calls.clone();
        let later = TransformPlugin::new("dev.later", &[1], ToolFailure::None);
        let later_calls = later.calls.clone();
        let mut host = Host::new(vec![Box::new(bad), Box::new(later)]);
        host.drain_events();
        assert!(matches!(
            host.transform_commands(BoundsOnly.bounds(), vec![named("input")]),
            Err(HostError::Faulted {
                phase: Phase::CommandTransform,
                ..
            })
        ));
        assert!(host.registry().for_plugin("dev.bad").is_none());
        assert_eq!(host.setting("dev.bad.setting"), None);
        assert!(
            !host
                .set_setting("dev.bad.setting", SettingValue::Bool(false))
                .unwrap()
        );
        assert!(later_calls.borrow().is_empty());
        assert!(matches!(
            host.drain_events().as_slice(),
            [HostEvent::Faulted {
                phase: Phase::CommandTransform,
                ..
            }]
        ));
        assert_eq!(
            host.timings()
                .get("dev.bad", "command_transform")
                .unwrap()
                .count,
            1
        );
        assert_eq!(
            host.transform_commands(BoundsOnly.bounds(), vec![named("input")])
                .unwrap(),
            [named("input"), named("dev.later")]
        );
        assert!(!host.has_command_transform("dev.bad.transform_0").unwrap());
        assert_eq!(calls.borrow().len(), 1);
    }
}

#[test]
fn unbound_transforms_are_inert_but_fail_conformance() {
    let mut host = Host::new(vec![Box::new(TransformPlugin::new(
        "dev.unbound",
        &[0],
        ToolFailure::Unbound,
    ))]);
    assert_eq!(
        host.transform_commands(BoundsOnly.bounds(), vec![named("input")])
            .unwrap(),
        [named("input")]
    );
    conformance::check(|| Box::new(TransformPlugin::new("dev.bound", &[0], ToolFailure::None)));
    for failure in [ToolFailure::Unbound, ToolFailure::BindPanic] {
        assert!(
            std::panic::catch_unwind(|| conformance::check(|| Box::new(TransformPlugin::new(
                "dev.bad",
                &[0],
                failure
            ))))
            .is_err()
        );
    }
}

#[test]
fn runtime_settings_validate_schemas_and_survive_disable() {
    for (kind, default, valid, invalid) in [
        (
            SettingKind::Bool,
            SettingValue::Bool(false),
            SettingValue::Bool(true),
            SettingValue::Number(1.0),
        ),
        (
            SettingKind::Choice(vec!["a".into(), "b".into()]),
            SettingValue::Choice("a".into()),
            SettingValue::Choice("b".into()),
            SettingValue::Choice("c".into()),
        ),
        (
            SettingKind::Number,
            SettingValue::Number(0.0),
            SettingValue::Number(2.0),
            SettingValue::Number(f64::INFINITY),
        ),
        (
            SettingKind::Number,
            SettingValue::Number(0.0),
            SettingValue::Number(2.0),
            SettingValue::Number(f64::NAN),
        ),
    ] {
        let mut plugin = TestPlugin::new("dev.settings");
        plugin.declarations = vec![Contribution::Setting(SettingContribution {
            key: "dev.settings.value".into(),
            label_key: "Value".into(),
            kind,
            default: default.clone(),
        })];
        let mut host = Host::new(vec![Box::new(plugin)]);
        let key = "dev.settings.value";
        assert_eq!(host.setting(key), Some(&default));
        assert!(host.set_setting(key, valid.clone()).unwrap());
        assert!(!host.set_setting(key, valid.clone()).unwrap());
        assert!(matches!(
            host.set_setting(key, invalid),
            Err(HostError::InvalidSetting { .. })
        ));
        assert_eq!(host.setting(key), Some(&valid));
        host.disable("dev.settings").unwrap();
        assert_eq!(host.setting(key), None);
        assert!(!host.set_setting(key, default).unwrap());
        host.enable("dev.settings").unwrap();
        assert_eq!(host.setting(key), Some(&valid));
        assert!(!host.set_setting("dev.missing", valid).unwrap());
        assert_eq!(host.setting("dev.missing"), None);
    }
}
