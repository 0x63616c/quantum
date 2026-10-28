//! Layer structure and stable palette-slot edits.
use super::{Work, cells::live_slot};
use crate::*;
use std::collections::BTreeSet;

pub(super) fn layer_edit(s: &mut Snapshot, edit: &LayerEdit) -> Result<()> {
    match edit {
        LayerEdit::Create { id, name, index } => {
            if s.layers.key(*id).is_some() || *index > s.layers.len() {
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
            if s.layers.key(*id).is_some() {
                return Err(Error::Conflict("layer ID exists".into()));
            }
            let mut l = s.layer(*source)?.clone();
            l.id = *id;
            l.name = name.clone();
            l.locked = false;
            let source = s
                .layers
                .key(*source)
                .ok_or_else(|| Error::Conflict("source layer missing".into()))?;
            s.layers
                .insert_after(source, l)
                .ok_or_else(|| Error::Conflict("source layer missing".into()))?;
        }
        LayerEdit::Reorder(order) => {
            let old: BTreeSet<_> = s.layers.iter().map(|l| l.id).collect();
            if order.len() != old.len() || order.iter().copied().collect::<BTreeSet<_>>() != old {
                return Err(Error::Invalid("layer order must be a permutation".into()));
            }
            s.layers.reorder(order);
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
pub(super) fn palette_edit(s: &mut Snapshot, edit: &PaletteEdit, work: &mut Work) -> Result<()> {
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
