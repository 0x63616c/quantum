//! Editor scheduling for the platform surface viewport; Linux keeps software rendering.
#[cfg(target_os = "macos")]
use super::*;

pub(super) struct State {
    pub enabled: bool,
    #[cfg(target_os = "macos")]
    pub renderer: Option<voxelite_render::gpu::Worker>,
    #[cfg(target_os = "macos")]
    pub frame: Option<voxelite_render::gpu::Frame>,
    #[cfg(target_os = "macos")]
    content: Option<(u64, u64, Option<voxelite_app::history::PreviewSource>)>,
}
impl State {
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        let enabled = std::env::var("VOXELITE_CPU_VIEWPORT").as_deref() != Ok("1");
        #[cfg(not(target_os = "macos"))]
        let enabled = false;
        Self {
            enabled,
            #[cfg(target_os = "macos")]
            renderer: None,
            #[cfg(target_os = "macos")]
            frame: None,
            #[cfg(target_os = "macos")]
            content: None,
        }
    }
    pub fn has_frame(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.frame.is_some()
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }
    pub fn clear_frame(&mut self) {
        #[cfg(target_os = "macos")]
        {
            self.frame = None;
        }
    }
}

#[cfg(target_os = "macos")]
impl EditorView {
    pub(super) fn monitor_gpu(cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(250))
                    .await;
                let keep_running = this.update(cx, |this, cx| {
                    if let Some(error) = gpui_apple::metal_renderer::take_surface_error() {
                        this.gpu_fallback(voxelite_render::RenderError::Invalid(error), cx);
                    }
                    if let Some(renderer) = &this.gpu.renderer
                        && let Err(error) = renderer.check_health()
                    {
                        this.gpu_fallback(error, cx);
                    }
                    this.gpu.enabled
                });
                if !matches!(keep_running, Ok(true)) {
                    break;
                }
            }
        })
        .detach();
    }
    fn gpu_fallback(&mut self, error: voxelite_render::RenderError, cx: &mut Context<Self>) {
        self.gpu.enabled = false;
        self.gpu.clear_frame();
        self.gpu.renderer = None;
        self.gpu.content = None;
        self.displayed = None;
        self.painted_frame.set(None);
        self.interactive_render = false;
        self.render_size = [0, 0]; // Recompute the CPU budget at next layout.
        self.status = format!("GPU viewport unavailable; using CPU: {error}").into();
        self.render_pending = true;
        cx.notify();
    }

    pub(super) fn request_gpu_render(&mut self, cx: &mut Context<Self>) {
        if self.render_size.contains(&0) {
            return;
        }
        if self.rendering {
            self.render_pending = true;
            return;
        }
        self.rendering = true;
        self.render_pending = false;
        let identity = self.frame_identity();
        let content = (identity.generation, identity.revision, identity.preview);
        let changed = self.gpu.content != Some(content);
        let bounds = self.editor.display_snapshot().bounds;
        let snapshot = changed.then(|| self.editor.display_snapshot().clone());
        let mut renderer = self.gpu.renderer.take();
        let background = if self.theme == Theme::Light { 1. } else { 0. };
        let grid_color: Rgba = self.theme.line().into();
        let style = voxelite_render::gpu::Style {
            background: [background, background, background, 1.],
            outline: true,
            grid: self.grid,
            grid_color: [grid_color.r, grid_color.g, grid_color.b, 0.24],
        };
        cx.spawn(async move |this, cx| {
            let result = async {
                if renderer.is_none() {
                    let registry_id = gpui_apple::metal_renderer::surface_device_registry_id()
                        .ok_or_else(|| {
                            voxelite_render::RenderError::Invalid(
                                "GPUI Metal device unavailable".into(),
                            )
                        })?;
                    renderer = Some(voxelite_render::gpu::Worker::new(registry_id)?);
                }
                renderer
                    .as_mut()
                    .unwrap()
                    .render(
                        snapshot,
                        bounds,
                        identity.camera.camera(),
                        identity.size,
                        style,
                    )
                    .await
            }
            .await;
            let _ = this.update(cx, |this, cx| {
                this.rendering = false;
                // Consumer import failure can disable the GPU while a producer
                // request is outstanding. Never reinstall that worker or frame.
                if !this.gpu.enabled {
                    cx.notify();
                    return;
                }
                match result {
                    Ok(frame) => {
                        this.gpu.renderer = renderer;
                        this.gpu.content = Some(content);
                        // Camera motion may queue a newer frame; keep completed frames
                        // flowing, but never install a different document/preview.
                        if identity.generation == this.document_generation
                            && identity.revision == this.editor.engine.revision()
                            && identity.preview == this.frame_identity().preview
                            && identity.size == this.render_size
                        {
                            this.display_camera = identity.camera;
                            this.display_size = identity.size;
                            this.displayed = Some(identity);
                            this.image = None;
                            this.gpu.frame = Some(frame);
                        }
                    }
                    Err(error) => {
                        this.gpu_fallback(error, cx);
                    }
                }
                // Multiple input events coalesce to the latest camera at layout.
                // CoreVideo owns reuse; paint is not a GPU completion fence.
                cx.notify();
            });
        })
        .detach();
    }
}
