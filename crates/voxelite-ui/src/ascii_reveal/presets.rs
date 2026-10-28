// Ported from ascii-reveal by Quan Hoang (MIT), https://github.com/quanhoangindex/ascii-reveal — see crates/voxelite-ui/NOTICE
//! The four upstream glyph and media presets.

/// Built-in reveal style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Preset {
    /// Light-to-dense glyphs, then alphanumeric scrambling.
    #[default]
    Decode,
    /// Shaded blocks with source-colored image glyphs.
    Dither,
    /// Leading block cursor and binary/punctuation scrambling.
    Terminal,
    /// Accent-colored construction marks.
    Blueprint,
}

/// How a character approaches its final value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Condense through the ramp for the first 70 percent.
    Ramp,
    /// Scramble throughout (after an optional cursor).
    Random,
}

/// Color source for an image glyph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    /// Text color.
    Mono,
    /// Sampled image color.
    Source,
    /// Theme accent.
    Accent,
}

/// Constants from upstream `presets.ts`, in logical pixels and milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresetSpec {
    /// Empty-to-dense ramp (including the leading space).
    pub ramp: &'static str,
    /// Scramble alphabet.
    pub scramble: &'static str,
    /// Condense or scramble directly.
    pub mode: Mode,
    /// Image glyph color source.
    pub color: Color,
    /// Image cell heights, coarse to fine.
    pub cells: [f64; 5],
    /// Milliseconds between glyph changes.
    pub tick: f64,
    /// Leading cursor, if any.
    pub cursor: Option<&'static str>,
    /// Use accent color for text scrambling.
    pub accent_glyphs: bool,
}

impl Preset {
    /// The unmodified upstream constants for this preset.
    pub const fn spec(self) -> PresetSpec {
        match self {
            Self::Decode => PresetSpec {
                ramp: " .:-=+*#%@",
                scramble: "ABCDEFGHJKLMNPQRSTUVWXYZ0123456789#%&*+=?/",
                mode: Mode::Ramp,
                color: Color::Mono,
                cells: [24., 16., 11., 8., 6.],
                tick: 60.,
                cursor: None,
                accent_glyphs: false,
            },
            Self::Dither => PresetSpec {
                ramp: " ░▒▓█",
                scramble: "░▒▓█",
                mode: Mode::Ramp,
                color: Color::Source,
                cells: [20., 14., 10., 7., 5.],
                tick: 85.,
                cursor: None,
                accent_glyphs: false,
            },
            Self::Terminal => PresetSpec {
                ramp: " .-:=+*#",
                scramble: "01<>/\\{}[]|_=+-;:",
                mode: Mode::Random,
                color: Color::Mono,
                cells: [22., 15., 10., 7., 6.],
                tick: 40.,
                cursor: Some("█"),
                accent_glyphs: false,
            },
            Self::Blueprint => PresetSpec {
                ramp: " ·-+=x#",
                scramble: "+-/\\|x·",
                mode: Mode::Random,
                color: Color::Accent,
                cells: [26., 18., 13., 9., 7.],
                tick: 95.,
                cursor: None,
                accent_glyphs: true,
            },
        }
    }
}
