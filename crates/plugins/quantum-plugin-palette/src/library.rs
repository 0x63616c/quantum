use quantum_plugin_api::{
    LibraryPalette, Palette, PaletteError, PaletteLibrary, PaletteState, Swatch, Uuid, json,
    validate_name,
};

const MAX_LIBRARY_BYTES: usize = 4 * 1024 * 1024;

type Result<T> = std::result::Result<T, PaletteError>;

#[cfg(test)]
pub(crate) fn load(bytes: Option<&[u8]>) -> Result<PaletteLibrary> {
    Ok(load_state(bytes)?.library)
}

pub(crate) fn load_state(bytes: Option<&[u8]>) -> Result<PaletteState> {
    let Some(bytes) = bytes else {
        return Ok(PaletteState::default());
    };
    if bytes.len() > MAX_LIBRARY_BYTES {
        return Err(PaletteError::Message("Settings file too large".into()));
    }
    let value: json::Value = json::from_slice(bytes).map_err(json_error)?;
    let recent_colors: Vec<[u8; 3]> = value
        .get("recent_colors")
        .cloned()
        .map(json::from_value)
        .transpose()
        .map_err(json_error)?
        .unwrap_or_default();
    validate_recent(&recent_colors)?;
    let library = json::from_value(value).map_err(json_error)?;
    validate(&library)?;
    Ok(PaletteState {
        library,
        recent_colors,
        ..Default::default()
    })
}

fn json_error(error: json::Error) -> PaletteError {
    PaletteError::Message(error.to_string())
}

fn validate_recent(recent: &[[u8; 3]]) -> Result<()> {
    if recent.len() > 16
        || recent
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != recent.len()
    {
        return Err(PaletteError::Message(
            "Recent colors must contain at most 16 distinct RGB colors".into(),
        ));
    }
    Ok(())
}

pub(crate) fn remember(state: &mut PaletteState, rgb: [u8; 3]) {
    state.recent_colors.retain(|color| *color != rgb);
    state.recent_colors.insert(0, rgb);
    state.recent_colors.truncate(16);
}

pub(crate) fn encode_state(state: &PaletteState) -> Result<Vec<u8>> {
    validate_recent(&state.recent_colors)?;
    if state.recent_colors.is_empty() {
        return encode(&state.library);
    }
    validate(&state.library)?;
    let mut value = json::to_value(&state.library).map_err(json_error)?;
    value["recent_colors"] = json::to_value(&state.recent_colors).map_err(json_error)?;
    bounded_bytes(json::to_vec_pretty(&value).map_err(json_error)?, || {
        json::to_vec(&value)
    })
}

// A valid in-memory library can exceed the bounded reader once pretty-printed.
// Preserve the legacy formatting when possible; compact large files, then reject
// anything that could not be loaded again before the host adopts the edit.
fn bounded_bytes(
    pretty: Vec<u8>,
    compact: impl FnOnce() -> std::result::Result<Vec<u8>, json::Error>,
) -> Result<Vec<u8>> {
    let bytes = if pretty.len() > MAX_LIBRARY_BYTES {
        compact().map_err(json_error)?
    } else {
        pretty
    };
    if bytes.len() > MAX_LIBRARY_BYTES {
        return Err(PaletteError::Message("Settings file too large".into()));
    }
    Ok(bytes)
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
    bounded_bytes(json::to_vec_pretty(library).map_err(json_error)?, || {
        json::to_vec(library)
    })
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

/// Bound filename stems in both characters and portable UTF-8 bytes.
pub(crate) fn import_name(source: &str) -> String {
    let source: String = source.chars().take(100).collect();
    prefix(&source, 128).to_owned()
}

fn prefix(source: &str, max_bytes: usize) -> &str {
    let mut end = source.len().min(max_bytes);
    while !source.is_char_boundary(end) {
        end -= 1;
    }
    &source[..end]
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
        let name = format!("{}{suffix}", prefix(source, 128 - suffix.len()));
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
/// The caller must provide a palette that passes `Palette::validate`.
#[expect(clippy::expect_used, reason = "internal invariant: valid palette")]
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
