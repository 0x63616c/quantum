// Ported from ascii-reveal by Quan Hoang (MIT), https://github.com/quanhoangindex/ascii-reveal — see crates/voxelite-ui/NOTICE
//! Still patterns, two flicker frames, and the 16-strip skeleton sweep.

use super::core::{hash, sweep_ease};

/// Animation cadence in milliseconds (not every display vsync).
pub const FPS_MS: u64 = 33;
/// Wave pattern switches every 120 ms.
pub const FLICK_MS: f64 = 120.;
/// Delay between paragraph lines; rows inside a block share a single sweep.
pub const STAGGER_MS: f64 = 90.;
/// Number of constant-alpha strips in the upstream soft band.
pub const STRIPS: usize = 16;
/// Maximum source glyphs in a paragraph line.
pub const LINE_LEN: usize = 120;
/// Maximum source glyphs in a block row.
pub const BLOCK_LEN: usize = 96;
/// Default paragraph widths, as fractions of the container width.
pub const SKELETON_WIDTHS: [f32; 5] = [1., 0.94, 0.72, 0.88, 0.60];

/// Skeleton texture, independent of the reveal preset.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonVariant {
    /// Shaded blocks.
    #[default]
    Dither,
    /// Dots and dense ASCII marks.
    Scan,
    /// Fine braille texture.
    Braille,
    /// Binary digits with an accent wave.
    Terminal,
}

/// Unmodified upstream variant constants, including its CSS overrides.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VariantSpec {
    /// Base alphabet.
    pub base: &'static str,
    /// Wave alphabet (duplicates intentionally weight glyph frequencies).
    pub wave: &'static str,
    /// Fraction of empty cells.
    pub gap: f64,
    /// Exponent bias toward lighter glyphs.
    pub bias: f64,
    /// Still text alpha from SKELETON_CSS.
    pub base_alpha: f64,
    /// CSS sweep settings for this variant.
    pub sweep: Sweep,
}

impl SkeletonVariant {
    /// Pattern and appearance constants from skeleton.ts.
    pub const fn spec(self) -> VariantSpec {
        match self {
            Self::Dither => VariantSpec {
                base: "░░░░▒",
                wave: "▒▓▓█▓▒",
                gap: 0.,
                bias: 0.9,
                base_alpha: 0.22,
                sweep: Sweep {
                    band: 0.55,
                    opacity: 0.75,
                    speed_ms: 1800.,
                },
            },
            Self::Scan => VariantSpec {
                base: "··:·:-",
                wave: "+=*#%@&$",
                gap: 0.08,
                bias: 1.6,
                base_alpha: 0.28,
                sweep: Sweep {
                    band: 0.38,
                    opacity: 0.95,
                    speed_ms: 1800.,
                },
            },
            Self::Braille => VariantSpec {
                base: "⠢⠔⠑⠊⠡⠌⠒⠤⠕⠪",
                wave: "⣿⣷⣯⣟⡿⢿⣻⣽⣾⣶",
                gap: 0.02,
                bias: 1.,
                base_alpha: 0.35,
                sweep: Sweep {
                    band: 0.4,
                    opacity: 1.,
                    speed_ms: 1800.,
                },
            },
            Self::Terminal => VariantSpec {
                base: "01",
                wave: "01",
                gap: 0.12,
                bias: 1.,
                base_alpha: 0.16,
                sweep: Sweep {
                    band: 0.35,
                    opacity: 1.,
                    speed_ms: 1500.,
                },
            },
        }
    }
}

/// Deterministic `glyphRow`; an empty ramp produces spaces instead of JS `undefined`.
pub fn glyph_row(seed: i32, len: usize, ramp: &str, gap: f64, bias: f64) -> String {
    let ramp: Vec<char> = ramp.chars().collect();
    (0..len)
        .map(|i| {
            if ramp.is_empty() || hash(i as i32, seed, 17) < gap {
                ' '
            } else {
                let index =
                    (hash(i as i32, seed, 29).powf(bias) * ramp.len() as f64).floor() as usize;
                ramp[index.min(ramp.len() - 1)]
            }
        })
        .collect()
}

/// The exact still glyph text for a row group, including newline separators.
pub fn skeleton_base(variant: SkeletonVariant, seed: i32, rows: usize, len: usize) -> String {
    let v = variant.spec();
    (0..rows)
        .map(|r| glyph_row(seed.wrapping_add(r as i32), len, v.base, v.gap, v.bias))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Precomputed text for one row: a base plus two denser flicker patterns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkeletonRow {
    /// Still pattern.
    pub base: String,
    /// Two alternate patterns; seeds are offset by 300 and 600.
    pub wave: [String; 2],
    /// Stagger index; all rows of a block have group zero.
    pub group: usize,
}

impl SkeletonRow {
    /// Cache one group row. `row` is zero for a paragraph line.
    pub fn new(variant: SkeletonVariant, seed: i32, group: usize, row: usize, len: usize) -> Self {
        let v = variant.spec();
        let seed = seed
            .wrapping_mul(97)
            .wrapping_add((group as i32).wrapping_mul(13))
            .wrapping_add(row as i32);
        Self {
            base: glyph_row(seed, len, v.base, v.gap, v.bias),
            wave: [300, 600]
                .map(|off| glyph_row(seed.wrapping_add(off), len, v.wave, v.gap, v.bias)),
            group,
        }
    }
}

/// Soft band settings. Defaults are the base `.ars` CSS values; variants override them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sweep {
    /// Fraction of the row width occupied by the band.
    pub band: f64,
    /// Peak opacity.
    pub opacity: f64,
    /// Time for a complete pass, in milliseconds.
    pub speed_ms: f64,
}
impl Default for Sweep {
    fn default() -> Self {
        Self {
            band: 0.45,
            opacity: 0.9,
            speed_ms: 1800.,
        }
    }
}

/// Clipped strip in row-local logical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Strip {
    /// Left edge.
    pub x: f64,
    /// Clipped width (zero outside the row).
    pub width: f64,
    /// Wave opacity.
    pub alpha: f64,
}

impl Sweep {
    /// Wave strips at an absolute elapsed time. Static/reduced skeletons pass `animate=false`.
    pub fn strips(
        self,
        width: f64,
        elapsed_ms: f64,
        group: usize,
        animate: bool,
    ) -> [Strip; STRIPS] {
        if !animate
            || !width.is_finite()
            || width <= 0.
            || !elapsed_ms.is_finite()
            || !self.speed_ms.is_finite()
            || self.speed_ms <= 0.
            || !self.band.is_finite()
            || self.band <= 0.
        {
            return [Strip::default(); STRIPS];
        }
        let p = (elapsed_ms - group as f64 * STAGGER_MS).rem_euclid(self.speed_ms) / self.speed_ms;
        let bw = width * self.band;
        let bx = (width - bw) * (-1.2 + 3.4 * sweep_ease(p));
        std::array::from_fn(|k| {
            let a0 = bx + bw * k as f64 / STRIPS as f64;
            let a1 = bx + bw * (k + 1) as f64 / STRIPS as f64;
            let x = a0.max(0.);
            let end = a1.min(width);
            let m = (k as f64 + 0.5) / STRIPS as f64;
            let alpha = if m < 0.45 {
                m / 0.45
            } else if m > 0.55 {
                (1. - m) / 0.45
            } else {
                1.
            };
            Strip {
                x,
                width: (end - x).max(0.),
                alpha: alpha * self.opacity,
            }
        })
    }
}

/// Select one of the two cached wave rows without accumulating state.
pub fn flicker_frame(elapsed_ms: f64) -> usize {
    (elapsed_ms.max(0.) / FLICK_MS).floor().rem_euclid(2.) as usize
}
