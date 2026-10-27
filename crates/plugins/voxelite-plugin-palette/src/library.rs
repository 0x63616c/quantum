use voxelite_plugin_api::{
    LibraryPalette, Palette, PaletteError, PaletteLibrary, Swatch, Uuid, json, validate_name,
};

type Result<T> = std::result::Result<T, PaletteError>;

pub(crate) fn load(bytes: Option<&[u8]>) -> Result<PaletteLibrary> {
    let Some(bytes) = bytes else {
        return Ok(PaletteLibrary::default());
    };
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(PaletteError::Message("Settings file too large".into()));
    }
    let library = json::from_slice(bytes).map_err(|e| PaletteError::Message(e.to_string()))?;
    validate(&library)?;
    Ok(library)
}

pub(crate) fn validate(library: &PaletteLibrary) -> Result<()> {
    if library.entries.len() > 128 {
        return Err(PaletteError::Message("128 custom palettes maximum".into()));
    }
    let mut ids = std::collections::BTreeSet::new();
    for entry in &library.entries {
        validate_name(&entry.name)?;
        entry.palette.validate()?;
        if !ids.insert(entry.id) {
            return Err(PaletteError::Message("Duplicate palette identity".into()));
        }
    }
    Ok(())
}

pub(crate) fn encode(library: &PaletteLibrary) -> Result<Vec<u8>> {
    validate(library)?;
    json::to_vec_pretty(library).map_err(|e| PaletteError::Message(e.to_string()))
}

pub(crate) fn create(library: &mut PaletteLibrary, name: String, palette: Palette) -> Result<Uuid> {
    validate_name(&name)?;
    palette.validate()?;
    if library.entries.len() >= 128 {
        return Err(PaletteError::Message("128 custom palettes maximum".into()));
    }
    let id = Uuid::new_v4();
    library.entries.push(LibraryPalette { id, name, palette });
    Ok(id)
}

/// Reserve suffix space in UTF-8 bytes, keeping the portable name limit.
pub(crate) fn copy_name(library: &PaletteLibrary, source: &str) -> Result<String> {
    validate_name(source)?;
    for number in 1..=library.entries.len() + 1 {
        let suffix = if number == 1 {
            " copy".into()
        } else {
            format!(" copy {number}")
        };
        let mut end = source.len().min(128 - suffix.len());
        while !source.is_char_boundary(end) {
            end -= 1;
        }
        let name = format!("{}{suffix}", &source[..end]);
        if !library.entries.iter().any(|entry| entry.name == name) {
            return Ok(name);
        }
    }
    Err(PaletteError::Message(
        "Cannot find an unused palette name".into(),
    ))
}

#[cfg(test)]
pub(crate) fn set_colors(
    library: &mut PaletteLibrary,
    index: usize,
    colors: Vec<Swatch>,
) -> Result<()> {
    let palette = Palette::from_colors(colors)?;
    library
        .entries
        .get_mut(index)
        .ok_or_else(|| PaletteError::Message("Palette no longer exists".into()))?
        .palette = palette;
    Ok(())
}

/// Parse exactly one opaque hexadecimal swatch.
pub fn parse_color(text: &str) -> Result<Swatch> {
    let palette = crate::from_hex(text)?;
    if palette.slots.len() != 1 {
        return Err(PaletteError::Message(
            "Enter one six-digit hex color".into(),
        ));
    }
    Ok(palette.slots[0].color.clone())
}

/// Step through display order, matching duplicate swatches by their original RGBA.
pub fn step_swatch(palette: &Palette, color: &Swatch, delta: isize) -> (usize, Swatch) {
    let current = palette
        .order
        .iter()
        .position(|slot| palette.get(*slot).is_some_and(|s| s.rgba == color.rgba))
        .unwrap_or(0);
    let next = (current as isize + delta.rem_euclid(palette.order.len() as isize))
        .rem_euclid(palette.order.len() as isize) as usize;
    (
        next,
        palette
            .get(palette.order[next])
            .expect("valid palette")
            .clone(),
    )
}
