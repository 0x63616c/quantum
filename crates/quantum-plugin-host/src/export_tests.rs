use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Failure {
    None,
    Unbound,
    Bind,
    Encode,
    Error,
}
struct ExportPlugin {
    metadata: TestPlugin,
    failure: Failure,
}
impl ExportPlugin {
    fn new(failure: Failure) -> Self {
        let mut metadata = TestPlugin::new("dev.export");
        metadata.declarations = [true, false]
            .into_iter()
            .map(|export| {
                Contribution::Format(FormatContribution {
                    id: if export {
                        "dev.export.write"
                    } else {
                        "dev.export.read"
                    }
                    .into(),
                    content: FormatContent::Document,
                    label_key: "Test".into(),
                    extensions: vec!["test".into()],
                    import: !export,
                    export,
                })
            })
            .collect();
        Self { metadata, failure }
    }
}
impl Plugin for ExportPlugin {
    fn manifest(&self) -> Manifest {
        self.metadata.manifest()
    }
    fn contribute(&self, registrar: &mut Registrar) {
        self.metadata.contribute(registrar);
    }
    fn exporter(&self, _: &str) -> Option<ExportCallback> {
        // Intentionally binds unknown/import-only IDs too: the host must gate lookup.
        match self.failure {
            Failure::Unbound => None,
            Failure::Bind => panic!("binding panic"),
            Failure::Encode => Some(|_| panic!("encoding panic")),
            Failure::Error => Some(|_| Err(ExportError::Invalid("bad view".into()))),
            Failure::None => Some(|cx| {
                assert_eq!(cx.palette(), &cx.snapshot().palette);
                assert_eq!(cx.bounds(), cx.snapshot().bounds);
                Ok(cx.render_view()?.rgba)
            }),
        }
    }
}
struct Context(Snapshot);
impl Default for Context {
    fn default() -> Self {
        Self(Snapshot {
            name: "Snapshot".into(),
            bounds: Region {
                min: [-1; 3],
                max: [1; 3],
            },
            palette: Palette::default(),
            layers: Default::default(),
            comments: Default::default(),
            extra: Default::default(),
        })
    }
}
impl ExportCx for Context {
    fn snapshot(&self) -> &Snapshot {
        &self.0
    }
    fn render_view(&self) -> ExportResult<ExportImage> {
        Ok(ExportImage {
            width: 1,
            height: 1,
            rgba: vec![1, 2, 3, 255],
        })
    }
}

#[test]
fn only_enabled_declared_export_bindings_dispatch_and_worker_results_are_rechecked() {
    let cx = Context::default();
    let before = cx.0.clone();
    let mut host = Host::new(vec![Box::new(ExportPlugin::new(Failure::None))]);
    for id in ["dev.export.unknown", "dev.export.read"] {
        assert!(!host.has_exporter(id).unwrap());
        assert!(host.dispatch_export(id, &cx).unwrap().is_none());
    }
    assert_eq!(
        host.dispatch_export("dev.export.write", &cx).unwrap(),
        Some(vec![1, 2, 3, 255])
    );
    let job = host.prepare_export("dev.export.write").unwrap().unwrap();
    let completion = std::thread::spawn(move || job.run(&Context::default()))
        .join()
        .unwrap();
    host.disable("dev.export").unwrap();
    assert!(host.finish_export(completion).unwrap().is_none());
    assert!(!host.has_exporter("dev.export.write").unwrap());
    assert!(
        host.dispatch_export("dev.export.write", &cx)
            .unwrap()
            .is_none()
    );
    host.enable("dev.export").unwrap();
    let job = host.prepare_export("dev.export.write").unwrap().unwrap();
    let mut replacement = Host::new(vec![Box::new(ExportPlugin::new(Failure::None))]);
    assert!(replacement.finish_export(job.run(&cx)).unwrap().is_none());
    assert!(
        host.dispatch_export("dev.export.write", &cx)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        host.timings()
            .get("dev.export", "export_encode")
            .unwrap()
            .count,
        3
    );
    assert_eq!(cx.0, before);
}

#[test]
fn missing_bindings_errors_and_panics_release_no_bytes() {
    for failure in [
        Failure::Unbound,
        Failure::Error,
        Failure::Bind,
        Failure::Encode,
    ] {
        let mut host = Host::new(vec![
            Box::new(ExportPlugin::new(failure)),
            Box::new(TestPlugin::new("dev.unrelated")),
        ]);
        let result = host.dispatch_export("dev.export.write", &Context::default());
        match failure {
            Failure::Unbound => assert!(result.unwrap().is_none()),
            Failure::Error => assert!(matches!(result, Err(HostError::Export { .. }))),
            Failure::Bind | Failure::Encode => {
                assert!(matches!(
                    result,
                    Err(HostError::Faulted {
                        phase: Phase::Export,
                        ..
                    })
                ));
                assert_eq!(host.registry().formats().count(), 0);
                assert!(!host.has_exporter("dev.export.write").unwrap());
                assert!(host.drain_events().iter().any(|e| matches!(
                    e,
                    HostEvent::Faulted {
                        phase: Phase::Export,
                        ..
                    }
                )));
            }
            Failure::None => unreachable!(),
        }
        assert!(host.registry().get("dev.unrelated.run").is_some());
        if matches!(failure, Failure::Unbound | Failure::Error) {
            assert_eq!(host.state("dev.export"), Some(&PluginState::Enabled));
        }
    }
}

#[test]
fn conformance_requires_export_bindings() {
    conformance::check(|| Box::new(ExportPlugin::new(Failure::None)));
    for failure in [Failure::Unbound, Failure::Bind] {
        assert!(
            std::panic::catch_unwind(|| conformance::check(|| Box::new(ExportPlugin::new(
                failure
            ))))
            .is_err()
        );
    }
}
