//! Shared assertions for each built-in plugin's unit tests.

use voxelite_plugin_api::Plugin;

use crate::{EnabledSet, Host, HostError, HostEvent, PluginState};

/// Assert manifest, declaration, conflict and lifecycle conformance.
///
/// The factory must return a fresh plugin. The harness checks local validation,
/// conflicts in an otherwise empty host, idempotent lifecycle requests, removal
/// while disabled and identical declarations after an enable/disable/enable cycle.
/// Every declared tool must bind while enabled and be unavailable while disabled.
/// It recollects declarations to detect plugin state leaking into metadata rather
/// than merely comparing the host's cached declarations. Activation and
/// deactivation must complete without panicking, including final cleanup.
///
/// For a required plugin, public disablement must be rejected; the harness then
/// exercises the same internal deactivation path to test cleanup and reactivation.
/// The harness cannot inspect arbitrary plugin-owned resources or external effects.
///
/// # Panics
/// Panics with a diagnostic when the factory or plugin fails these assertions.
///
/// ```
/// use voxelite_plugin_api::{API_VERSION, Manifest, Plugin, PluginId, Registrar, Version};
///
/// struct Example;
/// impl Plugin for Example {
///     fn manifest(&self) -> Manifest {
///         Manifest {
///             id: PluginId::new("dev.example.plugin").unwrap(),
///             name_key: "example.name".into(),
///             description_key: "example.description".into(),
///             version: Version::new(1, 0, 0),
///             api_version: API_VERSION,
///             builtin: true,
///             default_enabled: true,
///             can_disable: true,
///         }
///     }
///     fn contribute(&self, _: &mut Registrar) {}
/// }
/// voxelite_plugin_host::conformance::check(|| Box::new(Example));
/// ```
pub fn check(plugin_factory: impl FnOnce() -> Box<dyn Plugin>) {
    let mut host = Host::with_enabled(vec![plugin_factory()], EnabledSet::default());
    let events = host.drain_events();
    assert!(
        events
            .iter()
            .all(|event| matches!(event, HostEvent::Enabled(_))),
        "plugin registration failed: {events:?}"
    );
    let manifest = host
        .plugins()
        .next()
        .expect("plugin was not registered")
        .0
        .clone();
    let id = manifest.id.as_str();
    assert!(
        host.conflicts().is_empty(),
        "plugin conflicts with an empty host: {:?}",
        host.conflicts()
    );
    host.enable(id).expect("first activation must succeed");
    let baseline = host.registry().for_plugin(id).unwrap().clone();
    let tools: Vec<_> = host.registry().tools().map(|(_, t)| t.id.clone()).collect();
    for tool in &tools {
        assert!(
            host.has_tool(tool).expect("tool binding panicked"),
            "unbound tool: {tool}"
        );
    }
    host.enable(id).expect("repeated enable must be a no-op");
    assert_eq!(host.timings().get(id, "activate").unwrap().count, 1);

    if manifest.can_disable {
        host.disable(id).expect("deactivation must succeed");
        host.disable(id).expect("repeated disable must be a no-op");
    } else {
        assert!(matches!(
            host.disable(id),
            Err(HostError::CannotDisable { .. })
        ));
        assert_eq!(host.state(id), Some(&PluginState::Enabled));
        host.deactivate(0)
            .expect("required plugin cleanup must succeed");
    }
    assert_eq!(host.state(id), Some(&PluginState::Disabled));
    assert!(
        host.registry().iter().next().is_none(),
        "disabled declarations remain published"
    );
    assert!(host.enabled_set().as_set().is_empty());
    for tool in &tools {
        assert!(
            !host.has_tool(tool).unwrap(),
            "disabled tool remains bound: {tool}"
        );
    }
    assert_eq!(host.timings().get(id, "deactivate").unwrap().count, 1);
    assert_eq!(
        host.collect(0)
            .expect("contribute panicked after disable")
            .expect("invalid declarations after disable"),
        baseline,
        "plugin declarations changed after cleanup"
    );

    host.enable(id).expect("reactivation must succeed");
    for tool in &tools {
        assert!(
            host.has_tool(tool)
                .expect("tool binding panicked after reenable")
        );
    }
    assert_eq!(host.registry().for_plugin(id), Some(&baseline));
    assert_eq!(
        host.collect(0)
            .expect("contribute panicked after reenable")
            .expect("invalid declarations after reenable"),
        baseline,
        "plugin declarations changed after reactivation"
    );
    assert_eq!(host.timings().get(id, "activate").unwrap().count, 2);
    host.deactivate(0).expect("final cleanup must succeed");
    assert!(host.registry().iter().next().is_none());
}
