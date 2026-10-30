//! Bounded portable text interchange, shared by the manager and export registry.
use voxelite_plugin_api::{Palette, PaletteError, PaletteFormat};

/// Formats accepted for both importing and exporting palette text.
pub const FORMATS: &[PaletteFormat] = &[
    PaletteFormat {
        id: "hex",
        label: "HEX (Lospec)",
        extension: "hex",
    },
    PaletteFormat {
        id: "gpl",
        label: "GIMP Palette",
        extension: "gpl",
    },
    PaletteFormat {
        id: "pal",
        label: "JASC-PAL (Aseprite)",
        extension: "pal",
    },
];

/// Resolve a supported palette text format.
pub fn format(id: &str) -> Result<PaletteFormat, PaletteError> {
    FORMATS
        .iter()
        .find(|format| format.id == id)
        .copied()
        .ok_or_else(|| PaletteError::Message("Choose a .hex, .gpl or .pal palette".into()))
}

/// Import at most 256 opaque colors from at most 64 KiB of text.
pub fn import(format_id: &str, text: &str) -> Result<Palette, PaletteError> {
    Ok(match format(format_id)?.id {
        "hex" => crate::from_hex(text)?,
        "gpl" => crate::from_gpl(text)?,
        _ => crate::from_pal(text)?,
    })
}

/// Export exact colors in display order. GPL also preserves swatch names.
pub fn export(format_id: &str, palette: &Palette, name: &str) -> Result<String, PaletteError> {
    palette.validate()?;
    Ok(match format(format_id)?.id {
        "hex" => palette.to_hex(),
        "gpl" => palette.to_gpl(name)?,
        _ => to_pal(palette)?,
    })
}

/// Export JASC-PAL version 0100, with RGB bytes in display order.
/// Names and stable slot identities are not representable in this format.
pub fn to_pal(palette: &Palette) -> Result<String, PaletteError> {
    palette.validate()?;
    let mut out = format!("JASC-PAL\n0100\n{}\n", palette.order.len());
    for id in &palette.order {
        let color = palette
            .get(*id)
            .ok_or_else(|| PaletteError::Message("Invalid palette order".into()))?;
        let [r, g, b, _] = color.rgba;
        out.push_str(&format!("{r} {g} {b}\n"));
    }
    Ok(out)
}
