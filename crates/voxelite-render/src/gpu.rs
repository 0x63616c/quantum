//! Full-resolution wgpu viewport: persistent chunks, vertex AO, fitted shadows,
//! four-sample MSAA and shader outlines. CPU rendering remains for exports.
//!
//! Presentation stays on the GPU: IOSurface planes consumed by GPUI on macOS.
use crate::shadow::{SHADOW_SIZE, ShadowCache, ShadowInput};
use crate::{Camera, ChunkMeshes, MeshVertex, Projection, RenderError, RenderResult};
use glam::{Mat4, Vec3};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use voxelite_core::{Pos, Region};
use wgpu::util::DeviceExt;

#[allow(unsafe_code)]
mod mac;
pub use mac::{Frame, Worker};

/// Appearance is composited before presentation (NV12 has no alpha channel).
#[derive(Clone, Copy)]
pub struct Style {
    /// Opaque sRGB background, with alpha ignored.
    pub background: [f32; 4],
    /// Draw voxel edge outlines.
    pub outline: bool,
    /// Ground grid, composited behind the model.
    pub grid: bool,
    /// sRGB grid color and opacity.
    pub grid_color: [f32; 4],
}

const SAMPLES: u32 = 4;
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
    background: [f32; 4],
    inverse_projection: [[f32; 4]; 4],
    grid: [f32; 4],
    grid_color: [f32; 4],
    shadow_settings: [f32; 4],
}
struct ChunkBuffer {
    buffer: wgpu::Buffer,
    vertices: u32,
}
struct Target {
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
}
/// Persistent GPU resources for one editor viewport. Camera updates refit shadows;
/// edits replace the buffers returned by the chunk invalidation cache.
struct Viewport {
    device: wgpu::Device,
    errors: Arc<Mutex<Option<String>>>,
    queue: wgpu::Queue,
    buffers: BTreeMap<Pos, ChunkBuffer>,
    uniform: wgpu::Buffer,
    globals: wgpu::BindGroup,
    shadow: Target,
    shadow_cache: ShadowCache,
    shadow_bind: wgpu::BindGroup,
    main_pipeline: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    resolve_pipeline: wgpu::RenderPipeline,
    resolve_layout: wgpu::BindGroupLayout,
    targets: Option<Targets>,
    presenter: mac::Surfaces,
}
impl Viewport {
    /// Initialize Metal on the viewport worker. Failure permits CPU fallback.
    async fn new(registry_id: u64) -> RenderResult<Self> {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::METAL;
        let instance = wgpu::Instance::new(descriptor);
        let adapter = mac::adapter(&instance, registry_id).await?;
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
                required_limits: wgpu::Limits::default()
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
            *sink.lock().unwrap() = Some(format!("device lost ({reason:?}): {message}"));
        });
        let message = format!(
            "Voxelite GPU viewport: {:?} / {} / full resolution / 4x MSAA / {SHADOW_SIZE} fitted shadows",
            adapter.get_info().backend,
            adapter.get_info().name
        );
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
            label: Some("filtered directional shadow"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
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
        let presenter = mac::Surfaces::new(&device)?;
        if let Some(e) = scope.pop().await {
            return Err(error(e));
        }
        Ok(Self {
            device,
            errors,
            queue,
            buffers: BTreeMap::new(),
            uniform,
            globals,
            shadow,
            shadow_cache: ShadowCache::default(),
            shadow_bind,
            main_pipeline,
            shadow_pipeline,
            resolve_pipeline,
            resolve_layout,
            targets: None,
            presenter,
        })
    }
    /// Check asynchronous validation/device-loss errors even while the view is idle.
    fn check_health(&self) -> RenderResult<()> {
        self.device.poll(wgpu::PollType::Poll).map_err(error)?;
        self.check_error()
    }
    fn check_error(&self) -> RenderResult<()> {
        if let Some(message) = &*self.errors.lock().unwrap() {
            Err(error(message))
        } else {
            Ok(())
        }
    }
    /// Render an engine-validated snapshot using prepared chunk changes.
    /// Meshing may run on a worker; camera motion supplies an empty update list.
    async fn render(
        &mut self,
        bounds: Region,
        camera: Camera,
        size: [u32; 2],
        mesh: &ChunkMeshes,
        updates: &[Pos],
        style: Style,
    ) -> RenderResult<Frame> {
        self.check_error()?;
        camera.validate()?;
        // Render the scene at 2x *device* dimensions, then chroma subsample.
        // Budget the actual MSAA/resolve/pool allocations, never silently reduce
        // moving-frame quality. Oversized viewports use the software fallback.
        let size = super::surface_size(size, self.device.limits().max_texture_dimension_2d)?;
        {
            for &at in updates {
                if let Some(mesh) = mesh.meshes.get(&at) {
                    let data = bytemuck::cast_slice(&mesh.vertices);
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
        }
        if self.targets.as_ref().is_none_or(|t| t.size != size) {
            self.targets = None; // Retire the old allocation before creating its replacement.
            self.device.poll(wgpu::PollType::Poll).map_err(error)?;
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
        let view_projection = projection * Mat4::look_at_rh(eye, center, up);
        let redraw_shadow = self.shadow_cache.update(
            ShadowInput {
                bounds,
                view_projection,
                light,
            },
            !updates.is_empty(),
        );
        let shadow_fit = self.shadow_cache.fit;
        let globals = Globals {
            view_projection: view_projection.to_cols_array_2d(),
            light_projection: shadow_fit
                .map_or(Mat4::IDENTITY, |fit| fit.projection)
                .to_cols_array_2d(),
            light: light.extend(0.).to_array(),
            eye: eye.extend(0.).to_array(),
            forward: forward.extend(0.).to_array(),
            settings: [
                size[0] as f32,
                size[1] as f32,
                f32::from(shadow_fit.is_some()),
                f32::from(style.outline),
            ],
            background: style.background,
            inverse_projection: view_projection.inverse().to_cols_array_2d(),
            grid: [
                bounds.min[1] as f32 - 0.05,
                bounds.min[0].min(bounds.min[2]) as f32 - 16.,
                bounds.max[0].max(bounds.max[2]) as f32 + 16.,
                f32::from(style.grid),
            ],
            grid_color: style.grid_color,
            shadow_settings: [
                1.0 / SHADOW_SIZE as f32,
                shadow_fit.map_or(0.0, |fit| fit.normal_offset),
                0.0,
                0.0,
            ],
        };

        self.queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&globals));
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("voxel viewport"),
            });
        if redraw_shadow {
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
        {
            self.presenter
                .present(
                    &self.device,
                    &self.queue,
                    encoder,
                    size,
                    &target.output.view,
                    &self.errors,
                )
                .await
        }
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
    Target { view }
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
            wgpu::TextureUsages::TEXTURE_BINDING,
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
        Self {
            size,
            color_ms,
            geometry_ms,
            depth,
            color,
            geometry,
            output,
            resolve,
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
            bias: if vs == "vs_shadow" {
                wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 1.5,
                    clamp: 0.0,
                }
            } else {
                Default::default()
            },
        }),
        multisample: wgpu::MultisampleState {
            count: samples,
            ..Default::default()
        },
        multiview_mask: None,
        cache: None,
    })
}
