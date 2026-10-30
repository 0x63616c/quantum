use super::*;
use std::time::Instant;
use voxelite_plugin_api::{PassCx, RenderKernel, RenderPassSlot};
use voxelite_render_plan::{RenderPlan, ScheduledPass};

struct PassContext {
    slot: RenderPassSlot,
    settings: BTreeMap<String, SettingValue>,
    kernel: Option<RenderKernel>,
}

impl PassCx for PassContext {
    fn slot(&self) -> RenderPassSlot {
        self.slot
    }
    fn setting(&self, key: &str) -> Option<&SettingValue> {
        self.settings.get(key)
    }
    fn select(&mut self, kernel: RenderKernel) -> bool {
        if self.slot != kernel.slot() {
            return false;
        }
        self.kernel = Some(kernel);
        true
    }
}

impl Host {
    /// Describe enabled passes in graph order, retaining registration order within
    /// a slot. Missing bindings are inert. Panic isolation discards all selections
    /// from a faulted owner, including selections from its earlier callbacks.
    /// Timings measure description callbacks, not asynchronous GPU execution.
    pub fn render_plan(&mut self) -> RenderPlan {
        let mut declarations: Vec<_> = self
            .registry()
            .render_passes()
            .map(|(owner, pass)| (owner.clone(), pass.clone()))
            .collect();
        declarations.sort_by_key(|(_, pass)| pass.slot);
        let mut staged = Vec::new();
        for (owner, pass) in declarations {
            let Some(index) = self.overlay_owner(&pass.id) else {
                continue;
            };
            let mut cx = PassContext {
                slot: pass.slot,
                settings: self.plugins[index].settings.clone(),
                kernel: None,
            };
            let start = Instant::now();
            let result = self.invoke(index, Phase::RenderPass, |plugin| {
                if let Some(binding) = plugin.render_pass(&pass.id) {
                    binding.describe(&mut cx);
                }
            });
            self.timings.record(
                owner.as_str(),
                &format!("render_pass:{}", pass.id),
                start.elapsed(),
            );
            self.collect_budget_notices();
            if result.is_ok()
                && let Some(kernel) = cx.kernel
            {
                staged.push(ScheduledPass {
                    plugin_id: owner.to_string(),
                    id: pass.id,
                    slot: pass.slot,
                    kernel,
                });
            }
        }
        let mut plan = RenderPlan::default();
        for pass in staged {
            if self.overlay_owner(&pass.id).is_some() {
                plan.insert(pass);
            }
        }
        plan
    }

    /// Whether a declared enabled pass has a portable kernel description binding.
    pub fn has_render_pass(&mut self, id: &str) -> Result<bool, HostError> {
        let Some(index) = self.overlay_owner(id) else {
            return Ok(false);
        };
        self.invoke(index, Phase::RenderPass, |plugin| {
            plugin.render_pass(id).is_some()
        })
    }
}
