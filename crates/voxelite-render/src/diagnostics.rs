//! Portable renderer measurements; no GPU or UI dependencies.

/// Actual shadow implementation selected by the production viewport.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShadowPath {
    /// Exact voxel occupancy traversal, including AO.
    #[default]
    Exact,
    /// Production shadow-map fallback after occupancy allocation/upload failure.
    Fallback,
}
impl ShadowPath {
    /// Stable report spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Fallback => "fallback",
        }
    }
}

/// GPU timestamps in milliseconds, absent when unsupported or not requested.
#[derive(Clone, Copy, Debug, Default)]
pub struct GpuTimes {
    /// Four-sample MSAA scene pass.
    pub main_ms: f64,
    /// Linear-to-sRGB resolve and outlines.
    pub resolve_ms: f64,
    /// Shadow-map redraw, if used this frame.
    pub shadow_ms: Option<f64>,
    /// First pass start to final resolve end, not the sum of overlapping stages.
    pub total_ms: f64,
}

/// Per-frame diagnostics, collected only when explicitly enabled.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameStats {
    /// Snapshot synchronization through GPU completion, excluding diagnostic readback.
    pub wall_ms: f64,
    /// Queue submit through completion (includes the submit call).
    pub submit_complete_ms: f64,
    /// CPU occupancy cache synchronization.
    pub occupancy_sync_ms: f64,
    /// CPU chunk invalidation and meshing.
    pub mesh_sync_ms: f64,
    /// CPU occupancy upload, including the production staging flush.
    pub occupancy_upload_ms: f64,
    /// CPU mesh buffer uploads.
    pub mesh_upload_ms: f64,
    /// CPU target setup and command encoding, excluding uploads and synchronization.
    pub encode_ms: f64,
    /// Optional GPU pass times.
    pub gpu: Option<GpuTimes>,
    /// Main pass draw calls (one per resident chunk); resolve adds one fullscreen draw.
    pub draw_calls: u32,
    /// Additional draw calls in a fallback shadow redraw.
    pub shadow_draw_calls: u32,
    /// Main pass vertices.
    pub vertices: u64,
    /// Main pass triangles.
    pub triangles: u64,
    /// Resident chunk vertex buffers.
    pub resident_chunks: u32,
    /// Nonempty 8³ occupancy bricks in the CPU shadow cache.
    pub occupied_bricks: u32,
    /// Total 8³ occupancy bricks allocated in the CPU shadow cache.
    pub total_bricks: u32,
    /// Chunk buffers replaced, written or deleted.
    pub chunks_updated: u32,
    /// Occupancy, vertex and uniform bytes written this frame.
    pub upload_bytes: u64,
    /// Supersampled target dimensions.
    pub target_size: [u32; 2],
    /// Resident vertex buffer capacity, including unused capacity after shrinkage.
    pub vertex_bytes: u64,
    /// Resident GPU occupancy bytes (zero on fallback).
    pub occupancy_bytes: u64,
    /// Estimated render target bytes, excluding NV12 and diagnostic readback buffers.
    pub target_bytes: u64,
    /// MTLDevice currentAllocatedSize; unavailable on other backends.
    pub device_allocated_bytes: Option<u64>,
    /// Actual shadow path, never inferred from scene name.
    pub shadow_path: ShadowPath,
}
