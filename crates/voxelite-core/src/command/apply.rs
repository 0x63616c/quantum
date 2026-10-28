//! Dispatch and preparation against private, evolving document state.
use super::{
    cells::{apply_cells, editable, live_slot, region_in, transform},
    shapes::shape_cells,
    structure::{layer_edit, palette_edit},
};
use crate::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

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
            if doc.content.comments.key(*id).is_some() {
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
        .by_uuid_mut(id)
        .ok_or_else(|| Error::NotFound(id.to_string()))
}
pub(crate) fn validate_restore(current: &Snapshot, target: &Snapshot) -> Result<()> {
    for l in &current.layers {
        if l.locked
            && target
                .layers
                .by_uuid(l.id)
                .is_none_or(|t| t.voxels != l.voxels)
        {
            return Err(Error::LayerBlocked(l.id));
        }
    }
    Ok(())
}
