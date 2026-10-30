//! Full-resolution wgpu viewport: persistent chunks, vertex AO, hard shadows,
//! four-sample MSAA and shader outlines. CPU rendering remains for exports.
//!
//! GPUI 1.25 has no portable external-texture element. This implementation returns
//! an asynchronously read-back image for that compositor; no CPU raster is involved.
use crate::{
    Camera, ChunkMeshes, MeshVertex, Projection, RenderError, RenderResult, RenderedImage, perf,
};
use glam::{Mat4, Vec3};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use voxelite_core::{Pos, Snapshot};
use wgpu::util::DeviceExt;

const SHADOW_SIZE: u32 = 2048;
const SAMPLES: u32 = 4;
/// Adapter choice. Auto prefers WebGPU in browsers, Metal on macOS.
#[derive(Clone, Copy, Default)]
pub enum Backend {
    #[default]
    /// Prefer WebGPU in browsers and Metal on macOS.
    Auto,
    /// Require WebGL2 in browsers.
    WebGl,
    /// Require WebGPU in browsers.
    WebGpu,
}
fn error(e: impl std::fmt::Display) -> RenderError {
    RenderError::Invalid(format!("GPU viewport: {e}"))
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    view_projection: [[f32; 4]; 4],
    light_projection: [[f32; 4]; 4],
    light: [f32; 4],
    eye: [f32; 4],
    forward: [f32; 4],
    settings: [f32; 4],
}
struct ChunkBuffer {
    buffer: wgpu::Buffer,
    vertices: u32,
}
struct Target {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}
struct Targets {
    size: [u32; 2],
    color_ms: Target,
    geometry_ms: Target,
    depth: Target,
    color: Target,
    geometry: Target,
    output: Target,
    resolve: wgpu::BindGroup,
    readback: wgpu::Buffer,
    row_bytes: u32,
}
/// Persistent GPU resources for one editor viewport. Camera updates only change
/// uniforms; edits replace the buffers returned by the chunk invalidation cache.
pub struct Viewport {
    device: wgpu::Device,
    errors: Arc<Mutex<Option<String>>>,
    queue: wgpu::Queue,
    mesh: ChunkMeshes,
    buffers: BTreeMap<Pos, ChunkBuffer>,
    uniform: wgpu::Buffer,
    globals: wgpu::BindGroup,
    shadow: Target,
    shadow_bind: wgpu::BindGroup,
    main_pipeline: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    resolve_pipeline: wgpu::RenderPipeline,
    resolve_layout: wgpu::BindGroupLayout,
    targets: Option<Targets>,
    #[cfg(target_family = "wasm")]
    _surface: wgpu::Surface<'static>,
    #[cfg(target_family = "wasm")]
    browser_poll: bool,
}
#[cfg(target_family = "wasm")]
#[derive(Debug)]
struct WebDisplay;
#[cfg(target_family = "wasm")]
impl raw_window_handle::HasDisplayHandle for WebDisplay {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        Ok(raw_window_handle::DisplayHandle::web())
    }
}
impl Viewport {
    /// Initialize an adapter asynchronously. Failure permits a full-quality CPU fallback.
    pub async fn new(backend: Backend) -> RenderResult<Self> {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        #[cfg(target_family = "wasm")]
        {
            descriptor.backends = match backend {
                Backend::Auto => wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
                Backend::WebGl => wgpu::Backends::GL,
                Backend::WebGpu => wgpu::Backends::BROWSER_WEBGPU,
            };
            descriptor.display = Some(Box::new(WebDisplay));
        }
        #[cfg(not(target_family = "wasm"))]
        {
            let _ = backend;
            descriptor.backends = if cfg!(target_os = "macos") {
                wgpu::Backends::METAL
            } else {
                wgpu::Backends::VULKAN | wgpu::Backends::GL
            };
        }
        #[cfg(target_family = "wasm")]
        let instance = wgpu::util::new_instance_with_webgpu_detection(descriptor).await;
        #[cfg(not(target_family = "wasm"))]
        let instance = wgpu::Instance::new(descriptor);
        #[cfg(target_family = "wasm")]
        let surface = {
            use wasm_bindgen::JsCast;
            let canvas: web_sys::HtmlCanvasElement = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("canvas")
                .map_err(|e| error(format!("{e:?}")))?
                .unchecked_into();
            instance
                .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
                .map_err(error)?
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                #[cfg(target_family = "wasm")]
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(error)?;
        // These formats/sample counts are required for every frame; never silently
        // lower resolution, disable AO/shadows, or drop AA during interaction.
        for format in [
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureFormat::Depth24Plus,
        ] {
            if !adapter
                .get_texture_format_features(format)
                .flags
                .sample_count_supported(SAMPLES)
            {
                return Err(error(format!("{format:?} lacks 4x MSAA")));
            }
        }
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Voxelite viewport"),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits())
                    .using_alignment(adapter.limits()),
                ..Default::default()
            })
            .await
            .map_err(error)?;
        let errors = Arc::new(Mutex::new(None));
        let sink = errors.clone();
        device.on_uncaptured_error(Arc::new(move |error| {
            *sink.lock().unwrap() = Some(error.to_string());
        }));
        let sink = errors.clone();
        device.set_device_lost_callback(move |reason, message| {
            if reason != wgpu::DeviceLostReason::Destroyed {
                *sink.lock().unwrap() = Some(message);
            }
        });
        let message = format!(
            "Voxelite GPU viewport: {:?} / {} / full resolution / 4x MSAA / 2048 shadows",
            adapter.get_info().backend,
            adapter.get_info().name
        );
        #[cfg(target_family = "wasm")]
        web_sys::console::info_1(&message.into());
        #[cfg(not(target_family = "wasm"))]
        eprintln!("{message}");
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewport globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let global_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewport globals"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let globals = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &global_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let shadow_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow input"),
            entries: &[
                texture_entry(0, wgpu::TextureSampleType::Depth),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });
        let shadow = target(
            &device,
            [SHADOW_SIZE; 2],
            wgpu::TextureFormat::Depth32Float,
            1,
            wgpu::TextureUsages::TEXTURE_BINDING,
            "shadow",
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("hard directional shadow"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let shadow_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &shadow_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadow.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let resolve_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("resolve input"),
            entries: &[
                texture_entry(0, wgpu::TextureSampleType::Float { filterable: false }),
                texture_entry(1, wgpu::TextureSampleType::Float { filterable: false }),
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("voxel shading"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gpu/viewport.wgsl").into()),
        });
        let resolve_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("linear resolve and outline"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gpu/resolve.wgsl").into()),
        });
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x4],
        };
        let main_pipeline = pipeline(
            &device,
            &shader,
            &[&global_layout, &shadow_layout],
            "vs_main",
            Some("fs_main"),
            std::slice::from_ref(&vertex_layout),
            &[
                wgpu::TextureFormat::Rgba8Unorm,
                wgpu::TextureFormat::Rgba16Float,
            ],
            Some(wgpu::TextureFormat::Depth24Plus),
            SAMPLES,
        );
        let shadow_pipeline = pipeline(
            &device,
            &shader,
            &[&global_layout],
            "vs_shadow",
            None,
            &[vertex_layout],
            &[],
            Some(wgpu::TextureFormat::Depth32Float),
            1,
        );
        let resolve_pipeline = pipeline(
            &device,
            &resolve_shader,
            &[&global_layout, &resolve_layout],
            "vs_main",
            Some("fs_main"),
            &[],
            &[wgpu::TextureFormat::Rgba8Unorm],
            None,
            1,
        );
        if let Some(e) = scope.pop().await {
            return Err(error(e));
        }
        Ok(Self {
            device,
            errors,
            queue,
            mesh: ChunkMeshes::default(),
            buffers: BTreeMap::new(),
            uniform,
            globals,
            shadow,
            shadow_bind,
            main_pipeline,
            shadow_pipeline,
            resolve_pipeline,
            resolve_layout,
            targets: None,
            #[cfg(target_family = "wasm")]
            _surface: surface,
            #[cfg(target_family = "wasm")]
            browser_poll: adapter.get_info().backend == wgpu::Backend::Gl,
        })
    }
    fn check_error(&self) -> RenderResult<()> {
        if let Some(message) = &*self.errors.lock().unwrap() {
            Err(error(message))
        } else {
            Ok(())
        }
    }
    /// Render an engine-validated snapshot. `content_changed` is independent of
    /// camera state, so navigating neither copies the scene nor rebuilds geometry.
    pub async fn render(
        &mut self,
        snapshot: &Snapshot,
        camera: Camera,
        size: [u32; 2],
        content_changed: bool,
        outline: bool,
    ) -> RenderResult<RenderedImage> {
        self.check_error()?;
        camera.validate()?;
        if size.contains(&0)
            || size
                .iter()
                .any(|s| *s > self.device.limits().max_texture_dimension_2d)
            || u64::from(size[0]) * u64::from(size[1]) > 16_777_216
        {
            return Err(error("viewport exceeds target budget"));
        }
        if content_changed {
            let updates = self.mesh.sync_validated(snapshot)?;
            let _span = perf::Span::new("gpu_buffer_update");
            let mut bytes = 0;
            for at in updates {
                if let Some(mesh) = self.mesh.meshes.get(&at) {
                    let data = bytemuck::cast_slice(&mesh.vertices);
                    bytes += data.len();
                    if let Some(old) = self
                        .buffers
                        .get_mut(&at)
                        .filter(|old| old.buffer.size() >= data.len() as u64)
                    {
                        self.queue.write_buffer(&old.buffer, 0, data);
                        old.vertices = mesh.vertices.len() as u32;
                    } else {
                        self.buffers.insert(
                            at,
                            ChunkBuffer {
                                buffer: self.device.create_buffer_init(
                                    &wgpu::util::BufferInitDescriptor {
                                        label: Some("voxel chunk"),
                                        contents: data,
                                        usage: wgpu::BufferUsages::VERTEX
                                            | wgpu::BufferUsages::COPY_DST,
                                    },
                                ),
                                vertices: mesh.vertices.len() as u32,
                            },
                        );
                    }
                } else {
                    self.buffers.remove(&at);
                }
            }
            perf::event("gpu_upload_bytes", bytes as f64, 0.);
        }
        if self.targets.as_ref().is_none_or(|t| t.size != size) {
            self.targets = Some(Targets::new(&self.device, &self.resolve_layout, size));
        }
        let target = self.targets.as_ref().unwrap();
        let eye = Vec3::from_array(camera.eye.map(|v| v as f32));
        let center = Vec3::from_array(camera.target.map(|v| v as f32));
        let up = Vec3::from_array(camera.up.map(|v| v as f32));
        let forward = (center - eye).normalize();
        let right = forward.cross(up).normalize();
        let up = right.cross(forward);
        let light = (-right * 0.55 + up * 0.7 - forward * 0.65 + Vec3::Y * 0.65).normalize();
        let aspect = size[0] as f32 / size[1] as f32;
        let projection = match camera.projection {
            Projection::Orthographic { height } => {
                let h = height as f32 * 0.5;
                Mat4::orthographic_rh(-h * aspect, h * aspect, -h, h, 0.001, 4096.)
            }
            Projection::Perspective { fov_y_degrees } => {
                Mat4::perspective_rh((fov_y_degrees as f32).to_radians(), aspect, 0.001, 4096.)
            }
        };
        let lo = Vec3::from_array(snapshot.bounds.min.map(|v| v as f32));
        let hi = Vec3::from_array(snapshot.bounds.max.map(|v| v as f32));
        let midpoint = (lo + hi) * 0.5;
        let radius = (hi - lo).length() * 0.5 + 2.;
        let light_view = Mat4::look_at_rh(
            midpoint + light * (radius * 2.),
            midpoint,
            if light.dot(Vec3::Y).abs() > 0.99 {
                Vec3::Z
            } else {
                Vec3::Y
            },
        );
        let light_projection =
            Mat4::orthographic_rh(-radius, radius, -radius, radius, 0.1, radius * 4.) * light_view;
        let globals = Globals {
            view_projection: (projection * Mat4::look_at_rh(eye, center, up)).to_cols_array_2d(),
            light_projection: light_projection.to_cols_array_2d(),
            light: light.extend(0.).to_array(),
            eye: eye.extend(0.).to_array(),
            forward: forward.extend(0.).to_array(),
            settings: [size[0] as f32, size[1] as f32, 1., f32::from(outline)],
        };
        let span = perf::Span::new("render_submission");
        self.queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&globals));
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("voxel viewport"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("voxel shadow"),
                color_attachments: &[],
                depth_stencil_attachment: Some(depth_attachment(&self.shadow.view)),
                ..Default::default()
            });
            pass.set_pipeline(&self.shadow_pipeline);
            pass.set_bind_group(0, &self.globals, &[]);
            for chunk in self.buffers.values() {
                pass.set_vertex_buffer(0, chunk.buffer.slice(..));
                pass.draw(0..chunk.vertices, 0..1);
            }
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("voxel MSAA"),
                color_attachments: &[
                    Some(color_attachment(
                        &target.color_ms.view,
                        Some(&target.color.view),
                    )),
                    Some(color_attachment(
                        &target.geometry_ms.view,
                        Some(&target.geometry.view),
                    )),
                ],
                depth_stencil_attachment: Some(depth_attachment(&target.depth.view)),
                ..Default::default()
            });
            pass.set_pipeline(&self.main_pipeline);
            pass.set_bind_group(0, &self.globals, &[]);
            pass.set_bind_group(1, &self.shadow_bind, &[]);
            for chunk in self.buffers.values() {
                pass.set_vertex_buffer(0, chunk.buffer.slice(..));
                pass.draw(0..chunk.vertices, 0..1);
            }
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("voxel sRGB resolve"),
                color_attachments: &[Some(color_attachment(&target.output.view, None))],
                ..Default::default()
            });
            pass.set_pipeline(&self.resolve_pipeline);
            pass.set_bind_group(0, &self.globals, &[]);
            pass.set_bind_group(1, &target.resolve, &[]);
            pass.draw(0..3, 0..1);
        }
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target.output.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &target.readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(target.row_bytes),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);
        drop(span);
        // The wait includes GPU execution and GPUI's required image readback; it
        // is deliberately separate from CPU render-submission timings.
        let readback_span = perf::Span::new("gpu_wait_readback");
        let (tx, rx) = futures_channel::oneshot::channel();
        target
            .readback
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
        #[cfg(not(target_family = "wasm"))]
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(error)?;
        #[cfg(target_family = "wasm")]
        if self.browser_poll {
            let mut rx = rx;
            loop {
                self.check_error()?;
                self.device.poll(wgpu::PollType::Poll).map_err(error)?;
                if let Some(result) = rx.try_recv().map_err(error)? {
                    result.map_err(error)?;
                    break;
                }
                // WebGL fences do not wake Rust futures by themselves. Yield the
                // event loop, then poll again; never spin or block browser input.
                wasm_bindgen_futures::JsFuture::from(yield_for_gpu())
                    .await
                    .map_err(|e| error(format!("{e:?}")))?;
            }
        } else {
            rx.await.map_err(error)?.map_err(error)?;
        }
        #[cfg(not(target_family = "wasm"))]
        rx.await.map_err(error)?.map_err(error)?;
        self.check_error()?;
        let data = match target.readback.slice(..).get_mapped_range() {
            Ok(data) => data,
            Err(e) => {
                target.readback.unmap();
                return Err(error(e));
            }
        };
        let mut rgba = Vec::with_capacity((size[0] * size[1] * 4) as usize);
        for row in data.chunks_exact(target.row_bytes as usize) {
            rgba.extend_from_slice(&row[..size[0] as usize * 4]);
        }
        drop(data);
        target.readback.unmap();
        drop(readback_span);
        Ok(RenderedImage {
            width: size[0],
            height: size[1],
            rgba,
            camera,
            quads: self.buffers.values().map(|b| b.vertices as usize / 6).sum(),
        })
    }
}
fn texture_entry(binding: u32, sample_type: wgpu::TextureSampleType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}
fn target(
    device: &wgpu::Device,
    size: [u32; 2],
    format: wgpu::TextureFormat,
    samples: u32,
    usage: wgpu::TextureUsages,
    label: &str,
) -> Target {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: samples,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | usage,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    Target { texture, view }
}
impl Targets {
    fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, size: [u32; 2]) -> Self {
        let color_ms = target(
            device,
            size,
            wgpu::TextureFormat::Rgba8Unorm,
            SAMPLES,
            wgpu::TextureUsages::empty(),
            "MSAA color",
        );
        let geometry_ms = target(
            device,
            size,
            wgpu::TextureFormat::Rgba16Float,
            SAMPLES,
            wgpu::TextureUsages::empty(),
            "MSAA normals/depth",
        );
        let depth = target(
            device,
            size,
            wgpu::TextureFormat::Depth24Plus,
            SAMPLES,
            wgpu::TextureUsages::empty(),
            "MSAA depth",
        );
        let color = target(
            device,
            size,
            wgpu::TextureFormat::Rgba8Unorm,
            1,
            wgpu::TextureUsages::TEXTURE_BINDING,
            "resolved color",
        );
        let geometry = target(
            device,
            size,
            wgpu::TextureFormat::Rgba16Float,
            1,
            wgpu::TextureUsages::TEXTURE_BINDING,
            "resolved normals/depth",
        );
        let output = target(
            device,
            size,
            wgpu::TextureFormat::Rgba8Unorm,
            1,
            wgpu::TextureUsages::COPY_SRC,
            "sRGB viewport",
        );
        let resolve = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("resolve"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&color.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&geometry.view),
                },
            ],
        });
        let row_bytes = (size[0] * 4).div_ceil(256) * 256;
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewport compositor transfer"),
            size: u64::from(row_bytes) * u64::from(size[1]),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            size,
            color_ms,
            geometry_ms,
            depth,
            color,
            geometry,
            output,
            resolve,
            readback,
            row_bytes,
        }
    }
}
fn color_attachment<'a>(
    view: &'a wgpu::TextureView,
    resolve_target: Option<&'a wgpu::TextureView>,
) -> wgpu::RenderPassColorAttachment<'a> {
    wgpu::RenderPassColorAttachment {
        view,
        resolve_target,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            store: wgpu::StoreOp::Store,
        },
        depth_slice: None,
    }
}
fn depth_attachment(view: &wgpu::TextureView) -> wgpu::RenderPassDepthStencilAttachment<'_> {
    wgpu::RenderPassDepthStencilAttachment {
        view,
        depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.),
            store: wgpu::StoreOp::Store,
        }),
        stencil_ops: None,
    }
}
#[allow(clippy::too_many_arguments)]
fn pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layouts: &[&wgpu::BindGroupLayout],
    vs: &str,
    fs: Option<&str>,
    vertices: &[wgpu::VertexBufferLayout<'_>],
    formats: &[wgpu::TextureFormat],
    depth: Option<wgpu::TextureFormat>,
    samples: u32,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(vs),
        bind_group_layouts: &layouts.iter().copied().map(Some).collect::<Vec<_>>(),
        immediate_size: 0,
    });
    let targets: Vec<_> = formats
        .iter()
        .map(|&format| {
            Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
        })
        .collect();
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(vs),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some(vs),
            compilation_options: Default::default(),
            buffers: &vertices.iter().cloned().map(Some).collect::<Vec<_>>(),
        },
        fragment: fs.map(|entry| wgpu::FragmentState {
            module: shader,
            entry_point: Some(entry),
            compilation_options: Default::default(),
            targets: &targets,
        }),
        primitive: wgpu::PrimitiveState {
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: depth.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: samples,
            ..Default::default()
        },
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function yield_for_gpu() { return new Promise(resolve => setTimeout(resolve, 1)); }"
)]
extern "C" {
    fn yield_for_gpu() -> js_sys::Promise;
}
