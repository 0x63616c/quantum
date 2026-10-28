use crate::*;
use slotmap::{SlotMap, new_key_type};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

new_key_type! { struct HistoryId; }

#[cfg(test)]
mod compare_tests;
#[cfg(all(test, feature = "unbounded-voxels"))]
mod measurements;

/// Runtime history provenance. Only the role is portable project content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Actor {
    /// Human or agent origin.
    pub origin: Origin,
    /// Opaque local client identity.
    pub client_id: Uuid,
    /// Public command/tool name.
    pub tool: String,
}
impl Actor {
    /// Construct a role-only actor for local tools and deterministic tests.
    pub fn new(origin: Origin) -> Self {
        Self {
            origin,
            client_id: Uuid::nil(),
            tool: "core".into(),
        }
    }
}
/// Captured revision and human-readable transaction attribution.
#[derive(Clone, Debug)]
pub struct Transaction {
    /// Captured revision, checked before preparation.
    pub expected_revision: u64,
    /// Caller-supplied identity used as the resulting history entry ID.
    pub id: Uuid,
    /// Who initiated this edit.
    pub actor: Actor,
    /// Display label (1–128 bytes).
    pub label: String,
    /// Typed edit.
    pub command: Command,
}
/// Actual cell changes, separate from charged preparation work.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Changes {
    /// Previously empty layer-cells.
    pub added: usize,
    /// Removed layer-cells.
    pub removed: usize,
    /// Changed palette references.
    pub recolored: usize,
    /// Bounding region of changed cell coordinates.
    pub bounds: Option<Region>,
    /// Layers whose cells or metadata changed.
    pub layers: BTreeSet<Uuid>,
    /// Palette appearance or ordering changed independently of voxel slots.
    pub palette_changed: bool,
    /// Document metadata, layer ordering, or comments changed.
    pub metadata_changed: bool,
}
impl Changes {
    /// Number of occupied-state/reference changes over all layers.
    pub fn voxel_count(&self) -> usize {
        self.added + self.removed + self.recolored
    }
}
/// Successful preparation or commit summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    /// Resulting revision (unchanged for no-ops or previews).
    pub revision: u64,
    /// New history entry, or None for a no-op.
    pub entry: Option<Uuid>,
    /// Cell and metadata changes.
    pub changes: Changes,
    /// Candidate work charged using estimator version 1.
    pub work_units: usize,
}
/// Retained reversible patch. Chunks are shared, with copy-on-write on mutation.
#[derive(Clone, Debug)]
pub struct HistoryEntry {
    /// Stable entry and operation identity.
    pub id: Uuid,
    /// Actor attribution.
    pub actor: Actor,
    /// Display label.
    pub label: String,
    /// Revision before original application.
    pub base_revision: u64,
    /// Revision after original application (undo/redo do not rewrite it).
    pub result_revision: u64,
    /// Original effect.
    pub changes: Changes,
    before: Arc<Document>,
    after: Arc<Document>,
    bytes: usize,
    applied: bool,
}
/// Budget limits independent of the hosting UI or transport.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum candidate cells across all subcommands.
    pub work_units: usize,
    /// Retained history capacity in bytes.
    pub history_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            work_units: MAX_WORK,
            history_bytes: 128 * 1024 * 1024,
        }
    }
}
/// A bookmark into retained history, invalidated with its underlying branch.
#[derive(Clone, Debug)]
pub struct Checkpoint {
    /// Stable bookmark identity.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// History cursor, None for the initial state.
    pub head: Option<Uuid>,
}
/// Shared human/agent authority for one validated document.
///
/// Every mutation is prepared privately and validated before installation, reusing
/// voxel checks for unchanged chunks from the already-validated current document.
/// Revisions are monotonic, including undo/redo. Hosts add epoch/durability guards.
pub struct Engine {
    document: Arc<Document>,
    revision: u64,
    undo: VecDeque<HistoryId>,
    redo: Vec<HistoryId>,
    entries: SlotMap<HistoryId, HistoryEntry>,
    entry_ids: std::collections::HashMap<Uuid, HistoryId>,
    limits: Limits,
    history_bytes: usize,
    initial_retained: bool,
    saved: Arc<Document>,
    checkpoints: BTreeMap<Uuid, Checkpoint>,
}
impl Engine {
    /// Validate and open at revision zero with empty history.
    pub fn new(document: Document) -> Result<Self> {
        Self::with_limits(document, Limits::default())
    }
    /// Create an engine with explicit work/history budgets.
    pub fn with_limits(document: Document, limits: Limits) -> Result<Self> {
        document.validate()?;
        if limits.work_units > MAX_WORK || limits.work_units == 0 || limits.history_bytes == 0 {
            return Err(Error::Invalid("engine limits".into()));
        }
        let document = Arc::new(document);
        Ok(Self {
            saved: document.clone(),
            document,
            revision: 0,
            undo: VecDeque::new(),
            redo: vec![],
            entries: SlotMap::with_key(),
            entry_ids: Default::default(),
            limits,
            history_bytes: 0,
            initial_retained: true,
            checkpoints: BTreeMap::new(),
        })
    }
    /// Current immutable document.
    pub fn document(&self) -> &Document {
        &self.document
    }
    /// Captured immutable snapshot, safe to render while future edits proceed.
    pub fn snapshot(&self) -> Arc<Document> {
        self.document.clone()
    }
    /// Monotonic revision; adapters serialize this as a decimal string.
    pub fn revision(&self) -> u64 {
        self.revision
    }
    /// Current top history ID, or None when the retained undo stack is empty.
    pub fn head(&self) -> Option<Uuid> {
        self.undo.back().map(|key| self.entries[*key].id)
    }
    /// Next redo entry ID.
    pub fn redo_head(&self) -> Option<Uuid> {
        self.redo.last().map(|key| self.entries[*key].id)
    }
    /// Applied entries in chronological order.
    pub fn history(&self) -> impl DoubleEndedIterator<Item = &HistoryEntry> {
        self.undo.iter().map(|key| &self.entries[*key])
    }
    /// All retained steps in chronological order, including undone steps.
    pub fn timeline(&self) -> impl DoubleEndedIterator<Item = &HistoryEntry> {
        self.undo
            .iter()
            .chain(self.redo.iter().rev())
            .map(|key| &self.entries[*key])
    }
    /// Restore a retained preview as one new edit without moving the undo cursor backward.
    /// The caller must still be at the revision and head captured when previewing.
    pub fn restore_history(
        &mut self,
        revision: u64,
        expected_head: Option<Uuid>,
        entry: Option<Uuid>,
        id: Uuid,
        actor: Actor,
    ) -> Result<Receipt> {
        self.guard(revision)?;
        if self.head() != expected_head {
            return Err(Error::Conflict("history head changed".into()));
        }
        let target = self.preview(entry)?;
        let mut staged = (*self.document).clone();
        staged.content = target.content.clone();
        staged.extensions = target.extensions.clone();
        self.commit_restore(staged, id, actor, "Restore history state".into())
    }
    /// Retained conservative history byte accounting.
    pub fn history_bytes(&self) -> usize {
        self.history_bytes
    }
    /// Whether the original state is still available to preview.
    pub fn initial_retained(&self) -> bool {
        self.initial_retained
    }
    /// Whether content differs from the last explicitly marked saved state.
    pub fn is_dirty(&self) -> bool {
        self.document != self.saved
    }
    /// Mark only a successfully persisted current state as saved.
    pub fn mark_saved(&mut self) {
        self.saved = self.document.clone();
    }
    /// Record a successfully persisted snapshot, including a background save.
    /// Later edits are compared with this snapshot, even after undo or redo.
    /// Snapshots of other documents are ignored.
    pub fn mark_saved_snapshot(&mut self, snapshot: Arc<Document>) {
        if snapshot.id == self.document.id {
            self.saved = snapshot;
        }
    }
    fn guard(&self, revision: u64) -> Result<()> {
        if revision != self.revision {
            return Err(Error::Conflict(format!(
                "expected revision {revision}, current {}",
                self.revision
            )));
        }
        Ok(())
    }
    fn prepare(&self, tx: &Transaction) -> Result<(Document, Receipt, usize)> {
        self.guard(tx.expected_revision)?;
        validate_name(&tx.label)?;
        validate_name(&tx.actor.tool)?;
        if self.entry_ids.contains_key(&tx.id) {
            return Err(Error::Conflict("history entry ID already exists".into()));
        }
        let mut staged = (*self.document).clone();
        let mut work = crate::command::Work {
            used: 0,
            limit: self.limits.work_units,
        };
        crate::command::apply(
            &mut staged,
            &tx.command,
            tx.actor.origin,
            self.head(),
            &mut work,
        )?;
        staged.validate_against(&self.document)?;
        let mut changes = compare(&self.document.content, &staged.content);
        changes.metadata_changed |= self.document.extensions != staged.extensions;
        let changed = staged != *self.document;
        let bytes = if changed {
            patch_bytes(&self.document, &staged)
        } else {
            0
        };
        if bytes > self.limits.history_bytes {
            return Err(Error::Limit(format!(
                "single history entry needs {bytes} bytes, budget {}",
                self.limits.history_bytes
            )));
        }
        let revision = if changed {
            self.revision
                .checked_add(1)
                .ok_or_else(|| Error::Limit("revision overflow".into()))?
        } else {
            self.revision
        };
        Ok((
            staged,
            Receipt {
                revision,
                entry: changed.then_some(tx.id),
                changes,
                work_units: work.used,
            },
            bytes,
        ))
    }
    /// Identical preflight to commit, with no live mutation or reservation.
    pub fn validate(&self, tx: &Transaction) -> Result<Receipt> {
        self.prepare(tx).map(|(_, mut r, _)| {
            r.revision = self.revision;
            r
        })
    }
    /// Atomically apply a typed edit with exactly one history entry (unless no-op).
    pub fn execute(&mut self, tx: Transaction) -> Result<Receipt> {
        let (staged, receipt, bytes) = self.prepare(&tx)?;
        if receipt.entry.is_some() {
            self.install(
                staged,
                tx.id,
                tx.actor,
                tx.label,
                receipt.changes.clone(),
                bytes,
            )?;
        }
        Ok(receipt)
    }
    fn install(
        &mut self,
        staged: Document,
        id: Uuid,
        actor: Actor,
        label: String,
        changes: Changes,
        bytes: usize,
    ) -> Result<()> {
        let next = self
            .revision
            .checked_add(1)
            .ok_or_else(|| Error::Limit("revision overflow".into()))?;
        for key in self.redo.drain(..) {
            let entry = self.entries.remove(key).expect("retained redo key");
            self.entry_ids.remove(&entry.id);
            self.history_bytes -= entry.bytes;
        }
        let after = Arc::new(staged);
        let key = self.entries.insert(HistoryEntry {
            id,
            actor,
            label,
            base_revision: self.revision,
            result_revision: next,
            changes,
            before: self.document.clone(),
            after: after.clone(),
            bytes,
            applied: true,
        });
        self.entry_ids.insert(id, key);
        self.undo.push_back(key);
        self.history_bytes += bytes;
        self.document = after;
        self.revision = next;
        while self.history_bytes > self.limits.history_bytes {
            let key = self.undo.pop_front().expect("entry admission checked");
            let old = self.entries.remove(key).expect("retained undo key");
            self.entry_ids.remove(&old.id);
            self.history_bytes -= old.bytes;
            self.initial_retained = false;
        }
        let retained: BTreeSet<_> = self.undo.iter().map(|key| self.entries[*key].id).collect();
        self.checkpoints.retain(|_, c| {
            c.head
                .map_or(self.initial_retained, |id| retained.contains(&id))
        });
        Ok(())
    }
    /// Undo exactly the expected top entry, even when another actor authored it.
    pub fn undo(&mut self, revision: u64, entry: Uuid) -> Result<Receipt> {
        self.guard(revision)?;
        if self.head() != Some(entry) {
            return Err(Error::Conflict("undo head changed or empty".into()));
        }
        let next = self
            .revision
            .checked_add(1)
            .ok_or_else(|| Error::Limit("revision overflow".into()))?;
        let key = self.undo.pop_back().expect("head checked");
        let e = &mut self.entries[key];
        e.applied = false;
        let mut changes = compare(&self.document.content, &e.before.content);
        changes.metadata_changed |= self.document.extensions != e.before.extensions;
        self.document = e.before.clone();
        self.redo.push(key);
        self.revision = next;
        Ok(Receipt {
            revision: next,
            entry: Some(entry),
            changes,
            work_units: 0,
        })
    }
    /// Redo exactly the captured redo head; any intervening edit clears redo.
    pub fn redo(&mut self, revision: u64, entry: Uuid) -> Result<Receipt> {
        self.guard(revision)?;
        if self.redo_head() != Some(entry) {
            return Err(Error::Conflict("redo head changed or empty".into()));
        }
        let next = self
            .revision
            .checked_add(1)
            .ok_or_else(|| Error::Limit("revision overflow".into()))?;
        let key = self.redo.pop().expect("head checked");
        let e = &mut self.entries[key];
        e.applied = true;
        let mut changes = compare(&self.document.content, &e.after.content);
        changes.metadata_changed |= self.document.extensions != e.after.extensions;
        self.document = e.after.clone();
        self.undo.push_back(key);
        self.revision = next;
        Ok(Receipt {
            revision: next,
            entry: Some(entry),
            changes,
            work_units: 0,
        })
    }
    /// Capture a read-only retained state without moving the live history cursor.
    pub fn preview(&self, entry: Option<Uuid>) -> Result<Arc<Document>> {
        if let Some(id) = entry {
            return self
                .entry_ids
                .get(&id)
                .and_then(|key| self.entries.get(*key))
                .map(|e| e.after.clone())
                .ok_or_else(|| Error::NotFound("history expired".into()));
        }
        if !self.initial_retained {
            return Err(Error::NotFound("initial history expired".into()));
        }
        Ok(self.undo.front().or_else(|| self.redo.last()).map_or_else(
            || self.document.clone(),
            |key| self.entries[*key].before.clone(),
        ))
    }
    /// Bookmark the current retained cursor; this does not change content/revision.
    pub fn checkpoint(&mut self, id: Uuid, name: String) -> Result<()> {
        validate_name(&name)?;
        if self.checkpoints.len() >= 64 {
            return Err(Error::Limit("64 checkpoints".into()));
        }
        if self.checkpoints.contains_key(&id) {
            return Err(Error::Conflict("checkpoint ID exists".into()));
        }
        self.checkpoints.insert(
            id,
            Checkpoint {
                id,
                name,
                head: self.head(),
            },
        );
        Ok(())
    }
    /// Enumerate temporary retained checkpoints.
    pub fn checkpoints(&self) -> impl Iterator<Item = &Checkpoint> {
        self.checkpoints.values()
    }
    /// Restore a bookmark as a new reversible edit, preserving later audit entries.
    pub fn restore_checkpoint(
        &mut self,
        revision: u64,
        head: Option<Uuid>,
        checkpoint: Uuid,
        id: Uuid,
        actor: Actor,
    ) -> Result<Receipt> {
        self.guard(revision)?;
        if head != self.head() {
            return Err(Error::Conflict("history head changed".into()));
        }
        let c = self
            .checkpoints
            .get(&checkpoint)
            .ok_or_else(|| Error::NotFound("checkpoint expired".into()))?;
        let target = self.preview(c.head)?;
        let mut staged = (*self.document).clone();
        staged.content = target.content.clone();
        staged.extensions = target.extensions.clone();
        self.commit_restore(staged, id, actor, "Restore checkpoint".into())
    }
    /// Commit host-scoped extension edits or migrations as one reversible operation.
    /// The caller supplies ownership checks; core enforces revision and history limits.
    /// Project data is independent of named voxel snapshots and build recipes.
    pub fn set_extensions(
        &mut self,
        revision: u64,
        id: Uuid,
        actor: Actor,
        extensions: Extensions,
    ) -> Result<Receipt> {
        self.guard(revision)?;
        validate_name(&actor.tool)?;
        let mut staged = (*self.document).clone();
        staged.extensions = extensions;
        self.commit_restore(staged, id, actor, "Update plugin data".into())
    }
    fn commit_restore(
        &mut self,
        staged: Document,
        id: Uuid,
        actor: Actor,
        label: String,
    ) -> Result<Receipt> {
        if self.entry_ids.contains_key(&id) {
            return Err(Error::Conflict("history entry ID exists".into()));
        }
        crate::command::validate_restore(&self.document.content, &staged.content)?;
        staged.validate()?;
        let mut changes = compare(&self.document.content, &staged.content);
        changes.metadata_changed |= self.document.extensions != staged.extensions;
        if staged == *self.document {
            return Ok(Receipt {
                revision: self.revision,
                entry: None,
                changes,
                work_units: 0,
            });
        }
        let bytes = patch_bytes(&self.document, &staged);
        if bytes > self.limits.history_bytes {
            return Err(Error::Limit("restore history budget".into()));
        }
        self.install(staged, id, actor, label, changes.clone(), bytes)?;
        Ok(Receipt {
            revision: self.revision,
            entry: Some(id),
            changes,
            work_units: 0,
        })
    }
    /// Revert a retained step only where current fields still match its after-state.
    /// Unrelated edits survive; conflicts leave everything unchanged.
    pub fn revert(
        &mut self,
        revision: u64,
        entry: Uuid,
        id: Uuid,
        actor: Actor,
    ) -> Result<Receipt> {
        self.guard(revision)?;
        let e = self
            .entry_ids
            .get(&entry)
            .and_then(|key| self.entries.get(*key))
            .filter(|entry| entry.applied)
            .ok_or_else(|| Error::NotFound("retained applied entry".into()))?;
        let mut staged = (*self.document).clone();
        revert_document(&mut staged, &e.before, &e.after)?;
        self.commit_restore(staged, id, actor, "Revert history step".into())
    }
    /// List durable versions independently of undo retention.
    pub fn versions(&self) -> impl Iterator<Item = &NamedVersion> {
        self.document.versions.values()
    }
    /// Create an independent project from a named version, remapping all internal IDs.
    pub fn branch(
        &self,
        version: Uuid,
        new_id: Uuid,
        name: String,
    ) -> Result<(Document, BTreeMap<Uuid, Uuid>)> {
        validate_name(&name)?;
        if new_id == self.document.id {
            return Err(Error::Conflict(
                "branch needs a new document identity".into(),
            ));
        }
        let v = self
            .document
            .versions
            .get(&version)
            .ok_or_else(|| Error::NotFound(version.to_string()))?;
        let mut content = (*v.snapshot).clone();
        content.name = name;
        let mut map = BTreeMap::from([(self.document.id, new_id)]);
        // Deterministic mapping derived from the caller-supplied new document ID.
        let remap = |old: Uuid, map: &mut BTreeMap<Uuid, Uuid>| -> Uuid {
            *map.entry(old).or_insert_with(|| {
                use sha2::{Digest, Sha256};
                let mut h = Sha256::new();
                h.update(new_id.as_bytes());
                h.update(old.as_bytes());
                let digest = h.finalize();
                let mut bytes = [0; 16];
                bytes.copy_from_slice(&digest[..16]);
                bytes[6] = (bytes[6] & 15) | 0x40;
                bytes[8] = (bytes[8] & 63) | 0x80;
                Uuid::from_bytes(bytes)
            })
        };
        for l in &mut content.layers {
            l.id = remap(l.id, &mut map);
        }
        for c in &mut content.comments {
            c.id = remap(c.id, &mut map);
            if let Some(l) = c.layer {
                c.layer = Some(remap(l, &mut map));
            }
            for m in &mut c.messages {
                m.id = remap(m.id, &mut map);
            }
        }
        let doc = Document {
            id: new_id,
            content,
            versions: BTreeMap::new(),
            extensions: self.document.extensions.clone(),
            lineage: Some(Lineage {
                document: self.document.id,
                version,
            }),
        };
        doc.validate()?;
        Ok((doc, map))
    }
}
// Conservative admission counts both snapshots, even where chunk Arcs are shared.
fn patch_bytes(a: &Document, b: &Document) -> usize {
    512 + a.content.bytes()
        + b.content.bytes()
        + a.extensions.bytes()
        + b.extensions.bytes()
        + a.versions
            .values()
            .map(|v| v.snapshot.bytes())
            .sum::<usize>()
        + b.versions
            .values()
            .map(|v| v.snapshot.bytes())
            .sum::<usize>()
}
/// Compare layer cells and metadata; palette-driven appearance is reported separately.
pub fn compare(a: &Snapshot, b: &Snapshot) -> Changes {
    let mut out = Changes {
        palette_changed: a.palette != b.palette,
        metadata_changed: a.name != b.name
            || a.bounds != b.bounds
            || a.comments != b.comments
            || a.extra != b.extra
            || a.layers.iter().map(|l| l.id).collect::<Vec<_>>()
                != b.layers.iter().map(|l| l.id).collect::<Vec<_>>(),
        ..Changes::default()
    };
    let ids: BTreeSet<_> = a.layers.iter().chain(&b.layers).map(|l| l.id).collect();
    for id in ids {
        let before = a.layers.by_uuid(id);
        let after = b.layers.by_uuid(id);
        if before == after {
            continue;
        }
        out.layers.insert(id);
        let empty = Volume::default();
        let previous = before.map_or(&empty, |l| &l.voxels);
        let current = after.map_or(&empty, |l| &l.voxels);
        for (p, old, new) in current.changed_cells(previous) {
            if old == 0 {
                out.added += 1;
            } else if new == 0 {
                out.removed += 1;
            } else {
                out.recolored += 1;
            }
            if let Some(r) = &mut out.bounds {
                for (axis, value) in p.iter().enumerate() {
                    r.min[axis] = r.min[axis].min(*value);
                    r.max[axis] = r.max[axis].max(*value + 1);
                }
            } else {
                out.bounds = Some(Region {
                    min: p,
                    max: p.map(|v| v + 1),
                });
            }
        }
    }
    out
}
fn revert_field<T: Clone + PartialEq>(current: &mut T, before: &T, after: &T) -> Result<()> {
    if before != after {
        if current != after {
            return Err(Error::Conflict("history field changed".into()));
        }
        *current = before.clone();
    }
    Ok(())
}
fn revert_document(current: &mut Document, before: &Document, after: &Document) -> Result<()> {
    revert_field(
        &mut current.extensions,
        &before.extensions,
        &after.extensions,
    )?;
    revert_field(&mut current.versions, &before.versions, &after.versions)?;
    let c = &mut current.content;
    let b = &before.content;
    let a = &after.content;
    revert_field(&mut c.name, &b.name, &a.name)?;
    revert_field(&mut c.bounds, &b.bounds, &a.bounds)?;
    revert_field(&mut c.palette, &b.palette, &a.palette)?;
    revert_field(&mut c.comments, &b.comments, &a.comments)?;
    revert_field(&mut c.extra, &b.extra, &a.extra)?;
    let order = |s: &Snapshot| s.layers.iter().map(|l| l.id).collect::<Vec<_>>();
    if order(b) != order(a) {
        revert_field(&mut c.layers, &b.layers, &a.layers)?;
        return Ok(());
    }
    for (bl, al) in b.layers.iter().zip(&a.layers) {
        if bl == al {
            continue;
        }
        let cl = c.layer_mut(al.id)?;
        revert_field(&mut cl.name, &bl.name, &al.name)?;
        revert_field(&mut cl.visible, &bl.visible, &al.visible)?;
        revert_field(&mut cl.locked, &bl.locked, &al.locked)?;
        let coords: BTreeSet<_> = bl
            .voxels
            .iter()
            .chain(al.voxels.iter())
            .map(|(p, _)| p)
            .collect();
        for p in coords {
            let old = bl.voxels.get(p);
            let new = al.voxels.get(p);
            if old == new {
                continue;
            }
            if cl.locked || !cl.visible {
                return Err(Error::LayerBlocked(cl.id));
            }
            if cl.voxels.get(p) != new {
                return Err(Error::Conflict(format!("history cell changed at {p:?}")));
            }
            cl.voxels.set(p, old);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rename(engine: &mut Engine, id: Uuid, name: &str) -> Result<Receipt> {
        engine.execute(Transaction {
            expected_revision: engine.revision(),
            id,
            actor: Actor::new(Origin::Human),
            label: "Rename".into(),
            command: Command::EditDocument {
                name: Some(name.into()),
                bounds: None,
            },
        })
    }

    #[test]
    fn discarded_redo_keys_cannot_resolve_reused_slots() {
        let mut engine = Engine::new(Document::default()).unwrap();
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        rename(&mut engine, first, "First").unwrap();
        let stale = engine.entry_ids[&first];
        engine.undo(engine.revision(), first).unwrap();
        assert_eq!(engine.preview(Some(first)).unwrap().content.name, "First");
        assert!(rename(&mut engine, first, "Duplicate").is_err());
        assert!(
            engine
                .revert(engine.revision(), first, second, Actor::new(Origin::Human))
                .is_err()
        );
        rename(&mut engine, second, "Second").unwrap();
        assert!(engine.entries.get(stale).is_none());
        assert!(engine.preview(Some(first)).is_err());
        assert_eq!(engine.preview(Some(second)).unwrap().content.name, "Second");
        assert_eq!(
            engine.timeline().map(|e| e.id).collect::<Vec<_>>(),
            vec![second]
        );
        // A discarded portable UUID is reusable, but receives a fresh generation.
        rename(&mut engine, first, "Third").unwrap();
        assert_ne!(engine.entry_ids[&first], stale);
    }

    #[test]
    fn eviction_removes_uuid_mapping_and_invalidates_key() {
        let mut engine = Engine::new(Document::default()).unwrap();
        let first = Uuid::from_u128(1);
        rename(&mut engine, first, "First").unwrap();
        let stale = engine.entry_ids[&first];
        engine.limits.history_bytes = engine.history_bytes();
        rename(&mut engine, Uuid::from_u128(2), "Other").unwrap();
        assert!(engine.entries.get(stale).is_none());
        assert!(!engine.entry_ids.contains_key(&first));
        assert!(engine.preview(Some(first)).is_err());
        assert_eq!(engine.entries.len(), engine.timeline().count());
    }

    #[test]
    fn revision_overflow_rejects_without_mutation() {
        let mut e = Engine::new(Document::default()).unwrap();
        e.revision = u64::MAX;
        let before = e.snapshot();
        let tx = Transaction {
            expected_revision: u64::MAX,
            id: Uuid::new_v4(),
            actor: Actor::new(Origin::Human),
            label: "Rename".into(),
            command: Command::EditDocument {
                name: Some("Changed".into()),
                bounds: None,
            },
        };
        assert!(matches!(e.execute(tx), Err(Error::Limit(_))));
        assert_eq!(e.snapshot(), before);
        assert_eq!(e.history().count(), 0);
    }
}
