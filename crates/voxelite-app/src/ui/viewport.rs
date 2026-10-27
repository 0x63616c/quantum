//! Viewport frame ownership, scheduling, sizing and presentation.
//!
//! EditorView supplies the document and GPUI task callbacks; only this module
//! owns render state. Picking and playback use the same displayed-frame identity.
use super::*;
use voxelite_app::frame_meter::{FrameMeter, RendererKind};
mod gpu_viewport;

/// Image identity includes the editor incarnation, since reopening the same ID
/// resets its revision. Camera and dimensions bind picking to displayed pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
struct FrameIdentity {
    generation: u64,
    revision: u64,
    preview: Option<voxelite_app::history::PreviewSource>,
    camera: OrbitCamera,
    size: [u32; 2],
}

pub(super) struct Viewport {
    gpu: gpu_viewport::State,
    image: Option<Arc<RenderImage>>,
    painted_image: Option<Arc<RenderImage>>,
    painted_frame: Rc<Cell<Option<FrameIdentity>>>,
    rendering: bool,
    frame_meter: FrameMeter,
    render_pending: bool,
    render_size: [u32; 2],
    display_camera: OrbitCamera,
    document_generation: u64,
    displayed: Option<FrameIdentity>,
    display_size: [u32; 2],
    navigation_epoch: u64,
    interactive_render: bool,
    bounds: Rc<Cell<Bounds<Pixels>>>,
}

impl Viewport {
    pub(super) fn new(camera: OrbitCamera) -> Self {
        let gpu = gpu_viewport::State::new();
        let renderer = if gpu.enabled {
            RendererKind::Gpu
        } else {
            RendererKind::Cpu
        };
        Self {
            gpu,
            image: None,
            painted_image: None,
            painted_frame: Rc::new(Cell::new(None)),
            rendering: false,
            frame_meter: FrameMeter::new(renderer),
            render_pending: false,
            render_size: [0, 0],
            display_camera: camera,
            document_generation: 0,
            displayed: None,
            display_size: [0, 0],
            navigation_epoch: 0,
            interactive_render: false,
            bounds: Rc::new(Cell::new(Bounds::default())),
        }
    }

    pub(super) fn gpu_enabled(&self) -> bool {
        self.gpu.enabled
    }
    pub(super) fn bounds(&self) -> Bounds<Pixels> {
        self.bounds.get()
    }
    pub(super) fn display_camera(&self) -> OrbitCamera {
        self.display_camera
    }
    pub(super) fn frame_meter(&self) -> &FrameMeter {
        &self.frame_meter
    }
    pub(super) fn rendering(&self) -> bool {
        self.rendering
    }

    pub(super) fn is_current(&self, editor: &Editor) -> bool {
        self.displayed == Some(self.identity(editor))
    }

    pub(super) fn invalidate_preview(&mut self) {
        self.document_generation = self.document_generation.wrapping_add(1);
    }

    pub(super) fn replace_editor(&mut self, editor: &mut Editor) {
        self.frame_meter = FrameMeter::new(if self.gpu.enabled {
            RendererKind::Gpu
        } else {
            RendererKind::Cpu
        });
        self.invalidate_preview();
        self.image = None;
        self.gpu.clear_frame();
        self.displayed = None;
        let aspect = f64::from(self.render_size[0]) / f64::from(self.render_size[1].max(1));
        editor.camera = OrbitCamera::fit(editor.display_snapshot(), aspect.max(0.01));
        self.display_camera = editor.camera;
    }

    pub(super) fn ray(
        &self,
        editor: &Editor,
        p: Point<Pixels>,
    ) -> Option<voxelite_app::editor::Ray> {
        // Pointer edits must use the exact camera of the image the user can see.
        if self.rendering
            || self.interactive_render
            || (self.image.is_none() && !self.gpu.has_frame())
            || self.painted_frame.get() != self.displayed
            || self.displayed != Some(self.identity(editor))
        {
            return None;
        }
        let b = self.bounds.get();
        let w = f64::from(f32::from(b.size.width));
        let h = f64::from(f32::from(b.size.height));
        self.display_camera.ray(
            [
                f64::from(f32::from(p.x - b.origin.x)) / w,
                f64::from(f32::from(p.y - b.origin.y)) / h,
            ],
            f64::from(self.render_size[0]) / f64::from(self.render_size[1].max(1)),
        )
    }
    fn identity(&self, editor: &Editor) -> FrameIdentity {
        FrameIdentity {
            generation: self.document_generation,
            revision: editor.engine.revision(),
            preview: editor.preview.as_ref().map(|p| p.source),
            camera: editor.camera,
            size: self.render_size,
        }
    }

    // Camera motion may queue a newer frame; keep completed frames flowing,
    // but never install a different document, revision, preview or size.
    fn accepts(&self, identity: FrameIdentity, editor: &Editor) -> bool {
        let current = self.identity(editor);
        identity.generation == current.generation
            && identity.revision == current.revision
            && identity.preview == current.preview
            && identity.size == current.size
    }

    fn cpu_options(&self, camera: OrbitCamera, antialiasing: bool) -> RenderOptions {
        let interactive = self.interactive_render;
        RenderOptions {
            width: if interactive {
                self.render_size[0].div_ceil(2)
            } else {
                self.render_size[0]
            },
            height: if interactive {
                self.render_size[1].div_ceil(2)
            } else {
                self.render_size[1]
            },
            supersampling: if interactive || !antialiasing { 1 } else { 2 },
            view: View::Camera(camera.camera()),
            background: [0; 4],
            ..Default::default()
        }
    }

    fn begin_navigation(&mut self) -> u64 {
        self.navigation_epoch = self.navigation_epoch.wrapping_add(1);
        self.interactive_render = true;
        self.navigation_epoch
    }

    fn settle_navigation(&mut self, epoch: u64) -> bool {
        if self.navigation_epoch != epoch {
            return false;
        }
        self.interactive_render = false;
        true
    }

    fn begin_render(&mut self, at: web_time::Instant) -> bool {
        if self.rendering {
            self.render_pending = true;
            return false;
        }
        if !self.render_pending {
            self.frame_meter.begin_frame(at);
        }
        self.rendering = true;
        self.render_pending = false;
        true
    }

    #[cfg(any(target_os = "macos", test))]
    fn fallback_to_cpu(&mut self) {
        self.gpu.enabled = false;
        self.frame_meter = FrameMeter::new(RendererKind::Cpu);
        self.gpu.clear_frame();
        #[cfg(target_os = "macos")]
        {
            self.gpu.renderer = None;
            self.gpu.content = None;
        }
        self.displayed = None;
        self.painted_frame.set(None);
        self.interactive_render = false;
        self.render_size = [0, 0]; // Recompute the CPU budget at next layout.
        self.render_pending = true;
    }

    fn dimensions(&self, available: [f32; 2], scale_factor: f32) -> [u32; 2] {
        let actual = self.bounds.get().size;
        let (width, height) = if self.gpu.enabled && actual.width > px(0.) && actual.height > px(0.)
        {
            (f32::from(actual.width), f32::from(actual.height))
        } else {
            (available[0], available[1])
        };
        let scale = if self.gpu.enabled {
            scale_factor
        } else {
            (1024.0 / width.max(height)).min(1.0)
        };
        [
            ((width * scale).round() as u32).max(1),
            ((height * scale).round() as u32).max(1),
        ]
    }

    pub(super) fn release_retired_image(&mut self, window: &mut Window) {
        if self.image.as_ref().map(|i| i.id) != self.painted_image.as_ref().map(|i| i.id) {
            if let Some(previous) = self.painted_image.take() {
                let _ = window.drop_image(previous);
            }
            self.painted_image = self.image.clone();
        }
    }
}

impl EditorView {
    pub(super) fn request_render(&mut self, cx: &mut Context<Self>) {
        #[cfg(target_os = "macos")]
        if self.viewport.gpu.enabled {
            self.request_gpu_render(cx);
            return;
        }
        self.request_cpu_render(cx);
    }
    fn request_cpu_render(&mut self, cx: &mut Context<Self>) {
        if self.viewport.render_size[0] == 0 {
            return;
        }
        if !self.viewport.begin_render(cx.background_executor().now()) {
            return;
        }
        let snapshot = self.editor.display_snapshot().clone();
        let camera = self.editor.camera;
        let identity = self.frame_identity();
        let options = self
            .viewport
            .cpu_options(camera, self.preferences.antialiasing);
        let task = cx
            .background_executor()
            .spawn(self.editor.raster.task(snapshot, options));
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.viewport.rendering = false;
                // Never install a render from a replaced document or an older edit.
                if this.viewport.accepts(identity, &this.editor) {
                    match result {
                        Ok(mut frame) => {
                            for pixel in frame.rgba.as_chunks_mut::<4>().0 {
                                pixel.swap(0, 2);
                            }
                            this.viewport.display_size = [frame.width, frame.height];
                            let buffer =
                                image::RgbaImage::from_raw(frame.width, frame.height, frame.rgba)
                                    .expect("renderer dimensions");
                            this.viewport.display_camera = camera;
                            this.viewport.displayed = Some(identity);
                            this.viewport.image =
                                Some(Arc::new(RenderImage::new(vec![image::Frame::new(buffer)])));
                        }
                        Err(e) => {
                            this.pause_history(cx);
                            this.error = true;
                            this.status = e.to_string().into();
                        }
                    }
                }
                if this.viewport.render_pending {
                    this.request_render(cx);
                }
                cx.notify();
            });
        })
        .detach();
    }
    pub(super) fn request_navigation(&mut self, cx: &mut Context<Self>) {
        if self.viewport.gpu.enabled {
            self.request_render(cx);
            return;
        }
        let epoch = self.viewport.begin_navigation();
        self.request_render(cx);
        let timer = cx
            .background_executor()
            .timer(std::time::Duration::from_millis(180));
        cx.spawn(async move |this, cx| {
            timer.await;
            let _ = this.update(cx, |this, cx| {
                if this.viewport.settle_navigation(epoch) {
                    this.request_render(cx);
                }
            });
        })
        .detach();
    }
    fn frame_identity(&self) -> FrameIdentity {
        self.viewport.identity(&self.editor)
    }
    pub(super) fn layout_viewport(&mut self, window: &Window, cx: &mut Context<Self>) {
        let size = window.viewport_size();
        let width = (f32::from(size.width) - SIDEBAR_WIDTH - TOOL_RAIL_WIDTH).max(1.0);
        let height =
            (f32::from(size.height) - MENUBAR_HEIGHT - TOOLBAR_HEIGHT - STATUSBAR_HEIGHT).max(1.0);
        let dimensions = self
            .viewport
            .dimensions([width, height], window.scale_factor());
        if self.viewport.render_size != dimensions {
            if self.camera_fitted {
                self.editor.camera = OrbitCamera::fit_view(
                    self.editor.display_snapshot(),
                    f64::from(dimensions[0]) / f64::from(dimensions[1].max(1)),
                    match self.camera_view {
                        Some(Op::Front) => PresetView::Front,
                        Some(Op::Right) => PresetView::Right,
                        Some(Op::Top) => PresetView::Top,
                        _ => PresetView::Iso,
                    },
                );
                self.zoom_reference_span = self.editor.camera.span;
                if self.viewport.image.is_none() && !self.viewport.gpu.has_frame() {
                    self.viewport.display_camera = self.editor.camera;
                }
            }
            self.viewport.render_size = dimensions;
            self.request_render(cx);
        }
        if self.viewport.render_pending && !self.viewport.rendering {
            self.request_render(cx);
        }
    }

    pub(super) fn viewport_surface(&self, cx: &Context<Self>) -> impl IntoElement {
        let t = self.theme;
        let bounds_cell = self.viewport.bounds.clone();
        let image = self.viewport.image.clone();
        let generation = self.viewport.document_generation;
        let view = cx.weak_entity();
        #[cfg(target_os = "macos")]
        let gpu_frame = self
            .viewport
            .gpu
            .frame
            .clone()
            .map(|frame| (frame, self.viewport.gpu.frame_serial));
        let gpu_active = self.viewport.gpu.enabled;
        let entity = cx.weak_entity();
        let painted_frame = self.viewport.painted_frame.clone();
        let identity = self.viewport.displayed;
        let camera = self.viewport.display_camera;
        let frame_size = self.viewport.displayed.map(|_| self.viewport.display_size);
        let tool_region = self.editor.tool_preview(self.hover);
        let grid = self.grid;
        let doc_bounds = self.editor.display_snapshot().bounds;
        let comment_region = self
            .discussion
            .draft
            .as_ref()
            .or_else(|| {
                self.discussion
                    .selected
                    .and_then(|id| {
                        self.editor
                            .display_snapshot()
                            .comments
                            .iter()
                            .find(|c| c.id == id)
                    })
                    .map(|c| &c.anchor)
            })
            .and_then(|a| a.region().ok());
        canvas(
            move |bounds, window, _| {
                let previous = bounds_cell.replace(bounds);
                if previous != bounds {
                    window.on_next_frame(move |_, cx| {
                        let _ = entity.update(cx, |_, cx| cx.notify());
                    });
                }
            },
            move |bounds, _, window, cx| {
                let aspect = frame_size
                    .map(|s| f64::from(s[0]) / f64::from(s[1].max(1)))
                    .unwrap_or_else(|| {
                        f64::from(f32::from(bounds.size.width) / f32::from(bounds.size.height))
                    });
                let project = |p: DVec3| {
                    let uv = camera.project(p, aspect);
                    point(
                        bounds.origin.x + bounds.size.width * uv[0] as f32,
                        bounds.origin.y + bounds.size.height * uv[1] as f32,
                    )
                };
                if grid && !gpu_active {
                    let mut path = PathBuilder::stroke(px(0.6));
                    let y = f64::from(doc_bounds.min[1]) - 0.05;
                    let extent = doc_bounds.min[0].min(doc_bounds.min[2]) - 16;
                    let end = doc_bounds.max[0].max(doc_bounds.max[2]) + 16;
                    for a in (extent..=end).step_by(4) {
                        path.move_to(project(DVec3::new(f64::from(a), y, f64::from(extent))));
                        path.line_to(project(DVec3::new(f64::from(a), y, f64::from(end))));
                        path.move_to(project(DVec3::new(f64::from(extent), y, f64::from(a))));
                        path.line_to(project(DVec3::new(f64::from(end), y, f64::from(a))));
                    }
                    if let Ok(path) = path.build() {
                        window.paint_path(path, t.line().opacity(0.24));
                    }
                }
                let mut presented = None;
                let visible = bounds.intersect(&window.content_mask().bounds);
                if visible.size.width > px(0.0) && visible.size.height > px(0.0) {
                    #[cfg(target_os = "macos")]
                    if let Some((frame, serial)) = gpu_frame {
                        window.paint_surface(bounds, frame.pixel_buffer());
                        presented = Some((RendererKind::Gpu, serial));
                    }
                    if presented.is_none()
                        && let Some(image) = image
                    {
                        let frame_id = image.id.0;
                        if window
                            .paint_image(bounds, bounds, Corners::default(), image, 0, false)
                            .is_ok()
                        {
                            presented = Some((RendererKind::Cpu, frame_id));
                        }
                    }
                }
                painted_frame.set(presented.and(identity));
                if let Some((renderer, frame_id)) = presented {
                    let at = cx.background_executor().now();
                    // Count distinct content handed to GPUI, not completions or
                    // repeated paints. Notify only after drawing has finished.
                    window.defer(cx, move |_, cx| {
                        let _ = view.update(cx, |this, cx| {
                            if this.viewport.document_generation == generation
                                && (renderer != RendererKind::Gpu || this.viewport.gpu.enabled)
                                && this
                                    .viewport
                                    .frame_meter
                                    .record_presented(renderer, frame_id, at)
                            {
                                cx.notify();
                            }
                        });
                    });
                }
                if let Some(region) = comment_region.or(tool_region) {
                    let mut path = PathBuilder::stroke(px(1.4));
                    let (lo, hi) = (region.min, region.max);
                    for bits in 0..8 {
                        for a in 0..3 {
                            if bits & (1 << a) == 0 {
                                let p: [f64; 3] = std::array::from_fn(|j| {
                                    f64::from(if bits & (1 << j) == 0 { lo[j] } else { hi[j] })
                                });
                                let mut q = p;
                                q[a] = f64::from(hi[a]);
                                path.move_to(project(p.into()));
                                path.line_to(project(q.into()));
                            }
                        }
                    }
                    if let Ok(path) = path.build() {
                        window.paint_path(path, t.accent());
                    }
                }
            },
        )
        .size_full()
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameIdentity, Viewport};
    use gpui::{Bounds, Point, RenderImage, point, px, size};
    use std::sync::Arc;
    use voxelite_app::history::PreviewSource;
    use voxelite_app::{editor::Editor, frame_meter::RendererKind};

    fn fixture() -> (Viewport, Editor) {
        let editor = Editor::blank().unwrap();
        let mut viewport = Viewport::new(editor.camera);
        // Exercise both backend policies without consulting the environment or
        // creating a GPU worker, GPUI app or window.
        viewport.gpu.enabled = false;
        viewport.render_size = [801, 601];
        viewport.bounds.set(Bounds::new(
            point(px(10.), px(20.)),
            size(px(801.), px(601.)),
        ));
        (viewport, editor)
    }

    #[test]
    fn five_k_gpu_fallback_waits_for_cpu_sizing_and_rejects_old_frame() {
        let (mut viewport, editor) = fixture();
        viewport.gpu.enabled = true;
        viewport
            .bounds
            .set(Bounds::new(Point::default(), size(px(2560.), px(1440.))));
        viewport.render_size = viewport.dimensions([2500., 1400.], 2.);
        assert_eq!(viewport.render_size, [5120, 2880]);
        let old = viewport.identity(&editor);
        viewport.displayed = Some(old);
        viewport.painted_frame.set(Some(old));
        viewport.interactive_render = true;
        viewport.rendering = true; // Consumer failure while producer is in flight.
        viewport.fallback_to_cpu();
        assert!(!viewport.gpu_enabled());
        assert_eq!(viewport.frame_meter().renderer(), RendererKind::Cpu);
        assert_eq!(viewport.displayed, None);
        assert_eq!(viewport.painted_frame.get(), None);
        assert!(!viewport.interactive_render);
        assert_eq!(viewport.render_size, [0, 0]);
        assert!(viewport.render_pending);
        assert!(viewport.rendering()); // Outstanding completion still owns this flag.
        assert!(!viewport.accepts(old, &editor));
        // CPU uses the layout budget, not the previous physical surface size.
        viewport.render_size = viewport.dimensions([2500., 1400.], 2.);
        assert_eq!(viewport.render_size, [1024, 573]);
        assert!(!viewport.accepts(old, &editor));
        viewport.rendering = false;
        assert!(viewport.begin_render(web_time::Instant::now()));
        assert!(!viewport.render_pending);
    }

    #[test]
    fn dimensions_preserve_backend_scaling_rounding_and_minimums() {
        let (mut viewport, _) = fixture();
        assert_eq!(viewport.dimensions([800., 600.], 2.), [800, 600]);
        assert_eq!(viewport.dimensions([2000., 3000.], 2.), [683, 1024]);
        assert_eq!(viewport.dimensions([0.1, 0.1], 2.), [1, 1]);
        viewport.gpu.enabled = true;
        assert_eq!(viewport.dimensions([800., 600.], 2.), [1602, 1202]);
        viewport.bounds.set(Bounds::default());
        assert_eq!(viewport.dimensions([800., 600.], 2.), [1600, 1200]);
    }

    #[test]
    fn accepts_camera_lag_but_rejects_other_stale_frame_identity() {
        let (mut viewport, mut editor) = fixture();
        let frame = viewport.identity(&editor);
        viewport.displayed = Some(frame);
        assert!(viewport.is_current(&editor));
        editor.camera.orbit(20., 10.);
        assert!(viewport.accepts(frame, &editor));
        assert!(!viewport.is_current(&editor));
        for stale in [
            FrameIdentity {
                generation: frame.generation + 1,
                ..frame
            },
            FrameIdentity {
                revision: frame.revision + 1,
                ..frame
            },
            FrameIdentity {
                preview: Some(PreviewSource::History(None)),
                ..frame
            },
            FrameIdentity {
                size: [802, 601],
                ..frame
            },
        ] {
            assert!(!viewport.accepts(stale, &editor), "{stale:?}");
        }
        viewport.invalidate_preview();
        assert!(!viewport.accepts(frame, &editor));
    }

    #[test]
    fn coalesces_requests_and_only_latest_navigation_timer_restores_quality() {
        let (mut viewport, editor) = fixture();
        let at = web_time::Instant::now();
        assert!(viewport.begin_render(at));
        assert!(!viewport.begin_render(at));
        assert!(!viewport.begin_render(at));
        assert!(viewport.render_pending);
        let first = viewport.begin_navigation();
        let latest = viewport.begin_navigation();
        assert!(!viewport.settle_navigation(first));
        let low = viewport.cpu_options(editor.camera, true);
        assert_eq!((low.width, low.height, low.supersampling), (401, 301, 1));
        viewport.rendering = false;
        assert!(viewport.begin_render(at));
        assert!(!viewport.render_pending);
        assert!(viewport.settle_navigation(latest));
        let full = viewport.cpu_options(editor.camera, true);
        assert_eq!((full.width, full.height, full.supersampling), (801, 601, 2));
        assert_eq!(viewport.cpu_options(editor.camera, false).supersampling, 1);
    }

    #[test]
    fn picking_requires_current_painted_frame_and_no_in_flight_navigation() {
        let (mut viewport, mut editor) = fixture();
        let p = point(px(400.), px(300.));
        let identity = viewport.identity(&editor);
        viewport.displayed = Some(identity);
        viewport.image = Some(Arc::new(RenderImage::new(vec![image::Frame::new(
            image::RgbaImage::new(1, 1),
        )])));
        assert!(viewport.ray(&editor, p).is_none());
        viewport.painted_frame.set(Some(identity));
        assert!(viewport.ray(&editor, p).is_some());
        viewport.rendering = true;
        assert!(viewport.ray(&editor, p).is_none());
        viewport.rendering = false;
        let epoch = viewport.begin_navigation();
        assert!(viewport.ray(&editor, p).is_none());
        viewport.settle_navigation(epoch);
        assert!(viewport.ray(&editor, p).is_some());
        editor.camera.orbit(10., 0.);
        assert!(viewport.ray(&editor, p).is_none());
    }

    #[test]
    fn replacing_document_invalidates_same_revision_and_retains_in_flight_work() {
        let (mut viewport, mut editor) = fixture();
        let identity = viewport.identity(&editor);
        viewport.displayed = Some(identity);
        viewport.painted_frame.set(Some(identity));
        viewport.rendering = true;
        viewport.replace_editor(&mut editor);
        assert!(!viewport.accepts(identity, &editor));
        assert!(!viewport.is_current(&editor));
        assert!(viewport.rendering());
        assert_eq!(viewport.display_camera(), editor.camera);
        assert_eq!(viewport.frame_meter().average(), None);
    }
}
