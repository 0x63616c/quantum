//! Logical pass graph shared by software rendering and the GPU viewport.
//!
//! `BeforeScene -> Geometry -> Lighting -> AfterScene -> Overlay -> PostProcess`.
//! Geometry/depth, coverage, outlines and resolve remain core operations. Portable
//! overlays are composed by the shell at their declared slots. Plugins select
//! renderer-owned kernels; they never receive textures, buffers or an encoder.
//! The graph is logical: GPU geometry and lighting remain fused in the existing
//! MSAA draw, and software lighting and resolve share their sample loop. Changing
//! that physical scheduling would change rounding and risk existing image bytes.

/// Ordered insertion points in the core render graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderPassSlot {
    /// Background overlays and preparation before geometry.
    BeforeScene,
    /// Core surface geometry and depth.
    Geometry,
    /// Surface illumination, before foreground overlays and resolve.
    Lighting,
    /// Scene additions before foreground overlays.
    AfterScene,
    /// Foreground viewport overlays such as gizmos and pins.
    Overlay,
    /// Core outlines, antialiasing resolve and presentation conversion.
    PostProcess,
}

/// Complete logical execution order, including slots with no contributions.
pub const PASS_GRAPH: [RenderPassSlot; 6] = [
    RenderPassSlot::BeforeScene,
    RenderPassSlot::Geometry,
    RenderPassSlot::Lighting,
    RenderPassSlot::AfterScene,
    RenderPassSlot::Overlay,
    RenderPassSlot::PostProcess,
];

/// Portable selection of an audited renderer-owned implementation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderKernel {
    /// Existing camera-relative diffuse light, AO and exact voxel shadows.
    /// GPU occupancy allocation failure retains the existing shadow-map path.
    ExactShadowsLighting,
}

impl RenderKernel {
    /// Stable implementation identity for diagnostics and API documentation.
    pub const fn id(self) -> &'static str {
        match self {
            Self::ExactShadowsLighting => "dev.voxelite.render.exact-shadows-lighting",
        }
    }

    /// Only this slot grants the capability to select this implementation.
    pub const fn slot(self) -> RenderPassSlot {
        match self {
            Self::ExactShadowsLighting => RenderPassSlot::Lighting,
        }
    }
}

/// Effective surface shading shared by both rendering backends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lighting {
    /// Constant linear ambient illumination, without AO or shadow traversal.
    #[default]
    Ambient,
    /// The existing diffuse, AO and shadow implementation, unchanged.
    Exact,
}

/// Linear illumination used when no lighting implementation is contributed.
pub const AMBIENT_ILLUMINATION: f64 = 0.44;

/// An admitted kernel with its declaration and plugin identity for diagnostics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduledPass {
    /// Owning plugin ID.
    pub plugin_id: String,
    /// Declared contribution ID.
    pub id: String,
    /// Requested logical insertion point.
    pub slot: RenderPassSlot,
    /// Renderer-owned implementation to execute.
    pub kernel: RenderKernel,
}

/// Immutable frame description once handed to a renderer or worker.
/// Empty plans use deterministic ambient shading. Equal-slot contributions retain
/// registration order; selecting the same lighting kernel is idempotent, never
/// additive (the surface is shaded exactly once).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderPlan {
    passes: Vec<ScheduledPass>,
}

impl RenderPlan {
    /// Admit a supported kernel only in its capability slot. Returns false without
    /// changing the plan on a slot mismatch.
    pub fn insert(&mut self, pass: ScheduledPass) -> bool {
        if pass.slot != pass.kernel.slot() {
            return false;
        }
        self.passes.push(pass);
        self.passes.sort_by_key(|pass| pass.slot);
        true
    }

    /// Admitted passes in logical execution order.
    pub fn passes(&self) -> &[ScheduledPass] {
        &self.passes
    }

    /// Compile the lighting slot for the fused CPU/GPU surface implementation.
    pub fn lighting(&self) -> Lighting {
        self.passes
            .iter()
            .fold(Lighting::Ambient, |_, pass| match pass.kernel {
                RenderKernel::ExactShadowsLighting => Lighting::Exact,
            })
    }

    /// Legacy standalone rendering preset. Hosts must instead build a plan from
    /// enabled declarations, so disabled or faulted plugins fall back to ambient.
    pub fn standard() -> Self {
        let mut plan = Self::default();
        plan.insert(ScheduledPass {
            plugin_id: "dev.voxelite.shadows-lighting".into(),
            id: "dev.voxelite.shadows-lighting.surface".into(),
            slot: RenderPassSlot::Lighting,
            kernel: RenderKernel::ExactShadowsLighting,
        });
        plan
    }
}
