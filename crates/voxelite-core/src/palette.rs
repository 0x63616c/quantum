use crate::{Error, Result, validate_name};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// An opaque sRGB swatch. Names do not determine stable slot identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Swatch {
    /// Red, green, blue, alpha; alpha must be 255.
    pub rgba: [u8; 4],
    /// Optional display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
impl Swatch {
    /// Construct an opaque unnamed RGB swatch.
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            rgba: [r, g, b, 255],
            name: None,
        }
    }
    /// Check opacity and name constraints.
    pub fn validate(&self) -> Result<()> {
        if self.rgba[3] != 255 {
            return Err(Error::Invalid("transparent palette color".into()));
        }
        if let Some(n) = &self.name {
            validate_name(n)?;
        }
        Ok(())
    }
}
/// A stable nonzero palette slot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaletteSlot {
    /// Slot in 1..=256; zero is air.
    pub index: u16,
    /// Color and optional name.
    #[serde(flatten)]
    pub color: Swatch,
}
/// Document palette: stable identity and independent display order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Palette {
    /// Slots sorted by stable index.
    pub slots: Vec<PaletteSlot>,
    /// Full permutation of live slots.
    pub order: Vec<u16>,
}
impl Default for Palette {
    fn default() -> Self {
        Self {
            slots: vec![PaletteSlot {
                index: 1,
                color: Swatch {
                    rgba: [180, 140, 100, 255],
                    name: Some("clay".into()),
                },
            }],
            order: vec![1],
        }
    }
}
impl Palette {
    /// Resolve a stable slot; zero never resolves.
    pub fn get(&self, slot: u16) -> Option<&Swatch> {
        self.slots
            .iter()
            .find(|s| s.index == slot)
            .map(|s| &s.color)
    }
    /// Resolve an unambiguous name, rejecting missing or ambiguous names.
    pub fn resolve(&self, name: &str) -> Result<u16> {
        let matches: Vec<_> = self
            .slots
            .iter()
            .filter(|s| s.color.name.as_deref() == Some(name))
            .map(|s| s.index)
            .collect();
        if matches.len() != 1 {
            return Err(Error::Invalid(format!(
                "palette name {name:?} matches {matches:?}"
            )));
        }
        Ok(matches[0])
    }
    /// Check identity, opacity, cardinality and order.
    pub fn validate(&self) -> Result<()> {
        if self.slots.is_empty() || self.slots.len() > 256 {
            return Err(Error::Limit("palette needs 1..=256 slots".into()));
        }
        let mut ids = BTreeSet::new();
        for s in &self.slots {
            if !(1..=256).contains(&s.index) || !ids.insert(s.index) {
                return Err(Error::Invalid("palette slot identity".into()));
            }
            s.color.validate()?;
        }
        if self.order.len() != ids.len()
            || self.order.iter().copied().collect::<BTreeSet<_>>() != ids
        {
            return Err(Error::Invalid("palette order must be a permutation".into()));
        }
        Ok(())
    }
    /// Build slots in source order, preserving duplicate colors.
    pub fn from_colors(colors: Vec<Swatch>) -> Result<Self> {
        let p = Self {
            order: (1..=colors.len() as u16).collect(),
            slots: colors
                .into_iter()
                .enumerate()
                .map(|(i, color)| PaletteSlot {
                    index: (i + 1) as u16,
                    color,
                })
                .collect(),
        };
        p.validate()?;
        Ok(p)
    }
    /// Export display order as six lowercase hexadecimal digits per line.
    pub fn to_hex(&self) -> String {
        self.order
            .iter()
            .filter_map(|s| self.get(*s))
            .map(|s| format!("{:02x}{:02x}{:02x}\n", s.rgba[0], s.rgba[1], s.rgba[2]))
            .collect()
    }
    /// Import HEX, accepting optional leading `#`, blank lines and CRLF.
    pub fn from_hex(input: &str) -> Result<Self> {
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
            .collect::<Result<Vec<_>>>()?;
        Self::from_colors(colors)
    }
    /// Export GIMP palette text with representable swatch names.
    pub fn to_gpl(&self, name: &str) -> Result<String> {
        validate_name(name)?;
        let mut out = format!("GIMP Palette\nName: {name}\nColumns: 16\n#\n");
        for id in &self.order {
            let s = self
                .get(*id)
                .ok_or_else(|| Error::Invalid("palette order".into()))?;
            out.push_str(&format!(
                "{:3} {:3} {:3}\t{}\n",
                s.rgba[0],
                s.rgba[1],
                s.rgba[2],
                s.name.as_deref().unwrap_or("")
            ));
        }
        Ok(out)
    }
    /// Import bounded GIMP palette text, retaining RGB order and color names.
    pub fn from_gpl(input: &str) -> Result<Self> {
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
        Self::from_colors(colors)
    }
}
fn bounded_text(input: &str) -> Result<()> {
    if input.len() > 65536 {
        return Err(Error::Limit("palette text exceeds 64 KiB".into()));
    }
    Ok(())
}
/// Exact built-in palettes. Originals are produced by value; library copies are editable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    /// Lexaloffle's standard sixteen colors.
    Pico8,
    /// ENDESGA's thirty-two colors in published order.
    Endesga32,
    /// GrafxKid's Sweetie 16.
    Sweetie16,
    /// Sixteen evenly spaced sRGB grays, black through white.
    Grayscale,
    /// Original Voxelite stone and slate tones.
    Neutral,
    /// Original Voxelite earth and terracotta tones.
    Warm,
    /// Original Voxelite saturated garden colors.
    Vivid,
}
impl Preset {
    /// Construct exact opaque source colors without assigning slot zero.
    pub fn palette(self) -> Palette {
        let hex = match self {
            Self::Pico8 => {
                "000000 1d2b53 7e2553 008751 ab5236 5f574f c2c3c7 fff1e8 ff004d ffa300 ffec27 00e436 29adff 83769c ff77a8 ffccaa"
            }
            Self::Endesga32 => {
                "be4a2f d77643 ead4aa e4a672 b86f50 733e39 3e2731 a22633 e43b44 f77622 feae34 fee761 63c74d 3e8948 265c42 193c3e 124e89 0099db 2ce8f5 ffffff c0cbdc 8b9bb4 5a6988 3a4466 262b44 181425 ff0044 68386c b55088 f6757a e8b796 c28569"
            }
            Self::Sweetie16 => {
                "1a1c2c 5d275d b13e53 ef7d57 ffcd75 a7f070 38b764 257179 29366f 3b5dc9 41a6f6 73eff7 f4f4f4 94b0c2 566c86 333c57"
            }
            Self::Grayscale => {
                "000000 111111 222222 333333 444444 555555 666666 777777 888888 999999 aaaaaa bbbbbb cccccc dddddd eeeeee ffffff"
            }
            Self::Neutral => "202833 35404c 4f5d69 76838b a2adae c9ceca e8e6da f8f4e7",
            Self::Warm => "352933 593d3b 815344 ab6c4b d78d5b eab878 f1d5a2 f9ebcc",
            Self::Vivid => "263747 326d66 55a36b a5c76b ebd27e e99964 c66264 815477",
        };
        Palette::from_hex(&hex.replace(' ', "\n")).expect("validated built-in palette")
    }
}
/// An editable local palette collection; selecting one never mutates a document.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaletteLibrary {
    /// Custom palettes keyed by stable opaque ID.
    pub entries: std::collections::BTreeMap<crate::Uuid, LibraryPalette>,
}
/// Named editable swatches in a local library.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryPalette {
    /// Library display name.
    pub name: String,
    /// Swatches and display order.
    pub palette: Palette,
}
impl PaletteLibrary {
    /// Create or replace a custom entry after validation; preset originals are separate.
    pub fn put(&mut self, id: crate::Uuid, name: String, palette: Palette) -> Result<()> {
        validate_name(&name)?;
        palette.validate()?;
        if !self.entries.contains_key(&id) && self.entries.len() >= 256 {
            return Err(Error::Limit("256 custom palettes".into()));
        }
        self.entries.insert(id, LibraryPalette { name, palette });
        Ok(())
    }
    /// Duplicate a custom entry with a new identity and name.
    pub fn duplicate(&mut self, source: crate::Uuid, id: crate::Uuid, name: String) -> Result<()> {
        if self.entries.contains_key(&id) {
            return Err(Error::Conflict("palette ID already exists".into()));
        }
        let palette = self
            .entries
            .get(&source)
            .ok_or_else(|| Error::NotFound(source.to_string()))?
            .palette
            .clone();
        self.put(id, name, palette)
    }
    /// Remove an editable custom palette.
    pub fn remove(&mut self, id: crate::Uuid) -> Option<LibraryPalette> {
        self.entries.remove(&id)
    }
}
