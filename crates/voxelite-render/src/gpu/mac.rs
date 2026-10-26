//! The only unsafe boundary: CoreVideo-owned IOSurface planes imported into wgpu.
use super::*;
use objc2_core_foundation::{CFBoolean, CFDictionary, CFNumber, CFRetained, CFType};
use objc2_core_video::*;
use objc2_metal::{MTLDevice, MTLPixelFormat, MTLTextureType, MTLTextureUsage};
use std::{
    ptr::{self, NonNull},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

/// A completed NV12 frame. GPUI retains the pixel buffer through its consumer fence.
#[derive(Clone)]
pub struct Frame(CFRetained<CVPixelBuffer>);
impl Frame {
    /// The full-range bi-planar pixel buffer accepted by `Window::paint_surface`.
    pub fn pixel_buffer(&self) -> CFRetained<CVPixelBuffer> {
        self.0.clone()
    }
}
// A private, non-cloneable, one-way transfer. No public pixel-buffer reference
// exists until the UI unwraps it into a !Send Frame. The worker has completed
// all writes and never touches this buffer's attachments. Its only remaining
// references are private retirement leases that do nothing except retain/release.
struct CompletedFrame(Frame);
unsafe impl Send for CompletedFrame {}

struct Request {
    snapshot: Option<voxelite_core::Snapshot>,
    bounds: Region,
    camera: Camera,
    size: [u32; 2],
    style: Style,
    reply: futures_channel::oneshot::Sender<RenderResult<CompletedFrame>>,
}

/// A single Metal owner thread. Dropping this handle disconnects its inbox; it
/// never joins or destroys GPU resources on the UI thread. Requests are serialized
/// by the editor, with at most one outstanding frame and one coalesced update.
pub struct Worker {
    requests: std::sync::mpsc::Sender<Request>,
    failure: Arc<Mutex<Option<String>>>,
}
impl Worker {
    /// Start the worker; adapter and pipeline initialization happens on it.
    pub fn new(registry_id: u64) -> RenderResult<Self> {
        let (requests, inbox) = std::sync::mpsc::channel::<Request>();
        let failure = Arc::new(Mutex::new(None));
        let sink = failure.clone();
        std::thread::Builder::new()
            .name("voxelite-metal".into())
            .spawn(move || {
                // AppKit does not provide autorelease pools on this thread.
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let mut renderer = None;
                    let mut mesh = ChunkMeshes::default();
                    loop {
                        let request = match inbox.recv_timeout(Duration::from_millis(250)) {
                            Ok(request) => request,
                            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                                if let Some(renderer) = &renderer {
                                    let renderer: &Viewport = renderer;
                                    if let Err(error) =
                                        objc2::rc::autoreleasepool(|_| renderer.check_health())
                                    {
                                        *sink.lock().unwrap() = Some(error.to_string());
                                        break;
                                    }
                                }
                                continue;
                            }
                        };
                        let result = objc2::rc::autoreleasepool(|_| {
                            pollster::block_on(async {
                                let updates = match request.snapshot {
                                    Some(snapshot) => mesh.sync_validated(&snapshot)?,
                                    None => Vec::new(),
                                };
                                if renderer.is_none() {
                                    renderer = Some(Viewport::new(registry_id).await?);
                                }
                                renderer
                                    .as_mut()
                                    .unwrap()
                                    .render(
                                        request.bounds,
                                        request.camera,
                                        request.size,
                                        &mesh,
                                        &updates,
                                        request.style,
                                    )
                                    .await
                            })
                        });
                        let failed = result.is_err();
                        if let Err(error) = &result {
                            *sink.lock().unwrap() = Some(error.to_string());
                        }
                        let _ = request.reply.send(result.map(CompletedFrame));
                        if failed {
                            break;
                        }
                    }
                    objc2::rc::autoreleasepool(|_| drop(renderer));
                }));
                if result.is_err() {
                    *sink.lock().unwrap() = Some("Metal worker panicked".into());
                }
            })
            .map_err(error)?;
        Ok(Self { requests, failure })
    }

    /// Read worker health without polling Metal on the UI thread.
    pub fn check_health(&self) -> RenderResult<()> {
        match &*self.failure.lock().unwrap() {
            Some(message) => Err(error(message)),
            None => Ok(()),
        }
    }

    /// Submit the latest camera/style; provide a snapshot only on content changes.
    pub async fn render(
        &self,
        snapshot: Option<voxelite_core::Snapshot>,
        bounds: Region,
        camera: Camera,
        size: [u32; 2],
        style: Style,
    ) -> RenderResult<Frame> {
        self.check_health()?;
        let (reply, result) = futures_channel::oneshot::channel();
        self.requests
            .send(Request {
                snapshot,
                bounds,
                camera,
                size,
                style,
                reply,
            })
            .map_err(|_| error("Metal worker stopped"))?;
        result
            .await
            .map_err(|_| error("Metal worker stopped"))?
            .map(|completed| completed.0)
    }
}
// A CVMetalTexture is an immutable CoreVideo/Metal ownership token here. Its
// retain/release are thread-safe; no attachments or image bytes are accessed on
// the wgpu retirement thread. Keep the opaque object inaccessible to callers.
struct PlaneLease {
    texture: CFRetained<CVMetalTexture>,
    // Explicitly retain the pool allocation even if CoreVideo changes how its
    // wrappers retain their source. wgpu may retire textures after cancellation.
    buffer: CFRetained<CVPixelBuffer>,
}
unsafe impl Send for PlaneLease {}
unsafe impl Sync for PlaneLease {}
impl PlaneLease {
    fn release(self) {
        drop(self.texture);
        drop(self.buffer);
    }
}
/// Select GPUI's actual physical device; never infer it from a power preference.
pub(super) async fn adapter(
    instance: &wgpu::Instance,
    registry_id: u64,
) -> RenderResult<wgpu::Adapter> {
    for adapter in instance.enumerate_adapters(wgpu::Backends::METAL).await {
        // SAFETY: inspect the device while its HAL guard is live; do not mutate it.
        let matches = unsafe {
            adapter
                .as_hal::<wgpu::hal::api::Metal>()
                .is_some_and(|hal| hal.raw_device().registryID() == registry_id)
        };
        if matches {
            return Ok(adapter);
        }
    }
    Err(error("wgpu cannot use GPUI's Metal device"))
}
pub(super) struct Surfaces {
    cache: CFRetained<CVMetalTextureCache>,
    pool: Option<CFRetained<CVPixelBufferPool>>,
    size: [u32; 2],
    layout: wgpu::BindGroupLayout,
    y_pipeline: wgpu::RenderPipeline,
    chroma_pipeline: wgpu::RenderPipeline,
}
fn status(code: CVReturn) -> RenderResult<()> {
    if code == kCVReturnSuccess {
        Ok(())
    } else {
        Err(error(format!("CoreVideo status {code}")))
    }
}
impl Surfaces {
    pub fn new(device: &wgpu::Device) -> RenderResult<Self> {
        // SAFETY: the HAL guard stays alive for the call. CoreVideo retains the
        // same MTLDevice that owns all wgpu textures; no cross-device import.
        let cache = unsafe {
            let hal = device
                .as_hal::<wgpu::hal::api::Metal>()
                .ok_or_else(|| error("Metal device unavailable"))?;
            let usage = CFNumber::new_i64(
                (MTLTextureUsage::ShaderRead | MTLTextureUsage::RenderTarget).0 as i64,
            );
            let attributes = CFDictionary::from_slices(&[kCVMetalTextureUsage], &[&*usage]);
            let mut raw = ptr::null_mut();
            status(CVMetalTextureCache::create(
                None,
                None,
                hal.raw_device(),
                Some(attributes.as_opaque()),
                NonNull::from(&mut raw),
            ))?;
            CFRetained::from_raw(NonNull::new(raw).ok_or_else(|| error("null texture cache"))?)
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("NV12 source"),
            entries: &[texture_entry(
                0,
                wgpu::TextureSampleType::Float { filterable: false },
            )],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GPUI full-range BT.601 inverse"),
            source: wgpu::ShaderSource::Wgsl(include_str!("nv12.wgsl").into()),
        });
        let y_pipeline = pipeline(
            device,
            &shader,
            &[&layout],
            "vs_main",
            Some("fs_y"),
            &[],
            &[wgpu::TextureFormat::R8Unorm],
            None,
            1,
        );
        let chroma_pipeline = pipeline(
            device,
            &shader,
            &[&layout],
            "vs_main",
            Some("fs_chroma"),
            &[],
            &[wgpu::TextureFormat::Rg8Unorm],
            None,
            1,
        );
        Ok(Self {
            cache,
            pool: None,
            size: [0, 0],
            layout,
            y_pipeline,
            chroma_pipeline,
        })
    }
    fn create_pool(size: [u32; 2]) -> RenderResult<CFRetained<CVPixelBufferPool>> {
        // SAFETY: CF dictionaries retain typed values through the create call;
        // the successful create result transfers one +1 reference.
        unsafe {
            let surface = CFDictionary::<CFType, CFType>::empty();
            let width = CFNumber::new_i64(i64::from(size[0]));
            let height = CFNumber::new_i64(i64::from(size[1]));
            let format =
                CFNumber::new_i64(i64::from(kCVPixelFormatType_420YpCbCr8BiPlanarFullRange));
            let values: [&CFType; 5] = [
                surface.as_ref(),
                CFBoolean::new(true).as_ref(),
                width.as_ref(),
                height.as_ref(),
                format.as_ref(),
            ];
            let attributes = CFDictionary::from_slices(
                &[
                    kCVPixelBufferIOSurfacePropertiesKey,
                    kCVPixelBufferMetalCompatibilityKey,
                    kCVPixelBufferWidthKey,
                    kCVPixelBufferHeightKey,
                    kCVPixelBufferPixelFormatTypeKey,
                ],
                &values,
            );
            let mut raw = ptr::null_mut();
            let minimum = CFNumber::new_i64(3);
            let pool_attributes =
                CFDictionary::from_slices(&[kCVPixelBufferPoolMinimumBufferCountKey], &[&*minimum]);
            status(CVPixelBufferPool::create(
                None,
                Some(pool_attributes.as_opaque()),
                Some(attributes.as_opaque()),
                NonNull::from(&mut raw),
            ))?;
            Ok(CFRetained::from_raw(
                NonNull::new(raw).ok_or_else(|| error("null pixel buffer pool"))?,
            ))
        }
    }
    fn acquire(&self) -> RenderResult<Option<CFRetained<CVPixelBuffer>>> {
        // CoreVideo, rather than a sampled CFRetainCount, owns the reuse decision.
        // Scene, producer texture and consumer completion leases all prevent reuse.
        // SAFETY: typed threshold attribute, valid out-pointer and owned pool.
        unsafe {
            let threshold = CFNumber::new_i64(3);
            let attributes = CFDictionary::from_slices(
                &[kCVPixelBufferPoolAllocationThresholdKey],
                &[&*threshold],
            );
            let mut raw = ptr::null_mut();
            let code = CVPixelBufferPool::create_pixel_buffer_with_aux_attributes(
                None,
                self.pool.as_ref().unwrap(),
                Some(attributes.as_opaque()),
                NonNull::from(&mut raw),
            );
            if code == kCVReturnWouldExceedAllocationThreshold {
                return Ok(None);
            }
            status(code)?;
            let buffer =
                CFRetained::from_raw(NonNull::new(raw).ok_or_else(|| error("null pixel buffer"))?);
            if CVPixelBufferGetIOSurface(Some(&buffer)).is_none()
                || CVPixelBufferGetPixelFormatType(&buffer)
                    != kCVPixelFormatType_420YpCbCr8BiPlanarFullRange
                || CVPixelBufferGetPlaneCount(&buffer) != 2
                || CVPixelBufferGetWidthOfPlane(&buffer, 0) != self.size[0] as usize
                || CVPixelBufferGetHeightOfPlane(&buffer, 0) != self.size[1] as usize
                || CVPixelBufferGetWidthOfPlane(&buffer, 1) != (self.size[0] / 2) as usize
                || CVPixelBufferGetHeightOfPlane(&buffer, 1) != (self.size[1] / 2) as usize
                || CVPixelBufferGetBytesPerRowOfPlane(&buffer, 0) < self.size[0] as usize
                || CVPixelBufferGetBytesPerRowOfPlane(&buffer, 1) < self.size[0] as usize
            {
                return Err(error("unexpected NV12 IOSurface layout"));
            }
            Ok(Some(buffer))
        }
    }
    fn plane(
        &self,
        device: &wgpu::Device,
        buffer: &CFRetained<CVPixelBuffer>,
        plane: usize,
        metal_format: MTLPixelFormat,
        format: wgpu::TextureFormat,
    ) -> RenderResult<wgpu::TextureView> {
        let width = CVPixelBufferGetWidthOfPlane(buffer, plane) as u32;
        let height = CVPixelBufferGetHeightOfPlane(buffer, plane) as u32;
        // SAFETY: plane dimensions, format, sample count, mip count and usage all
        // match CoreVideo's actual Metal allocation. The cache uses this device.
        // Retain the CVMetalTexture until wgpu retires the raw texture, including
        // on resize/drop. UNINITIALIZED forces the first use to clear the plane.
        unsafe {
            let mut raw = ptr::null_mut();
            status(CVMetalTextureCache::create_texture_from_image(
                None,
                &self.cache,
                buffer,
                None,
                metal_format,
                width as usize,
                height as usize,
                plane,
                NonNull::from(&mut raw),
            ))?;
            let cv_texture =
                CFRetained::from_raw(NonNull::new(raw).ok_or_else(|| error("null plane texture"))?);
            let metal = CVMetalTextureGetTexture(&cv_texture)
                .ok_or_else(|| error("plane has no Metal texture"))?;
            // Import the planes through CoreVideo, never by assuming tightly
            // packed rows. IOSurface row padding is handled by the Metal texture.
            use objc2_metal::MTLTexture;
            if metal.width() != width as usize
                || metal.height() != height as usize
                || metal.pixelFormat() != metal_format
                || metal.textureType() != MTLTextureType::Type2D
                || metal.sampleCount() != 1
                || metal.mipmapLevelCount() != 1
                || !metal
                    .usage()
                    .contains(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead)
            {
                return Err(error("unexpected CoreVideo Metal plane descriptor"));
            }
            let lease = PlaneLease {
                texture: cv_texture,
                buffer: buffer.clone(),
            };
            let hal_texture = wgpu::hal::metal::Device::texture_from_raw(
                metal,
                format,
                MTLTextureType::Type2D,
                1,
                1,
                wgpu::hal::CopyExtent {
                    width,
                    height,
                    depth: 1,
                },
                Some(Box::new(move || lease.release())),
            );
            let texture = device.create_texture_from_hal::<wgpu::hal::api::Metal>(
                hal_texture,
                &wgpu::TextureDescriptor {
                    label: Some("IOSurface NV12 plane"),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
                wgpu::TextureUses::UNINITIALIZED,
            );
            Ok(texture.create_view(&Default::default()))
        }
    }
    pub async fn present(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mut encoder: wgpu::CommandEncoder,
        size: [u32; 2],
        source: &wgpu::TextureView,
        errors: &Arc<Mutex<Option<String>>>,
    ) -> RenderResult<Frame> {
        // The source scene and luma are already 2x device dimensions; CbCr has
        // half their width/height. Replacing the pool does not invalidate leases
        // retained by older GPUI scenes or in-flight consumer command buffers.
        if self.size != size {
            self.pool = Some(Self::create_pool(size)?);
            self.size = size;
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        let buffer = loop {
            self.cache.flush(0);
            if let Some(buffer) = self.acquire()? {
                break buffer;
            }
            Self::yield_and_poll(device, errors, deadline).await?;
        };
        let y = self.plane(
            device,
            &buffer,
            0,
            MTLPixelFormat::R8Unorm,
            wgpu::TextureFormat::R8Unorm,
        )?;
        let chroma = self.plane(
            device,
            &buffer,
            1,
            MTLPixelFormat::RG8Unorm,
            wgpu::TextureFormat::Rg8Unorm,
        )?;
        self.cache.flush(0);
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("RGBA to NV12"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source),
            }],
        });
        for (view, pipeline) in [(&y, &self.y_pipeline), (&chroma, &self.chroma_pipeline)] {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("NV12 plane"),
                color_attachments: &[Some(color_attachment(view, None))],
                ..Default::default()
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
        let complete = Arc::new(AtomicBool::new(false));
        let signal = complete.clone();
        queue.on_submitted_work_done(move || signal.store(true, Ordering::Release));
        let deadline = Instant::now() + Duration::from_secs(5);
        while !complete.load(Ordering::Acquire) {
            Self::yield_and_poll(device, errors, deadline).await?;
        }
        if let Some(message) = &*errors.lock().unwrap() {
            return Err(error(message));
        }
        Ok(Frame(buffer))
    }
    async fn yield_and_poll(
        device: &wgpu::Device,
        errors: &Arc<Mutex<Option<String>>>,
        deadline: Instant,
    ) -> RenderResult<()> {
        // A timer wakes the worker; even pool starvation is bounded. No Metal
        // polling, allocation or driver teardown occurs on the UI thread.
        futures_timer::Delay::new(Duration::from_millis(2)).await;
        device.poll(wgpu::PollType::Poll).map_err(error)?;
        if let Some(message) = &*errors.lock().unwrap() {
            return Err(error(message));
        }
        if Instant::now() >= deadline {
            return Err(error("surface completion/reuse timed out"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Readback is deliberately confined to this opt-in hardware regression.
    fn first_pixel(frame: &Frame) -> [u8; 3] {
        unsafe {
            status(CVPixelBufferLockBaseAddress(
                &frame.0,
                CVPixelBufferLockFlags::ReadOnly,
            ))
            .unwrap();
            let y = CVPixelBufferGetBaseAddressOfPlane(&frame.0, 0).cast::<u8>();
            let uv = CVPixelBufferGetBaseAddressOfPlane(&frame.0, 1).cast::<u8>();
            let pixel = [*y, *uv, *uv.add(1)];
            status(CVPixelBufferUnlockBaseAddress(
                &frame.0,
                CVPixelBufferLockFlags::ReadOnly,
            ))
            .unwrap();
            pixel
        }
    }

    #[test]
    #[ignore = "requires a real macOS Metal device"]
    fn metal_planes_completion_leases_and_resize() {
        objc2::rc::autoreleasepool(|_| {
            pollster::block_on(async {
                let id = objc2_metal::MTLCreateSystemDefaultDevice()
                    .unwrap()
                    .registryID();
                let mut viewport = Viewport::new(id).await.unwrap();
                let camera = Camera {
                    eye: [3.; 3],
                    target: [0.; 3],
                    up: [0., 1., 0.],
                    projection: Projection::Orthographic { height: 4. },
                };
                let bounds = Region::new([0; 3], [1; 3]).unwrap();
                let mesh = ChunkMeshes::default();
                let style = Style {
                    background: [1., 0., 0., 1.],
                    outline: false,
                    grid: false,
                    grid_color: [0.; 4],
                };
                let mut frames = Vec::new();
                for _ in 0..3 {
                    frames.push(
                        viewport
                            .render(bounds, camera, [17, 11], &mesh, &[], style)
                            .await
                            .unwrap(),
                    );
                }
                assert_eq!(CVPixelBufferGetWidth(&frames[0].0), 34);
                assert_eq!(CVPixelBufferGetHeight(&frames[0].0), 22);
                assert_eq!(CVPixelBufferGetWidthOfPlane(&frames[0].0, 1), 17);
                assert_eq!(CVPixelBufferGetHeightOfPlane(&frames[0].0, 1), 11);
                assert_eq!(first_pixel(&frames[0]), [76, 84, 255]);
                viewport.presenter.cache.flush(0);
                assert!(viewport.presenter.acquire().unwrap().is_none());

                // A scene/consumer clone must prevent reuse even after Frame drops.
                let lease = frames.remove(0).pixel_buffer();
                viewport.device.poll(wgpu::PollType::Poll).unwrap();
                viewport.presenter.cache.flush(0);
                assert!(viewport.presenter.acquire().unwrap().is_none());
                drop(lease);
                let deadline = Instant::now() + Duration::from_secs(5);
                loop {
                    viewport.presenter.cache.flush(0);
                    if viewport.presenter.acquire().unwrap().is_some() {
                        break;
                    }
                    Surfaces::yield_and_poll(&viewport.device, &viewport.errors, deadline)
                        .await
                        .unwrap();
                }
                let resized = viewport
                    .render(
                        bounds,
                        camera,
                        [23, 13],
                        &mesh,
                        &[],
                        Style {
                            background: [0., 0., 1., 1.],
                            ..style
                        },
                    )
                    .await
                    .unwrap();
                assert_eq!(CVPixelBufferGetWidth(&resized.0), 46);
                assert_eq!(first_pixel(&resized), [29, 255, 107]);
                // Old-size allocations stay intact through a pool replacement.
                assert_eq!(first_pixel(&frames[0]), [76, 84, 255]);
                assert_eq!(CVPixelBufferGetWidth(&frames[0].0), 34);
            })
        });
    }
}
