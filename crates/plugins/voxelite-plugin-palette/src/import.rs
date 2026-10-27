use CoreError as Error;
use voxelite_plugin_api::{CoreError, Palette, Swatch};

/// Import HEX, accepting optional leading `#`, blank lines and CRLF.
pub fn from_hex(input: &str) -> Result<Palette, CoreError> {
    bounded_text(input)?;
    let colors = input
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| {
            let l = l.strip_prefix('#').unwrap_or(l);
            if l.len() != 6 || !l.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(Error::Invalid(
                    "HEX requires six hex digits per color".into(),
                ));
            }
            let n = u32::from_str_radix(l, 16).map_err(|e| Error::Invalid(e.to_string()))?;
            Ok(Swatch::rgb((n >> 16) as u8, (n >> 8) as u8, n as u8))
        })
        .collect::<Result<Vec<_>, CoreError>>()?;
    Palette::from_colors(colors)
}
/// Import bounded GIMP palette text, retaining RGB order and color names.
pub fn from_gpl(input: &str) -> Result<Palette, CoreError> {
    bounded_text(input)?;
    let mut lines = input.lines();
    if lines.next().map(str::trim) != Some("GIMP Palette") {
        return Err(Error::Invalid("missing GIMP Palette header".into()));
    }
    let mut colors = Vec::new();
    for line in lines {
        let mut rest = line.trim();
        if rest.is_empty() || rest.starts_with('#') || rest.starts_with("Name:") {
            continue;
        }
        if let Some(columns) = rest.strip_prefix("Columns:") {
            columns
                .trim()
                .parse::<u16>()
                .map_err(|_| Error::Invalid("GPL columns".into()))?;
            continue;
        }
        let mut rgb = [0; 3];
        for c in &mut rgb {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            *c = rest[..end]
                .parse::<u8>()
                .map_err(|_| Error::Invalid("GPL requires three RGB bytes".into()))?;
            rest = rest[end..].trim_start();
        }
        colors.push(Swatch {
            rgba: [rgb[0], rgb[1], rgb[2], 255],
            name: if rest.is_empty() {
                None
            } else {
                Some(rest.into())
            },
        });
    }
    Palette::from_colors(colors)
}
fn bounded_text(input: &str) -> Result<(), CoreError> {
    if input.len() > 65536 {
        return Err(Error::Limit("palette text exceeds 64 KiB".into()));
    }
    Ok(())
}
