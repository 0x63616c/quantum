//! Shared assertions for each built-in plugin's unit tests.

use voxelite_plugin_api::Plugin;

use crate::{EnabledSet, Host, HostError, HostEvent, PluginState};

/// Assert manifest, declaration, conflict and lifecycle conformance.
///
/// The factory must return a fresh plugin. The harness checks local validation,
/// conflicts in an otherwise empty host, idempotent lifecycle requests, removal
/// while disabled and identical declarations after an enable/disable/enable cycle.
/// Every declared tool and transform must bind only while enabled.
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
    let transforms: Vec<_> = host
        .registry()
        .command_transforms()
        .map(|(_, t)| t.id.clone())
        .collect();
    for transform in &transforms {
        assert!(
            host.has_command_transform(transform)
                .expect("transform binding panicked"),
            "unbound transform: {transform}"
        );
    }
    let comment_panels: Vec<_> = host
        .registry()
        .panels()
        .map(|(_, p)| p.id.clone())
        .collect();
    let comment_panels: Vec<_> = comment_panels
        .into_iter()
        .filter(|id| {
            host.has_comments_panel(id)
                .expect("comments binding panicked")
        })
        .collect();
    let overlays: Vec<_> = host
        .registry()
        .render_passes()
        .map(|(_, p)| p.id.clone())
        .collect();
    let passes: Vec<_> = overlays
        .iter()
        .filter(|id| host.has_render_pass(id).expect("pass binding panicked"))
        .cloned()
        .collect();
    let baseline_plan = host.render_plan();
    assert_eq!(
        host.state(id),
        Some(&PluginState::Enabled),
        "pass description panicked"
    );
    // Exercise every portable overlay binding that this plugin supplies.
    let overlays: Vec<_> = overlays
        .into_iter()
        .filter(|id| host.has_overlay(id).expect("overlay binding panicked"))
        .collect();
    let formats: Vec<_> = host
        .registry()
        .formats()
        .filter(|(_, f)| f.export)
        .map(|(_, f)| f.id.clone())
        .collect();
    let panels: Vec<_> = host
        .registry()
        .panels()
        .map(|(_, p)| p.id.clone())
        .collect();
    let panels: Vec<_> = panels
        .into_iter()
        .filter(|id| {
            host.has_palette_panel(id)
                .expect("palette binding panicked")
        })
        .collect();
    for format in &formats {
        assert!(
            host.has_exporter(format).expect("export binding panicked"),
            "unbound exporter: {format}"
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
    assert!(host.render_plan().passes().is_empty());
    for pass in &passes {
        assert!(
            !host.has_render_pass(pass).unwrap(),
            "disabled pass remains bound: {pass}"
        );
    }
    for panel in &comment_panels {
        assert!(!host.has_comments_panel(panel).unwrap());
    }
    for panel in &panels {
        assert!(
            !host.has_palette_panel(panel).unwrap(),
            "disabled palette panel remains bound: {panel}"
        );
    }
    for tool in &tools {
        assert!(
            !host.has_tool(tool).unwrap(),
            "disabled tool remains bound: {tool}"
        );
    }
    for overlay in &overlays {
        assert!(
            !host.has_overlay(overlay).unwrap(),
            "disabled overlay remains bound: {overlay}"
        );
    }
    for format in &formats {
        assert!(
            !host.has_exporter(format).unwrap(),
            "disabled exporter remains bound: {format}"
        );
    }
    assert_eq!(host.timings().get(id, "deactivate").unwrap().count, 1);
    for transform in &transforms {
        assert!(
            !host.has_command_transform(transform).unwrap(),
            "disabled transform remains bound: {transform}"
        );
    }
    assert_eq!(
        host.collect(0)
            .expect("contribute panicked after disable")
            .expect("invalid declarations after disable"),
        baseline,
        "plugin declarations changed after cleanup"
    );

    host.enable(id).expect("reactivation must succeed");
    assert_eq!(host.render_plan(), baseline_plan);
    for pass in &passes {
        assert!(
            host.has_render_pass(pass)
                .expect("pass binding panicked after reenable")
        );
    }
    for panel in &comment_panels {
        assert!(
            host.has_comments_panel(panel)
                .expect("comments binding panicked after reenable")
        );
    }
    for panel in &panels {
        assert!(
            host.has_palette_panel(panel)
                .expect("palette binding panicked after reenable")
        );
    }
    for format in &formats {
        assert!(
            host.has_exporter(format)
                .expect("export binding panicked after reenable")
        );
    }
    for overlay in &overlays {
        assert!(
            host.has_overlay(overlay)
                .expect("overlay binding panicked after reenable")
        );
    }
    for transform in &transforms {
        assert!(
            host.has_command_transform(transform)
                .expect("transform binding panicked after reenable")
        );
    }
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

/// Generate a unit test using the host's full conformance harness.
///
/// Keep the host in plugin dev-dependencies and invoke this inside a test module.
/// The factory expression must return a fresh boxed plugin.
///
/// ```
/// use voxelite_plugin_api::{Manifest, Plugin, PluginId, Registrar, Version};
///
/// struct Example;
/// impl Plugin for Example {
///     fn manifest(&self) -> Manifest {
///         Manifest::builtin(
///             PluginId::new("dev.example.plugin").unwrap(),
///             "example.name",
///             "example.description",
///             Version::new(1, 0, 0),
///         )
///     }
///     fn contribute(&self, _: &mut Registrar) {}
/// }
/// voxelite_plugin_host::plugin_conformance_test!(
///     conforms_to_plugin_contract,
///     || Box::new(Example)
/// );
/// ```
#[macro_export]
macro_rules! plugin_conformance_test {
    ($name:ident, $factory:expr $(,)?) => {
        #[test]
        fn $name() {
            $crate::conformance::check($factory);
        }
    };
}
