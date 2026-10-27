use super::{RenderResult, error};
use std::time::Instant;

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

/// Per-frame diagnostics. Production workers leave collection disabled.
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

pub(super) fn elapsed_ms(start: Option<Instant>) -> f64 {
    start.map_or(0.0, |t| t.elapsed().as_secs_f64() * 1000.0)
}
pub(super) fn target_bytes(size: [u32; 2], fallback: bool) -> u64 {
    // MSAA color 16 + RGBA16F geometry 32 + depth (estimated 32-bit) 16;
    // resolved color 4 + geometry 8 + output 4. Driver alignment is not estimated.
    u64::from(size[0]) * u64::from(size[1]) * 80
        + if fallback {
            u64::from(crate::shadow::SHADOW_SIZE).pow(2) * 4
        } else {
            0
        }
}

pub(super) struct GpuTimer {
    set: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    readback: wgpu::Buffer,
    period: f64,
    shadow: bool,
}
impl GpuTimer {
    pub(super) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let buffer = |label, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: 48,
                usage,
                mapped_at_creation: false,
            })
        };
        Self {
            set: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("harness timestamps"),
                ty: wgpu::QueryType::Timestamp,
                count: 6,
            }),
            resolve: buffer(
                "timestamp resolve",
                wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            ),
            readback: buffer(
                "timestamp readback",
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            ),
            period: f64::from(queue.get_timestamp_period()),
            shadow: false,
        }
    }
    pub(super) fn writes(&self, begin: u32, end: u32) -> wgpu::RenderPassTimestampWrites<'_> {
        wgpu::RenderPassTimestampWrites {
            query_set: &self.set,
            beginning_of_pass_write_index: Some(begin),
            end_of_pass_write_index: Some(end),
        }
    }
    pub(super) fn resolve(&mut self, encoder: &mut wgpu::CommandEncoder, shadow: bool) {
        self.shadow = shadow;
        // Resolve only queries written this frame, always to an aligned offset of zero.
        let first = if shadow { 0 } else { 2 };
        encoder.resolve_query_set(&self.set, first..6, &self.resolve, 0);
        encoder.copy_buffer_to_buffer(
            &self.resolve,
            0,
            &self.readback,
            0,
            u64::from(6 - first) * 8,
        );
    }
    pub(super) fn read(&self, device: &wgpu::Device) -> RenderResult<GpuTimes> {
        let bytes = read_buffer(device, &self.readback)?;
        // Avoid alignment assumptions about a Vec<u8>.
        let ticks: Vec<_> = bytes
            .as_chunks::<8>()
            .0
            .iter()
            .map(|v| u64::from_ne_bytes(*v))
            .collect();
        timestamp_times(&ticks, self.period, self.shadow)
    }
}
fn timestamp_times(ticks: &[u64], period: f64, shadow: bool) -> RenderResult<GpuTimes> {
    let main = if shadow { 2 } else { 0 };
    let ms = |a: usize, b: usize| {
        ticks[b]
            .checked_sub(ticks[a])
            .map(|n| n as f64 * period / 1e6)
            .ok_or_else(|| error("GPU timestamps went backwards"))
    };
    Ok(GpuTimes {
        main_ms: ms(main, main + 1)?,
        resolve_ms: ms(main + 2, main + 3)?,
        shadow_ms: if shadow { Some(ms(0, 1)?) } else { None },
        total_ms: ms(0, main + 3)?,
    })
}

pub(super) fn read_buffer(device: &wgpu::Device, buffer: &wgpu::Buffer) -> RenderResult<Vec<u8>> {
    let slice = buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(error)?;
    rx.recv().map_err(error)?.map_err(error)?;
    let result = slice
        .get_mapped_range()
        .map(|view| view.to_vec())
        .map_err(error);
    buffer.unmap();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn byte_estimates_and_timestamp_span() {
        assert_eq!(target_bytes([1280, 800], false), 81_920_000);
        assert!(target_bytes([1280, 800], true) > 81_920_000);
        let t = timestamp_times(&[10, 20, 18, 30], 1e6, false).unwrap();
        assert_eq!(t.main_ms, 10.0);
        assert_eq!(t.resolve_ms, 12.0);
        assert_eq!(t.total_ms, 20.0); // span, not sum
        let t = timestamp_times(&[1, 3, 4, 8, 9, 10], 1e6, true).unwrap();
        assert_eq!(t.shadow_ms, Some(2.0));
        assert_eq!(t.total_ms, 9.0);
        assert!(timestamp_times(&[20, 10, 30, 40], 1.0, false).is_err());
    }
}
