//! Library edits preserve stable slots independently of display positions.
use crate::library;
use quantum_plugin_api::{
    Palette, PaletteError, PaletteOperation, PaletteOutcome, PaletteSlot, PaletteView, Swatch,
    validate_name,
};

type Result<T> = std::result::Result<T, PaletteError>;

pub(crate) enum Edit {
    Create,
    Duplicate(String),
    Delete,
    Rename(String),
    SetColor([u8; 3]),
    Add { color: Swatch, at: Option<usize> },
    Remove(usize),
    Move { from: usize, to: usize },
    DuplicateSwatch(usize),
}

pub(crate) fn legacy(
    view: &PaletteView,
    operation: PaletteOperation,
    name: String,
    color: String,
) -> Result<Edit> {
    use PaletteOperation::*;
    let slot = view.state.slot;
    Ok(match operation {
        Create => Edit::Create,
        Duplicate => Edit::Duplicate(name),
        Delete => Edit::Delete,
        Rename => Edit::Rename(name),
        ApplyColor => {
            let [r, g, b, _] = library::parse_color(&color)?.rgba;
            Edit::SetColor([r, g, b])
        }
        AddSlot => Edit::Add {
            color: library::parse_color(&color)?,
            at: None,
        },
        RemoveSlot => Edit::Remove(slot),
        MoveLeft => Edit::Move {
            from: slot,
            to: slot.saturating_sub(1),
        },
        MoveRight => Edit::Move {
            from: slot,
            to: slot.saturating_add(1),
        },
    })
}

/// Return the committed color, if any, for the recent-color list.
pub(crate) fn edit(
    view: &PaletteView,
    out: &mut PaletteOutcome,
    edit: Edit,
) -> Result<Option<[u8; 3]>> {
    view.palette.validate()?;
    out.state.slot = view.state.slot;
    match edit {
        Edit::Create => {
            out.state.custom = Some(library::create(
                &mut out.state.library,
                "Untitled palette".into(),
                Palette::default(),
            )?);
            out.state.slot = 0;
            return Ok(None);
        }
        Edit::Duplicate(name) => {
            let name = library::copy_name(&out.state.library, &name)?;
            out.state.custom = Some(library::create(
                &mut out.state.library,
                name,
                view.palette.clone(),
            )?);
            return Ok(None);
        }
        Edit::Delete => {
            let index = custom_index(out).ok_or_else(|| {
                PaletteError::Message("Only custom palettes can be deleted".into())
            })?;
            out.state.library.entries.remove(index);
            out.state.custom = None;
            out.state.slot = out.state.slot.min(
                view.choices[view.preset]
                    .palette
                    .order
                    .len()
                    .saturating_sub(1),
            );
            return Ok(None);
        }
        Edit::Rename(name) => {
            validate_name(&name)?;
            if view.read_only || name != view.name {
                let index = editable(view, out)?;
                out.state.library.entries[index].name = name;
            }
            return Ok(None);
        }
        _ => {}
    }
    let copy_unchanged_color = view.read_only && matches!(edit, Edit::SetColor(_));
    let mut palette = view.palette.clone();
    let mut selected = slot_at(&palette, view.state.slot)?;
    let mut recent = None;
    match edit {
        Edit::SetColor(rgb) => {
            let swatch = palette
                .slots
                .iter_mut()
                .find(|s| s.index == selected)
                .ok_or_else(missing_swatch)?;
            swatch.color.rgba = [rgb[0], rgb[1], rgb[2], 255];
            recent = Some(rgb);
        }
        Edit::Add { color, at } => {
            recent = Some(rgb(&color));
            selected = insert(&mut palette, color, at)?;
        }
        Edit::Remove(position) => {
            let removed = slot_at(&palette, position)?;
            if palette.order.len() == 1 {
                return Err(PaletteError::Message(
                    "Cannot remove the last swatch from a palette".into(),
                ));
            }
            palette.order.remove(position);
            palette.slots.retain(|s| s.index != removed);
            if selected == removed {
                selected = palette.order[position.min(palette.order.len() - 1)];
            }
        }
        Edit::Move { from, to } => {
            let moved = slot_at(&palette, from)?;
            let to = to.min(palette.order.len() - 1);
            if from != to {
                palette.order.remove(from);
                palette.order.insert(to, moved);
            }
        }
        Edit::DuplicateSwatch(position) => {
            let id = slot_at(&palette, position)?;
            let color = palette.get(id).ok_or_else(missing_swatch)?.clone();
            recent = Some(rgb(&color));
            selected = insert(&mut palette, color, Some(position + 1))?;
        }
        _ => unreachable!("non-swatch operations return above"),
    }
    if palette != view.palette || copy_unchanged_color {
        let index = editable(view, out)?;
        palette.validate()?;
        out.state.slot = palette
            .order
            .iter()
            .position(|id| *id == selected)
            .ok_or_else(missing_swatch)?;
        out.state.library.entries[index].palette = palette;
    }
    Ok(recent)
}

fn rgb(color: &Swatch) -> [u8; 3] {
    let [r, g, b, _] = color.rgba;
    [r, g, b]
}

fn missing_swatch() -> PaletteError {
    PaletteError::Message("Palette swatch no longer exists".into())
}

fn slot_at(palette: &Palette, position: usize) -> Result<u16> {
    palette
        .order
        .get(position)
        .copied()
        .ok_or_else(missing_swatch)
}

fn insert(palette: &mut Palette, color: Swatch, at: Option<usize>) -> Result<u16> {
    color.validate()?;
    let index = (1..=256)
        .find(|id| palette.get(*id).is_none())
        .ok_or_else(|| PaletteError::Message("256 swatches maximum per palette".into()))?;
    palette.slots.push(PaletteSlot { index, color });
    palette.slots.sort_by_key(|s| s.index);
    palette.order.insert(
        at.unwrap_or(palette.order.len()).min(palette.order.len()),
        index,
    );
    Ok(index)
}

fn custom_index(out: &PaletteOutcome) -> Option<usize> {
    out.state
        .custom
        .and_then(|id| out.state.library.entries.iter().position(|e| e.id == id))
}

fn editable(view: &PaletteView, out: &mut PaletteOutcome) -> Result<usize> {
    if view.read_only {
        let name = library::copy_name(&out.state.library, &view.name)?;
        out.state.custom = Some(library::create(
            &mut out.state.library,
            name,
            view.palette.clone(),
        )?);
        out.copied_from = Some(view.name.clone());
    }
    custom_index(out).ok_or_else(|| PaletteError::Message("Palette no longer exists".into()))
}
