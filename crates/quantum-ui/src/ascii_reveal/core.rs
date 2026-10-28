// Ported from ascii-reveal by Quan Hoang (MIT), https://github.com/quanhoangindex/ascii-reveal — see crates/quantum-ui/NOTICE
//! Deterministic text decoding and easing. No GPUI types or frame accumulation.

use std::{ops::Range, sync::LazyLock};
use unicode_segmentation::UnicodeSegmentation;

use super::presets::{Mode, Preset};

/// Unsigned result of the upstream JavaScript integer hash.
pub fn hash_bits(a: i32, b: i32, c: i32) -> u32 {
    let mut h = (a as u32).wrapping_mul(0x27d4_eb2d)
        ^ (b as u32)
            .wrapping_add(0x9e37_79b9)
            .wrapping_mul(0x85eb_ca6b)
        ^ (c as u32)
            .wrapping_add(0x632b_e5ab)
            .wrapping_mul(0xc2b2_ae35);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    h
}

/// Bit-exact JavaScript `hash(a,b,c)` in `[0,1)` for integer inputs.
pub fn hash(a: i32, b: i32, c: i32) -> f64 {
    f64::from(hash_bits(a, b, c)) / 4_294_967_296.
}

/// Upstream clamp; NaN passes through, just as in JavaScript.
pub fn clamp(x: f64, a: f64, b: f64) -> f64 {
    if x < a {
        a
    } else if x > b {
        b
    } else {
        x
    }
}

/// Cubic ease-out used by media glyph opacity and ordinary fades.
pub fn ease(x: f64) -> f64 {
    1. - (1. - clamp(x, 0., 1.)).powi(3)
}

/// Cubic ease-in-out used by the image crossfade.
pub fn ease_io(x: f64) -> f64 {
    let x = clamp(x, 0., 1.);
    if x < 0.5 {
        4. * x * x * x
    } else {
        1. - (-2. * x + 2.).powi(3) / 2.
    }
}

fn bezier(p: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let (mut lo, mut hi, mut u) = (0., 1., p);
    for _ in 0..28 {
        u = (lo + hi) / 2.;
        let x = 3. * (1. - u).powi(2) * u * x1 + 3. * (1. - u) * u * u * x2 + u.powi(3);
        if x < p {
            lo = u;
        } else {
            hi = u;
        }
    }
    3. * (1. - u).powi(2) * u * y1 + 3. * (1. - u) * u * u * y2 + u.powi(3)
}

/// Upstream Float32 LUT for cubic-bezier(.45,.05,.55,.95), computed once.
pub static SWEEP_LUT: LazyLock<[f32; 257]> = LazyLock::new(|| {
    std::array::from_fn(|k| bezier(k as f64 / 256., 0.45, 0.05, 0.55, 0.95) as f32)
});

/// Nearest LUT entry, including the original Float32 rounding.
pub fn sweep_ease(p: f64) -> f64 {
    f64::from(SWEEP_LUT[(clamp(p, 0., 1.) * 256.).round() as usize])
}

/// Upstream reduced-motion reveal duration (CSS `ease-out`).
pub const REDUCED_FADE_MS: f64 = 220.;
/// Upstream optional landed-character flash duration.
pub const FLASH_MS: f64 = 420.;

/// Playback direction; hide reverses local time, not character order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    /// Decode into readable text.
    #[default]
    Reveal,
    /// Scramble back to hidden.
    Hide,
}

/// Reduced motion shows the real content with a 220 ms CSS ease-out fade;
/// hides are immediate, as in upstream `Controller.run`.
pub fn reduced_motion_alpha(elapsed_ms: f64, direction: Direction) -> f64 {
    if direction == Direction::Hide {
        return 0.;
    }
    let p = clamp(elapsed_ms, 0., REDUCED_FADE_MS) / REDUCED_FADE_MS;
    if p <= 0. || p >= 1. {
        p
    } else {
        bezier(p, 0., 0., 0.58, 1.)
    }
}

/// The two upstream timing tables.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Timing {
    /// Data becomes readable in roughly 400 ms.
    #[default]
    Fast,
    /// Slower showcase wave.
    Cinematic,
}

/// Upstream timing constants, all durations in milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimingSpec {
    /// Wave velocity in logical pixels per millisecond.
    pub velocity: f64,
    /// Maximum travel delay.
    pub max_wave: f64,
    /// Character duration: base plus hash times range.
    pub char_range: [f64; 2],
    /// Media duration.
    pub media: f64,
    /// Surface duration.
    pub surface: f64,
    /// Ordinary fade duration.
    pub fade: f64,
    /// Extra reveal delay for content on a surface.
    pub on_surface: f64,
}

impl Timing {
    /// Constants from `engine.ts`'s TIMING table.
    pub const fn spec(self) -> TimingSpec {
        match self {
            Self::Fast => TimingSpec {
                velocity: 3.2,
                max_wave: 220.,
                char_range: [90., 110.],
                media: 450.,
                surface: 300.,
                fade: 220.,
                on_surface: 60.,
            },
            Self::Cinematic => TimingSpec {
                velocity: 1.15,
                max_wave: f64::INFINITY,
                char_range: [200., 240.],
                media: 1150.,
                surface: 720.,
                fade: 420.,
                on_surface: 140.,
            },
        }
    }
    /// Time to start all non-whitespace graphemes, before speed scaling.
    pub fn spread(self, n: usize) -> f64 {
        match self {
            Self::Fast => (40. + n as f64 * 4.).min(200.),
            Self::Cinematic => (90. + n as f64 * 13.).min(720.),
        }
    }
    /// Delay for a target at `distance` logical pixels from the wave origin.
    pub fn wave_delay(self, distance: f64, speed: f64, on_surface: bool) -> f64 {
        let t = self.spec();
        ((distance.max(0.) / t.velocity).min(t.max_wave)
            + if on_surface { t.on_surface } else { 0. })
            * time_scale(speed)
    }
}

pub(super) fn time_scale(speed: f64) -> f64 {
    1. / if speed.is_finite() {
        speed.max(0.1)
    } else {
        1.
    }
}

/// Match ECMAScript `\s`, rather than Rust's slightly different whitespace set.
pub(super) fn whitespace(c: char) -> bool {
    matches!(c, '\u{9}'..='\u{d}' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}

/// Decode configuration; constructing a plan precomputes all per-character data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecodeOptions {
    /// Glyph preset.
    pub preset: Preset,
    /// Timing table.
    pub timing: Timing,
    /// Stable seed; no random seed is generated by the port.
    pub seed: i32,
    /// Playback multiplier, clamped to at least 0.1.
    pub speed: f64,
    /// Enable the optional accent-to-text flash on landing.
    pub flash: bool,
    /// Force digits-only scrambling, in addition to automatic numeric detection.
    pub digits: bool,
}
impl Default for DecodeOptions {
    fn default() -> Self {
        Self {
            preset: Preset::Decode,
            timing: Timing::Fast,
            seed: 1,
            speed: 1.,
            flash: false,
            digits: false,
        }
    }
}

/// Visible stage of one grapheme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellKind {
    /// Not reached yet; no ink.
    Pending,
    /// Leading terminal cursor.
    Cursor,
    /// Ramp or random glyph.
    Scramble,
    /// Real grapheme, including unchanged whitespace.
    Landed,
}

/// Borrowed, allocation-free output for one grapheme (including whitespace).
#[derive(Clone, Debug, PartialEq)]
pub struct Cell<'a> {
    /// UTF-8 byte range in the original string.
    pub source: Range<usize>,
    /// Display glyph; empty when pending.
    pub glyph: &'a str,
    /// State of the character.
    pub kind: CellKind,
    /// Opacity with the default theme text/accent colors.
    pub alpha: f64,
    /// Accent-to-real-color mix weight for optional landing flash.
    pub flash: f64,
}

#[derive(Debug)]
struct Character {
    source: Range<usize>,
    timing: Option<(i32, f32, f32, f32)>,
}

/// Cached, immutable text decode plan. Frame evaluation neither allocates nor mutates it.
#[derive(Debug)]
pub struct Decode {
    text: String,
    options: DecodeOptions,
    characters: Vec<Character>,
    ramp: Vec<&'static str>,
    scramble: Vec<&'static str>,
    digits: bool,
    duration: f64,
}

impl Decode {
    /// Segment graphemes and cache the upstream s0/s1/ph Float32 arrays.
    pub fn new(text: impl Into<String>, options: DecodeOptions) -> Self {
        let text = text.into();
        let n = text
            .graphemes(true)
            .filter(|g| !g.chars().all(whitespace))
            .count();
        let spread = options.timing.spread(n) * time_scale(options.speed);
        let [base, range] = options.timing.spec().char_range;
        let mut k = 0i32;
        let mut duration: f64 = if n == 0 { 0. } else { 1. };
        let characters = text
            .grapheme_indices(true)
            .map(|(i, g)| {
                let timing = if g.chars().all(whitespace) {
                    None
                } else {
                    let s0 = (f64::from(k) / n as f64 * spread * 0.72
                        + hash(k, options.seed, 1) * spread * 0.28)
                        as f32;
                    let s1 = (f64::from(s0)
                        + (base + hash(k, options.seed, 2) * range) * time_scale(options.speed))
                        as f32;
                    let ph = (hash(k, options.seed, 3) * 1000.) as f32;
                    duration = duration.max(f64::from(s1));
                    let result = Some((k, s0, s1, ph));
                    k = k.wrapping_add(1);
                    result
                };
                Character {
                    source: i..i + g.len(),
                    timing,
                }
            })
            .collect();
        let trimmed = text.trim_matches(whitespace);
        let digits = options.digits
            || (!trimmed.is_empty()
                && trimmed
                    .chars()
                    .all(|c| c.is_ascii_digit() || whitespace(c) || ".,%:+-€$£¥".contains(c)));
        Self {
            text,
            options,
            characters,
            duration,
            digits,
            ramp: options.preset.spec().ramp.graphemes(true).collect(),
            scramble: if digits {
                "0123456789"
            } else {
                options.preset.spec().scramble
            }
            .graphemes(true)
            .collect(),
        }
    }
    /// Original text, suitable for accessibility and final layout.
    pub fn text(&self) -> &str {
        &self.text
    }
    /// Configuration of this cached plan.
    pub fn options(&self) -> DecodeOptions {
        self.options
    }
    /// Local duration, without DOM target travel or mount delay.
    pub fn duration_ms(&self) -> f64 {
        self.duration
    }
    /// Whether no further animation frames are required.
    pub fn is_finished(&self, elapsed_ms: f64) -> bool {
        elapsed_ms >= self.duration
    }
    /// A deterministic frame at local elapsed milliseconds; reverse evaluates `dur - t`.
    /// Like upstream baking, completion removes any remaining flash immediately.
    pub fn frame(
        &self,
        elapsed_ms: f64,
        direction: Direction,
    ) -> impl ExactSizeIterator<Item = Cell<'_>> {
        let elapsed = if elapsed_ms.is_nan() {
            0.
        } else {
            elapsed_ms.max(0.)
        };
        let lt = match direction {
            Direction::Reveal => elapsed.min(self.duration),
            Direction::Hide => (self.duration - elapsed).max(0.),
        };
        let p = self.options.preset.spec();
        self.characters.iter().map(move |ch| {
            let mut cell = Cell {
                source: ch.source.clone(),
                glyph: &self.text[ch.source.clone()],
                kind: CellKind::Landed,
                alpha: 1.,
                flash: 0.,
            };
            let Some((k, s0, s1, ph)) = ch.timing else {
                return cell;
            };
            let (s0, s1, ph) = (f64::from(s0), f64::from(s1), f64::from(ph));
            if lt < s0 || (direction == Direction::Hide && self.is_finished(elapsed)) {
                cell.glyph = "";
                cell.kind = CellKind::Pending;
                cell.alpha = 0.;
            } else if lt >= s1 {
                if self.options.flash
                    && direction == Direction::Reveal
                    && !self.is_finished(elapsed)
                {
                    cell.flash = 1. - clamp((lt - s1) / FLASH_MS, 0., 1.);
                }
            } else {
                let q = (lt - s0) / (s1 - s0);
                // JS ToInt32 inside hash wraps; float-to-int casts alone saturate in Rust.
                let tick = ((lt + ph) / p.tick).floor().rem_euclid(4_294_967_296.) as u32 as i32;
                if let Some(cursor) = p.cursor.filter(|_| q < 0.2) {
                    cell.glyph = cursor;
                    cell.kind = CellKind::Cursor;
                } else {
                    cell.kind = CellKind::Scramble;
                    cell.alpha = if p.accent_glyphs { 0.9 } else { 0.5 };
                    cell.glyph = if p.mode == Mode::Ramp && !self.digits && q < 0.7 {
                        let last = (self.ramp.len() - 1) as i32;
                        let mut idx = 1 + ((q / 0.7) * f64::from(last)).floor() as i32;
                        if hash(k, tick, 5) < 0.35 {
                            idx += if hash(k, tick, 6) < 0.5 { -1 } else { 1 };
                        }
                        self.ramp[idx.clamp(1, last) as usize]
                    } else {
                        self.scramble
                            [(hash(k, tick, 7) * self.scramble.len() as f64).floor() as usize]
                    };
                }
            }
            cell
        })
    }
}
