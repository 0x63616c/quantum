use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Failure {
    None,
    Unbound,
    Bind,
    View,
    Interact,
    Error,
    ForeignSetting,
}

struct PanelPlugin {
    metadata: TestPlugin,
    failure: Failure,
}
impl PanelPlugin {
    fn new(failure: Failure) -> Self {
        let mut metadata = TestPlugin::new("dev.palette");
        metadata.declarations.extend([
            Contribution::Panel(PanelContribution {
                id: "dev.palette.panel".into(),
                label_key: "Palette".into(),
                icon: None,
            }),
            Contribution::Setting(SettingContribution {
                key: "dev.palette.choice".into(),
                label_key: "Choice".into(),
                kind: SettingKind::Bool,
                default: SettingValue::Bool(true),
            }),
        ]);
        Self { metadata, failure }
    }
}
impl Plugin for PanelPlugin {
    fn manifest(&self) -> Manifest {
        self.metadata.manifest()
    }
    fn contribute(&self, registrar: &mut Registrar) {
        self.metadata.contribute(registrar);
    }
    fn palette_panel(&self, _: &str) -> Option<&dyn PalettePanel> {
        assert!(self.failure != Failure::Bind, "binding panic");
        (self.failure != Failure::Unbound).then_some(self)
    }
}
impl PalettePanel for PanelPlugin {
    fn view(&self, cx: &dyn PaletteCx) -> PaletteView {
        assert!(self.failure != Failure::View, "view panic");
        assert_eq!(
            cx.setting("dev.palette.choice"),
            Some(&SettingValue::Bool(true))
        );
        assert_eq!(cx.setting("dev.secret.value"), None);
        PaletteView {
            selected: PaletteId::Source(0),
            kind: PaletteKind::Project,
            read_only: true,
            name: "Test".into(),
            palette: cx.palette().clone(),
            choices: vec![],
            preset: 0,
            state: cx.state().clone(),
        }
    }
    fn interact(
        &self,
        cx: &dyn PaletteCx,
        _: PaletteAction,
    ) -> Result<PaletteOutcome, PaletteError> {
        self.view(cx);
        assert!(self.failure != Failure::Interact, "interaction panic");
        if self.failure == Failure::Error {
            return Err(PaletteError::Message("rejected".into()));
        }
        Ok(PaletteOutcome {
            color: Some(cx.active_color().clone()),
            state: cx.state().clone(),
            setting: (self.failure == Failure::ForeignSetting)
                .then(|| ("dev.secret.value".into(), SettingValue::Bool(false))),
            ..Default::default()
        })
    }
}
#[derive(Default)]
struct Context {
    palette: Palette,
    color: Swatch,
    state: PaletteState,
}
impl PaletteCx for Context {
    fn palette(&self) -> &Palette {
        &self.palette
    }
    fn active_color(&self) -> &Swatch {
        &self.color
    }
    fn state(&self) -> &PaletteState {
        &self.state
    }
    fn setting(&self, _: &str) -> Option<&SettingValue> {
        Some(&SettingValue::Bool(false))
    }
}
fn host(failure: Failure) -> Host {
    let mut secret = TestPlugin::new("dev.secret");
    secret
        .declarations
        .push(Contribution::Setting(SettingContribution {
            key: "dev.secret.value".into(),
            label_key: "Secret".into(),
            kind: SettingKind::Bool,
            default: SettingValue::Bool(true),
        }));
    Host::new(vec![Box::new(PanelPlugin::new(failure)), Box::new(secret)])
}

#[test]
fn palette_lookup_is_declared_enabled_and_capability_scoped() {
    let cx = Context::default();
    let mut host = host(Failure::None);
    assert!(
        host.palette_view("dev.palette.undeclared", &cx)
            .unwrap()
            .is_none()
    );
    assert!(
        host.dispatch_palette(
            "dev.palette.panel",
            &cx,
            PaletteAction::Command("dev.secret.run".into())
        )
        .unwrap()
        .is_none()
    );
    assert!(
        host.palette_view("dev.palette.panel", &cx)
            .unwrap()
            .is_some()
    );
    assert!(
        host.dispatch_palette(
            "dev.palette.panel",
            &cx,
            PaletteAction::Command("dev.palette.run".into())
        )
        .unwrap()
        .is_some()
    );
    assert_eq!(
        host.timings().get("dev.palette", "palette").unwrap().count,
        2
    );
    host.disable("dev.palette").unwrap();
    assert!(!host.has_palette_panel("dev.palette.panel").unwrap());
    assert!(
        host.palette_view("dev.palette.panel", &cx)
            .unwrap()
            .is_none()
    );
    host.enable("dev.palette").unwrap();
    assert!(host.has_palette_panel("dev.palette.panel").unwrap());
}

#[test]
fn palette_errors_are_atomic_and_panics_quarantine_only_the_owner() {
    let cx = Context::default();
    for failure in [
        Failure::Unbound,
        Failure::Bind,
        Failure::View,
        Failure::Interact,
        Failure::Error,
        Failure::ForeignSetting,
    ] {
        let mut host = host(failure);
        let out = host.dispatch_palette("dev.palette.panel", &cx, PaletteAction::StepSwatch(1));
        if failure == Failure::Unbound {
            assert!(out.unwrap().is_none());
        } else {
            assert!(out.is_err());
        }
        let fault = matches!(failure, Failure::Bind | Failure::View | Failure::Interact);
        assert_eq!(
            matches!(
                host.state("dev.palette"),
                Some(PluginState::Faulted {
                    phase: Phase::Palette,
                    ..
                })
            ),
            fault
        );
        if fault {
            assert!(
                host.palette_view("dev.palette.panel", &cx)
                    .unwrap()
                    .is_none()
            );
        }
        assert_eq!(host.state("dev.secret"), Some(&PluginState::Enabled));
        assert_eq!(
            host.setting("dev.secret.value"),
            Some(&SettingValue::Bool(true))
        );
        assert_eq!(cx.state, PaletteState::default());
    }
    let mut host = host(Failure::View);
    assert!(host.palette_view("dev.palette.panel", &cx).is_err());
    assert!(host.registry().panels().next().is_none());
}
