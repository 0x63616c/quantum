use super::*;
use quantum_render_plan::{Lighting as Shading, PASS_GRAPH, RenderPlan, ScheduledPass};

struct PassPlugin {
    id: &'static str,
    passes: Vec<Pass>,
    panic_binding: bool,
}
struct Pass {
    id: String,
    slot: RenderPassSlot,
    calls: Rc<RefCell<Vec<String>>>,
    panic: bool,
}
impl Plugin for PassPlugin {
    fn manifest(&self) -> Manifest {
        TestPlugin::new(self.id).manifest
    }
    fn contribute(&self, r: &mut Registrar) {
        r.setting(SettingContribution {
            key: format!("{}.enabled", self.id),
            label_key: "Enabled".into(),
            kind: SettingKind::Bool,
            default: SettingValue::Bool(true),
        });
        for pass in &self.passes {
            r.render_pass(RenderPassContribution {
                id: pass.id.clone(),
                slot: pass.slot,
            });
        }
    }
    fn render_pass(&self, id: &str) -> Option<&dyn RenderPass> {
        assert!(!self.panic_binding, "binding panic");
        self.passes
            .iter()
            .find(|pass| pass.id == id)
            .map(|pass| pass as &dyn RenderPass)
    }
}
impl RenderPass for Pass {
    fn describe(&self, cx: &mut dyn PassCx) {
        self.calls.borrow_mut().push(self.id.clone());
        assert_eq!(cx.slot(), self.slot);
        let owner = self.id.rsplit_once('.').unwrap().0;
        assert_eq!(
            cx.setting(&format!("{owner}.enabled")),
            Some(&SettingValue::Bool(true))
        );
        assert_eq!(cx.setting("dev.foreign.enabled"), None);
        assert_eq!(
            cx.select(RenderKernel::ExactShadowsLighting),
            self.slot == RenderPassSlot::Lighting
        );
        assert!(!self.panic, "description panic after selection");
    }
}
fn plugin(
    id: &'static str,
    slots: &[RenderPassSlot],
    calls: &Rc<RefCell<Vec<String>>>,
) -> PassPlugin {
    PassPlugin {
        id,
        panic_binding: false,
        passes: slots
            .iter()
            .enumerate()
            .map(|(i, slot)| Pass {
                id: format!("{id}.pass-{i}"),
                slot: *slot,
                calls: calls.clone(),
                panic: false,
            })
            .collect(),
    }
}

#[test]
fn slots_execute_in_graph_order_with_stable_ties_and_scoped_settings() {
    use RenderPassSlot::*;
    let calls = Rc::default();
    let mut host = Host::new(vec![
        Box::new(plugin(
            "dev.first",
            &[
                PostProcess,
                Lighting,
                BeforeScene,
                Geometry,
                Overlay,
                AfterScene,
                Lighting,
            ],
            &calls,
        )),
        Box::new(plugin("dev.second", &[Lighting], &calls)),
        Box::new(plugin("dev.foreign", &[], &calls)),
    ]);
    assert_eq!(
        PASS_GRAPH,
        [
            BeforeScene,
            Geometry,
            Lighting,
            AfterScene,
            Overlay,
            PostProcess
        ]
    );
    let plan = host.render_plan();
    assert_eq!(
        *calls.borrow(),
        [
            "dev.first.pass-2",
            "dev.first.pass-3",
            "dev.first.pass-1",
            "dev.first.pass-6",
            "dev.second.pass-0",
            "dev.first.pass-5",
            "dev.first.pass-4",
            "dev.first.pass-0"
        ]
    );
    assert_eq!(
        plan.passes()
            .iter()
            .map(|p| p.id.as_str())
            .collect::<Vec<_>>(),
        ["dev.first.pass-1", "dev.first.pass-6", "dev.second.pass-0"]
    );
    assert_eq!(plan.lighting(), Shading::Exact);
    assert!(!host.has_render_pass("dev.first.undeclared").unwrap());
    assert_eq!(
        host.timings()
            .get("dev.first", "render_pass:dev.first.pass-1")
            .unwrap()
            .count,
        1
    );
    host.disable("dev.first").unwrap();
    host.disable("dev.second").unwrap();
    calls.borrow_mut().clear();
    assert_eq!(host.render_plan().lighting(), Shading::Ambient);
    assert!(calls.borrow().is_empty());
    assert!(!host.has_render_pass("dev.first.pass-1").unwrap());
}

#[test]
fn failed_binding_or_later_description_discards_all_passes_from_its_owner() {
    for binding in [false, true] {
        let calls = Rc::default();
        let mut bad = plugin("dev.bad", &[RenderPassSlot::Lighting; 2], &calls);
        bad.panic_binding = binding;
        bad.passes[1].panic = true;
        let mut host = Host::new(vec![Box::new(bad)]);
        assert_eq!(host.render_plan().lighting(), Shading::Ambient);
        assert!(matches!(
            host.state("dev.bad"),
            Some(PluginState::Faulted {
                phase: Phase::RenderPass,
                ..
            })
        ));
        assert!(host.registry().render_passes().next().is_none());
        let count = calls.borrow().len();
        assert!(host.render_plan().passes().is_empty());
        assert_eq!(calls.borrow().len(), count);
        assert!(host.timings().get("dev.bad", "render_pass").is_some());
    }
}

#[test]
fn absent_bindings_and_incompatible_kernel_slots_are_inert() {
    let mut base = TestPlugin::new("dev.unbound");
    base.declarations = vec![Contribution::RenderPass(RenderPassContribution {
        id: "dev.unbound.pass".into(),
        slot: RenderPassSlot::Lighting,
    })];
    let mut host = Host::new(vec![Box::new(base)]);
    assert!(!host.has_render_pass("dev.unbound.pass").unwrap());
    assert!(host.render_plan().passes().is_empty());
    let mut plan = RenderPlan::default();
    assert!(!plan.insert(ScheduledPass {
        plugin_id: "dev.unbound".into(),
        id: "dev.unbound.pass".into(),
        slot: RenderPassSlot::Geometry,
        kernel: RenderKernel::ExactShadowsLighting
    }));
    assert_eq!(plan.lighting(), Shading::Ambient);
}
