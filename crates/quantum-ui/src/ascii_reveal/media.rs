// Ported from ascii-reveal by Quan Hoang (MIT), https://github.com/quanhoangindex/ascii-reveal — see crates/quantum-ui/NOTICE
//! Precomputed coarse-to-fine image grids over caller-fitted RGBA pixels.

use super::{
    core::{Direction, Timing, clamp, ease, ease_io, hash, time_scale},
    presets::{Color, Preset},
};
use std::{error::Error, fmt};
use unicode_segmentation::UnicodeSegmentation;

/// Maximum edge of a supplied thumbnail grid. Bounds work and memory before allocation.
pub const MAX_MEDIA_EDGE: u32 = 1024;

/// Invalid caller-supplied image grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaError {
    /// Dimensions must be nonzero and at most MAX_MEDIA_EDGE.
    Dimensions,
    /// Pixel count must equal width times height.
    PixelCount,
    /// Origin coordinates must be finite.
    Origin,
}
impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Dimensions => "ASCII image dimensions must be in 1..=1024",
            Self::PixelCount => "ASCII image pixel count does not match dimensions",
            Self::Origin => "ASCII image origin must be finite",
        })
    }
}
impl Error for MediaError {}

/// Image sampling configuration. Pixel coordinates are logical pixels (DPR 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MediaOptions {
    /// Glyph preset.
    pub preset: Preset,
    /// Playback timing.
    pub timing: Timing,
    /// Speed multiplier, as for text.
    pub speed: f64,
    /// Opaque backdrop behind transparent source pixels.
    pub background: [u8; 3],
    /// Wave origin in image coordinates; None means image center.
    pub origin: Option<[f64; 2]>,
}
impl Default for MediaOptions {
    fn default() -> Self {
        Self {
            preset: Preset::Decode,
            timing: Timing::Fast,
            speed: 1.,
            background: [255; 3],
            origin: None,
        }
    }
}

#[derive(Debug)]
struct Sample {
    x: f32,
    y: f32,
    gate: f32,
    ink: f32,
    color: [u8; 3],
}
#[derive(Debug)]
struct GridStep {
    height: f64,
    samples: Vec<Sample>,
}

/// A visible image glyph, borrowing its alphabet from the cached grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MediaCell {
    /// Glyph centered at x/y.
    pub glyph: &'static str,
    /// Horizontal center in logical pixels.
    pub x: f64,
    /// Vertical center in logical pixels.
    pub y: f64,
    /// Rounded upstream font size (`cell height * 1.02`).
    pub font_size: f64,
    /// Source RGB for dither; None uses the preset's mono or accent color.
    pub color: Option<[u8; 3]>,
}

/// Cached image grids. Construction samples each cell five times; frames only hash and select.
#[derive(Debug)]
pub struct Media {
    width: u32,
    height: u32,
    options: MediaOptions,
    steps: [GridStep; 5],
    ramp: Vec<&'static str>,
}

fn luminance(rgb: [f64; 3]) -> f64 {
    (0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]) / 255.
}

impl Media {
    /// Precompute the five upstream cell sizes. Pixels must already be fitted/cropped to
    /// the final thumbnail; no image I/O, resampling, renderer or GPUI dependency is used.
    pub fn new(
        width: u32,
        height: u32,
        rgba: &[[u8; 4]],
        options: MediaOptions,
    ) -> Result<Self, MediaError> {
        if width == 0 || height == 0 || width > MAX_MEDIA_EDGE || height > MAX_MEDIA_EDGE {
            return Err(MediaError::Dimensions);
        }
        if rgba.len() != width as usize * height as usize {
            return Err(MediaError::PixelCount);
        }
        let [ox, oy] = options
            .origin
            .unwrap_or([f64::from(width) / 2., f64::from(height) / 2.]);
        if !ox.is_finite() || !oy.is_finite() {
            return Err(MediaError::Origin);
        }
        let (w, h) = (f64::from(width), f64::from(height));
        let max_d = [
            ox.hypot(oy),
            (w - ox).hypot(oy),
            ox.hypot(h - oy),
            (w - ox).hypot(h - oy),
        ]
        .into_iter()
        .fold(0., f64::max)
        .max(f64::MIN_POSITIVE);
        let bg = options.background.map(f64::from);
        let bg_lum = luminance(bg);
        let sample = |x: f64, y: f64| {
            let ix = clamp(x.trunc(), 0., w - 1.) as usize;
            let iy = clamp(y.trunc(), 0., h - 1.) as usize;
            let pixel = rgba[iy * width as usize + ix];
            let a = f64::from(pixel[3]) / 255.;
            let rgb = std::array::from_fn(|i| f64::from(pixel[i]) * a + bg[i] * (1. - a));
            (
                (luminance(rgb) - bg_lum).abs() / bg_lum.max(1. - bg_lum),
                rgb.map(|c| c as u8),
            )
        };
        let scale = clamp(w.min(h) / 230., 0.4, 1.);
        let steps = options.preset.spec().cells.map(|height| {
            let ch = (height * scale).max(3.);
            let cw = ch * 0.62;
            let mut samples = Vec::new();
            for gy in 0..(h / ch).ceil() as usize {
                let cy = (gy as f64 + 0.5) * ch;
                for gx in 0..(w / cw).ceil() as usize {
                    let cx = (gx as f64 + 0.5) * cw;
                    let (dx, dy) = (cw / 3., ch / 3.);
                    let mut best = (-1., [0; 3]);
                    for (x, y) in [
                        (cx, cy),
                        (cx - dx, cy - dy),
                        (cx + dx, cy - dy),
                        (cx - dx, cy + dy),
                        (cx + dx, cy + dy),
                    ] {
                        let next = sample(x, y);
                        if next.0 > best.0 {
                            best = next;
                        }
                    }
                    if best.0 < 0.04 {
                        continue;
                    }
                    samples.push(Sample {
                        x: cx as f32,
                        y: cy as f32,
                        ink: best.0 as f32,
                        color: best.1,
                        gate: (0.5 * hash((cx / 6.) as i32, (cy / 6.) as i32, 11)
                            + 0.5 * (cx - ox).hypot(cy - oy) / max_d)
                            as f32,
                    });
                }
            }
            GridStep {
                height: ch,
                samples,
            }
        });
        Ok(Self {
            width,
            height,
            options,
            steps,
            ramp: options.preset.spec().ramp.graphemes(true).collect(),
        })
    }
    /// Final thumbnail dimensions, in logical pixels.
    pub fn size(&self) -> [u32; 2] {
        [self.width, self.height]
    }
    /// Options used to build the cached grids.
    pub fn options(&self) -> MediaOptions {
        self.options
    }
    /// Local playback duration.
    pub fn duration_ms(&self) -> f64 {
        self.options.timing.spec().media * time_scale(self.options.speed)
    }
    /// Whether scheduling can stop.
    pub fn is_finished(&self, elapsed_ms: f64) -> bool {
        elapsed_ms >= self.duration_ms()
    }
    /// Evaluate a frame without allocation, including source-image and glyph opacity.
    pub fn frame(&self, elapsed_ms: f64, direction: Direction) -> MediaFrame<'_> {
        let elapsed = if elapsed_ms.is_nan() {
            0.
        } else {
            elapsed_ms.max(0.)
        };
        let lt = match direction {
            Direction::Reveal => elapsed.min(self.duration_ms()),
            Direction::Hide => (self.duration_ms() - elapsed).max(0.),
        };
        let q = lt / self.duration_ms();
        let step = ((q / 0.8 * self.steps.len() as f64).floor() as usize).min(self.steps.len() - 1);
        MediaFrame {
            media: self,
            step,
            q,
            tick: (lt / self.options.preset.spec().tick).floor() as i32,
            image_alpha: if q < 0.68 {
                0.
            } else {
                // setSrc writes o.toFixed(3) to the upstream CSS opacity.
                (ease_io((q - 0.68) / 0.32) * 1000.).round() / 1000.
            },
            glyph_alpha: if q <= 0. || q >= 1. {
                0.
            } else if q < 0.74 {
                1.
            } else {
                1. - ease((q - 0.74) / 0.26)
            },
        }
    }
}

/// Borrowed image frame. The source image and glyph grid overlap during the crossfade.
pub struct MediaFrame<'a> {
    media: &'a Media,
    step: usize,
    q: f64,
    tick: i32,
    /// Opacity for the caller's original image.
    pub image_alpha: f64,
    /// Opacity for the entire glyph grid.
    pub glyph_alpha: f64,
}
impl MediaFrame<'_> {
    /// Current coarse-to-fine cell height, for layout and diagnostics.
    pub fn cell_height(&self) -> f64 {
        self.media.steps[self.step].height
    }
    /// Visible cells. Hash indices include every retained sample, even gated-out cells.
    pub fn cells(&self) -> impl Iterator<Item = MediaCell> + '_ {
        self.media.steps[self.step]
            .samples
            .iter()
            .enumerate()
            .filter_map(move |(i, s)| {
                if self.glyph_alpha <= 0. || f64::from(s.gate) > self.q * 2.3 {
                    return None;
                }
                let mut level = f64::from(s.ink);
                if hash(i as i32, self.tick, 3) < 0.16 {
                    level += (hash(i as i32, self.tick, 77) - 0.5) * 0.35;
                }
                let idx = (clamp(level * 1.15, 0., 1.) * (self.media.ramp.len() - 1) as f64).round()
                    as usize;
                (idx > 0).then(|| MediaCell {
                    glyph: self.media.ramp[idx],
                    x: f64::from(s.x),
                    y: f64::from(s.y),
                    font_size: (self.cell_height() * 1.02).round(),
                    color: (self.media.options.preset.spec().color == Color::Source)
                        .then_some(s.color),
                })
            })
    }
}
