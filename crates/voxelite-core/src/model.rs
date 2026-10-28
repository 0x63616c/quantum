use crate::{Error, Palette, Result, Uuid, Volume};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// Integer world cell coordinate, with Y up.
pub type Pos = [i32; 3];
/// Maximum occupied cells summed over all layers.
#[cfg(not(feature = "unbounded-voxels"))]
pub const MAX_VOXELS: usize = 1_048_576;
/// Benchmark-only admission limit; production builds retain 1,048,576 cells.
#[cfg(feature = "unbounded-voxels")]
pub const MAX_VOXELS: usize = 16_777_216;
/// Maximum nonempty chunks summed over all layers.
pub const MAX_CHUNKS: usize = 8192;
/// Maximum candidate work per transaction.
pub const MAX_WORK: usize = 1_048_576;
/// Portable unknown fields, retained inertly by format readers.
pub type Extra = BTreeMap<String, serde_json::Value>;

/// Finite half-open region. Construction and document validation enforce limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    /// Inclusive lower corner.
    pub min: Pos,
    /// Exclusive upper corner, which may equal 32768.
    pub max: Pos,
}
impl Default for Region {
    fn default() -> Self {
        Self {
            min: [-32, 0, -32],
            max: [32, 64, 32],
        }
    }
}
impl Region {
    /// Construct a nonempty bounded region with extents at most 256.
    pub fn new(min: Pos, max: Pos) -> Result<Self> {
        let r = Self { min, max };
        r.validate()?;
        Ok(r)
    }
    /// Validate coordinate range and positive extents.
    pub fn validate(self) -> Result<()> {
        for a in 0..3 {
            let extent = i64::from(self.max[a]) - i64::from(self.min[a]);
            if self.min[a] < -32768 || self.max[a] > 32768 || !(1..=256).contains(&extent) {
                return Err(Error::Invalid(
                    "region must have extents 1..=256 within [-32768,32768)".into(),
                ));
            }
        }
        Ok(())
    }
    /// Whether a cell belongs to this half-open region.
    pub fn contains(self, p: Pos) -> bool {
        (0..3).all(|a| p[a] >= self.min[a] && p[a] < self.max[a])
    }
    /// Whether another region is wholly contained.
    pub fn contains_region(self, r: Self) -> bool {
        (0..3).all(|a| r.min[a] >= self.min[a] && r.max[a] <= self.max[a])
    }
    /// Checked volume, including empty cells.
    pub fn volume(self) -> Result<usize> {
        self.validate()?;
        Ok((0..3)
            .map(|a| (self.max[a] - self.min[a]) as usize)
            .product())
    }
    /// Iterate cells deterministically, X fastest. Call `validate` before iteration.
    pub fn cells(self) -> impl Iterator<Item = Pos> {
        (self.min[2]..self.max[2]).flat_map(move |z| {
            (self.min[1]..self.max[1])
                .flat_map(move |y| (self.min[0]..self.max[0]).map(move |x| [x, y, z]))
        })
    }
}
/// Six face-neighbor offsets.
pub const NEIGHBORS: [Pos; 6] = [
    [-1, 0, 0],
    [1, 0, 0],
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
];
/// Validate a portable display name (1–128 UTF-8 bytes, no controls).
pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 128 || name.chars().any(char::is_control) {
        return Err(Error::Invalid(
            "name must be 1..=128 UTF-8 bytes without controls".into(),
        ));
    }
    Ok(())
}
/// One ordered voxel layer. Mutation is mediated by commands.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layer {
    /// Stable portable identity.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// Hidden layers persist but do not contribute to the composite.
    pub visible: bool,
    /// Locked layers reject voxel edits and deletion.
    pub locked: bool,
    /// Sparse voxel contents.
    pub voxels: Volume,
    /// Preserved optional format fields.
    #[serde(flatten)]
    pub extra: Extra,
}
impl Layer {
    /// Create an empty visible, unlocked layer with a caller-supplied identity.
    pub fn new(id: Uuid, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            visible: true,
            locked: false,
            voxels: Volume::default(),
            extra: Extra::new(),
        }
    }
}
/// Portable author attribution, without account or machine identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    /// A direct human edit.
    Human,
    /// An agent or plugin edit.
    Agent,
}
/// A persistent spatial anchor, retained even if geometry disappears.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Anchor {
    /// A single world cell.
    Voxel {
        /// Cell coordinate.
        at: Pos,
    },
    /// A half-open region.
    Region {
        /// Inclusive lower corner.
        min: Pos,
        /// Exclusive upper corner.
        max: Pos,
    },
}
impl Anchor {
    /// Anchor extent; all stored anchors must have valid global coordinates.
    pub fn region(&self) -> Result<Region> {
        match self {
            Self::Voxel { at } => {
                if at.iter().any(|c| !(-32768..32768).contains(c)) {
                    return Err(Error::OutOfBounds(*at));
                }
                Region::new(*at, at.map(|c| c + 1))
            }
            Self::Region { min, max } => Region::new(*min, *max),
        }
    }
}
/// Plain text contribution to a discussion.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Stable message identity.
    pub id: Uuid,
    /// Human or agent role.
    pub origin: Origin,
    /// Plain UTF-8 text, 1..=8192 bytes.
    pub text: String,
    /// Optional public tool ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// Preserved optional format fields.
    #[serde(flatten)]
    pub extra: Extra,
}
/// A portable discussion attached to model space.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    /// Stable thread identity.
    pub id: Uuid,
    /// Spatial location, independent of current occupancy.
    pub anchor: Anchor,
    /// Optional layer identity; deleted layers leave an orphan anchor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer: Option<Uuid>,
    /// Resolved threads remain part of document content.
    pub resolved: bool,
    /// Messages in authored order.
    pub messages: Vec<Message>,
    /// Preserved optional format fields.
    #[serde(flatten)]
    pub extra: Extra,
}
/// Derived visibility state for a retained comment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorState {
    /// Visible geometry intersects the anchor.
    Visible,
    /// Referenced layer is hidden.
    Hidden,
    /// Geometry/layer disappeared or bounds no longer contain the anchor.
    Orphaned,
}
/// Nonrecursive document content used by named versions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    /// Display name.
    pub name: String,
    /// Finite document grid.
    pub bounds: Region,
    /// Stable document colors.
    pub palette: Palette,
    /// Layers in bottom-to-top order.
    pub layers: crate::Layers,
    /// Threads in stable UUID order.
    #[serde(default, skip_serializing_if = "crate::Comments::is_empty")]
    pub comments: crate::Comments,
    /// Preserved optional snapshot fields.
    #[serde(flatten)]
    pub extra: Extra,
}
impl Snapshot {
    /// Find a layer by stable ID.
    pub fn layer(&self, id: Uuid) -> Result<&Layer> {
        self.layers
            .by_uuid(id)
            .ok_or_else(|| Error::NotFound(id.to_string()))
    }
    pub(crate) fn layer_mut(&mut self, id: Uuid) -> Result<&mut Layer> {
        self.layers
            .by_uuid_mut(id)
            .ok_or_else(|| Error::NotFound(id.to_string()))
    }
    /// Validate every model invariant, including hidden layers and orphan anchors.
    pub fn validate(&self) -> Result<()> {
        self.validate_against(None)
    }
    // Only reuse voxel checks from validated content with identical constraints.
    fn validate_against(&self, previous: Option<&Self>) -> Result<()> {
        let previous = previous.filter(|p| self.bounds == p.bounds && self.palette == p.palette);
        validate_name(&self.name)?;
        self.bounds.validate()?;
        self.palette.validate()?;
        if self.layers.is_empty() || self.layers.len() > 64 {
            return Err(Error::Limit("1..=64 layers required".into()));
        }
        let mut ids = BTreeSet::new();
        let mut voxels = 0;
        let mut chunks = 0;
        for l in &self.layers {
            validate_name(&l.name)?;
            if !ids.insert(l.id) {
                return Err(Error::Invalid("duplicate layer ID".into()));
            }
            voxels += l.voxels.len();
            chunks += l.voxels.chunk_count();
            let validate_voxel = |(p, s)| {
                if !self.bounds.contains(p) {
                    return Err(Error::OutOfBounds(p));
                }
                if self.palette.get(s).is_none() {
                    return Err(Error::Invalid(format!("undefined palette slot {s}")));
                }
                Ok(())
            };
            if let Some(old) = previous.and_then(|p| p.layers.by_uuid(l.id)) {
                let changed = l.voxels.changed_chunks(&old.voxels);
                // Retain the full scan's iteration order, including which invalid
                // voxel is reported first when multiple edited chunks are invalid.
                l.voxels
                    .chunk_positions()
                    .filter(|at| changed.binary_search(at).is_ok())
                    .flat_map(|at| l.voxels.chunk_entries(at))
                    .try_for_each(validate_voxel)?;
            } else {
                l.voxels.iter().try_for_each(validate_voxel)?;
            }
        }
        if voxels > MAX_VOXELS || chunks > MAX_CHUNKS {
            return Err(Error::Limit("document voxel/chunk budget".into()));
        }
        if self.comments.len() > 1024 {
            return Err(Error::Limit("1024 comment threads".into()));
        }
        ids.clear();
        let mut messages = BTreeSet::new();
        let mut bytes = 0;
        for c in &self.comments {
            if !ids.insert(c.id) {
                return Err(Error::Invalid("duplicate thread ID".into()));
            }
            c.anchor.region()?;
            if c.messages.is_empty() {
                return Err(Error::Invalid("empty comment thread".into()));
            }
            for m in &c.messages {
                if !messages.insert(m.id) {
                    return Err(Error::Invalid("duplicate message ID".into()));
                }
                if m.text.is_empty()
                    || m.text.len() > 8192
                    || m.text
                        .chars()
                        .any(|c| c.is_control() && c != '\n' && c != '\t' && c != '\r')
                {
                    return Err(Error::Invalid(
                        "comment text must be 1..=8192 bytes of plain text".into(),
                    ));
                }
                if let Some(tool) = &m.tool {
                    validate_name(tool)?;
                }
                bytes += m.text.len() + m.tool.as_ref().map_or(0, String::len) + 128;
            }
        }
        if messages.len() > 10_000 || bytes > 4 * 1024 * 1024 {
            return Err(Error::Limit("comment data budget".into()));
        }
        Ok(())
    }
    /// Topmost visible nonempty cell and its layer ID.
    pub fn sample(&self, p: Pos) -> Option<(u16, Uuid)> {
        self.layers
            .iter()
            .rev()
            .filter(|l| l.visible)
            .find_map(|l| {
                let s = l.voxels.get(p);
                (s != 0).then_some((s, l.id))
            })
    }
    /// Deterministic visible composite; overlaps count once.
    pub fn composite(&self) -> BTreeMap<Pos, u16> {
        let mut cells = BTreeMap::new();
        for l in self.layers.iter().filter(|l| l.visible) {
            cells.extend(l.voxels.iter());
        }
        cells
    }
    /// Current spatial state of a thread without modifying its stored anchor.
    pub fn anchor_state(&self, comment: &Comment) -> AnchorState {
        let Ok(r) = comment.anchor.region() else {
            return AnchorState::Orphaned;
        };
        if !self.bounds.contains_region(r) {
            return AnchorState::Orphaned;
        }
        if let Some(id) = comment.layer {
            let Ok(layer) = self.layer(id) else {
                return AnchorState::Orphaned;
            };
            if !layer.visible {
                return AnchorState::Hidden;
            }
            if layer.voxels.iter().any(|(p, _)| r.contains(p)) {
                return AnchorState::Visible;
            }
        } else if self
            .layers
            .iter()
            .filter(|l| l.visible)
            .any(|l| l.voxels.iter().any(|(p, _)| r.contains(p)))
        {
            return AnchorState::Visible;
        }
        AnchorState::Orphaned
    }
    /// Canonical immutable snapshot digest, excluding any version library.
    pub fn content_hash(&self) -> Result<String> {
        crate::canonical::hash(self)
    }
    pub(crate) fn bytes(&self) -> usize {
        1024 + self
            .layers
            .iter()
            .map(|l| l.voxels.bytes() + 256)
            .sum::<usize>()
            + self
                .comments
                .iter()
                .flat_map(|c| &c.messages)
                .map(|m| m.text.len() + 256)
                .sum::<usize>()
    }
}
/// A persistent, nonrecursive named snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedVersion {
    /// Stable version identity.
    pub id: Uuid,
    /// Human-readable name.
    pub name: String,
    /// Optional earlier version in the same library.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<Uuid>,
    /// Portable author role.
    pub origin: Origin,
    /// SHA-256 of canonical snapshot bytes.
    pub content_hash: String,
    /// Immutable nonrecursive content, sharing unchanged chunks.
    pub snapshot: Arc<Snapshot>,
    /// Preserved optional format fields.
    #[serde(flatten)]
    pub extra: Extra,
}
/// Optional external source of an independent branch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineage {
    /// Source document identity.
    pub document: Uuid,
    /// Source version identity.
    pub version: Uuid,
}
/// Portable project. Runtime revision and history belong to [`crate::Engine`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    /// Stable project identity.
    pub id: Uuid,
    /// Live content.
    pub content: Snapshot,
    /// Durable named snapshots ordered by UUID.
    pub versions: BTreeMap<Uuid, NamedVersion>,
    /// External branch source.
    pub lineage: Option<Lineage>,
    /// Opaque plugin data, preserved independently of plugin availability.
    pub extensions: crate::Extensions,
}
impl Document {
    /// Create the specified default model with deterministic supplied identities.
    pub fn new(id: Uuid, layer_id: Uuid) -> Self {
        Self {
            id,
            content: Snapshot {
                name: "Untitled".into(),
                bounds: Region::default(),
                palette: Palette::default(),
                layers: vec![Layer::new(layer_id, "Base")].into(),
                comments: Default::default(),
                extra: Extra::new(),
            },
            versions: BTreeMap::new(),
            lineage: None,
            extensions: crate::Extensions::default(),
        }
    }
    /// Validate live and version content, hashes, parent references and version bytes.
    pub fn validate(&self) -> Result<()> {
        self.validate_with_previous(None)
    }
    /// Validate all invariants, reusing voxel checks for unchanged layer chunks
    /// only when bounds and palette are unchanged. New layers are scanned in full.
    ///
    /// `previous` must already be validated, as guaranteed by the engine invariant.
    /// Public `validate()` always performs a full scan without this prerequisite.
    pub(crate) fn validate_against(&self, previous: &Self) -> Result<()> {
        self.validate_with_previous(Some(previous))
    }
    fn validate_with_previous(&self, previous: Option<&Self>) -> Result<()> {
        self.content
            .validate_against(previous.map(|p| &p.content))?;
        if self.content.extra.contains_key("extensions") {
            return Err(Error::Invalid(
                "extensions must use the project namespace".into(),
            ));
        }
        if self.versions.len() > 64 {
            return Err(Error::Limit("64 named versions".into()));
        }
        let mut bytes = 0;
        for (id, v) in &self.versions {
            if *id != v.id {
                return Err(Error::Invalid("version key mismatch".into()));
            }
            validate_name(&v.name)?;
            v.snapshot.validate()?;
            if v.content_hash != v.snapshot.content_hash()? {
                return Err(Error::Invalid("version snapshot hash mismatch".into()));
            }
            bytes += crate::canonical::to_vec(v.snapshot.as_ref())?.len();
            let mut seen = BTreeSet::from([*id]);
            let mut parent = v.parent;
            while let Some(p) = parent {
                if !seen.insert(p) {
                    return Err(Error::Invalid("version parent cycle".into()));
                }
                parent = self
                    .versions
                    .get(&p)
                    .ok_or_else(|| Error::Invalid("missing parent version".into()))?
                    .parent;
            }
        }
        if bytes > 64 * 1024 * 1024 {
            return Err(Error::Limit("64 MiB version snapshots".into()));
        }
        Ok(())
    }
}
impl Default for Document {
    fn default() -> Self {
        Self::new(Uuid::new_v4(), Uuid::new_v4())
    }
}
