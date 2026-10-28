//! GPU render targets, attachments and pipeline construction.
use super::*;

pub(super) fn texture_entry(
    binding: u32,
    sample_type: wgpu::TextureSampleType,
) -> wgpu::BindGroupLayoutEntry {
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
pub(super) fn target(
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
    pub(super) fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        size: [u32; 2],
        copy_src: bool,
        transient: bool,
    ) -> Self {
        // The multisampled attachments are only resolved, never read by a later
        // pass, so they can be discarded after the main pass (tile-memory only).
        let ms_usage = if transient {
            wgpu::TextureUsages::TRANSIENT_ATTACHMENT
        } else {
            wgpu::TextureUsages::empty()
        };
        let color_ms = target(
            device,
            size,
            wgpu::TextureFormat::Rgba8Unorm,
            SAMPLES,
            ms_usage,
            "MSAA color",
        );
        let geometry_ms = target(
            device,
            size,
            wgpu::TextureFormat::Rgba16Float,
            SAMPLES,
            ms_usage,
            "MSAA normals/depth",
        );
        let depth = target(
            device,
            size,
            wgpu::TextureFormat::Depth24Plus,
            SAMPLES,
            ms_usage,
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
            wgpu::TextureUsages::TEXTURE_BINDING
                | if copy_src {
                    wgpu::TextureUsages::COPY_SRC
                } else {
                    wgpu::TextureUsages::empty()
                },
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
            copy_src,
            size,
            transient,
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
pub(super) fn color_attachment<'a>(
    view: &'a wgpu::TextureView,
    resolve_target: Option<&'a wgpu::TextureView>,
    discard: bool,
) -> wgpu::RenderPassColorAttachment<'a> {
    wgpu::RenderPassColorAttachment {
        view,
        resolve_target,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            store: store_op(discard),
        },
        depth_slice: None,
    }
}
pub(super) fn store_op(discard: bool) -> wgpu::StoreOp {
    if discard {
        wgpu::StoreOp::Discard
    } else {
        wgpu::StoreOp::Store
    }
}
pub(super) fn depth_attachment(
    view: &wgpu::TextureView,
    discard: bool,
) -> wgpu::RenderPassDepthStencilAttachment<'_> {
    wgpu::RenderPassDepthStencilAttachment {
        view,
        depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.),
            store: store_op(discard),
        }),
        stencil_ops: None,
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn pipeline(
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
