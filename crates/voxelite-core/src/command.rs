//! Typed edits; batch preparation runs against private copy-on-write state.
use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Voxel editing semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Write, overwriting existing cells.
    Place,
    /// Recolor existing cells only.
    Paint,
    /// Remove cells.
    Erase,
}
/// World axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    /// X axis.
    X,
    /// Y axis.
    Y,
    /// Z axis.
    Z,
}
impl Axis {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        }
    }
}
/// Explicit independent mirror planes. `None` disables an axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mirror {
    /// Doubled plane positions; reflection is `plane2 - cell - 1`.
    pub plane2: [Option<i32>; 3],
}
impl Mirror {
    /// Enable selected axes at the document midplanes.
    pub fn centered(bounds: Region, axes: [bool; 3]) -> Self {
        Self {
            plane2: std::array::from_fn(|a| axes[a].then_some(bounds.min[a] + bounds.max[a])),
        }
    }
    fn candidates(self, at: Pos) -> Result<Vec<Pos>> {
        let mut points = vec![at];
        for a in 0..3 {
            if let Some(plane) = self.plane2[a] {
                if !(-65536..=65536).contains(&plane) {
                    return Err(Error::Invalid("mirror plane2 range".into()));
                }
                for i in 0..points.len() {
                    let mut p = points[i];
                    p[a] = plane
                        .checked_sub(p[a])
                        .and_then(|p| p.checked_sub(1))
                        .ok_or_else(|| Error::Invalid("mirror overflow".into()))?;
                    points.push(p);
                }
            }
        }
        Ok(points)
    }
}
/// An explicit cell write. Slot zero is accepted by SetVoxels to erase.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Voxel {
    /// World coordinate.
    pub at: Pos,
    /// Stable palette slot, or zero for air.
    pub slot: u16,
}
/// Integer shape occupancy; no platform-dependent floating point boundaries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Shape {
    /// Half-open box, optionally a one-cell shell.
    Box {
        /// Candidate region.
        region: Region,
        /// Emit only six-neighbor shell cells.
        hollow: bool,
    },
    /// Ellipsoid evaluated at doubled cell centers.
    Ellipsoid {
        /// Doubled center coordinates.
        center2: Pos,
        /// Positive doubled radii, at most 512.
        radii2: Pos,
        /// Emit only the six-neighbor shell.
        hollow: bool,
    },
    /// Axis-aligned elliptic cylinder, with caps when hollow.
    Cylinder {
        /// Cylinder axis.
        axis: Axis,
        /// Doubled center; axial component is ignored.
        center2: Pos,
        /// Doubled radii; axial component is ignored.
        radii2: Pos,
        /// Half-open axial interval.
        range: [i32; 2],
        /// Emit only the six-neighbor shell.
        hollow: bool,
    },
    /// Center-to-center supercover, including all cells touched at ties.
    Line {
        /// First cell.
        from: Pos,
        /// Last cell.
        to: Pos,
    },
}
/// Collision behavior for a region transform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collision {
    /// Reject any occupied destination.
    Reject,
    /// Overwrite destinations.
    Overwrite,
    /// Leave occupied destinations untouched (copy only).
    Skip,
}
/// Integer copy/move transform, in reflection → rotation → translation order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transform {
    /// Source layer identity.
    pub source: Uuid,
    /// Destination layer identity.
    pub destination: Uuid,
    /// Source region.
    pub region: Region,
    /// Remove source after all validation; false copies.
    pub move_cells: bool,
    /// Integer translation.
    pub translate: Pos,
    /// Doubled rotation/reflection pivot.
    pub pivot2: Pos,
    /// Axes to reflect.
    pub reflect: [bool; 3],
    /// Optional right-handed axis rotation and quarter-turn count (0..=3).
    pub rotate: Option<(Axis, u8)>,
    /// Explicit destination collision policy.
    pub collision: Collision,
}
/// Literal palette edits, suitable for ordered atomic batches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PaletteEdit {
    /// Add a stable slot; None chooses the lowest unused slot.
    Add {
        /// Optional requested identity.
        slot: Option<u16>,
        /// Opaque color.
        color: Swatch,
    },
    /// Recolor an existing stable slot, including appearance on locked layers.
    Set {
        /// Stable slot.
        slot: u16,
        /// Replacement color.
        color: Swatch,
    },
    /// Delete a slot, explicitly remapping any references.
    Remove {
        /// Slot to delete.
        slot: u16,
        /// Replacement for referenced cells.
        replacement: Option<u16>,
    },
    /// Reorder display swatches without recoloring references.
    Reorder(Vec<u16>),
}
/// Layer metadata and structural edits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayerEdit {
    /// Insert an empty layer at a bottom-to-top index.
    Create {
        /// Caller-supplied identity.
        id: Uuid,
        /// Display name.
        name: String,
        /// Insertion index.
        index: usize,
    },
    /// Change metadata. Unlocking must precede voxel edits in another command.
    Update {
        /// Target identity.
        id: Uuid,
        /// Optional replacement name.
        name: Option<String>,
        /// Optional visibility.
        visible: Option<bool>,
        /// Optional lock flag.
        locked: Option<bool>,
    },
    /// Duplicate cells and metadata into an unlocked layer just above the source.
    Duplicate {
        /// Source identity.
        source: Uuid,
        /// New identity.
        id: Uuid,
        /// New name.
        name: String,
    },
    /// Full bottom-to-top permutation of layer IDs.
    Reorder(Vec<Uuid>),
    /// Delete an unlocked layer, retaining any discussion anchors.
    Delete(Uuid),
}
/// Typed command shared by human tools and agent adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Last explicit duplicate wins before mirror expansion.
    SetVoxels {
        /// Target layer.
        layer: Uuid,
        /// Explicit writes (at most 65,536).
        voxels: Vec<Voxel>,
        /// Editing semantics.
        mode: Mode,
        /// Explicit reflection planes.
        mirror: Mirror,
    },
    /// Erase occupied cells in a half-open region.
    EraseRegion {
        /// Target layer.
        layer: Uuid,
        /// Finite erase extent.
        region: Region,
        /// Explicit reflection planes.
        mirror: Mirror,
    },
    /// Six-connected bounded fill of the seed's slot, including air.
    FloodFill {
        /// Target layer.
        layer: Uuid,
        /// Seed in region.
        seed: Pos,
        /// Nonzero replacement slot.
        slot: u16,
        /// Finite search extent; None uses document bounds.
        region: Option<Region>,
        /// Maximum distinct visited cells.
        cap: usize,
        /// Explicit reflection planes.
        mirror: Mirror,
    },
    /// Apply an exact integer primitive.
    Primitive {
        /// Target layer.
        layer: Uuid,
        /// Shape occupancy rule.
        shape: Shape,
        /// Editing semantics.
        mode: Mode,
        /// Nonzero for place/paint, None for erase.
        slot: Option<u16>,
        /// Explicit reflection planes.
        mirror: Mirror,
    },
    /// Copy or move occupied source cells atomically.
    TransformRegion(Transform),
    /// Ordered atomic commands; nesting and version/comment operations are forbidden.
    Batch(Vec<Command>),
    /// Modify literal palette data.
    Palette(PaletteEdit),
    /// Modify layers.
    Layer(LayerEdit),
    /// Rename or resize the document without clipping hidden cells.
    EditDocument {
        /// Optional display name.
        name: Option<String>,
        /// Optional finite grid.
        bounds: Option<Region>,
    },
    /// Create an anchored thread. Message authorship is assigned from the transaction.
    CreateComment {
        /// Thread identity.
        id: Uuid,
        /// Spatial anchor.
        anchor: Anchor,
        /// Optional layer anchor.
        layer: Option<Uuid>,
        /// First message identity.
        message: Uuid,
        /// Plain text.
        text: String,
    },
    /// Explicitly recover or move an anchor.
    Reanchor {
        /// Thread identity.
        id: Uuid,
        /// New spatial anchor.
        anchor: Anchor,
        /// Optional layer identity.
        layer: Option<Uuid>,
    },
    /// Append to a thread, preserving message order.
    Reply {
        /// Thread identity.
        id: Uuid,
        /// Message identity.
        message: Uuid,
        /// Plain text.
        text: String,
    },
    /// Resolve or reopen a discussion.
    Resolve {
        /// Thread identity.
        id: Uuid,
        /// Desired state.
        resolved: bool,
    },
    /// Persist the current content as a nonrecursive named snapshot.
    SaveVersion {
        /// Version identity.
        id: Uuid,
        /// Display name.
        name: String,
        /// Optional parent version.
        parent: Option<Uuid>,
    },
    /// Restore snapshot content while preserving the version library.
    RestoreVersion {
        /// Version identity.
        id: Uuid,
        /// Captured live history head.
        expected_head: Option<Uuid>,
    },
}

pub(crate) struct Work {
    pub used: usize,
    pub limit: usize,
}
impl Work {
    pub fn charge(&mut self, n: usize) -> Result<()> {
        self.used = self
            .used
            .checked_add(n)
            .ok_or_else(|| Error::Limit("work overflow".into()))?;
        if self.used > self.limit {
            return Err(Error::Limit(format!(
                "work {} exceeds {}",
                self.used, self.limit
            )));
        }
        Ok(())
    }
}
fn region_in(s: &Snapshot, r: Region) -> Result<()> {
    r.validate()?;
    if !s.bounds.contains_region(r) {
        return Err(Error::Invalid(
            "operation region outside document bounds".into(),
        ));
    }
    Ok(())
}
fn editable(s: &Snapshot, id: Uuid) -> Result<()> {
    let l = s.layer(id)?;
    if l.locked || !l.visible {
        return Err(Error::LayerBlocked(id));
    }
    Ok(())
}
fn live_slot(s: &Snapshot, slot: u16) -> Result<()> {
    if s.palette.get(slot).is_none() {
        return Err(Error::Invalid(format!("undefined palette slot {slot}")));
    }
    Ok(())
}
fn apply_cells(
    s: &mut Snapshot,
    layer: Uuid,
    cells: BTreeMap<Pos, u16>,
    mode: Mode,
    mirror: Mirror,
    work: &mut Work,
) -> Result<()> {
    editable(s, layer)?;
    let mut expanded = BTreeMap::new();
    for (at, slot) in cells {
        if slot != 0 {
            live_slot(s, slot)?;
        }
        let points = mirror.candidates(at)?;
        work.charge(points.len() - 1)?;
        for p in points {
            if !s.bounds.contains(p) {
                return Err(Error::OutOfBounds(p));
            }
            if expanded.insert(p, slot).is_some_and(|old| old != slot) {
                return Err(Error::Conflict(format!(
                    "mirrored values conflict at {p:?}"
                )));
            }
        }
    }
    let l = s.layer_mut(layer)?;
    for (p, slot) in expanded {
        if mode != Mode::Paint || l.voxels.get(p) != 0 {
            l.voxels.set(p, if mode == Mode::Erase { 0 } else { slot });
        }
    }
    Ok(())
}
fn shape_cells(shape: &Shape, work: &mut Work) -> Result<Vec<Pos>> {
    if let Shape::Line { from, to } = shape {
        return line_cells(*from, *to, work);
    }
    let (region, hollow) = match shape {
        Shape::Box { region, hollow } => {
            region.validate()?;
            (*region, *hollow)
        }
        Shape::Ellipsoid {
            center2,
            radii2,
            hollow,
        }
        | Shape::Cylinder {
            center2,
            radii2,
            hollow,
            ..
        } => {
            if center2.iter().any(|c| !(-65536..=65536).contains(c))
                || radii2.iter().any(|r| !(1..=512).contains(r))
            {
                return Err(Error::Invalid("shape center/radii2".into()));
            }
            let mut min = std::array::from_fn(|a| (center2[a] - radii2[a]).div_euclid(2));
            let mut max = std::array::from_fn(|a| (center2[a] + radii2[a] + 1).div_euclid(2));
            if let Shape::Cylinder { axis, range, .. } = shape {
                min[axis.index()] = range[0];
                max[axis.index()] = range[1];
            }
            // Candidate volume may be wider than a document; output is never clipped.
            if (0..3).any(|a| max[a] <= min[a] || i64::from(max[a]) - i64::from(min[a]) > 513) {
                return Err(Error::Invalid("shape extent".into()));
            }
            (Region { min, max }, *hollow)
        }
        Shape::Line { .. } => unreachable!(),
    };
    let volume = (0..3)
        .map(|a| (region.max[a] - region.min[a]) as usize)
        .product();
    work.charge(volume)?;
    let inside = |p: Pos| -> bool {
        if !region.contains(p) {
            return false;
        }
        match shape {
            Shape::Box { .. } => true,
            Shape::Ellipsoid {
                center2, radii2, ..
            }
            | Shape::Cylinder {
                center2, radii2, ..
            } => {
                let axes: Vec<_> = (0..3)
                    .filter(|a| !matches!(shape,Shape::Cylinder { axis,.. } if *a==axis.index()))
                    .collect();
                let denom: i128 = axes.iter().map(|a| i128::from(radii2[*a]).pow(2)).product();
                axes.iter()
                    .map(|a| {
                        i128::from(2 * p[*a] + 1 - center2[*a]).pow(2)
                            * (denom / i128::from(radii2[*a]).pow(2))
                    })
                    .sum::<i128>()
                    <= denom
            }
            _ => false,
        }
    };
    Ok(region
        .cells()
        .filter(|p| {
            inside(*p)
                && (!hollow
                    || NEIGHBORS
                        .iter()
                        .any(|d| !inside(std::array::from_fn(|a| p[a] + d[a]))))
        })
        .collect())
}
// Exact rational boundary times: numerator odd, denominator twice |delta|.
fn line_cells(from: Pos, to: Pos, work: &mut Work) -> Result<Vec<Pos>> {
    for p in [from, to] {
        if p.iter().any(|v| !(-32768..32768).contains(v)) {
            return Err(Error::OutOfBounds(p));
        }
    }
    let delta: Pos = std::array::from_fn(|a| to[a] - from[a]);
    let mut p = from;
    let mut next = [1i64; 3];
    let mut points = BTreeSet::from([p]);
    work.charge(1)?;
    while p != to {
        let axis = (0..3)
            .filter(|a| delta[*a] != 0 && p[*a] != to[*a])
            .min_by(|a, b| {
                (next[*a] * i64::from(delta[*b].abs()))
                    .cmp(&(next[*b] * i64::from(delta[*a].abs())))
            })
            .expect("nonterminal line");
        let tied: Vec<_> = (0..3)
            .filter(|a| {
                delta[*a] != 0
                    && p[*a] != to[*a]
                    && next[*a] * i64::from(delta[axis].abs())
                        == next[axis] * i64::from(delta[*a].abs())
            })
            .collect();
        for mask in 1..(1 << tied.len()) {
            let mut q = p;
            for (i, a) in tied.iter().enumerate() {
                if mask & (1 << i) != 0 {
                    q[*a] += delta[*a].signum();
                }
            }
            work.charge(1)?;
            points.insert(q);
        }
        for a in tied {
            p[a] += delta[a].signum();
            next[a] += 2;
        }
    }
    Ok(points.into_iter().collect())
}

pub(crate) fn apply(
    doc: &mut Document,
    cmd: &Command,
    origin: Origin,
    head: Option<Uuid>,
    work: &mut Work,
) -> Result<()> {
    match cmd {
        Command::Batch(commands) => {
            if commands.len() > 256 {
                return Err(Error::Limit("256 batch commands".into()));
            }
            for (index, c) in commands.iter().enumerate() {
                let result = if matches!(
                    c,
                    Command::Batch(_)
                        | Command::SaveVersion { .. }
                        | Command::RestoreVersion { .. }
                        | Command::CreateComment { .. }
                        | Command::Reanchor { .. }
                        | Command::Reply { .. }
                        | Command::Resolve { .. }
                ) {
                    Err(Error::Invalid("command not permitted in batch".into()))
                } else {
                    apply(doc, c, origin, head, work)
                };
                result
                    .and_then(|_| doc.content.validate())
                    .map_err(|source| Error::Batch {
                        index,
                        source: Box::new(source),
                    })?;
            }
        }
        Command::SetVoxels {
            layer,
            voxels,
            mode,
            mirror,
        } => {
            if voxels.len() > 65536 {
                return Err(Error::Limit("65,536 explicit voxels".into()));
            }
            work.charge(voxels.len())?;
            apply_cells(
                &mut doc.content,
                *layer,
                voxels.iter().map(|v| (v.at, v.slot)).collect(),
                *mode,
                *mirror,
                work,
            )?;
        }
        Command::EraseRegion {
            layer,
            region,
            mirror,
        } => {
            region_in(&doc.content, *region)?;
            editable(&doc.content, *layer)?;
            let l = doc.content.layer(*layer)?;
            work.charge(l.voxels.len())?;
            let cells = l
                .voxels
                .iter()
                .filter(|(p, _)| region.contains(*p))
                .map(|(p, _)| (p, 0))
                .collect();
            apply_cells(&mut doc.content, *layer, cells, Mode::Erase, *mirror, work)?;
        }
        Command::FloodFill {
            layer,
            seed,
            slot,
            region,
            cap,
            mirror,
        } => {
            editable(&doc.content, *layer)?;
            live_slot(&doc.content, *slot)?;
            let r = region.unwrap_or(doc.content.bounds);
            region_in(&doc.content, r)?;
            if !r.contains(*seed) {
                return Err(Error::OutOfBounds(*seed));
            }
            if *cap == 0 || *cap > MAX_WORK {
                return Err(Error::Limit("fill cap must be 1..=1,048,576".into()));
            }
            let l = doc.content.layer(*layer)?;
            let old = l.voxels.get(*seed);
            if old == *slot {
                return Ok(());
            }
            let mut seen = BTreeSet::new();
            let mut queue = VecDeque::from([*seed]);
            let mut cells = BTreeMap::new();
            seen.insert(*seed);
            while let Some(p) = queue.pop_front() {
                work.charge(1)?;
                if seen.len() > *cap {
                    return Err(Error::Limit("flood fill visit cap".into()));
                }
                if l.voxels.get(p) != old {
                    continue;
                }
                cells.insert(p, *slot);
                for d in NEIGHBORS {
                    let q = std::array::from_fn(|a| p[a] + d[a]);
                    if r.contains(q) && seen.insert(q) {
                        queue.push_back(q);
                    }
                }
            }
            apply_cells(&mut doc.content, *layer, cells, Mode::Place, *mirror, work)?;
        }
        Command::Primitive {
            layer,
            shape,
            mode,
            slot,
            mirror,
        } => {
            editable(&doc.content, *layer)?;
            let value = match (mode, slot) {
                (Mode::Erase, None) => 0,
                (Mode::Place | Mode::Paint, Some(s)) => {
                    live_slot(&doc.content, *s)?;
                    *s
                }
                _ => {
                    return Err(Error::Invalid(
                        "shape slot required for place/paint and forbidden for erase".into(),
                    ));
                }
            };
            let cells = shape_cells(shape, work)?
                .into_iter()
                .map(|p| (p, value))
                .collect();
            apply_cells(&mut doc.content, *layer, cells, *mode, *mirror, work)?;
        }
        Command::TransformRegion(t) => transform(&mut doc.content, t, work)?,
        Command::Palette(edit) => palette_edit(&mut doc.content, edit, work)?,
        Command::Layer(edit) => layer_edit(&mut doc.content, edit)?,
        Command::EditDocument { name, bounds } => {
            if let Some(n) = name {
                doc.content.name = n.clone();
            }
            if let Some(b) = bounds {
                doc.content.bounds = *b;
            }
        }
        Command::CreateComment {
            id,
            anchor,
            layer,
            message,
            text,
        } => {
            if doc.content.comments.iter().any(|c| c.id == *id) {
                return Err(Error::Conflict("thread ID exists".into()));
            }
            validate_anchor(&doc.content, anchor, *layer)?;
            doc.content.comments.push(Comment {
                id: *id,
                anchor: anchor.clone(),
                layer: *layer,
                resolved: false,
                messages: vec![Message {
                    id: *message,
                    origin,
                    text: text.clone(),
                    tool: None,
                    extra: Extra::new(),
                }],
                extra: Extra::new(),
            });
            doc.content.comments.sort_by_key(|c| c.id);
        }
        Command::Reanchor { id, anchor, layer } => {
            validate_anchor(&doc.content, anchor, *layer)?;
            let c = comment_mut(&mut doc.content, *id)?;
            c.anchor = anchor.clone();
            c.layer = *layer;
        }
        Command::Reply { id, message, text } => {
            comment_mut(&mut doc.content, *id)?.messages.push(Message {
                id: *message,
                origin,
                text: text.clone(),
                tool: None,
                extra: Extra::new(),
            })
        }
        Command::Resolve { id, resolved } => {
            comment_mut(&mut doc.content, *id)?.resolved = *resolved
        }
        Command::SaveVersion { id, name, parent } => {
            if doc.versions.contains_key(id) {
                return Err(Error::Conflict("version ID exists".into()));
            }
            let snapshot = std::sync::Arc::new(doc.content.clone());
            let v = NamedVersion {
                id: *id,
                name: name.clone(),
                parent: *parent,
                origin,
                content_hash: snapshot.content_hash()?,
                snapshot,
                extra: Extra::new(),
            };
            doc.versions.insert(*id, v);
        }
        Command::RestoreVersion { id, expected_head } => {
            if *expected_head != head {
                return Err(Error::Conflict("history head changed".into()));
            }
            let target = doc
                .versions
                .get(id)
                .ok_or_else(|| Error::NotFound(id.to_string()))?
                .snapshot
                .clone();
            validate_restore(&doc.content, &target)?;
            doc.content = (*target).clone();
        }
    }
    Ok(())
}
fn validate_anchor(s: &Snapshot, anchor: &Anchor, layer: Option<Uuid>) -> Result<()> {
    region_in(s, anchor.region()?)?;
    if let Some(id) = layer {
        s.layer(id)?;
    }
    Ok(())
}
fn comment_mut(s: &mut Snapshot, id: Uuid) -> Result<&mut Comment> {
    s.comments
        .iter_mut()
        .find(|c| c.id == id)
        .ok_or_else(|| Error::NotFound(id.to_string()))
}
pub(crate) fn validate_restore(current: &Snapshot, target: &Snapshot) -> Result<()> {
    for l in &current.layers {
        if l.locked
            && target
                .layers
                .iter()
                .find(|t| t.id == l.id)
                .is_none_or(|t| t.voxels != l.voxels)
        {
            return Err(Error::LayerBlocked(l.id));
        }
    }
    Ok(())
}
fn layer_edit(s: &mut Snapshot, edit: &LayerEdit) -> Result<()> {
    match edit {
        LayerEdit::Create { id, name, index } => {
            if s.layers.iter().any(|l| l.id == *id) || *index > s.layers.len() {
                return Err(Error::Invalid("layer ID or index".into()));
            }
            s.layers.insert(*index, Layer::new(*id, name));
        }
        LayerEdit::Update {
            id,
            name,
            visible,
            locked,
        } => {
            let l = s.layer_mut(*id)?;
            if let Some(n) = name {
                l.name = n.clone();
            }
            if let Some(v) = visible {
                l.visible = *v;
            }
            if let Some(v) = locked {
                l.locked = *v;
            }
        }
        LayerEdit::Duplicate { source, id, name } => {
            if s.layers.iter().any(|l| l.id == *id) {
                return Err(Error::Conflict("layer ID exists".into()));
            }
            let mut l = s.layer(*source)?.clone();
            l.id = *id;
            l.name = name.clone();
            l.locked = false;
            let index = s
                .layers
                .iter()
                .position(|l| l.id == *source)
                .expect("source exists");
            s.layers.insert(index + 1, l);
        }
        LayerEdit::Reorder(order) => {
            let old: BTreeSet<_> = s.layers.iter().map(|l| l.id).collect();
            if order.len() != old.len() || order.iter().copied().collect::<BTreeSet<_>>() != old {
                return Err(Error::Invalid("layer order must be a permutation".into()));
            }
            s.layers
                .sort_by_key(|l| order.iter().position(|id| *id == l.id));
        }
        LayerEdit::Delete(id) => {
            if s.layer(*id)?.locked {
                return Err(Error::LayerBlocked(*id));
            }
            if s.layers.len() == 1 {
                return Err(Error::Invalid("cannot delete final layer".into()));
            }
            s.layers.retain(|l| l.id != *id);
        }
    }
    Ok(())
}
fn palette_edit(s: &mut Snapshot, edit: &PaletteEdit, work: &mut Work) -> Result<()> {
    match edit {
        PaletteEdit::Add { slot, color } => {
            color.validate()?;
            let id = slot
                .or_else(|| (1..=256).find(|i| s.palette.get(*i).is_none()))
                .ok_or_else(|| Error::Limit("palette full".into()))?;
            if !(1..=256).contains(&id) || s.palette.get(id).is_some() {
                return Err(Error::Invalid("slot already exists or out of range".into()));
            }
            s.palette.slots.push(PaletteSlot {
                index: id,
                color: color.clone(),
            });
            s.palette.slots.sort_by_key(|s| s.index);
            s.palette.order.push(id);
        }
        PaletteEdit::Set { slot, color } => {
            color.validate()?;
            let c = s
                .palette
                .slots
                .iter_mut()
                .find(|c| c.index == *slot)
                .ok_or_else(|| Error::NotFound(format!("slot {slot}")))?;
            c.color = color.clone();
        }
        PaletteEdit::Remove { slot, replacement } => {
            live_slot(s, *slot)?;
            if s.palette.slots.len() == 1 {
                return Err(Error::Invalid("cannot delete final palette slot".into()));
            }
            if let Some(r) = replacement {
                live_slot(s, *r)?;
                if r == slot {
                    return Err(Error::Invalid("replacement equals removed slot".into()));
                }
            }
            for l in &mut s.layers {
                work.charge(l.voxels.len())?;
                let refs: Vec<_> = l
                    .voxels
                    .iter()
                    .filter(|(_, v)| v == slot)
                    .map(|(p, _)| p)
                    .collect();
                if refs.is_empty() {
                    continue;
                }
                if l.locked || !l.visible {
                    return Err(Error::LayerBlocked(l.id));
                }
                let replacement = replacement
                    .ok_or_else(|| Error::Invalid("referenced slot needs replacement".into()))?;
                for p in refs {
                    l.voxels.set(p, replacement);
                }
            }
            s.palette.slots.retain(|c| c.index != *slot);
            s.palette.order.retain(|i| i != slot);
        }
        PaletteEdit::Reorder(order) => s.palette.order = order.clone(),
    }
    Ok(())
}
fn transform(s: &mut Snapshot, t: &Transform, work: &mut Work) -> Result<()> {
    region_in(s, t.region)?;
    editable(s, t.source)?;
    editable(s, t.destination)?;
    if t.move_cells && t.collision == Collision::Skip {
        return Err(Error::Invalid("skip is copy-only".into()));
    }
    if t.pivot2.iter().any(|p| !(-65536..=65536).contains(p))
        || t.translate.iter().any(|p| !(-65536..=65536).contains(p))
    {
        return Err(Error::Invalid("transform range".into()));
    }
    let source = s.layer(t.source)?;
    work.charge(source.voxels.len())?;
    let cells: Vec<_> = source
        .voxels
        .iter()
        .filter(|(p, _)| t.region.contains(*p))
        .collect();
    let removed: BTreeSet<_> = cells.iter().map(|(p, _)| *p).collect();
    let mut destinations = Vec::new();
    for (p, slot) in cells {
        work.charge(1)?;
        let mut q: Pos = std::array::from_fn(|a| {
            (2 * p[a] + 1 - t.pivot2[a]) * if t.reflect[a] { -1 } else { 1 }
        });
        if let Some((axis, turns)) = t.rotate {
            if turns > 3 {
                return Err(Error::Invalid("quarter turns must be 0..=3".into()));
            }
            let a = axis.index();
            let u = (a + 1) % 3;
            let v = (a + 2) % 3;
            for _ in 0..turns {
                let old = q[u];
                q[u] = -q[v];
                q[v] = old;
            }
        }
        q = std::array::from_fn(|a| q[a] + t.pivot2[a] + 2 * t.translate[a]);
        if q.iter().any(|v| v.rem_euclid(2) != 1) {
            return Err(Error::Invalid(
                "transform does not preserve cell centers".into(),
            ));
        }
        q = q.map(|v| (v - 1) / 2);
        if !s.bounds.contains(q) {
            return Err(Error::OutOfBounds(q));
        }
        let occupied = s.layer(t.destination)?.voxels.get(q) != 0
            && !(t.move_cells && t.source == t.destination && removed.contains(&q));
        if occupied {
            match t.collision {
                Collision::Reject => {
                    return Err(Error::Conflict(format!("destination occupied at {q:?}")));
                }
                Collision::Skip => continue,
                Collision::Overwrite => {}
            }
        }
        destinations.push((q, slot));
    }
    if t.move_cells {
        for p in removed {
            s.layer_mut(t.source)?.voxels.set(p, 0);
        }
    }
    for (p, slot) in destinations {
        s.layer_mut(t.destination)?.voxels.set(p, slot);
    }
    Ok(())
}
