use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Failure {
    None,
    Unbound,
    Bind,
    Draw,
    Interact,
    Error,
}
struct OverlayPlugin {
    metadata: TestPlugin,
    failure: Failure,
    calls: Rc<RefCell<Vec<&'static str>>>,
}
impl OverlayPlugin {
    fn new(id: &str, failure: Failure) -> Self {
        let mut metadata = TestPlugin::new(id);
        metadata.declarations = vec![
            Contribution::RenderPass(RenderPassContribution {
                id: format!("{id}.first"),
                slot: RenderPassSlot::BeforeScene,
            }),
            Contribution::RenderPass(RenderPassContribution {
                id: format!("{id}.second"),
                slot: RenderPassSlot::Overlay,
            }),
            Contribution::Setting(SettingContribution {
                key: format!("{id}.setting"),
                label_key: "Grid".into(),
                kind: SettingKind::Bool,
                default: SettingValue::Bool(true),
            }),
        ];
        Self {
            metadata,
            failure,
            calls: Rc::default(),
        }
    }
}
impl Plugin for OverlayPlugin {
    fn manifest(&self) -> Manifest {
        self.metadata.manifest()
    }
    fn contribute(&self, registrar: &mut Registrar) {
        self.metadata.contribute(registrar);
    }
    fn overlay(&self, _: &str) -> Option<&dyn Overlay> {
        self.calls.borrow_mut().push("bind");
        assert!(self.failure != Failure::Bind, "binding panic");
        // Deliberately binds undeclared IDs; the host must gate them.
        (self.failure != Failure::Unbound).then_some(self)
    }
}
fn primitive() -> OverlayPrimitive {
    OverlayPrimitive::Grid(OverlayGrid {
        plane: -2,
        offset: -0.05,
        min: -16,
        max: 48,
        spacing: 4,
        width: 0.6,
        color: OverlayColor::Line,
        opacity: 0.24,
    })
}
impl Overlay for OverlayPlugin {
    fn draw(&self, cx: &mut dyn OverlayCx, _: Option<u32>) {
        self.calls.borrow_mut().push("draw");
        assert_eq!(cx.bounds(), BoundsOnly.bounds());
        assert_eq!(cx.camera_basis(), basis());
        assert_eq!(
            cx.setting(&format!("{}.setting", self.metadata.manifest.id)),
            Some(&SettingValue::Bool(true))
        );
        assert_eq!(cx.setting("dev.secret.setting"), None);
        cx.emit(primitive());
        let draws = self
            .calls
            .borrow()
            .iter()
            .filter(|call| **call == "draw")
            .count();
        assert!(
            !(self.failure == Failure::Draw && draws == 2),
            "second draw panic"
        );
    }
    fn interact(
        &self,
        _: &dyn OverlayCx,
        input: OverlayInput,
    ) -> Result<OverlayOutcome, ToolError> {
        self.calls.borrow_mut().push("interact");
        assert!(self.failure != Failure::Interact, "interaction panic");
        if self.failure == Failure::Error {
            return Err(ToolError::Gesture("rejected".into()));
        }
        Ok(OverlayOutcome {
            commands: vec![Command::EditDocument {
                name: Some("Overlay edit".into()),
                bounds: None,
            }],
            label: "Overlay edit".into(),
            drag: input.drag,
            ..Default::default()
        })
    }
}
fn basis() -> [[f64; 3]; 3] {
    [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]
}
fn input() -> OverlayInput {
    OverlayInput {
        gesture: OverlayGesture::Hover,
        position: [56.; 2],
        pointer: [56.; 2],
        drag: None,
    }
}
#[test]
fn overlay_admission_order_scoped_queries_and_disable_enable() {
    let plugin = OverlayPlugin::new("dev.overlay", Failure::None);
    let calls = plugin.calls.clone();
    let mut secret = TestPlugin::new("dev.secret");
    secret.declarations = vec![Contribution::Setting(SettingContribution {
        key: "dev.secret.setting".into(),
        label_key: "Secret".into(),
        kind: SettingKind::Bool,
        default: SettingValue::Bool(true),
    })];
    let mut host = Host::new(vec![Box::new(plugin), Box::new(secret)]);
    for id in ["dev.missing", "dev.overlay.setting"] {
        assert!(!host.has_overlay(id).unwrap());
        assert!(
            host.dispatch_overlay(id, BoundsOnly.bounds(), basis(), input())
                .unwrap()
                .is_none()
        );
    }
    assert!(calls.borrow().is_empty());
    let frames = host.draw_overlays(BoundsOnly.bounds(), basis(), None);
    assert_eq!(
        frames.iter().map(|f| f.id.as_str()).collect::<Vec<_>>(),
        ["dev.overlay.first", "dev.overlay.second"]
    );
    assert!(frames.iter().all(|f| f.primitives == vec![primitive()]));
    assert_eq!(
        host.dispatch_overlay("dev.overlay.first", BoundsOnly.bounds(), basis(), input())
            .unwrap()
            .unwrap()
            .commands,
        vec![Command::EditDocument {
            name: Some("Overlay edit".into()),
            bounds: None
        }]
    );
    host.disable("dev.overlay").unwrap();
    let previous = calls.borrow().clone();
    assert!(
        host.draw_overlays(BoundsOnly.bounds(), basis(), None)
            .is_empty()
    );
    assert!(
        host.dispatch_overlay("dev.overlay.first", BoundsOnly.bounds(), basis(), input())
            .unwrap()
            .is_none()
    );
    assert!(!host.has_overlay("dev.overlay.first").unwrap());
    assert_eq!(*calls.borrow(), previous);
    host.enable("dev.overlay").unwrap();
    assert_eq!(
        host.draw_overlays(BoundsOnly.bounds(), basis(), None),
        frames
    );
    assert!(host.timings().get("dev.overlay", "overlay").unwrap().count >= 5);
}
#[test]
fn faults_discard_partial_and_earlier_passes_but_leave_other_plugins_usable() {
    for failure in [Failure::Bind, Failure::Draw] {
        let mut host = Host::new(vec![
            Box::new(OverlayPlugin::new("dev.bad", failure)),
            Box::new(OverlayPlugin::new("dev.good", Failure::None)),
        ]);
        host.drain_events();
        let frames = host.draw_overlays(BoundsOnly.bounds(), basis(), None);
        assert_eq!(frames.len(), 2);
        assert!(frames.iter().all(|f| f.id.starts_with("dev.good.")));
        assert!(matches!(
            host.state("dev.bad"),
            Some(PluginState::Faulted {
                phase: Phase::Overlay,
                ..
            })
        ));
        assert!(host.registry().for_plugin("dev.bad").is_none());
        assert!(host.drain_events().iter().any(|e| matches!(
            e,
            HostEvent::Faulted {
                phase: Phase::Overlay,
                ..
            }
        )));
        assert!(!host.has_overlay("dev.bad.first").unwrap());
        assert_eq!(
            host.draw_overlays(BoundsOnly.bounds(), basis(), None),
            frames
        );
    }
}
#[test]
fn missing_bindings_errors_and_panics_never_release_an_outcome() {
    for failure in [
        Failure::Unbound,
        Failure::Bind,
        Failure::Interact,
        Failure::Error,
    ] {
        let mut host = Host::new(vec![Box::new(OverlayPlugin::new("dev.test", failure))]);
        let result = host.dispatch_overlay("dev.test.first", BoundsOnly.bounds(), basis(), input());
        match failure {
            Failure::Unbound => assert_eq!(result, Ok(None)),
            Failure::Error => {
                assert!(matches!(
                    result,
                    Err(HostError::Tool(ToolError::Gesture(_)))
                ));
                assert_eq!(host.state("dev.test"), Some(&PluginState::Enabled));
            }
            _ => {
                assert!(matches!(
                    result,
                    Err(HostError::Faulted {
                        phase: Phase::Overlay,
                        ..
                    })
                ));
                assert!(host.registry().iter().next().is_none());
            }
        }
    }
    conformance::check(|| Box::new(OverlayPlugin::new("dev.test", Failure::None)));
}
