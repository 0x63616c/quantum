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
impl Default for Swatch {
    fn default() -> Self {
        Self {
            rgba: [180, 140, 100, 255],
            name: Some("clay".into()),
        }
    }
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
                color: Swatch::default(),
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
}
