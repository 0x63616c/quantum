//! Human tool gestures translated into atomic shared-core commands.
use crate::persistence::{self, Fingerprint};
pub use crate::persistence::{SaveTask, SavedProject};
use glam::DVec3;
use std::path::{Path, PathBuf};
use voxelite_core::*;
use voxelite_format::Project;
use voxelite_render::{Camera, PresetView, Projection};

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
    /// Elevation in radians; clamped away from pole singularities.
    pub pitch: f64,
    /// Vertical world-space viewport extent.
    pub span: f64,
    distance: f64,
}
impl OrbitCamera {
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
        Self {
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
            span: height,
            distance: DVec3::from(camera.eye).distance(camera.target.into()),
        }
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
    /// Shared validated command engine.
    pub engine: Engine,
    /// Read-only preview, separate from the live authority.
    pub preview: Option<crate::history::Preview>,
    /// Active layer identity, reconciled after undo/redo.
    pub layer: Uuid,
    /// Selected editing tool.
    pub tool: Tool,
    /// Independent centered mirror axes.
    pub mirror: [bool; 3],
    /// Selected brush color; imported lazily as part of the edit transaction.
    pub color: Swatch,
    /// Camera state, unaffected by undo/redo.
    pub camera: OrbitCamera,
    /// First corner of an uncommitted box.
    pub box_start: Option<Pos>,
    /// Current successful open/save path; examples start untitled.
    pub path: Option<PathBuf>,
    project: Project,
    saved_file: Option<(PathBuf, Fingerprint)>,
}
impl Editor {
    /// Open a validated portable project, preserving unknown optional metadata.
    pub fn new(project: Project, path: Option<PathBuf>) -> EditorResult<Self> {
        let saved_file = path
            .as_deref()
            .map(|path| {
                let path = path.canonicalize()?;
                let hash = persistence::fingerprint(&persistence::read(&path)?);
                Ok::<_, EditorError>((path, hash))
            })
            .transpose()?;
        let engine = Engine::new(project.document.clone())?;
        let s = &engine.document().content;
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
            engine,
            preview: None,
            layer,
            tool: Tool::Place,
            mirror: [false; 3],
            color,
            camera,
            box_start: None,
            path,
            project,
            saved_file,
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
    pub fn open(path: &Path) -> EditorResult<Self> {
        let path = path.canonicalize()?;
        let bytes = persistence::read(&path)?;
        let mut editor = Self::new(voxelite_format::read(bytes.as_slice())?, None)?;
        editor.saved_file = Some((path.clone(), persistence::fingerprint(&bytes)));
        editor.path = Some(path);
        Ok(editor)
    }
    /// Capture portable metadata and current content for a local MCP attachment.
    pub fn project(&self) -> Project {
        let mut p = self.project.clone();
        p.document = self.engine.document().clone();
        p
    }
    /// Reconcile local selection after commands committed by the attached agent.
    pub fn agent_updated(&mut self) {
        self.reconcile();
        self.box_start = None;
    }
    /// Current document content.
    pub fn snapshot(&self) -> &Snapshot {
        &self.engine.document().content
    }
    /// Save through a synced temporary file in the destination directory, then atomically rename.
    /// Dirty state is cleared only after successful persistence.
    pub fn save(&mut self, path: &Path) -> EditorResult<()> {
        let saved = self.prepare_save(path).write()?;
        self.finish_save(saved);
        Ok(())
    }
    /// Capture the current document without performing filesystem I/O.
    pub fn prepare_save(&self, path: &Path) -> SaveTask {
        let mut project = self.project.clone();
        project.document = self.engine.document().clone();
        SaveTask {
            project,
            path: path.to_path_buf(),
            baseline: self.saved_file.clone(),
            revision: self.engine.revision(),
        }
    }
    /// Adopt a successful save; edits made after its snapshot remain dirty.
    pub fn finish_save(&mut self, saved: SavedProject) {
        if saved.project.document.id != self.engine.document().id {
            return;
        }
        self.project = saved.project;
        self.path = Some(saved.path.clone());
        self.saved_file = Some((saved.path, saved.fingerprint));
        if self.engine.revision() == saved.revision {
            self.engine.mark_saved();
        }
    }
    /// Execute a single attributed, atomic command.
    pub fn execute(&mut self, label: &str, command: Command) -> EditorResult<bool> {
        self.ensure_live()?;
        let receipt = self.engine.execute(Transaction {
            expected_revision: self.engine.revision(),
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
    pub fn ensure_live_revert(&mut self, id: Uuid) -> EditorResult<()> {
        self.ensure_live()?;
        self.engine.revert(
            self.engine.revision(),
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
        if let Some(id) = self.engine.head() {
            self.engine.undo(self.engine.revision(), id)?;
        }
        self.reconcile();
        Ok(())
    }
    /// Redo the most recently undone gesture.
    pub fn redo(&mut self) -> EditorResult<()> {
        self.ensure_live()?;
        if let Some(id) = self.engine.redo_head() {
            self.engine.redo(self.engine.revision(), id)?;
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
    /// Locate a tool target, using visible faces or the document's bottom work plane.
    pub fn target(&self, ray: Ray) -> Option<Pos> {
        if let Some(hit) = raycast(self.snapshot(), ray) {
            let p = if matches!(self.tool, Tool::Place | Tool::Box) {
                std::array::from_fn(|a| hit.at[a] + hit.normal[a])
            } else {
                hit.at
            };
            return self.snapshot().bounds.contains(p).then_some(p);
        }
        if !matches!(self.tool, Tool::Place | Tool::Box) || ray.direction.y.abs() < 1e-12 {
            return None;
        }
        let t = (f64::from(self.snapshot().bounds.min[1]) - ray.origin.y) / ray.direction.y;
        if !t.is_finite() || t < 0.0 {
            return None;
        }
        let mut p = (ray.origin + ray.direction * t)
            .floor()
            .as_ivec3()
            .to_array();
        p[1] = self.snapshot().bounds.min[1];
        self.snapshot().bounds.contains(p).then_some(p)
    }
    /// Apply a pointer click, sampling without mutation when the picker is active.
    pub fn click(&mut self, ray: Ray, sample_only: bool) -> EditorResult<bool> {
        let hit = raycast(self.snapshot(), ray);
        if sample_only || self.tool == Tool::Picker {
            if let Some(hit) = hit {
                self.color = self
                    .snapshot()
                    .palette
                    .get(hit.slot)
                    .expect("valid slot")
                    .clone();
            }
            return Ok(false);
        }
        if let Some(hit) = hit
            && hit.layer != self.layer
            && matches!(self.tool, Tool::Paint | Tool::Erase | Tool::Fill)
        {
            return Err(EditorError::Gesture(
                "Select the hit voxel’s layer to edit it".into(),
            ));
        }
        let Some(at) = self.target(ray) else {
            return Ok(false);
        };
        self.apply_at(at)
    }
    /// Apply a tool at an explicit integer coordinate, also used by the keyboard cursor.
    pub fn apply_at(&mut self, at: Pos) -> EditorResult<bool> {
        self.ensure_live()?;
        if !self.snapshot().bounds.contains(at) {
            return Err(EditorError::Gesture("Outside document bounds".into()));
        }
        if self.tool == Tool::Picker {
            if let Some((slot, _)) = self.snapshot().sample(at) {
                self.color = self
                    .snapshot()
                    .palette
                    .get(slot)
                    .expect("valid slot")
                    .clone();
            }
            return Ok(false);
        }
        let layer = self.snapshot().layer(self.layer)?;
        if layer.locked || !layer.visible {
            return Err(EditorError::Gesture(
                "The active layer is locked or hidden".into(),
            ));
        }
        if self.tool == Tool::Box && self.box_start.is_none() {
            self.box_start = Some(at);
            return Ok(false);
        }
        // A no-op brush must not add unused palette entries.
        if matches!(self.tool, Tool::Paint | Tool::Erase | Tool::Fill) && layer.voxels.get(at) == 0
        {
            return Ok(false);
        }
        let mut commands = Vec::new();
        let slot = if self.tool == Tool::Erase {
            0
        } else if let Some(s) = self
            .snapshot()
            .palette
            .slots
            .iter()
            .find(|s| s.color.rgba == self.color.rgba)
        {
            s.index
        } else {
            let slot = (1..=256)
                .find(|s| self.snapshot().palette.get(*s).is_none())
                .ok_or_else(|| EditorError::Gesture("Palette is full (256 colors)".into()))?;
            commands.push(Command::Palette(PaletteEdit::Add {
                slot: Some(slot),
                color: self.color.clone(),
            }));
            slot
        };
        let mirror = Mirror::centered(self.snapshot().bounds, self.mirror);
        commands.push(match self.tool {
            Tool::Fill => Command::FloodFill {
                layer: self.layer,
                seed: at,
                slot,
                region: None,
                cap: 1_048_576,
                mirror,
            },
            Tool::Box => {
                let start = self.box_start.expect("box start established");
                Command::Primitive {
                    layer: self.layer,
                    shape: Shape::Box {
                        region: Region {
                            min: std::array::from_fn(|a| start[a].min(at[a])),
                            max: std::array::from_fn(|a| start[a].max(at[a]) + 1),
                        },
                        hollow: false,
                    },
                    mode: Mode::Place,
                    slot: Some(slot),
                    mirror,
                }
            }
            _ => Command::SetVoxels {
                layer: self.layer,
                voxels: vec![Voxel { at, slot }],
                mode: match self.tool {
                    Tool::Erase => Mode::Erase,
                    Tool::Paint => Mode::Paint,
                    _ => Mode::Place,
                },
                mirror,
            },
        });
        self.execute(self.tool.name(), Command::Batch(commands))
    }
}

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
        e.mirror = [true; 3];
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
        e.color = Preset::Pico8.palette().get(4).unwrap().clone();
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
