//! Human tool gestures translated into atomic shared-core commands.
use glam::DVec3;
use std::cell::{Ref, RefCell};
#[cfg(not(target_family = "wasm"))]
use std::path::Path;
use std::path::PathBuf;
use voxelite_core::*;
#[cfg(not(target_family = "wasm"))]
use voxelite_format::session::SavePolicy;
#[cfg(not(target_family = "wasm"))]
pub use voxelite_format::session::{SaveTask, SavedProject};
use voxelite_format::{
    Project,
    session::{DocumentSession, SessionError},
};
use voxelite_plugin_api::{
    ExportImage, ExportResult, OverlayFrame, OverlayInput, OverlayNavigation, OverlayOutcome,
    SettingValue, ToolCx, ToolError, ToolGesture, ToolHit, ToolInput, ToolLayer, ToolOutcome,
};
use voxelite_plugin_host::{Host, HostError};
use voxelite_render::{Camera, OrbitPose, PresetView, Projection, RenderOptions};

/// Supply the CPU-rendered view selected by the app to a snapshot exporter.
pub fn render_view(
    options: &RenderOptions,
) -> impl Fn(&Snapshot) -> ExportResult<ExportImage> + '_ {
    move |snapshot| {
        let image = voxelite_render::render(snapshot, options)?;
        Ok(ExportImage {
            width: image.width,
            height: image.height,
            rgba: image.rgba,
        })
    }
}

/// Failures presented by the shell without dropping the current document.
#[derive(Debug, thiserror::Error)]
pub enum EditorError {
    /// Core validation or edit conflict.
    #[error(transparent)]
    Core(#[from] voxelite_core::Error),
    /// Portable format error.
    #[error(transparent)]
    Format(#[from] voxelite_format::FormatError),
    /// Filesystem error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Invalid human gesture.
    #[error("{0}")]
    Gesture(String),
}
impl From<SessionError> for EditorError {
    fn from(error: SessionError) -> Self {
        match error {
            SessionError::Format(e) => Self::Format(e),
            SessionError::Io(e) => Self::Io(e),
            e => Self::Gesture(e.to_string()),
        }
    }
}
impl From<HostError> for EditorError {
    fn from(error: HostError) -> Self {
        match error {
            HostError::Tool(ToolError::Core(error)) => Self::Core(error),
            HostError::Palette(error) => error.into(),
            error => Self::Gesture(error.to_string()),
        }
    }
}
/// Result of an editor operation.
pub type EditorResult<T> = std::result::Result<T, EditorError>;

/// Built-in editing tools. All mutations use the same engine as MCP.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tool {
    /// Add a cell beside a visible face, or on the ground plane.
    #[default]
    Place,
    /// Erase a cell in the active layer.
    Erase,
    /// Recolor an occupied cell in the active layer.
    Paint,
    /// Six-connected replacement within document bounds.
    Fill,
    /// Two corners define one solid box transaction.
    Box,
    /// Sample the original palette color of visible geometry.
    Picker,
}
impl Tool {
    /// Stable implementation ID in the enabled plugin registry.
    pub fn id(self) -> &'static str {
        match self {
            Self::Place => "dev.voxelite.tools.place.tool",
            Self::Erase => "dev.voxelite.tools.erase.tool",
            Self::Paint => "dev.voxelite.tools.paint.tool",
            Self::Fill => "dev.voxelite.tools.fill.tool",
            Self::Box => "dev.voxelite.tools.box.tool",
            Self::Picker => "dev.voxelite.tools.picker.tool",
        }
    }
    /// Display order for the tool picker.
    pub const ALL: [Self; 6] = [
        Self::Place,
        Self::Erase,
        Self::Paint,
        Self::Fill,
        Self::Box,
        Self::Picker,
    ];
    /// User-facing name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Place => "Place",
            Self::Erase => "Erase",
            Self::Paint => "Paint",
            Self::Fill => "Fill",
            Self::Box => "Box",
            Self::Picker => "Picker",
        }
    }
    /// Keyboard shortcut, independent of platform modifiers.
    pub fn key(self) -> &'static str {
        match self {
            Self::Place => "B",
            Self::Erase => "E",
            Self::Paint => "P",
            Self::Fill => "G",
            Self::Box => "U",
            Self::Picker => "I",
        }
    }
}

/// Orthographic orbit rig shared by rendering and picking.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitCamera {
    /// World-space look-at point, including pan.
    pub target: DVec3,
    /// Azimuth in radians.
    pub yaw: f64,
    /// Elevation in radians; direct orbit gestures clamp away from the poles.
    pub pitch: f64,
    /// Vertical world-space viewport extent.
    pub span: f64,
    distance: f64,
}
impl OrbitCamera {
    /// Complete pose for interpolation, including the private eye distance.
    pub fn pose(self) -> OrbitPose {
        OrbitPose {
            target: self.target.to_array(),
            yaw: self.yaw,
            pitch: self.pitch,
            span: self.span,
            distance: self.distance,
        }
    }
    /// Restore a pose unchanged, preserving exact top views and endpoints.
    pub fn from_pose(pose: OrbitPose) -> Self {
        Self {
            target: pose.target.into(),
            yaw: pose.yaw,
            pitch: pose.pitch,
            span: pose.span,
            distance: pose.distance,
        }
    }
    /// Fit visible content with a comfortable margin.
    pub fn fit(snapshot: &Snapshot, aspect: f64) -> Self {
        Self::fit_view(snapshot, aspect, PresetView::Iso)
    }
    /// Fit a named orthographic view using its own projected bounds.
    pub fn fit_view(snapshot: &Snapshot, aspect: f64, view: PresetView) -> Self {
        let camera = Camera::fit(snapshot, view, aspect, false).expect("validated editor document");
        let Projection::Orthographic { height } = camera.projection else {
            unreachable!()
        };
        let mut fitted = Self {
            target: camera.target.into(),
            yaw: match view {
                PresetView::Iso => std::f64::consts::FRAC_PI_4,
                PresetView::Right => std::f64::consts::FRAC_PI_2,
                PresetView::Left => -std::f64::consts::FRAC_PI_2,
                PresetView::Back => std::f64::consts::PI,
                _ => 0.0,
            },
            pitch: match view {
                PresetView::Iso => (1.0 / 3.0_f64.sqrt()).asin(),
                PresetView::Top => std::f64::consts::FRAC_PI_2,
                _ => 0.0,
            },
            span: height * 1.04,
            distance: DVec3::from(camera.eye).distance(camera.target.into()),
        };
        // The center of a world-space bounding box is not necessarily the
        // center of its visible silhouette (for example, a tree beside a house).
        // Project occupied cell extents, retaining the conservative fit scale
        // so sparse scenes do not jump to an unexpectedly tight zoom.
        let (right, up, _) = fitted.basis();
        let half_cell = [right.abs().element_sum(), up.abs().element_sum()].map(|v| v * 0.5);
        let mut lo = [f64::INFINITY; 2];
        let mut hi = [f64::NEG_INFINITY; 2];
        for layer in snapshot.layers.iter().filter(|layer| layer.visible) {
            for (position, _) in layer.voxels.iter() {
                let offset =
                    DVec3::from(position.map(f64::from)) + DVec3::splat(0.5) - fitted.target;
                for (axis, value) in [offset.dot(right), offset.dot(up)].into_iter().enumerate() {
                    lo[axis] = lo[axis].min(value - half_cell[axis]);
                    hi[axis] = hi[axis].max(value + half_cell[axis]);
                }
            }
        }
        if lo[0].is_finite() {
            fitted.target += right * ((lo[0] + hi[0]) * 0.5) + up * ((lo[1] + hi[1]) * 0.5);
        }
        fitted
    }
    /// Basis used by both the CPU renderer and screen rays.
    pub fn basis(self) -> (DVec3, DVec3, DVec3) {
        let back = DVec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        );
        // Derive right from yaw so exact top view has no pole singularity.
        let right = DVec3::new(self.yaw.cos(), 0.0, -self.yaw.sin());
        (right, back.cross(right).normalize(), back)
    }
    /// Portable camera vectors for plugin overlay queries.
    pub fn overlay_basis(self) -> [[f64; 3]; 3] {
        let (right, up, back) = self.basis();
        [right.to_array(), up.to_array(), back.to_array()]
    }
    /// Renderer camera; no hidden fit or projection adjustments.
    pub fn camera(self) -> Camera {
        let (_, up, back) = self.basis();
        Camera {
            eye: (self.target + back * self.distance).to_array(),
            target: self.target.to_array(),
            up: up.to_array(),
            projection: Projection::Orthographic { height: self.span },
        }
    }
    /// Orbit by logical pointer pixels.
    pub fn orbit(&mut self, dx: f64, dy: f64) {
        if !dx.is_finite() || !dy.is_finite() {
            return;
        }
        self.yaw = (self.yaw - dx * 0.008).rem_euclid(std::f64::consts::TAU);
        self.pitch = (self.pitch + dy * 0.008).clamp(-1.5, 1.5);
    }
    /// Pan by logical pixels at the current zoom.
    pub fn pan(&mut self, dx: f64, dy: f64, height: f64) {
        if !dx.is_finite() || !dy.is_finite() || !height.is_finite() || height <= 0.0 {
            return;
        }
        let (right, up, _) = self.basis();
        self.target += (-right * dx + up * dy) * self.span / height;
    }
    /// Exponential bounded zoom. Positive wheel delta zooms in.
    pub fn zoom(&mut self, delta: f64) {
        if delta.is_finite() {
            self.span = (self.span * (-delta * 0.002).exp()).clamp(2.0, 2048.0);
        }
    }
    /// Ray through normalized viewport coordinates (left/top = 0, right/bottom = 1).
    pub fn ray(self, uv: [f64; 2], aspect: f64) -> Option<Ray> {
        if !aspect.is_finite()
            || aspect <= 0.0
            || uv
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return None;
        }
        let (right, up, back) = self.basis();
        Some(Ray {
            origin: self.target
                + back * self.distance
                + right * (uv[0] - 0.5) * self.span * aspect
                + up * (0.5 - uv[1]) * self.span,
            direction: -back,
        })
    }
    /// Project a world coordinate to normalized screen coordinates.
    pub fn project(self, p: DVec3, aspect: f64) -> [f64; 2] {
        let (right, up, _) = self.basis();
        let offset = p - self.target;
        [
            0.5 + offset.dot(right) / (self.span * aspect),
            0.5 - offset.dot(up) / self.span,
        ]
    }
}
/// A world-space picking ray.
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    /// Origin before the visible scene.
    pub origin: DVec3,
    /// Nonzero finite direction.
    pub direction: DVec3,
}
/// The nearest visible cell and face.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    /// Occupied cell.
    pub at: Pos,
    /// Outward face normal, exactly one signed axis.
    pub normal: Pos,
    /// Topmost visible layer at this cell.
    pub layer: Uuid,
    /// Original stable palette slot.
    pub slot: u16,
}
/// Bounded voxel DDA, including negative coordinates and axis-aligned rays.
pub fn raycast(snapshot: &Snapshot, ray: Ray) -> Option<Hit> {
    if !ray.origin.is_finite()
        || !ray.direction.is_finite()
        || ray.direction.length_squared() < 1e-16
    {
        return None;
    }
    let d = ray.direction.normalize();
    let min = DVec3::from(snapshot.bounds.min.map(f64::from));
    let max = DVec3::from(snapshot.bounds.max.map(f64::from));
    let mut near: f64 = 0.0;
    let mut far = f64::INFINITY;
    let mut normal = [0; 3];
    for a in 0..3 {
        if d[a].abs() < 1e-12 {
            if ray.origin[a] < min[a] || ray.origin[a] >= max[a] {
                return None;
            }
        } else {
            let t1 = (min[a] - ray.origin[a]) / d[a];
            let t2 = (max[a] - ray.origin[a]) / d[a];
            if t1.min(t2) > near {
                near = t1.min(t2);
                normal = [0; 3];
                normal[a] = if d[a] > 0.0 { -1 } else { 1 };
            }
            far = far.min(t1.max(t2));
        }
    }
    if near > far || far < 0.0 {
        return None;
    }
    let start = ray.origin + d * (near + 1e-7);
    let mut cell = start.floor().as_ivec3().to_array();
    let steps = d.to_array().map(|v| if v > 0.0 { 1 } else { -1 });
    let delta = d.to_array().map(|v| {
        if v.abs() < 1e-12 {
            f64::INFINITY
        } else {
            1.0 / v.abs()
        }
    });
    let mut next: [f64; 3] = std::array::from_fn(|a| {
        if d[a].abs() < 1e-12 {
            f64::INFINITY
        } else {
            (f64::from(cell[a] + i32::from(steps[a] > 0)) - ray.origin[a]) / d[a]
        }
    });
    // Each axis is limited to 256 cells by the document contract.
    for _ in 0..772 {
        if !snapshot.bounds.contains(cell) {
            return None;
        }
        if let Some((slot, layer)) = snapshot.sample(cell) {
            return Some(Hit {
                at: cell,
                normal,
                layer,
                slot,
            });
        }
        let a = if next[0] <= next[1] && next[0] <= next[2] {
            0
        } else if next[1] <= next[2] {
            1
        } else {
            2
        };
        if next[a] > far {
            return None;
        }
        cell[a] += steps[a];
        next[a] += delta[a];
        normal = [0; 3];
        normal[a] = -steps[a];
    }
    None
}

/// One open project, with UI preferences separate from portable document content.
pub struct Editor {
    pub(crate) session: DocumentSession,
    // Hover/preview queries also record host timings and isolate plugin panics.
    pub(crate) plugins: RefCell<Host>,
    /// Persistent software viewport occupancy, shared with background render tasks.
    pub raster: voxelite_render::RasterCache,
    /// Read-only preview, separate from the live authority.
    pub preview: Option<crate::history::Preview>,
    pub(crate) playback: Option<crate::history::Playback>,
    /// Active layer identity, reconciled after undo/redo.
    pub layer: Uuid,
    /// Selected editing tool.
    pub tool: Tool,
    /// Selected brush color; imported lazily as part of the edit transaction.
    pub color: Swatch,
    pub(crate) palette_state: voxelite_plugin_api::PaletteState,
    /// Last admitted transient discussion state.
    pub comments_state: voxelite_plugin_api::CommentsState,
    /// Camera state, unaffected by undo/redo.
    pub camera: OrbitCamera,
    /// First corner of an uncommitted box.
    pub box_start: Option<Pos>,
    #[cfg(test)]
    last_command: Option<(String, Command)>,
}
// Preserve the shell's engine/path access while the shared session owns both.
impl std::ops::Deref for Editor {
    type Target = DocumentSession;
    fn deref(&self) -> &Self::Target {
        &self.session
    }
}
impl std::ops::DerefMut for Editor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.session
    }
}
impl Editor {
    /// Open a validated portable project, preserving unknown optional metadata.
    pub fn new(project: Project, path: Option<PathBuf>) -> EditorResult<Self> {
        Self::from_session(DocumentSession::new(project, path, Limits::default())?)
    }
    /// Attach UI-thread plugins to a session that may have been loaded in the background.
    pub fn from_session(mut session: DocumentSession) -> EditorResult<Self> {
        let mut plugins = crate::app_host();
        plugins.load_data(&mut session.engine)?;
        let s = &session.engine.document().content;
        let layer = s
            .layers
            .iter()
            .rev()
            .find(|l| l.visible && !l.locked)
            .unwrap_or(&s.layers[0])
            .id;
        let color = s
            .palette
            .get(s.palette.order[0])
            .expect("valid palette")
            .clone();
        let camera = OrbitCamera::fit(s, 1.3);
        Ok(Self {
            session,
            plugins: RefCell::new(plugins),
            raster: voxelite_render::RasterCache::default(),
            preview: None,
            playback: None,
            layer,
            tool: Tool::Place,
            color,
            palette_state: Default::default(),
            comments_state: Default::default(),
            camera,
            box_start: None,
            #[cfg(test)]
            last_command: None,
        })
    }
    /// Load the bundled diorama, independent of the current working directory.
    pub fn example() -> EditorResult<Self> {
        Self::new(
            voxelite_format::read(
                include_bytes!("../../../examples/autumn-workshop.voxl").as_slice(),
            )?,
            None,
        )
    }
    /// Create a clean 32-cube project.
    pub fn blank() -> EditorResult<Self> {
        Self::new(
            Project::new(Document::new(Uuid::new_v4(), Uuid::new_v4()))?,
            None,
        )
    }
    /// Read a file without touching the current editor on failure.
    #[cfg(not(target_family = "wasm"))]
    pub fn open(path: &Path) -> EditorResult<Self> {
        Self::from_session(DocumentSession::open(path, Limits::default())?)
    }
    /// Capture portable metadata and current content for a local MCP attachment.
    pub fn project(&self) -> Project {
        self.session.project()
    }
    /// Reconcile local selection after commands committed by the attached agent.
    pub fn agent_updated(&mut self) {
        self.pause_history();
        self.reconcile();
        self.box_start = None;
    }
    /// Current document content.
    pub fn snapshot(&self) -> &Snapshot {
        &self.session.engine.document().content
    }
    /// Save through a synced temporary file in the destination directory, then atomically rename.
    /// Dirty state is cleared only after successful persistence.
    #[cfg(not(target_family = "wasm"))]
    pub fn save(&mut self, path: &Path) -> EditorResult<()> {
        let saved = self.prepare_save(path).write()?;
        self.finish_save(saved);
        Ok(())
    }
    /// Capture the current document without performing filesystem I/O.
    #[cfg(not(target_family = "wasm"))]
    pub fn prepare_save(&self, path: &Path) -> SaveTask {
        self.session.prepare_save(path, SavePolicy::ReplaceSelected)
    }
    /// Adopt a successful save; edits made after its snapshot remain dirty.
    #[cfg(not(target_family = "wasm"))]
    pub fn finish_save(&mut self, saved: SavedProject) {
        self.session.finish_save(saved);
    }
    /// Execute a single attributed, atomic command.
    pub fn execute(&mut self, label: &str, command: Command) -> EditorResult<bool> {
        self.ensure_live()?;
        #[cfg(test)]
        {
            self.last_command = Some((label.into(), command.clone()));
        }
        let receipt = self.session.engine.execute(Transaction {
            expected_revision: self.session.engine.revision(),
            id: Uuid::new_v4(),
            actor: Actor {
                origin: Origin::Human,
                client_id: Uuid::nil(),
                tool: "native-editor".into(),
            },
            label: label.into(),
            command,
        })?;
        self.reconcile();
        Ok(receipt.entry.is_some())
    }
    pub(crate) fn reconcile(&mut self) {
        if self.snapshot().layer(self.layer).is_err() {
            self.layer = self.snapshot().layers[0].id;
        }
        self.box_start = None;
    }
    /// Compensate one applied step, retaining unrelated newer work.
    pub fn revert_step(&mut self, id: Uuid) -> EditorResult<()> {
        self.ensure_live()?;
        self.session.engine.revert(
            self.session.engine.revision(),
            id,
            Uuid::new_v4(),
            Actor::new(Origin::Human),
        )?;
        self.reconcile();
        Ok(())
    }
    /// Undo the most recent complete gesture.
    pub fn undo(&mut self) -> EditorResult<()> {
        self.ensure_live()?;
        if let Some(id) = self.session.engine.head() {
            self.session
                .engine
                .undo(self.session.engine.revision(), id)?;
        }
        self.reconcile();
        Ok(())
    }
    /// Redo the most recently undone gesture.
    pub fn redo(&mut self) -> EditorResult<()> {
        self.ensure_live()?;
        if let Some(id) = self.session.engine.redo_head() {
            self.session
                .engine
                .redo(self.session.engine.revision(), id)?;
        }
        self.reconcile();
        Ok(())
    }
    /// Add and select a new empty layer.
    pub fn add_layer(&mut self) -> EditorResult<()> {
        let id = Uuid::new_v4();
        let index = self.snapshot().layers.len();
        self.execute(
            "Add layer",
            Command::Layer(LayerEdit::Create {
                id,
                name: format!("Layer {}", index + 1),
                index,
            }),
        )?;
        self.layer = id;
        Ok(())
    }
    /// Change visibility or lock through undoable core metadata commands.
    pub fn update_layer(
        &mut self,
        id: Uuid,
        visible: Option<bool>,
        locked: Option<bool>,
    ) -> EditorResult<()> {
        self.execute(
            if visible.is_some() {
                "Toggle layer visibility"
            } else {
                "Toggle layer lock"
            },
            Command::Layer(LayerEdit::Update {
                id,
                name: None,
                visible,
                locked,
            }),
        )?;
        Ok(())
    }
    /// Enabled declarations, lifecycle state and tool invocation timings.
    pub fn plugin_host(&self) -> Ref<'_, Host> {
        self.plugins.borrow()
    }
    /// Capture enabled renderer passes for a viewport, export or preview worker.
    pub fn render_plan(&self) -> voxelite_render::RenderPlan {
        self.plugins.borrow_mut().render_plan()
    }
    /// Find a format by extension in the live registry, without a built-in format table.
    pub fn export_format(
        &self,
        extension: &str,
    ) -> EditorResult<voxelite_plugin_api::FormatContribution> {
        self.plugins
            .borrow()
            .registry()
            .formats()
            .find(|(_, format)| {
                format.export
                    && format
                        .extensions
                        .iter()
                        .any(|ext| ext.eq_ignore_ascii_case(extension))
            })
            .map(|(_, format)| format.clone())
            .ok_or_else(|| EditorError::Gesture("Export format is unavailable".into()))
    }
    /// Resolve an export format in the live registry before starting background work.
    pub fn prepare_export(&self, id: &str) -> EditorResult<voxelite_plugin_host::ExportJob> {
        self.plugins
            .borrow_mut()
            .prepare_export(id)?
            .ok_or_else(|| EditorError::Gesture("Export format is unavailable".into()))
    }
    /// Admit completed bytes only while the format's plugin remains enabled.
    pub fn finish_export(
        &self,
        result: voxelite_plugin_host::ExportCompletion,
    ) -> EditorResult<Vec<u8>> {
        self.plugins
            .borrow_mut()
            .finish_export(result)?
            .ok_or_else(|| EditorError::Gesture("Export format is unavailable".into()))
    }
    /// Draw the enabled portable overlays against the displayed document and camera.
    pub fn overlays(&self, camera: OrbitCamera, hovered: Option<(&str, u32)>) -> Vec<OverlayFrame> {
        self.plugins.borrow_mut().draw_overlays(
            self.display_snapshot().bounds,
            camera.overlay_basis(),
            hovered,
        )
    }
    /// Resolve a pointer input without applying edits or camera feedback.
    pub fn overlay_outcome(
        &self,
        id: &str,
        camera: OrbitCamera,
        input: OverlayInput,
    ) -> EditorResult<Option<OverlayOutcome>> {
        Ok(self.plugins.borrow_mut().dispatch_overlay(
            id,
            self.display_snapshot().bounds,
            camera.overlay_basis(),
            input,
        )?)
    }
    /// Admit typed overlay edits through the same transaction boundary as voxel tools.
    /// Navigation is adopted only after a successful transaction. Isometric fitting
    /// remains a shell action because it also needs viewport dimensions.
    pub fn interact_overlay(
        &mut self,
        id: &str,
        camera: OrbitCamera,
        input: OverlayInput,
    ) -> EditorResult<Option<OverlayOutcome>> {
        let Some(outcome) = self.overlay_outcome(id, camera, input)? else {
            return Ok(None);
        };
        if !outcome.commands.is_empty() {
            self.execute(&outcome.label, Command::Batch(outcome.commands.clone()))?;
        }
        match &outcome.navigation {
            Some(OverlayNavigation::Orbit([dx, dy])) => self.camera.orbit(*dx, *dy),
            Some(OverlayNavigation::Orient { yaw, pitch, .. }) => {
                self.camera.yaw = *yaw;
                self.camera.pitch = *pitch;
            }
            _ => {}
        }
        Ok(Some(outcome))
    }
    /// Synchronize the existing persisted grid preference into plugin-scoped state.
    pub fn set_grid(&mut self, enabled: bool) -> EditorResult<bool> {
        Ok(self.plugins.get_mut().set_setting(
            voxelite_plugin_gizmo_grid::GRID_SETTING,
            SettingValue::Bool(enabled),
        )?)
    }
    /// Effective grid setting; disabled or absent plugins expose no grid.
    pub fn grid(&self) -> bool {
        self.plugins
            .borrow()
            .setting(voxelite_plugin_gizmo_grid::GRID_SETTING)
            == Some(&SettingValue::Bool(true))
    }
    /// Enabled mirror axis settings, owned by the mirror plugin and local to this editor.
    pub fn mirror_axes(&self) -> [bool; 3] {
        let host = self.plugins.borrow();
        voxelite_plugin_mirror::AXIS_SETTINGS
            .map(|key| host.setting(key) == Some(&SettingValue::Bool(true)))
    }
    /// Change one plugin setting without affecting the document or undo history.
    /// Missing or disabled mirror commands are inert.
    pub fn set_mirror_axis(&mut self, axis: usize, enabled: bool) -> EditorResult<bool> {
        let Some(key) = voxelite_plugin_mirror::AXIS_SETTINGS.get(axis) else {
            return Ok(false);
        };
        Ok(self
            .plugins
            .get_mut()
            .set_setting(key, SettingValue::Bool(enabled))?)
    }
    /// Enable or disable an optional feature without changing document history.
    pub fn set_plugin_enabled(&mut self, id: &str, enabled: bool) -> EditorResult<()> {
        let host = self.plugins.get_mut();
        if enabled {
            host.enable(id)?;
        } else {
            host.disable(id)?;
        }
        if !host.has_tool(self.tool.id())? {
            self.box_start = None;
        }
        if !host.has_comments_panel(voxelite_plugin_comments::PANEL_ID)? {
            self.comments_state.placing = false;
            self.comments_state.first = None;
            self.comments_state.draft = None;
        }
        Ok(())
    }
    /// Region outlined for a tool hover or an unfinished box.
    pub fn tool_preview(&self, hover: Option<Pos>) -> Option<Region> {
        self.tool_outcome(self.tool, None, ToolGesture::Preview(hover))
            .ok()
            .flatten()
            .and_then(|outcome| outcome.preview)
    }
    /// Locate a tool target, using visible faces or the document's bottom work plane.
    pub fn target(&self, ray: Ray) -> Option<Pos> {
        self.tool_outcome(self.tool, Some(ray), ToolGesture::Hover)
            .ok()
            .flatten()
            .and_then(|outcome| outcome.target)
    }
    fn tool_outcome(
        &self,
        tool: Tool,
        ray: Option<Ray>,
        gesture: ToolGesture,
    ) -> EditorResult<Option<ToolOutcome>> {
        self.registered_tool_outcome(tool.id(), ray, gesture, self.box_start)
    }
    pub(crate) fn registered_tool_outcome(
        &self,
        id: &str,
        ray: Option<Ray>,
        gesture: ToolGesture,
        anchor: Option<Pos>,
    ) -> EditorResult<Option<ToolOutcome>> {
        let hit = ray
            .and_then(|ray| raycast(self.snapshot(), ray))
            .map(|hit| ToolHit {
                at: hit.at,
                normal: hit.normal,
                layer: hit.layer,
                slot: hit.slot,
            });
        let cx = EditorToolCx {
            editor: self,
            color: self.active_color().clone(),
            ray,
            hit,
        };
        Ok(self
            .plugins
            .borrow_mut()
            .dispatch_tool(id, &cx, ToolInput { gesture, anchor })?)
    }
    /// Apply a pointer click, sampling without mutation when the picker is active.
    pub fn click(&mut self, ray: Ray, sample_only: bool) -> EditorResult<bool> {
        self.pause_history();
        let tool = if sample_only { Tool::Picker } else { self.tool };
        let Some(outcome) = self.tool_outcome(tool, Some(ray), ToolGesture::Click)? else {
            self.box_start = None;
            return Ok(false);
        };
        if tool != Tool::Picker && outcome.target.is_some() {
            self.ensure_live()?;
        }
        self.apply_tool_outcome(outcome)
    }
    /// Apply a tool at an explicit integer coordinate, also used by the keyboard cursor.
    pub fn apply_at(&mut self, at: Pos) -> EditorResult<bool> {
        self.ensure_live()?;
        let Some(outcome) = self.tool_outcome(self.tool, None, ToolGesture::ApplyAt(at))? else {
            self.box_start = None;
            return Ok(false);
        };
        self.apply_tool_outcome(outcome)
    }
    fn apply_tool_outcome(&mut self, outcome: ToolOutcome) -> EditorResult<bool> {
        let changed = if outcome.commands.is_empty() {
            false
        } else {
            self.execute(&outcome.label, Command::Batch(outcome.commands))?
        };
        // Failed transactions leave the pending gesture and palette selection intact.
        self.box_start = outcome.anchor;
        if let Some(color) = outcome.sampled_color {
            self.color = color;
        }
        Ok(changed)
    }
}

// Only read-only, mirror-agnostic queries cross the tool boundary.
struct EditorToolCx<'a> {
    editor: &'a Editor,
    color: Swatch,
    ray: Option<Ray>,
    hit: Option<ToolHit>,
}
impl ToolCx for EditorToolCx<'_> {
    fn hovered_hit(&self) -> Option<ToolHit> {
        self.hit
    }
    fn ground_target(&self) -> Option<Pos> {
        let ray = self.ray?;
        let bounds = self.bounds();
        if ray.direction.y.abs() < 1e-12 {
            return None;
        }
        let t = (f64::from(bounds.min[1]) - ray.origin.y) / ray.direction.y;
        if !t.is_finite() || t < 0.0 {
            return None;
        }
        let mut p = (ray.origin + ray.direction * t)
            .floor()
            .as_ivec3()
            .to_array();
        p[1] = bounds.min[1];
        bounds.contains(p).then_some(p)
    }
    fn sample(&self, at: Pos) -> Option<(u16, Uuid)> {
        self.editor.snapshot().sample(at)
    }
    fn voxel(&self, layer: Uuid, at: Pos) -> voxelite_core::Result<u16> {
        Ok(self.editor.snapshot().layer(layer)?.voxels.get(at))
    }
    fn palette(&self) -> &Palette {
        &self.editor.snapshot().palette
    }
    fn active_color(&self) -> &Swatch {
        &self.color
    }
    fn active_layer(&self) -> Uuid {
        self.editor.layer
    }
    fn layer(&self, id: Uuid) -> voxelite_core::Result<ToolLayer> {
        let layer = self.editor.snapshot().layer(id)?;
        Ok(ToolLayer {
            visible: layer.visible,
            locked: layer.locked,
        })
    }
    fn bounds(&self) -> Region {
        self.editor.snapshot().bounds
    }
}

#[cfg(test)]
#[path = "editor/tool_tests.rs"]
mod tool_tests;

#[cfg(test)]
mod tests {
    use super::*;
    fn editor() -> Editor {
        Editor::blank().unwrap()
    }
    #[test]
    fn tools_and_mirror_are_atomic_and_reversible() {
        let mut e = editor();
        let original = e.engine.document().clone();
        for axis in 0..3 {
            e.set_mirror_axis(axis, true).unwrap();
        }
        e.color = Swatch::rgb(2, 5, 8);
        assert!(e.apply_at([1, 2, 3]).unwrap());
        assert_eq!(e.snapshot().layers[0].voxels.len(), 8);
        assert_eq!(e.engine.history().count(), 1);
        e.undo().unwrap();
        assert_eq!(e.engine.document(), &original);
        assert!(!e.engine.is_dirty());
        e.redo().unwrap();
        assert_eq!(e.snapshot().layers[0].voxels.len(), 8);
    }
    #[test]
    fn box_preview_cancels_without_mutation_and_commits_once() {
        let mut e = editor();
        e.tool = Tool::Box;
        assert!(!e.apply_at([1, 1, 1]).unwrap());
        assert!(!e.engine.is_dirty());
        e.box_start = None;
        e.apply_at([3, 3, 3]).unwrap();
        e.apply_at([2, 2, 2]).unwrap();
        assert_eq!(e.snapshot().layers[0].voxels.len(), 8);
        assert_eq!(e.engine.history().count(), 1);
    }
    #[test]
    fn paint_fill_picker_erase_and_locks() {
        let mut e = editor();
        e.apply_at([1, 1, 1]).unwrap();
        e.apply_at([1, 1, 2]).unwrap();
        e.tool = Tool::Paint;
        e.color = Swatch::rgb(100, 50, 30);
        e.apply_at([1, 1, 1]).unwrap();
        e.tool = Tool::Picker;
        e.apply_at([1, 1, 2]).unwrap();
        assert_ne!(e.color.rgba, [100, 50, 30, 255]);
        e.tool = Tool::Fill;
        e.apply_at([1, 1, 1]).unwrap();
        assert_eq!(
            e.snapshot().layers[0].voxels.get([1, 1, 1]),
            e.snapshot().layers[0].voxels.get([1, 1, 2])
        );
        e.update_layer(e.layer, None, Some(true)).unwrap();
        let before = e.engine.document().clone();
        e.tool = Tool::Erase;
        assert!(e.apply_at([1, 1, 1]).is_err());
        assert_eq!(e.engine.document(), &before);
        e.update_layer(e.layer, None, Some(false)).unwrap();
        e.apply_at([1, 1, 1]).unwrap();
        assert_eq!(e.snapshot().layers[0].voxels.len(), 1);
    }
    #[test]
    fn palette_choice_and_noop_do_not_recolor_or_dirty_document() {
        let mut e = editor();
        e.color = voxelite_plugin_palette::Preset::Pico8
            .palette()
            .get(4)
            .unwrap()
            .clone();
        assert!(!e.engine.is_dirty());
        e.tool = Tool::Paint;
        assert!(!e.apply_at([2, 2, 2]).unwrap());
        assert_eq!(e.snapshot().palette.slots.len(), 1);
        assert!(!e.engine.is_dirty());
    }
    #[test]
    fn fill_empty_cell_does_not_flood_space_or_import_a_color() {
        let mut e = editor();
        let original = e.engine.document().clone();
        e.tool = Tool::Fill;
        e.color = Swatch::rgb(1, 2, 3);
        assert!(!e.apply_at([2, 2, 2]).unwrap());
        assert_eq!(e.engine.document(), &original);
        assert_eq!(e.engine.revision(), 0);
        assert!(!e.engine.is_dirty());
    }
    #[test]
    fn keyboard_picker_samples_visible_composite_with_locked_or_hidden_active_layer() {
        let mut e = editor();
        e.color = Swatch::rgb(1, 2, 3);
        e.apply_at([2, 2, 2]).unwrap();
        e.add_layer().unwrap();
        e.update_layer(e.layer, Some(false), Some(true)).unwrap();
        let revision = e.engine.revision();
        e.tool = Tool::Picker;
        e.color = Swatch::rgb(4, 5, 6);
        assert!(!e.apply_at([2, 2, 2]).unwrap());
        assert_eq!(e.color.rgba, [1, 2, 3, 255]);
        assert_eq!(e.engine.revision(), revision);
    }
    #[test]
    fn save_rejects_external_changes_and_deletion_without_losing_edits() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("project.voxl");
        let mut e = editor();
        e.save(&path).unwrap();
        let mut external = Editor::open(&path).unwrap();
        e.apply_at([1, 2, 3]).unwrap();
        external.apply_at([4, 5, 6]).unwrap();
        external.save(&path).unwrap();
        let outside_bytes = std::fs::read(&path).unwrap();
        assert!(e.save(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), outside_bytes);
        assert!(e.engine.is_dirty());
        std::fs::remove_file(&path).unwrap();
        assert!(e.save(&path).is_err());
        assert!(!path.exists());
        e.save(&dir.path().join("recovered.voxl")).unwrap();
        assert!(!e.engine.is_dirty());
    }
    #[test]
    fn background_save_keeps_subsequent_edits_dirty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("project.voxl");
        let mut e = editor();
        e.apply_at([1, 2, 3]).unwrap();
        let save = e.prepare_save(&path);
        e.apply_at([4, 5, 6]).unwrap();
        e.finish_save(save.write().unwrap());
        assert!(e.engine.is_dirty());
        let opened = Editor::open(&path).unwrap();
        assert_eq!(opened.snapshot().layers[0].voxels.len(), 1);
        e.save(&path).unwrap();
        assert!(!e.engine.is_dirty());
        assert_eq!(
            Editor::open(&path).unwrap().snapshot().layers[0]
                .voxels
                .len(),
            2
        );
    }
    #[cfg(unix)]
    #[test]
    fn opening_symlink_saves_target_and_save_as_rejects_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("project.voxl");
        let link = dir.path().join("alias.voxl");
        let mut e = editor();
        e.save(&path).unwrap();
        std::os::unix::fs::symlink(&path, &link).unwrap();
        let mut opened = Editor::open(&link).unwrap();
        opened.apply_at([1, 2, 3]).unwrap();
        opened.save(&opened.path.clone().unwrap()).unwrap();
        assert!(link.is_symlink());
        assert_eq!(
            Editor::open(&path).unwrap().snapshot().layers[0]
                .voxels
                .len(),
            1
        );
        assert!(opened.save(&link).is_err());
    }
    #[test]
    fn save_roundtrip_and_failed_save_preserve_dirty_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.voxl");
        let mut e = editor();
        e.apply_at([2, 3, 4]).unwrap();
        assert!(e.save(&dir.path().join("missing/test.voxl")).is_err());
        assert!(e.engine.is_dirty());
        e.save(&path).unwrap();
        assert!(!e.engine.is_dirty());
        assert_eq!(
            Editor::open(&path).unwrap().engine.document(),
            e.engine.document()
        );
        let bytes = std::fs::read(&path).unwrap();
        e.save(&path).unwrap();
        assert_eq!(bytes, std::fs::read(path).unwrap());
    }
    #[test]
    fn layer_selection_reconciles_after_undo() {
        let mut e = editor();
        let first = e.layer;
        e.add_layer().unwrap();
        assert_ne!(e.layer, first);
        e.undo().unwrap();
        assert_eq!(e.layer, first);
        e.apply_at([1, 1, 1]).unwrap();
    }
    #[test]
    fn camera_pose_roundtrips_and_interpolation_preserves_destination() {
        let e = editor();
        let a = OrbitCamera::fit_view(e.snapshot(), 1.3, PresetView::Left);
        let b = OrbitCamera::fit_view(e.snapshot(), 0.8, PresetView::Top);
        assert_eq!(OrbitCamera::from_pose(a.pose()), a);
        assert_eq!(OrbitCamera::from_pose(b.pose()), b);
        assert_eq!(OrbitCamera::from_pose(a.pose().lerp(b.pose(), 1.0)), b);
        assert_eq!(b.pitch, std::f64::consts::FRAC_PI_2);
    }
    #[test]
    fn edits_undo_and_redo_leave_the_camera_unchanged() {
        let mut e = editor();
        e.camera.orbit(23.0, -7.0);
        e.camera.pan(20.0, -10.0, 600.0);
        e.camera.zoom(30.0);
        let camera = e.camera;
        e.apply_at([2, 3, 4]).unwrap();
        assert_eq!(e.camera, camera);
        e.undo().unwrap();
        assert_eq!(e.camera, camera);
        e.redo().unwrap();
        assert_eq!(e.camera, camera);
    }
    #[test]
    fn rays_hit_nearest_visible_layer_and_placement_face() {
        let mut e = editor();
        e.apply_at([2, 2, 2]).unwrap();
        e.apply_at([2, 2, 4]).unwrap();
        let ray = Ray {
            origin: DVec3::new(2.5, 2.5, 40.0),
            direction: DVec3::NEG_Z,
        };
        let hit = raycast(e.snapshot(), ray).unwrap();
        assert_eq!(hit.at, [2, 2, 4]);
        assert_eq!(hit.normal, [0, 0, 1]);
        assert_eq!(e.target(ray), Some([2, 2, 5]));
        e.add_layer().unwrap();
        e.apply_at([2, 2, 4]).unwrap();
        assert_eq!(raycast(e.snapshot(), ray).unwrap().layer, e.layer);
        e.update_layer(e.layer, Some(false), None).unwrap();
        assert_ne!(raycast(e.snapshot(), ray).unwrap().layer, e.layer);
    }
    #[test]
    fn negative_coordinates_parallel_misses_and_invalid_rays() {
        let mut e = editor();
        e.execute(
            "Resize",
            Command::EditDocument {
                name: None,
                bounds: Some(Region::new([-8; 3], [8; 3]).unwrap()),
            },
        )
        .unwrap();
        e.apply_at([-3, -2, -1]).unwrap();
        let ray = Ray {
            origin: DVec3::new(-2.5, -1.5, 20.0),
            direction: DVec3::NEG_Z,
        };
        assert_eq!(raycast(e.snapshot(), ray).unwrap().at, [-3, -2, -1]);
        assert!(
            raycast(
                e.snapshot(),
                Ray {
                    origin: DVec3::splat(50.0),
                    ..ray
                }
            )
            .is_none()
        );
        assert!(
            raycast(
                e.snapshot(),
                Ray {
                    direction: DVec3::ZERO,
                    ..ray
                }
            )
            .is_none()
        );
    }
    #[test]
    fn camera_projection_and_ray_remain_inverse_after_navigation() {
        let e = editor();
        let mut c = e.camera;
        c.orbit(120.0, -30.0);
        c.pan(40.0, -20.0, 700.0);
        c.zoom(75.0);
        let p = DVec3::new(3.5, 4.5, 5.5);
        let uv = c.project(p, 1.4);
        let ray = c.ray(uv, 1.4).unwrap();
        assert!((p - ray.origin).cross(ray.direction).length() < 1e-10);
        for _ in 0..100 {
            c.orbit(20.0, 100.0);
            c.zoom(500.0);
        }
        assert!(c.camera().validate().is_ok());
        assert!(c.span >= 2.0);
        assert!(c.ray([f64::NAN, 0.5], 1.4).is_none());
    }
    #[test]
    fn named_views_fit_bounds_and_top_points_exactly_down() {
        let mut e = editor();
        e.apply_at([0, 0, 0]).unwrap();
        e.apply_at([31, 10, 20]).unwrap();
        for view in [
            PresetView::Iso,
            PresetView::Front,
            PresetView::Right,
            PresetView::Top,
        ] {
            let camera = OrbitCamera::fit_view(e.snapshot(), 0.8, view);
            assert!(camera.camera().validate().is_ok());
            for bits in 0..8 {
                let corner = DVec3::new(
                    if bits & 1 == 0 { 0.0 } else { 32.0 },
                    if bits & 2 == 0 { 0.0 } else { 11.0 },
                    if bits & 4 == 0 { 0.0 } else { 21.0 },
                );
                assert!(
                    camera
                        .project(corner, 0.8)
                        .into_iter()
                        .all(|v| (0.0..=1.0).contains(&v))
                );
            }
            if view == PresetView::Top {
                let ray = camera.ray([0.5, 0.5], 0.8).unwrap();
                assert!((ray.direction - DVec3::NEG_Y).length() < 1e-12);
            }
        }
    }
    #[test]
    fn asymmetric_scene_is_centered_and_clear_of_viewport_controls() {
        let e = Editor::example().unwrap();
        for aspect in [616.0 / 512.0, 1032.0 / 772.0, 0.8, 2.0] {
            for view in [
                PresetView::Iso,
                PresetView::Front,
                PresetView::Right,
                PresetView::Top,
            ] {
                let camera = OrbitCamera::fit_view(e.snapshot(), aspect, view);
                let mut lo = [f64::INFINITY; 2];
                let mut hi = [f64::NEG_INFINITY; 2];
                for layer in e.snapshot().layers.iter().filter(|layer| layer.visible) {
                    for (position, _) in layer.voxels.iter() {
                        for bits in 0..8 {
                            let corner = DVec3::from(std::array::from_fn(|axis| {
                                f64::from(position[axis] + ((bits >> axis) & 1))
                            }));
                            for (axis, value) in
                                camera.project(corner, aspect).into_iter().enumerate()
                            {
                                lo[axis] = lo[axis].min(value);
                                hi[axis] = hi[axis].max(value);
                            }
                        }
                    }
                }
                for axis in 0..2 {
                    assert!((lo[axis] + hi[axis] - 1.0).abs() < 1e-10);
                    assert!(lo[axis] > 0.05 && hi[axis] < 0.95);
                }
                // The minimum window reserves the bottom 58px for navigation.
                if view == PresetView::Iso {
                    assert!(hi[1] < 1.0 - 58.0 / 512.0);
                }
            }
        }
    }
    #[test]
    fn ground_plane_allows_empty_document_editing() {
        let e = editor();
        let ray = Ray {
            origin: DVec3::new(4.5, 20.0, 5.5),
            direction: DVec3::NEG_Y,
        };
        assert_eq!(e.target(ray), Some([4, 0, 5]));
    }
    #[test]
    fn example_loads_and_renders_without_display() {
        let e = Editor::example().unwrap();
        assert!(
            e.snapshot()
                .layers
                .iter()
                .map(|l| l.voxels.len())
                .sum::<usize>()
                > 1000
        );
        let image = voxelite_render::render(
            e.snapshot(),
            &voxelite_render::RenderOptions {
                width: 64,
                height: 64,
                supersampling: 1,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(image.rgba.as_chunks::<4>().0.iter().any(|p| p[3] > 0));
    }
    #[test]
    fn foreign_layer_hit_is_not_silently_edited() {
        let mut e = editor();
        e.apply_at([2, 2, 2]).unwrap();
        e.add_layer().unwrap();
        e.tool = Tool::Erase;
        let before = e.engine.document().clone();
        let ray = Ray {
            origin: DVec3::new(2.5, 2.5, 40.0),
            direction: DVec3::NEG_Z,
        };
        assert!(e.click(ray, false).is_err());
        assert_eq!(e.engine.document(), &before);
        assert!(!e.click(ray, true).unwrap());
    }
    #[test]
    fn save_preserves_forward_metadata_and_portable_view() {
        let base = editor();
        let project = Project::new(base.engine.document().clone()).unwrap();
        let mut json: serde_json::Value =
            serde_json::from_slice(&project.to_bytes().unwrap()).unwrap();
        json["future_extension"] = serde_json::json!({"note": "retain me"});
        json["view"] = serde_json::json!({"camera": "iso"});
        let mut e = Editor::new(
            voxelite_format::read(serde_json::to_vec(&json).unwrap().as_slice()).unwrap(),
            None,
        )
        .unwrap();
        e.apply_at([1, 2, 3]).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("metadata.voxl");
        e.save(&path).unwrap();
        let saved: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(saved["future_extension"], json["future_extension"]);
        assert_eq!(saved["view"], json["view"]);
    }
    #[test]
    fn full_palette_failure_is_atomic_including_history() {
        let mut e = editor();
        let commands = (2..=256)
            .map(|slot| {
                Command::Palette(PaletteEdit::Add {
                    slot: Some(slot),
                    color: Swatch::rgb(0, (slot - 1) as u8, 0),
                })
            })
            .collect();
        e.execute("Fill palette", Command::Batch(commands)).unwrap();
        let before = e.engine.document().clone();
        let revision = e.engine.revision();
        e.color = Swatch::rgb(123, 45, 67);
        assert!(e.apply_at([1, 1, 1]).is_err());
        assert_eq!(e.engine.document(), &before);
        assert_eq!(e.engine.revision(), revision);
    }
    #[test]
    fn hidden_layers_and_background_picker_leave_color_unchanged() {
        let mut e = editor();
        e.apply_at([2, 2, 2]).unwrap();
        e.update_layer(e.layer, Some(false), None).unwrap();
        let ray = Ray {
            origin: DVec3::new(2.5, 2.5, 40.0),
            direction: DVec3::NEG_Z,
        };
        assert!(raycast(e.snapshot(), ray).is_none());
        e.color = Swatch::rgb(23, 45, 67);
        let color = e.color.clone();
        assert!(!e.click(ray, true).unwrap());
        assert_eq!(e.color, color);
        assert!(e.apply_at([2, 2, 2]).is_err());
    }
    #[test]
    fn projection_and_raycast_agree_across_orbits() {
        let mut e = editor();
        e.apply_at([15, 15, 15]).unwrap();
        let mut camera = OrbitCamera::fit(e.snapshot(), 1.4);
        for step in 0..80 {
            camera.yaw = f64::from(step) * 0.13;
            camera.pitch = -1.3 + f64::from(step % 20) * 0.13;
            let uv = camera.project(DVec3::splat(15.5), 1.4);
            let hit = raycast(e.snapshot(), camera.ray(uv, 1.4).unwrap()).unwrap();
            assert_eq!(hit.at, [15; 3]);
            assert_eq!(hit.normal.iter().map(|v| v.abs()).sum::<i32>(), 1);
        }
    }
}

#[cfg(test)]
mod overlay_tests;

#[cfg(test)]
#[path = "export_tests.rs"]
mod export_tests;
