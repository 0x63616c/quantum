//! Immutable timeline previews and guarded explicit restoration.
use crate::editor::{Editor, EditorError, EditorResult};
use std::sync::Arc;
use voxelite_core::*;
/// Source of a captured, read-only viewport state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewSource {
    /// A retained edit, or the initial state.
    History(Option<Uuid>),
    /// A durable named version.
    Version(Uuid),
}
/// Preview pins immutable content and the live restore preconditions.
#[derive(Clone)]
pub struct Preview {
    /// Captured content, never installed by scrubbing.
    pub snapshot: Arc<Snapshot>,
    /// Stable source identity.
    pub source: PreviewSource,
    /// Human-readable description.
    pub label: String,
    /// Live revision captured on entry.
    pub revision: u64,
    /// Live history head captured on entry.
    pub head: Option<Uuid>,
}
impl Editor {
    /// Content shown by the viewport and inspector; saving always uses live content.
    pub fn display_snapshot(&self) -> &Snapshot {
        self.preview
            .as_ref()
            .map_or_else(|| self.snapshot(), |p| &p.snapshot)
    }
    pub(crate) fn ensure_live(&self) -> EditorResult<()> {
        if self.preview.is_some() {
            Err(EditorError::Gesture(
                "History preview is read-only. Return to live or Restore.".into(),
            ))
        } else {
            Ok(())
        }
    }
    /// Scrub without changing revision, dirty state, or undo/redo.
    pub fn preview_history(&mut self, entry: Option<Uuid>) -> EditorResult<()> {
        let doc = self.engine.preview(entry)?;
        let label = entry
            .and_then(|id| self.engine.timeline().find(|e| e.id == id))
            .map_or_else(|| "Initial state".into(), |e| e.label.clone());
        self.set_preview(
            PreviewSource::History(entry),
            Arc::new(doc.content.clone()),
            label,
        );
        Ok(())
    }
    /// Preview a durable named snapshot without altering the live project.
    pub fn preview_version(&mut self, id: Uuid) -> EditorResult<()> {
        let v = self
            .engine
            .versions()
            .find(|v| v.id == id)
            .ok_or_else(|| EditorError::Gesture("Version no longer exists".into()))?;
        let (snapshot, label) = (v.snapshot.clone(), v.name.clone());
        self.set_preview(PreviewSource::Version(id), snapshot, label);
        Ok(())
    }
    fn set_preview(&mut self, source: PreviewSource, snapshot: Arc<Snapshot>, label: String) {
        self.preview = Some(Preview {
            snapshot,
            source,
            label,
            revision: self.engine.revision(),
            head: self.engine.head(),
        });
        self.box_start = None;
    }
    /// Exit preview; live edits that arrived while previewing remain intact.
    pub fn return_live(&mut self) {
        self.preview = None;
    }
    /// Explicit restore adds one reversible transaction and rejects an intervening edit.
    pub fn restore_preview(&mut self) -> EditorResult<()> {
        let p = self
            .preview
            .as_ref()
            .ok_or_else(|| EditorError::Gesture("Choose a history state first".into()))?;
        match p.source {
            PreviewSource::History(entry) => {
                self.engine.restore_history(
                    p.revision,
                    p.head,
                    entry,
                    Uuid::new_v4(),
                    Actor::new(Origin::Human),
                )?;
            }
            PreviewSource::Version(id) => {
                self.engine.execute(Transaction {
                    expected_revision: p.revision,
                    id: Uuid::new_v4(),
                    actor: Actor::new(Origin::Human),
                    label: format!("Restore {}", p.label.chars().take(110).collect::<String>()),
                    command: Command::RestoreVersion {
                        id,
                        expected_head: p.head,
                    },
                })?;
            }
        }
        self.preview = None;
        self.reconcile();
        Ok(())
    }
    /// Save a persistent, nonrecursive snapshot of live content.
    pub fn save_version(&mut self, name: String) -> EditorResult<Uuid> {
        let id = Uuid::new_v4();
        self.execute(
            "Save named version",
            Command::SaveVersion {
                id,
                name,
                parent: None,
            },
        )?;
        Ok(id)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scrub_is_read_only_restore_is_new_undoable_edit() {
        let mut e = Editor::blank().unwrap();
        e.apply_at([0, 0, 0]).unwrap();
        let first = e.engine.head();
        e.apply_at([1, 0, 0]).unwrap();
        let live = e.engine.snapshot();
        let revision = e.engine.revision();
        let head = e.engine.head();
        e.preview_history(first).unwrap();
        assert_eq!(e.display_snapshot().layers[0].voxels.len(), 1);
        assert_eq!(e.engine.snapshot(), live);
        assert_eq!(e.engine.revision(), revision);
        assert_eq!(e.engine.head(), head);
        assert!(e.apply_at([2, 0, 0]).is_err());
        assert!(e.undo().is_err());
        e.restore_preview().unwrap();
        assert_eq!(e.snapshot().layers[0].voxels.len(), 1);
        assert_eq!(e.engine.revision(), revision + 1);
        e.undo().unwrap();
        assert_eq!(e.snapshot(), &live.content);
    }
    #[test]
    fn preview_conflict_does_not_overwrite_live_and_versions_persist() {
        let mut e = Editor::blank().unwrap();
        let v = e.save_version("Before".into()).unwrap();
        e.apply_at([0, 0, 0]).unwrap();
        e.preview_version(v).unwrap();
        e.engine
            .execute(Transaction {
                expected_revision: e.engine.revision(),
                id: Uuid::new_v4(),
                actor: Actor::new(Origin::Agent),
                label: "Agent edit".into(),
                command: Command::EditDocument {
                    name: Some("Changed".into()),
                    bounds: None,
                },
            })
            .unwrap();
        assert!(e.restore_preview().is_err());
        assert_eq!(e.snapshot().name, "Changed");
        e.return_live();
        e.preview_version(v).unwrap();
        e.restore_preview().unwrap();
        assert_eq!(e.snapshot().layers[0].voxels.len(), 0);
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("versions.voxl");
        e.save(&p).unwrap();
        let r = Editor::open(&p).unwrap();
        assert_eq!(r.engine.versions().next().unwrap().id, v);
    }
    #[test]
    fn timeline_includes_undone_steps() {
        let mut e = Editor::blank().unwrap();
        e.apply_at([0, 0, 0]).unwrap();
        let id = e.engine.head();
        e.undo().unwrap();
        assert_eq!(e.engine.timeline().count(), 1);
        e.preview_history(id).unwrap();
        assert_eq!(e.display_snapshot().layers[0].voxels.len(), 1);
        assert_eq!(e.snapshot().layers[0].voxels.len(), 0);
    }
}
