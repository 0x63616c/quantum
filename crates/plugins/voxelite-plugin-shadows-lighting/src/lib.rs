//! Built-in selection of exact shadows and lighting through a portable pass ID.
use voxelite_plugin_api::*;

/// Independently disableable built-in identity.
pub const PLUGIN_ID: &str = "dev.voxelite.shadows-lighting";
/// Surface lighting pass; kernels and resources remain owned by the renderer.
pub const SURFACE_PASS: &str = "dev.voxelite.shadows-lighting.surface";

/// Enables the existing diffuse, ambient occlusion and exact shadow implementation.
pub struct ShadowsLightingPlugin;

impl Plugin for ShadowsLightingPlugin {
    fn manifest(&self) -> Manifest {
        Manifest {
            id: PluginId::new(PLUGIN_ID).unwrap(),
            name_key: "plugins.shadows-lighting.name".into(),
            description_key: "plugins.shadows-lighting.description".into(),
            version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            api_version: API_VERSION,
            builtin: true,
            default_enabled: true,
            can_disable: true,
        }
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
