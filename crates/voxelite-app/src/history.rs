//! Immutable timeline previews and guarded explicit restoration.
#[cfg(test)]
mod playback_tests;

use crate::editor::{Editor, EditorError, EditorResult};
use std::sync::Arc;
use std::time::{Duration, Instant};
use voxelite_core::*;

/// Default playback cadence: four retained history states per second.
pub const HISTORY_STEP_INTERVAL: Duration = Duration::from_millis(250);

pub(crate) struct Playback {
    next_step: Instant,
    revision: u64,
}
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
/// User-facing labels for old MCP entries; custom labels are preserved verbatim.
pub fn display_label(label: &str) -> &str {
    match label {
        "comments add" => "Add comment",
        "comments reply" => "Reply to comment",
        "comments resolve" => "Resolve comment",
        "versions save" => "Save named version",
        "versions restore" => "Restore named version",
        "history undo" => "Undo edit",
        "history redo" => "Redo edit",
        _ => label,
    }
}
impl Editor {
    /// Retained states in playback order, including redo entries.
    pub fn timeline_entries(&self) -> impl Iterator<Item = Option<Uuid>> + '_ {
        std::iter::once(None)
            .filter(|_| self.session.engine.initial_retained())
            .chain(self.session.engine.timeline().map(|e| Some(e.id)))
    }

    /// Whether history is advancing automatically.
    pub fn history_playing(&self) -> bool {
        self.playback.is_some()
    }

    /// Pause at the displayed state without changing the live document.
    pub fn pause_history(&mut self) {
        self.playback = None;
    }

    /// Start at the current timeline state, or restart at the first when at the end.
    /// Returns whether a preview was installed; fewer than two states cannot play.
    pub fn play_history(&mut self, now: Instant) -> EditorResult<bool> {
        self.pause_history();
        let entries: Vec<_> = self.timeline_entries().collect();
        if entries.len() < 2 {
            return Ok(false);
        }
        let index = self.timeline_index();
        let index = if index >= entries.len() - 1 { 0 } else { index };
        self.load_history_preview(entries[index])?;
        self.playback = Some(Playback {
            next_step: now + HISTORY_STEP_INTERVAL,
            revision: self.session.engine.revision(),
        });
        Ok(true)
    }

    /// Advance at most one state when due. Delayed ticks never skip build steps.
    /// Stops on the final state, an intervening live edit, or a preview error.
    pub fn tick_history(&mut self, now: Instant) -> EditorResult<bool> {
        let Some(playback) = &self.playback else {
            return Ok(false);
        };
        if playback.revision != self.session.engine.revision() {
            self.pause_history();
            return Ok(false);
        }
        if now < playback.next_step {
            return Ok(false);
        }
        let entries: Vec<_> = self.timeline_entries().collect();
        let next = self.timeline_index() + 1;
        self.pause_history();
        let Some(entry) = entries.get(next) else {
            return Ok(false);
        };
        self.load_history_preview(*entry)?;
        if next + 1 < entries.len() {
            self.playback = Some(Playback {
                next_step: now + HISTORY_STEP_INTERVAL,
                revision: self.session.engine.revision(),
            });
        }
        Ok(true)
    }

    /// Timeline cursor follows the preview or the actual applied head, never a redo entry.
    pub fn timeline_index(&self) -> usize {
        let source = self
            .preview
            .as_ref()
            .map(|p| p.source)
            .unwrap_or(PreviewSource::History(self.session.engine.head()));
        self.timeline_entries()
            .position(|id| source == PreviewSource::History(id))
            .unwrap_or(0)
    }

    /// Content shown by the viewport and inspector; saving always uses live content.
    pub fn display_snapshot(&self) -> &Snapshot {
        self.preview
            .as_ref()
            .map_or_else(|| self.snapshot(), |p| &p.snapshot)
    }
    pub(crate) fn ensure_live(&mut self) -> EditorResult<()> {
        self.pause_history();
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
        self.pause_history();
        self.load_history_preview(entry)
    }
    fn load_history_preview(&mut self, entry: Option<Uuid>) -> EditorResult<()> {
        let doc = self.session.engine.preview(entry)?;
        let label = entry
            .and_then(|id| self.session.engine.timeline().find(|e| e.id == id))
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
        self.pause_history();
        let v = self
            .session
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
            revision: self.session.engine.revision(),
            head: self.session.engine.head(),
        });
        self.box_start = None;
    }
    /// Exit preview; live edits that arrived while previewing remain intact.
    pub fn return_live(&mut self) {
        self.pause_history();
        self.preview = None;
    }
    /// Explicit restore adds one reversible transaction and rejects an intervening edit.
    pub fn restore_preview(&mut self) -> EditorResult<()> {
        self.pause_history();
        let p = self
            .preview
            .as_ref()
            .ok_or_else(|| EditorError::Gesture("Choose a history state first".into()))?;
        match p.source {
            PreviewSource::History(entry) => {
                self.session.engine.restore_history(
                    p.revision,
                    p.head,
                    entry,
                    Uuid::new_v4(),
                    Actor::new(Origin::Human),
                )?;
            }
            PreviewSource::Version(id) => {
                self.session.engine.execute(Transaction {
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
    fn bundled_build_history_opens_scrubs_and_survives_an_unchanged_save() {
        let mut e = Editor::example().unwrap();
        let count = e.engine.history().count();
        assert!((20..=60).contains(&count));
        assert_eq!(e.timeline_index(), count);
        assert!(!e.engine.is_dirty());
        let live = e.engine.snapshot();
        let revision = e.engine.revision();
        e.preview_history(None).unwrap();
        assert!(
            e.display_snapshot()
                .layers
                .iter()
                .all(|l| l.voxels.is_empty())
        );
        assert_eq!(e.engine.snapshot(), live);
        assert_eq!(e.engine.revision(), revision);
        assert_eq!(e.timeline_index(), 0);
        e.return_live();
        e.undo().unwrap();
        assert_eq!(e.timeline_index(), count - 1);
        e.redo().unwrap();
        assert_eq!(e.engine.snapshot(), live);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("demo.voxl");
        e.save(&path).unwrap();
        let reopened = Editor::open(&path).unwrap();
        assert_eq!(reopened.engine.history().count(), count);
        assert_eq!(reopened.engine.snapshot(), live);
        assert!(!reopened.engine.is_dirty());
    }

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
        e.session
            .engine
            .execute(Transaction {
                expected_revision: e.session.engine.revision(),
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
        assert_eq!(e.timeline_index(), 0);
        e.preview_history(id).unwrap();
        assert_eq!(e.display_snapshot().layers[0].voxels.len(), 1);
        assert_eq!(e.timeline_index(), 1);
        assert_eq!(e.snapshot().layers[0].voxels.len(), 0);
    }
}
