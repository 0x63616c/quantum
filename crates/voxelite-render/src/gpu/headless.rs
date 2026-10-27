//! Portable owner of the real viewport, stopping at the sRGB texture consumed by NV12.
use super::*;
use voxelite_core::Region;

/// Adapter/device options; no quality controls are exposed.
#[derive(Clone, Copy, Debug, Default)]
pub struct HeadlessOptions {
    /// Request TIMESTAMP_QUERY only if supported. CPU stats are always collected.
    pub profile: bool,
    /// None selects Metal on macOS, Vulkan elsewhere.
    pub backends: Option<Backends>,
}
/// Identifies the actual adapter, including software drivers.
#[derive(Clone, Debug)]
pub struct AdapterInfo {
    /// Adapter's driver-provided name.
    pub name: String,
    /// Lowercase backend name.
    pub backend: String,
    /// Driver name.
    pub driver: String,
    /// Driver version/details.
    pub driver_info: String,
}
/// A completed frame at production quality.
pub struct HeadlessFrame {
    /// Timings and resident resource counts.
    pub stats: FrameStats,
    /// Tight RGBA8 sRGB bytes when capture was requested (no NV12 conversion).
    pub rgba: Option<Vec<u8>>,
    /// Actual supersampled dimensions.
    pub size: [u32; 2],
}
/// Serialized headless renderer. Owns the same caches and Viewport as the Mac worker.
pub struct Headless {
    viewport: Viewport,
    mesh: ChunkMeshes,
    occupancy: OccupancyCache,
    adapter: AdapterInfo,
}
impl Headless {
    /// Initialize adapter, device and production pipelines separately from frame timing.
    pub async fn new(options: HeadlessOptions) -> RenderResult<Self> {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = options.backends.unwrap_or(if cfg!(target_os = "macos") {
            Backends::METAL
        } else {
            Backends::VULKAN
        });
        let instance = wgpu::Instance::new(descriptor);
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .map_err(error)?;
        let info = adapter.get_info();
        let viewport = Viewport::with_options(&adapter, true, options.profile).await?;
        Ok(Self {
            viewport,
            mesh: ChunkMeshes::default(),
            occupancy: OccupancyCache::default(),
            adapter: AdapterInfo {
                name: info.name,
                backend: format!("{:?}", info.backend).to_lowercase(),
                driver: info.driver,
                driver_info: info.driver_info,
            },
        })
    }
    /// Actual device identity, for machine-bound baselines.
    pub fn adapter_info(&self) -> &AdapterInfo {
        &self.adapter
    }
    /// Drop scene caches and GPU scene resources, retaining the device/pipelines.
    /// The next frame is cold, including targets and any shadow fallback decision.
    pub fn reset_scene(&mut self) -> RenderResult<()> {
        self.viewport
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(error)?;
        self.mesh = ChunkMeshes::default();
        self.occupancy = OccupancyCache::default();
        self.viewport.buffers.clear();
        self.viewport.occupancy = None;
        self.viewport.occupancy_bind = None;
        self.viewport.fallback = None;
        self.viewport.targets = None;
        self.viewport.check_error()
    }
    /// Sync on content changes, encode, submit and wait for GPU completion.
    /// `None` reuses the last snapshot exactly as camera-only production requests do.
    #[allow(clippy::too_many_arguments)]
    pub async fn render(
        &mut self,
        snapshot: Option<Snapshot>,
        bounds: Region,
        camera: Camera,
        display_size: [u32; 2],
        style: Style,
        capture: bool,
    ) -> RenderResult<HeadlessFrame> {
        let started = Instant::now();
        let mut sync = FrameStats::default();
        let updates = sync_snapshot(
            &mut self.occupancy,
            &mut self.mesh,
            snapshot,
            Some(&mut sync),
        )?;
        let occupancy = self
            .occupancy
            .grid_mut()
            .ok_or_else(|| error("first frame needs a snapshot"))?;
        if occupancy.bounds() != bounds {
            return Err(error("snapshot bounds mismatch"));
        }
        self.viewport.capture = capture;
        let mut frame = self
            .viewport
            .render(camera, display_size, &self.mesh, &updates, occupancy, style)
            .await?;
        let readback = capture.then(|| {
            let row = (frame.size[0] * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let buffer = self.viewport.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("sRGB capture"),
                size: u64::from(row) * u64::from(frame.size[1]),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            frame.encoder.copy_texture_to_buffer(
                frame.output.texture().as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(row),
                        rows_per_image: Some(frame.size[1]),
                    },
                },
                wgpu::Extent3d {
                    width: frame.size[0],
                    height: frame.size[1],
                    depth_or_array_layers: 1,
                },
            );
            (buffer, row)
        });
        let submitted = Instant::now();
        let submission = self.viewport.queue.submit([frame.encoder.finish()]);
        self.viewport
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: None,
            })
            .map_err(error)?;
        let mut stats = self.viewport.stats.unwrap();
        stats.submit_complete_ms = submitted.elapsed().as_secs_f64() * 1000.0;
        stats.wall_ms = started.elapsed().as_secs_f64() * 1000.0;
        self.viewport.check_error()?;
        stats.occupancy_sync_ms = sync.occupancy_sync_ms;
        stats.mesh_sync_ms = sync.mesh_sync_ms;
        if let Some(timer) = &self.viewport.timer {
            stats.gpu = Some(timer.read(&self.viewport.device)?);
        }
        #[cfg(target_os = "macos")]
        {
            stats.device_allocated_bytes = mac::device_allocated_bytes(&self.viewport.device);
        }
        let rgba = readback
            .map(|(buffer, row)| {
                let data = stats::read_buffer(&self.viewport.device, &buffer)?;
                unpad_rows(&data, frame.size, row)
            })
            .transpose()?;
        self.viewport.check_error()?;
        Ok(HeadlessFrame {
            stats,
            rgba,
            size: frame.size,
        })
    }
}
fn unpad_rows(data: &[u8], size: [u32; 2], padded_row: u32) -> RenderResult<Vec<u8>> {
    let row = size[0]
        .checked_mul(4)
        .ok_or_else(|| error("capture width overflow"))? as usize;
    let stride = padded_row as usize;
    let height = size[1] as usize;
    let needed = stride
        .checked_mul(height)
        .ok_or_else(|| error("capture length overflow"))?;
    if row == 0 || height == 0 || stride < row || data.len() != needed {
        return Err(error("invalid padded RGBA capture"));
    }
    Ok(data
        .chunks_exact(stride)
        .flat_map(|line| line[..row].iter().copied())
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn readback_unpadding_keeps_channels_and_row_order() {
        let mut bytes = vec![99; 512];
        bytes[..12].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        bytes[256..268].fill(42);
        let tight = unpad_rows(&bytes, [3, 2], 256).unwrap();
        assert_eq!(&tight[..12], &bytes[..12]);
        assert_eq!(&tight[12..], &[42; 12]);
        assert!(unpad_rows(&bytes, [65, 2], 256).is_err());
        assert!(unpad_rows(&bytes[..511], [3, 2], 256).is_err());
        assert!(unpad_rows(&[], [0, 0], 0).is_err());
        assert_eq!(unpad_rows(&[7; 256], [64, 1], 256).unwrap(), vec![7; 256]);
    }
}
