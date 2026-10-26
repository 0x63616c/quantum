//! Editor scheduling for the platform surface viewport; Linux keeps software rendering.
#[cfg(target_os = "macos")]
use super::*;

pub(super) struct State {
    pub enabled: bool,
    #[cfg(target_os = "macos")]
    pub renderer: Option<voxelite_render::gpu::Worker>,
    #[cfg(target_os = "macos")]
    pub frame: Option<voxelite_render::gpu::Frame>,
    // Pixel-buffer pool addresses and scene identities can both be reused.
    // Advance only when installing new surface content, never on UI repaints.
    #[cfg(target_os = "macos")]
    pub frame_serial: usize,
    #[cfg(target_os = "macos")]
    pub(super) content: Option<(u64, u64, Option<voxelite_app::history::PreviewSource>)>,
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
            frame_serial: 0,
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
    pub(in crate::ui) fn monitor_gpu(cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(250))
                    .await;
                let keep_running = this.update(cx, |this, cx| {
                    if let Some(error) = gpui_apple::metal_renderer::take_surface_error() {
                        this.gpu_fallback(voxelite_render::RenderError::Invalid(error), cx);
                    }
                    if let Some(renderer) = &this.viewport.gpu.renderer
                        && let Err(error) = renderer.check_health()
                    {
                        this.gpu_fallback(error, cx);
                    }
                    this.viewport.gpu.enabled
                });
                if !matches!(keep_running, Ok(true)) {
                    break;
                }
            }
        })
        .detach();
    }
    fn gpu_fallback(&mut self, error: voxelite_render::RenderError, cx: &mut Context<Self>) {
        self.viewport.fallback_to_cpu();
        self.status = format!("GPU viewport unavailable; using CPU: {error}").into();
        cx.notify();
    }

    pub(super) fn request_gpu_render(&mut self, cx: &mut Context<Self>) {
        if self.viewport.render_size.contains(&0) {
            return;
        }
        if !self.viewport.begin_render(cx.background_executor().now()) {
            return;
        }
        let identity = self.frame_identity();
        let content = (identity.generation, identity.revision, identity.preview);
        let changed = self.viewport.gpu.content != Some(content);
        let bounds = self.editor.display_snapshot().bounds;
        let snapshot = changed.then(|| self.editor.display_snapshot().clone());
        let mut renderer = self.viewport.gpu.renderer.take();
        let background = if self.theme == Theme::Light { 1. } else { 0. };
        let grid_color: Rgba = self.theme.line().into();
        let style = voxelite_render::gpu::Style {
            background: [background, background, background, 1.],
            outline: RenderOptions::default().outline,
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
                this.viewport.rendering = false;
                // Consumer import failure can disable the GPU while a producer
                // request is outstanding. Never reinstall that worker or frame.
                if !this.viewport.gpu.enabled {
                    cx.notify();
                    return;
                }
                match result {
                    Ok(frame) => {
                        this.viewport.gpu.renderer = renderer;
                        this.viewport.gpu.content = Some(content);
                        // Camera motion may queue a newer frame; keep completed frames
                        // flowing, but never install a different document/preview.
                        if this.viewport.accepts(identity, &this.editor) {
                            this.viewport.display_camera = identity.camera;
                            this.viewport.display_size = identity.size;
                            this.viewport.displayed = Some(identity);
                            this.viewport.image = None;
                            this.viewport.gpu.frame_serial =
                                this.viewport.gpu.frame_serial.wrapping_add(1);
                            this.viewport.gpu.frame = Some(frame);
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
