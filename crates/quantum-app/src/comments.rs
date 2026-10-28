//! Host adapters for registered discussion, placement and overlay behavior.
use crate::editor::{Editor, EditorError, EditorResult, OrbitCamera, Ray, raycast};

use quantum_plugin_api::*;
use quantum_plugin_comments::{PANEL_ID, TOOL_ID};

impl Editor {
    /// Whether the discussion panel is declared by an enabled plugin.
    pub fn comments_enabled(&self) -> bool {
        self.plugins
            .borrow()
            .registry()
            .panels()
            .any(|(_, p)| p.id == PANEL_ID)
    }
    /// Read presentation data from the enabled discussion panel.
    pub fn comments_view(&self) -> Option<CommentsView> {
        self.plugins
            .borrow_mut()
            .comments_view(PANEL_ID, &EditorCommentsCx(self))
            .ok()
            .flatten()
    }
    /// Admit a discussion proposal before adopting transient state.
    pub fn comments_action(
        &mut self,
        action: CommentsAction,
    ) -> EditorResult<Option<CommentsOutcome>> {
        Ok(self.admit_comments(action)?.map(|(out, _)| out))
    }
    fn admit_comments(
        &mut self,
        action: CommentsAction,
    ) -> EditorResult<Option<(CommentsOutcome, bool)>> {
        let outcome = self.plugins.borrow_mut().dispatch_comments(
            PANEL_ID,
            &EditorCommentsCx(self),
            action,
        )?;
        let Some(outcome) = outcome else {
            return Ok(None);
        };
        let changed = match outcome.commands.as_slice() {
            [] => false,
            [command] => self.execute(&outcome.label, command.clone())?,
            commands => self.execute(&outcome.label, Command::Batch(commands.to_vec()))?,
        };
        self.comments_state = outcome.state.clone();
        Ok(Some((outcome, changed)))
    }
    /// Place a draft through the enabled tool, then the enabled panel.
    pub fn comment_gesture(
        &mut self,
        ray: Option<Ray>,
        gesture: ToolGesture,
    ) -> EditorResult<Option<CommentsOutcome>> {
        let outcome =
            self.registered_tool_outcome(TOOL_ID, ray, gesture, self.comments_state.first)?;
        match outcome.and_then(|o| o.target) {
            Some(at) => self.comments_action(CommentsAction::Anchor(at)),
            None => Ok(None),
        }
    }
    /// Add a plain-text thread using the enabled discussion implementation.
    pub fn comment(&mut self, anchor: Anchor, text: String) -> EditorResult<Uuid> {
        let id = Uuid::new_v4();
        self.comments_action(CommentsAction::Create {
            id,
            anchor,
            layer: CommentLayer::Automatic,
            message: Uuid::new_v4(),
            text,
        })?
        .ok_or_else(|| EditorError::Gesture("Comments plugin is unavailable".into()))?;
        Ok(id)
    }
    /// Append an attributed message through shared history.
    pub fn reply(&mut self, id: Uuid, text: String) -> EditorResult<bool> {
        Ok(self
            .admit_comments(CommentsAction::Reply {
                id,
                message: Uuid::new_v4(),
                text,
            })?
            .is_some_and(|(_, changed)| changed))
    }
    /// Resolve or reopen without deleting the discussion.
    pub fn resolve_comment(&mut self, id: Uuid, resolved: bool) -> EditorResult<bool> {
        Ok(self
            .admit_comments(CommentsAction::Resolve { id, resolved })?
            .is_some_and(|(_, changed)| changed))
    }
    /// Project enabled pin primitives with the host renderer's camera and picking.
    pub fn comment_pins(&self, camera: OrbitCamera, aspect: f64) -> Vec<OverlayPin> {
        self.plugins
            .borrow_mut()
            .draw_overlays_with(
                &CommentsOverlayCx {
                    comments: EditorCommentsCx(self),
                    camera,
                    aspect,
                },
                None,
            )
            .into_iter()
            .filter(|frame| frame.slot == RenderPassSlot::Overlay)
            .flat_map(|frame| frame.primitives)
            .filter_map(|p| match p {
                OverlayPrimitive::Pin(pin) => Some(pin),
                _ => None,
            })
            .collect()
    }
}

struct EditorCommentsCx<'a>(&'a Editor);
impl CommentsCx for EditorCommentsCx<'_> {
    fn snapshot(&self) -> &Snapshot {
        self.0.display_snapshot()
    }
    fn live_snapshot(&self) -> &Snapshot {
        self.0.snapshot()
    }
    fn state(&self) -> &CommentsState {
        &self.0.comments_state
    }
    fn preview(&self) -> bool {
        self.0.preview.is_some()
    }
}

struct CommentsOverlayCx<'a> {
    comments: EditorCommentsCx<'a>,
    camera: OrbitCamera,
    aspect: f64,
}
impl OverlayCx for CommentsOverlayCx<'_> {
    fn bounds(&self) -> Region {
        self.comments.snapshot().bounds
    }
    fn camera_basis(&self) -> [[f64; 3]; 3] {
        self.camera.overlay_basis()
    }
    fn setting(&self, _: &str) -> Option<&SettingValue> {
        None
    }
    fn comments(&self) -> Option<&dyn CommentsCx> {
        Some(&self.comments)
    }
    fn project(&self, point: [f64; 3]) -> Option<[f64; 2]> {
        (self.aspect.is_finite() && self.aspect > 0.0)
            .then(|| self.camera.project(point.into(), self.aspect))
    }
    fn pick(&self, uv: [f64; 2]) -> Option<Pos> {
        self.camera
            .ray(uv, self.aspect)
            .and_then(|ray| raycast(self.comments.snapshot(), ray))
            .map(|hit| hit.at)
    }
    fn emit(&mut self, _: OverlayPrimitive) {}
}

#[cfg(test)]
#[path = "comment_tests.rs"]
mod tests;
