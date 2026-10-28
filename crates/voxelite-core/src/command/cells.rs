//! Cell admission, mirror expansion and copy/move transforms.
use super::Work;
use crate::*;
use std::collections::{BTreeMap, BTreeSet};

impl Mirror {
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

pub(super) fn region_in(s: &Snapshot, r: Region) -> Result<()> {
    r.validate()?;
    if !s.bounds.contains_region(r) {
        return Err(Error::Invalid(
            "operation region outside document bounds".into(),
        ));
    }
    Ok(())
}
pub(super) fn editable(s: &Snapshot, id: Uuid) -> Result<()> {
    let l = s.layer(id)?;
    if l.locked || !l.visible {
        return Err(Error::LayerBlocked(id));
    }
    Ok(())
}
pub(super) fn live_slot(s: &Snapshot, slot: u16) -> Result<()> {
    if s.palette.get(slot).is_none() {
        return Err(Error::Invalid(format!("undefined palette slot {slot}")));
    }
    Ok(())
}
pub(super) fn apply_cells(
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
pub(super) fn transform(s: &mut Snapshot, t: &Transform, work: &mut Work) -> Result<()> {
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
