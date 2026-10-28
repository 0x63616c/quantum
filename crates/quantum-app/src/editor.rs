//! Human tool gestures translated into atomic shared-core commands.
#[cfg(test)]
use glam::DVec3;
use quantum_core::*;
use quantum_format::session::SavePolicy;
pub use quantum_format::session::{SaveTask, SavedProject};
use quantum_format::{Project, session::DocumentSession};
use quantum_plugin_api::{
    ExportImage, ExportResult, OverlayFrame, OverlayInput, OverlayNavigation, OverlayOutcome,
    SettingValue, ToolCx, ToolGesture, ToolHit, ToolInput, ToolLayer, ToolOutcome,
};
use quantum_plugin_host::Host;
use quantum_render::RenderOptions;

use std::cell::{Ref, RefCell};
use std::path::Path;
use std::path::PathBuf;

mod camera;
pub use camera::{Hit, OrbitCamera, Ray, raycast};

/// Supply the CPU-rendered view selected by the app to a snapshot exporter.
pub fn render_view(
    options: &RenderOptions,
) -> impl Fn(&Snapshot) -> ExportResult<ExportImage> + '_ {
    move |snapshot| {
        let image = quantum_render::render(snapshot, options)?;
        Ok(ExportImage {
            width: image.width,
            height: image.height,
            rgba: image.rgba,
        })
    }
}

mod error;
pub use error::{EditorError, EditorResult};

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
            Self::Place => "dev.quantum.tools.place.tool",
            Self::Erase => "dev.quantum.tools.erase.tool",
            Self::Paint => "dev.quantum.tools.paint.tool",
            Self::Fill => "dev.quantum.tools.fill.tool",
            Self::Box => "dev.quantum.tools.box.tool",
            Self::Picker => "dev.quantum.tools.picker.tool",
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

/// One open project, with UI preferences separate from portable document content.
pub struct Editor {
    pub(crate) session: DocumentSession,
    // Hover/preview queries also record host timings and isolate plugin panics.
    pub(crate) plugins: RefCell<Host>,
    /// Persistent software viewport occupancy, shared with background render tasks.
    pub raster: quantum_render::RasterCache,
    /// Read-only preview, separate from the live authority.
    pub preview: Option<crate::history::Preview>,
    pub(crate) playback: Option<crate::history::Playback>,
    /// Active layer identity, reconciled after undo/redo.
    pub layer: Uuid,
    /// Selected editing tool.
    pub tool: Tool,
    /// Selected brush color; imported lazily as part of the edit transaction.
    pub color: Swatch,
    pub(crate) palette_state: quantum_plugin_api::PaletteState,
    /// Last admitted transient discussion state.
    pub comments_state: quantum_plugin_api::CommentsState,
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
// Use a single static allocation across the example entry point and launcher.
// Repeating include_bytes! embeds another 2.75 MB copy in native debug builds.
pub(crate) static AUTUMN_PROJECT_BYTES: &[u8] =
    include_bytes!("../../../examples/autumn-workshop.voxl");

impl Editor {
    /// Open a validated portable project, preserving unknown optional metadata.
    pub fn new(project: Project, path: Option<PathBuf>) -> EditorResult<Self> {
        Self::from_session(DocumentSession::new(project, path, Limits::default())?)
    }
    /// Attach UI-thread plugins to a session that may have been loaded in the background.
    #[expect(clippy::expect_used, reason = "internal invariant: valid palette")]
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
            raster: quantum_render::RasterCache::default(),
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
        Self::new(quantum_format::read(AUTUMN_PROJECT_BYTES)?, None)
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
        Self::load_session(path).and_then(Self::from_session)
    }
    /// Load a session with filename context, before replacing any live editor.
    pub fn load_session(path: &Path) -> EditorResult<DocumentSession> {
        DocumentSession::open(path, Limits::default()).map_err(|error| EditorError::Open {
            path: path.to_owned(),
            source: Box::new(error.into()),
        })
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
    pub fn save(&mut self, path: &Path) -> EditorResult<()> {
        let saved = self.prepare_save(path).write()?;
        self.finish_save(saved);
        Ok(())
    }
    /// Capture the current document without performing filesystem I/O.
    pub fn prepare_save(&self, path: &Path) -> SaveTask {
        self.session.prepare_save(path, SavePolicy::ReplaceSelected)
    }
    /// Adopt a successful save; edits made after its snapshot remain dirty.
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
    pub fn render_plan(&self) -> quantum_render::RenderPlan {
        self.plugins.borrow_mut().render_plan()
    }
    /// Find a format by extension in the live registry, without a built-in format table.
    pub fn export_format(
        &self,
        extension: &str,
    ) -> EditorResult<quantum_plugin_api::FormatContribution> {
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
            .ok_or(EditorError::ExportUnavailable)
    }
    /// Resolve an export format in the live registry before starting background work.
    pub fn prepare_export(&self, id: &str) -> EditorResult<quantum_plugin_host::ExportJob> {
        self.plugins
            .borrow_mut()
            .prepare_export(id)?
            .ok_or(EditorError::ExportUnavailable)
    }
    /// Admit completed bytes only while the format's plugin remains enabled.
    pub fn finish_export(
        &self,
        result: quantum_plugin_host::ExportCompletion,
    ) -> EditorResult<Vec<u8>> {
        self.plugins
            .borrow_mut()
            .finish_export(result)?
            .ok_or(EditorError::ExportUnavailable)
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
            quantum_plugin_gizmo_grid::GRID_SETTING,
            SettingValue::Bool(enabled),
        )?)
    }
    /// Effective grid setting; disabled or absent plugins expose no grid.
    pub fn grid(&self) -> bool {
        self.plugins
            .borrow()
            .setting(quantum_plugin_gizmo_grid::GRID_SETTING)
            == Some(&SettingValue::Bool(true))
    }
    /// Enabled mirror axis settings, owned by the mirror plugin and local to this editor.
    pub fn mirror_axes(&self) -> [bool; 3] {
        let host = self.plugins.borrow();
        quantum_plugin_mirror::AXIS_SETTINGS
            .map(|key| host.setting(key) == Some(&SettingValue::Bool(true)))
    }
    /// Change one plugin setting without affecting the document or undo history.
    /// Missing or disabled mirror commands are inert.
    pub fn set_mirror_axis(&mut self, axis: usize, enabled: bool) -> EditorResult<bool> {
        let Some(key) = quantum_plugin_mirror::AXIS_SETTINGS.get(axis) else {
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
        if !host.has_comments_panel(quantum_plugin_comments::PANEL_ID)? {
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
    fn voxel(&self, layer: Uuid, at: Pos) -> quantum_core::Result<u16> {
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
    fn layer(&self, id: Uuid) -> quantum_core::Result<ToolLayer> {
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
mod overlay_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "export_tests.rs"]
mod export_tests;
