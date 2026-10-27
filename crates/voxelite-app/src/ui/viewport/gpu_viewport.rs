//! Editor scheduling for the platform surface viewport; Linux keeps software rendering.
#[cfg(target_os = "macos")]
use super::*;

pub(super) struct State {
    pub enabled: bool,
    // A surface-size failure only suspends GPU rendering at this physical size.
    pub limited_size: Option<[u32; 2]>,
    pub stats: Option<voxelite_render::diagnostics::FrameStats>,
    diagnostics_epoch: u64,
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
            limited_size: None,
            stats: None,
            diagnostics_epoch: 0,
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
    pub fn clear_diagnostics(&mut self) {
        self.stats = None;
        self.diagnostics_epoch = self.diagnostics_epoch.wrapping_add(1);
    }
    pub fn clear_frame(&mut self) {
        self.clear_diagnostics();
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
                        this.gpu_fallback(
                            voxelite_render::RenderError::Invalid(error),
                            this.viewport.render_size,
                            cx,
                        );
                    }
                    if let Some(renderer) = &this.viewport.gpu.renderer
                        && let Err(error) = renderer.check_health()
                    {
                        this.gpu_fallback(error, this.viewport.render_size, cx);
                    }
                    this.viewport.gpu.enabled || this.viewport.gpu.limited_size.is_some()
                });
                if !matches!(keep_running, Ok(true)) {
                    break;
                }
            }
        })
        .detach();
    }
    fn gpu_fallback(
        &mut self,
        error: voxelite_render::RenderError,
        size: [u32; 2],
        cx: &mut Context<Self>,
    ) {
        self.viewport.fallback_to_cpu(&error, size);
        self.status = if self.viewport.gpu.limited_size.is_some() {
            // Describe the failed request so this remains true after a resize recovers.
            format!(
                "GPU viewport could not render at {}×{} pixels: {error}",
                size[0], size[1]
            )
        } else {
            format!("GPU viewport unavailable; using CPU: {error}")
        }
        .into();
        cx.notify();
    }

    pub(super) fn request_gpu_render(&mut self, cx: &mut Context<Self>) {
        if self.viewport.render_size.contains(&0) {
            return;
        }
        self.viewport.sync_scene_grid(&self.editor);
        if !self.viewport.begin_render(cx.background_executor().now()) {
            return;
        }
        let identity = self.frame_identity();
        let content = (identity.generation, identity.revision, identity.preview);
        let changed = self.viewport.gpu.content != Some(content);
        let bounds = self.editor.display_snapshot().bounds;
        let snapshot = changed.then(|| self.editor.display_snapshot().clone());
        let mut renderer = self.viewport.gpu.renderer.take();
        let diagnostics = self.preferences.show_debug_overlay;
        let diagnostics_epoch = self.viewport.gpu.diagnostics_epoch;
        let background = if self.theme == Theme::Light { 1. } else { 0. };
        let grid = self.viewport.scene_grid;
        let grid_color: Rgba = grid
            .map_or(self.theme.line(), |grid| {
                overlays::color(self.theme, grid.color)
            })
            .into();
        let style = voxelite_render::gpu::Style {
            lighting: identity.lighting,
            background: [background, background, background, 1.],
            outline: RenderOptions::default().outline,
            ambient_occlusion_strength: self.viewport.ambient_occlusion_strength,
            grid: grid.map(|grid| {
                [
                    grid.plane as f32 + grid.offset as f32,
                    grid.min as f32,
                    grid.max as f32,
                    grid.spacing as f32,
                ]
            }),
            grid_color: [
                grid_color.r,
                grid_color.g,
                grid_color.b,
                grid.map_or(0., |g| g.opacity),
            ],
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
                renderer.as_mut().unwrap().set_diagnostics(diagnostics);
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
                        this.viewport.gpu.content = Some(content);
                        // Camera motion may queue a newer frame; keep completed frames
                        // flowing, but never install a different document/preview.
                        if this.viewport.accepts(identity, &this.editor) {
                            if this.preferences.show_debug_overlay
                                && this.viewport.gpu.diagnostics_epoch == diagnostics_epoch
                            {
                                this.viewport.gpu.stats =
                                    renderer.as_ref().and_then(|r| r.last_stats());
                            }
                            this.viewport.display_camera = identity.camera;
                            this.viewport.display_size = identity.size;
                            this.viewport.displayed = Some(identity);
                            this.viewport.image = None;
                            this.viewport.gpu.frame_serial =
                                this.viewport.gpu.frame_serial.wrapping_add(1);
                            this.viewport.gpu.frame = Some(frame);
                        }
                        this.viewport.gpu.renderer = renderer;
                    }
                    Err(error) => {
                        this.gpu_fallback(error, identity.size, cx);
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
