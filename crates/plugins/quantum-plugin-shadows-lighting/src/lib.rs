//! Built-in selection of exact shadows and lighting through a portable pass ID.
use quantum_plugin_api::*;

/// Independently disableable built-in identity.
pub const PLUGIN_ID: &str = "dev.quantum.shadows-lighting";
/// Surface lighting pass; kernels and resources remain owned by the renderer.
pub const SURFACE_PASS: &str = "dev.quantum.shadows-lighting.surface";

/// Enables the existing diffuse, ambient occlusion and exact shadow implementation.
pub struct ShadowsLightingPlugin;

impl Plugin for ShadowsLightingPlugin {
    fn manifest(&self) -> Manifest {
        Manifest::builtin(
            PLUGIN_ID,
            "plugins.shadows-lighting",
            env!("CARGO_PKG_VERSION"),
        )
    }
    fn contribute(&self, registrar: &mut Registrar) {
        registrar.render_pass(RenderPassContribution {
            id: SURFACE_PASS.into(),
            slot: RenderPassSlot::Lighting,
        });
    }
    fn render_pass(&self, id: &str) -> Option<&dyn RenderPass> {
        (id == SURFACE_PASS).then_some(self)
    }
}

impl RenderPass for ShadowsLightingPlugin {
    fn describe(&self, cx: &mut dyn PassCx) {
        cx.select(RenderKernel::ExactShadowsLighting);
    }
}

#[cfg(test)]
mod tests;
