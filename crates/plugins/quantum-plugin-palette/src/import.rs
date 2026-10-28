use CoreError as Error;
use quantum_plugin_api::{CoreError, Palette, Swatch};

/// Import HEX, accepting optional leading `#`, blank lines and CRLF.
pub fn from_hex(input: &str) -> Result<Palette, CoreError> {
    bounded_text(input)?;
    let colors = input
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .enumerate()
        .map(|(index, l)| {
            if index == 256 {
                return Err(Error::Limit("palette needs 1..=256 slots".into()));
            }
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
        if colors.len() == 256 {
            return Err(Error::Limit("palette needs 1..=256 slots".into()));
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

/// Import JASC-PAL 0100 text with an exact 1..=256 color count.
/// Accepts Aseprite's optional alpha column only when it is fully opaque.
pub fn from_pal(input: &str) -> Result<Palette, CoreError> {
    bounded_text(input)?;
    let mut lines = input.lines().map(str::trim);
    if lines.next() != Some("JASC-PAL") {
        return Err(Error::Invalid("missing JASC-PAL header".into()));
    }
    if lines.next() != Some("0100") {
        return Err(Error::Invalid("JASC-PAL requires version 0100".into()));
    }
    let count = lines
        .next()
        .and_then(|line| line.parse::<usize>().ok())
        .filter(|count| (1..=256).contains(count))
        .ok_or_else(|| Error::Limit("JASC-PAL requires 1..=256 colors".into()))?;
    let mut colors = Vec::with_capacity(count);
    for line in lines.filter(|line| !line.is_empty()) {
        if colors.len() == count {
            return Err(Error::Invalid(
                "JASC-PAL color count does not match header".into(),
            ));
        }
        let mut columns = line.split_whitespace();
        let mut rgb = [0; 3];
        for channel in &mut rgb {
            *channel = columns
                .next()
                .and_then(|v| v.parse::<u8>().ok())
                .ok_or_else(|| Error::Invalid("JASC-PAL requires three RGB bytes".into()))?;
        }
        if let Some(alpha) = columns.next()
            && alpha.parse::<u8>() != Ok(255)
        {
            return Err(Error::Invalid("JASC-PAL colors must be opaque".into()));
        }
        if columns.next().is_some() {
            return Err(Error::Invalid("unexpected JASC-PAL color columns".into()));
        }
        colors.push(Swatch::rgb(rgb[0], rgb[1], rgb[2]));
    }
    if colors.len() != count {
        return Err(Error::Invalid(
            "JASC-PAL color count does not match header".into(),
        ));
    }
    Palette::from_colors(colors)
}
