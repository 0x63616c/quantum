// Ported from ascii-reveal by Quan Hoang (MIT), https://github.com/quanhoangindex/ascii-reveal — see crates/quantum-ui/NOTICE
//! GPUI builders. The pure plans and shaped rows survive parent renders.

use super::{
    clock::{Clock, Playback, schedule},
    core::{
        CellKind, Decode, DecodeOptions, Direction, REDUCED_FADE_MS, Timing, reduced_motion_alpha,
    },
    media::Media,
    paint::{paint, shape},
    presets::{Color, Preset},
    skeleton::{BLOCK_LEN, LINE_LEN, SKELETON_WIDTHS, SkeletonRow, SkeletonVariant, flicker_frame},
};
use crate::tokens::{Theme, radius, space, type_size};
use gpui::{prelude::*, *};
use std::{
    rc::Rc,
    time::{Duration, Instant},
};
use unicode_segmentation::UnicodeSegmentation;

/// A decorative ASCII skeleton. Give each instance a stable, unique element ID.
/// Widths are GPUI lengths; default line widths follow the upstream paragraph shape.
#[derive(IntoElement)]
pub struct AsciiSkeleton {
    id: ElementId,
    theme: Theme,
    lines: usize,
    rows: Option<usize>,
    widths: Vec<Length>,
    variant: SkeletonVariant,
    seed: i32,
    animate: bool,
    reduced: bool,
    playback: Playback,
}

impl AsciiSkeleton {
    /// Three dither lines, seed one, with the shimmer enabled.
    pub fn new(id: impl Into<ElementId>, theme: Theme) -> Self {
        Self {
            id: id.into(),
            theme,
            lines: 3,
            rows: None,
            widths: Vec::new(),
            variant: SkeletonVariant::Dither,
            seed: 1,
            animate: true,
            reduced: false,
            playback: Playback::default(),
        }
    }
    /// Paragraph line count. Clears block mode; zero renders an empty placeholder.
    pub fn lines(mut self, lines: usize) -> Self {
        self.lines = lines;
        self.rows = None;
        self
    }
    /// Repeating line widths (pixels or relative lengths). Empty restores the defaults.
    pub fn widths(mut self, widths: impl IntoIterator<Item = impl Into<Length>>) -> Self {
        self.widths = widths.into_iter().map(Into::into).collect();
        self
    }
    /// Block mode: tightly stacked glyph rows, sharing one sweep.
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = Some(rows);
        self
    }
    /// Glyph texture and matching upstream CSS sweep parameters.
    pub fn variant(mut self, variant: SkeletonVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Stable integer pattern seed.
    pub fn seed(mut self, seed: i32) -> Self {
        self.seed = seed;
        self
    }
    /// Disable the shimmer for a still, inexpensive placeholder.
    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }
    /// App-supplied preference; true means no sweep, flicker or timer.
    pub fn reduced_motion(mut self, reduced: bool) -> Self {
        self.reduced = reduced;
        self
    }
    /// Synchronize the sweep to a caller-owned clock.
    pub fn started_at(mut self, start: Instant) -> Self {
        self.playback.start = Some(start);
        self
    }
    /// Render an exact snapshot; disables automatic scheduling.
    pub fn elapsed(mut self, elapsed: Duration) -> Self {
        self.playback.elapsed = Some(elapsed);
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct SkeletonKey {
    variant: SkeletonVariant,
    seed: i32,
    count: usize,
    block: bool,
}
struct ShapedRow {
    base: ShapedLine,
    wave: [ShapedLine; 2],
    group: usize,
}
struct SkeletonCache {
    key: SkeletonKey,
    rows: Rc<Vec<ShapedRow>>,
}

impl RenderOnce for AsciiSkeleton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let key = SkeletonKey {
            variant: self.variant,
            seed: self.seed,
            count: self.rows.unwrap_or(self.lines),
            block: self.rows.is_some(),
        };
        let (cache, clock) = window.with_element_namespace(self.id.clone(), |window| {
            (
                window.use_keyed_state("pattern", cx, |_, _| None::<SkeletonCache>),
                window.use_keyed_state("clock", cx, |_, _| Clock::new()),
            )
        });
        let rows = cache.update(cx, |cache, _| {
            if cache.as_ref().is_none_or(|cache| cache.key != key) {
                let advance = type_size::BODY
                    * (type_size::MONO_ADVANCE
                        + if key.variant == SkeletonVariant::Dither && !key.block {
                            type_size::ASCII_DITHER_TRACKING_EM
                        } else {
                            0.
                        });
                let rows = (0..key.count)
                    .map(|i| {
                        let row = SkeletonRow::new(
                            key.variant,
                            key.seed,
                            if key.block { 0 } else { i },
                            if key.block { i } else { 0 },
                            if key.block { BLOCK_LEN } else { LINE_LEN },
                        );
                        ShapedRow {
                            base: shape(row.base, type_size::BODY, Some(advance), window),
                            wave: row
                                .wave
                                .map(|s| shape(s, type_size::BODY, Some(advance), window)),
                            group: row.group,
                        }
                    })
                    .collect();
                *cache = Some(SkeletonCache {
                    key,
                    rows: Rc::new(rows),
                });
            }
            cache
                .as_ref()
                .map(|cache| cache.rows.clone())
                .unwrap_or_default()
        });
        let elapsed = clock.update(cx, |clock, _| clock.elapsed(self.playback, true, false));
        let animate = self.animate && !self.reduced;
        let active = animate && self.playback.elapsed.is_none() && !rows.is_empty();
        if !active {
            clock.update(cx, |clock, _| clock.stop());
        }
        let line_height = type_size::BODY
            * if key.block || key.variant == SkeletonVariant::Dither {
                1.
            } else {
                type_size::ASCII_LINE_EM
            };
        let gap = if key.block {
            space::NONE
        } else {
            type_size::BODY * type_size::ASCII_GAP_EM
        };
        let spec = key.variant.spec();
        let theme = self.theme;
        let wave_color = if key.variant == SkeletonVariant::Terminal {
            theme.accent()
        } else {
            theme.text()
        };
        let mut root = div()
            .id(self.id)
            .flex()
            .flex_col()
            .w_full()
            .gap(px(gap))
            .overflow_hidden();
        if key.block {
            root = root.rounded(px(radius::LG)).bg(theme.text().opacity(0.06));
        }
        for i in 0..rows.len() {
            let rows = rows.clone();
            let clock = clock.clone();
            let width: Length = if key.block {
                relative(1.).into()
            } else if self.widths.is_empty() {
                relative(SKELETON_WIDTHS[i % SKELETON_WIDTHS.len()]).into()
            } else {
                self.widths[i % self.widths.len()]
            };
            root = root.child(
                canvas(
                    move |bounds, window, cx| {
                        schedule(&clock, bounds, active, window, cx);
                    },
                    move |bounds, (), window, cx| {
                        let row = &rows[i];
                        window.with_content_mask(Some(ContentMask { bounds }), |window| {
                            paint(
                                &row.base,
                                bounds.origin,
                                px(line_height),
                                theme.text().opacity(spec.base_alpha as f32),
                                window,
                                cx,
                            );
                            for strip in spec.sweep.strips(
                                f64::from(f32::from(bounds.size.width)),
                                elapsed,
                                row.group,
                                animate,
                            ) {
                                if strip.width <= 0. {
                                    continue;
                                }
                                let mask = Bounds::new(
                                    bounds.origin + point(px(strip.x as f32), px(space::NONE)),
                                    size(px(strip.width as f32), bounds.size.height),
                                );
                                window.with_content_mask(
                                    Some(ContentMask { bounds: mask }),
                                    |window| {
                                        paint(
                                            &row.wave[flicker_frame(elapsed)],
                                            bounds.origin,
                                            px(line_height),
                                            wave_color.opacity(strip.alpha as f32),
                                            window,
                                            cx,
                                        );
                                    },
                                );
                            }
                        });
                    },
                )
                .w(width)
                .h(px(line_height))
                .flex_shrink_0(),
            );
        }
        root
    }
}

/// A fixed-layout text reveal. Real text remains its accessible label throughout.
/// Explicit newlines are supported; soft wrapping is deliberately not applied.
#[derive(IntoElement)]
pub struct AsciiReveal {
    id: ElementId,
    text: SharedString,
    theme: Theme,
    options: DecodeOptions,
    direction: Direction,
    loading: bool,
    reduced: bool,
    variant: SkeletonVariant,
    playback: Playback,
}
impl AsciiReveal {
    /// Decode text on first display. Loading-to-ready transitions restart automatically.
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>, theme: Theme) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            theme,
            options: DecodeOptions::default(),
            direction: Direction::Reveal,
            loading: false,
            reduced: false,
            variant: SkeletonVariant::Dither,
            playback: Playback::default(),
        }
    }
    /// Reveal glyph preset.
    pub fn preset(mut self, preset: Preset) -> Self {
        self.options.preset = preset;
        self
    }
    /// Fast or cinematic timing.
    pub fn timing(mut self, timing: Timing) -> Self {
        self.options.timing = timing;
        self
    }
    /// Stable seed.
    pub fn seed(mut self, seed: i32) -> Self {
        self.options.seed = seed;
        self
    }
    /// Playback multiplier (minimum 0.1).
    pub fn speed(mut self, speed: f64) -> Self {
        self.options.speed = if speed.is_finite() {
            speed.max(0.1)
        } else {
            1.
        };
        self
    }
    /// Use only decimal digits while scrambling, even for nonnumeric text.
    pub fn digits(mut self, digits: bool) -> Self {
        self.options.digits = digits;
        self
    }
    /// Accent flash on landing, fading for up to 420 ms before the final bake.
    pub fn flash(mut self, flash: bool) -> Self {
        self.options.flash = flash;
        self
    }
    /// Reverse the state machine to scramble out.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
    /// Show a skeleton in the reserved text layout while true.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }
    /// Texture used while loading.
    pub fn skeleton_variant(mut self, variant: SkeletonVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Replace decoding with the original short fade; hides are immediate.
    pub fn reduced_motion(mut self, reduced: bool) -> Self {
        self.reduced = reduced;
        self
    }
    /// Caller-owned local animation start. Keep it stable across renders.
    pub fn started_at(mut self, start: Instant) -> Self {
        self.playback.start = Some(start);
        self
    }
    /// Exact frame for previews/captures; schedules no redraws.
    pub fn elapsed(mut self, elapsed: Duration) -> Self {
        self.playback.elapsed = Some(elapsed);
        self
    }
}

struct TextCell {
    real: ShapedLine,
    origin: Point<Pixels>,
    width: Pixels,
}
struct TextCache {
    decode: Decode,
    direction: Direction,
    lines: Vec<ShapedLine>,
    cells: Vec<TextCell>,
    glyphs: Vec<(&'static str, ShapedLine)>,
    size: Size<Pixels>,
}
impl TextCache {
    fn new(text: &str, options: DecodeOptions, direction: Direction, window: &Window) -> Self {
        let decode = Decode::new(text, options);
        let lines: Vec<_> = text
            .split('\n')
            .map(|s| shape(s, type_size::BODY, None, window))
            .collect();
        let mut cells = Vec::new();
        let mut line_index = 0;
        let mut byte = 0;
        for g in text.graphemes(true) {
            let line = &lines[line_index];
            let x = line.x_for_index(byte);
            let newline = g.ends_with('\n');
            let width = if newline {
                px(space::NONE)
            } else {
                line.x_for_index(byte + g.len()) - x
            };
            cells.push(TextCell {
                real: shape(if newline { "" } else { g }, type_size::BODY, None, window),
                origin: point(x, px(line_index as f32 * type_size::LINE_SM)),
                width,
            });
            if newline {
                line_index += 1;
                byte = 0;
            } else {
                byte += g.len();
            }
        }
        let spec = options.preset.spec();
        let mut glyphs = Vec::new();
        for glyph in spec
            .ramp
            .graphemes(true)
            .chain(spec.scramble.graphemes(true))
            .chain("0123456789".graphemes(true))
            .chain(spec.cursor)
        {
            if !glyphs.iter().any(|(g, _)| *g == glyph) {
                glyphs.push((glyph, shape(glyph, type_size::BODY, None, window)));
            }
        }
        let width = lines
            .iter()
            .map(ShapedLine::width)
            .fold(px(space::NONE), Pixels::max);
        let height = px(lines.len() as f32 * type_size::LINE_SM);
        Self {
            decode,
            direction,
            lines,
            cells,
            glyphs,
            size: size(width, height),
        }
    }
}

impl RenderOnce for AsciiReveal {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (cache, clock) = window.with_element_namespace(self.id.clone(), |window| {
            (
                window.use_keyed_state("text", cx, |_, _| None::<Rc<TextCache>>),
                window.use_keyed_state("clock", cx, |_, _| Clock::new()),
            )
        });
        let (cache, restart) = cache.update(cx, |cache, _| {
            let restart = cache.as_ref().is_none_or(|c| {
                c.decode.text() != self.text.as_ref()
                    || c.decode.options() != self.options
                    || c.direction != self.direction
            });
            if restart {
                *cache = Some(Rc::new(TextCache::new(
                    &self.text,
                    self.options,
                    self.direction,
                    window,
                )));
            }
            (cache.as_ref().cloned(), restart)
        });
        let Some(cache) = cache else {
            return div().into_any_element();
        };
        let elapsed = clock.update(cx, |clock, _| {
            clock.elapsed(self.playback, self.loading, restart)
        });
        let finished = if self.reduced {
            self.direction == Direction::Hide || elapsed >= REDUCED_FADE_MS
        } else {
            cache.decode.is_finished(elapsed)
        };
        let active = !self.loading && !finished && self.playback.elapsed.is_none();
        if !active {
            clock.update(cx, |clock, _| clock.stop());
        }
        let mut root = div()
            .id(self.id)
            .role(Role::Label)
            .aria_label(self.text.clone())
            .relative()
            .w(cache.size.width)
            .h(cache.size.height)
            .flex_shrink_0()
            .overflow_hidden();
        if self.loading {
            // The real line widths and heights continue to reserve layout through loading.
            for (i, line) in cache.lines.iter().enumerate() {
                let mut sk = AsciiSkeleton::new(("loading", i), self.theme)
                    .lines(1)
                    .variant(self.variant)
                    .seed(self.options.seed.wrapping_add(i as i32))
                    .reduced_motion(self.reduced);
                sk.playback = self.playback;
                root = root.child(
                    div()
                        .absolute()
                        .left(px(space::NONE))
                        .top(px(i as f32 * type_size::LINE_SM))
                        .w(line.width())
                        .h(px(type_size::LINE_SM))
                        .child(sk),
                );
            }
        } else {
            root = root.child(
                canvas(
                    move |bounds, window, cx| {
                        schedule(&clock, bounds, active, window, cx);
                    },
                    move |bounds, (), window, cx| {
                        let alpha = if self.reduced {
                            reduced_motion_alpha(elapsed, self.direction)
                        } else {
                            1.
                        };
                        if self.reduced || finished {
                            if self.direction == Direction::Hide {
                                return;
                            }
                            for (i, line) in cache.lines.iter().enumerate() {
                                paint(
                                    line,
                                    bounds.origin
                                        + point(px(space::NONE), px(i as f32 * type_size::LINE_SM)),
                                    px(type_size::LINE_SM),
                                    self.theme.text().opacity(alpha as f32),
                                    window,
                                    cx,
                                );
                            }
                            return;
                        }
                        for (cell, layout) in cache
                            .decode
                            .frame(elapsed, self.direction)
                            .zip(&cache.cells)
                        {
                            let origin = bounds.origin + layout.origin;
                            match cell.kind {
                                CellKind::Pending => {}
                                CellKind::Landed => {
                                    let color = flash_color(self.theme, cell.flash);
                                    paint(
                                        &layout.real,
                                        origin,
                                        px(type_size::LINE_SM),
                                        color,
                                        window,
                                        cx,
                                    );
                                }
                                CellKind::Cursor | CellKind::Scramble => {
                                    if let Some((_, glyph)) =
                                        cache.glyphs.iter().find(|(g, _)| *g == cell.glyph)
                                    {
                                        let color = if cell.kind == CellKind::Cursor
                                            || self.options.preset.spec().accent_glyphs
                                        {
                                            self.theme.accent()
                                        } else {
                                            self.theme.text()
                                        };
                                        let center = origin
                                            + point(
                                                (layout.width - glyph.width()) / 2.,
                                                px(space::NONE),
                                            );
                                        paint(
                                            glyph,
                                            center,
                                            px(type_size::LINE_SM),
                                            color.opacity(cell.alpha as f32),
                                            window,
                                            cx,
                                        );
                                    }
                                }
                            }
                        }
                    },
                )
                .size_full(),
            );
        }
        root.into_any_element()
    }
}

fn flash_color(theme: Theme, flash: f64) -> Hsla {
    if flash <= 0. {
        return theme.text();
    }
    let accent: Rgba = theme.accent().into();
    let text: Rgba = theme.text().into();
    let blend =
        |a: f32, b: f32| ((f64::from(b) + f64::from(a - b) * flash) * 255.).round() as f32 / 255.;
    Rgba {
        r: blend(accent.r, text.r),
        g: blend(accent.g, text.g),
        b: blend(accent.b, text.b),
        a: text.a,
    }
    .into()
}

/// Reveal a caller-owned image element using a precomputed RGBA thumbnail grid.
/// Supply an image already fitted to `Media::size()` and reuse the same `Rc<Media>`.
#[derive(IntoElement)]
pub struct AsciiImageReveal {
    id: ElementId,
    media: Rc<Media>,
    image: AnyElement,
    label: SharedString,
    theme: Theme,
    direction: Direction,
    reduced: bool,
    playback: Playback,
}
impl AsciiImageReveal {
    /// Create an accessible thumbnail reveal without loading or decoding an image.
    pub fn new(
        id: impl Into<ElementId>,
        media: Rc<Media>,
        image: impl IntoElement,
        label: impl Into<SharedString>,
        theme: Theme,
    ) -> Self {
        Self {
            id: id.into(),
            media,
            image: image.into_any_element(),
            label: label.into(),
            theme,
            direction: Direction::Reveal,
            reduced: false,
            playback: Playback::default(),
        }
    }
    /// Reverse the glyph grid and crossfade to hide the image.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
    /// Replace the grid with the upstream short fade.
    pub fn reduced_motion(mut self, reduced: bool) -> Self {
        self.reduced = reduced;
        self
    }
    /// Caller-supplied local start time.
    pub fn started_at(mut self, start: Instant) -> Self {
        self.playback.start = Some(start);
        self
    }
    /// Exact frame for capture; no automatic scheduling.
    pub fn elapsed(mut self, elapsed: Duration) -> Self {
        self.playback.elapsed = Some(elapsed);
        self
    }
}

struct MediaCache {
    media: Rc<Media>,
    direction: Direction,
    glyphs: Vec<(f64, &'static str, ShapedLine)>,
}
impl RenderOnce for AsciiImageReveal {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (cache, clock) = window.with_element_namespace(self.id.clone(), |window| {
            (
                window.use_keyed_state("media", cx, |_, _| None::<Rc<MediaCache>>),
                window.use_keyed_state("clock", cx, |_, _| Clock::new()),
            )
        });
        let (cache, restart) = cache.update(cx, |cache, _| {
            let restart = cache.as_ref().is_none_or(|c| {
                !Rc::ptr_eq(&c.media, &self.media) || c.direction != self.direction
            });
            if restart {
                let [width, height] = self.media.size();
                let scale = super::core::clamp(f64::from(width.min(height)) / 230., 0.4, 1.);
                let mut glyphs = Vec::new();
                for height in self.media.options().preset.spec().cells {
                    let size = ((height * scale).max(3.) * 1.02).round();
                    for glyph in self.media.options().preset.spec().ramp.graphemes(true) {
                        glyphs.push((size, glyph, shape(glyph, size as f32, None, window)));
                    }
                }
                *cache = Some(Rc::new(MediaCache {
                    media: self.media.clone(),
                    direction: self.direction,
                    glyphs,
                }));
            }
            (cache.as_ref().cloned(), restart)
        });
        let Some(cache) = cache else {
            return div().into_any_element();
        };
        let elapsed = clock.update(cx, |clock, _| clock.elapsed(self.playback, false, restart));
        let frame = self.media.frame(elapsed, self.direction);
        let alpha = if self.reduced {
            reduced_motion_alpha(elapsed, self.direction)
        } else {
            frame.image_alpha
        };
        let finished = if self.reduced {
            self.direction == Direction::Hide || elapsed >= REDUCED_FADE_MS
        } else {
            self.media.is_finished(elapsed)
        };
        let active = !finished && self.playback.elapsed.is_none();
        if !active {
            clock.update(cx, |clock, _| clock.stop());
        }
        let [width, height] = self.media.size();
        let direction = self.direction;
        let theme = self.theme;
        let reduced = self.reduced;
        div()
            .id(self.id)
            .role(Role::Image)
            .aria_label(self.label)
            .relative()
            .w(px(width as f32))
            .h(px(height as f32))
            .overflow_hidden()
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .opacity(alpha as f32)
                    .child(self.image),
            )
            .child(
                canvas(
                    move |bounds, window, cx| {
                        schedule(&clock, bounds, active, window, cx);
                    },
                    move |bounds, (), window, cx| {
                        if reduced {
                            return;
                        }
                        let frame = cache.media.frame(elapsed, direction);
                        for cell in frame.cells() {
                            let Some((_, _, glyph)) = cache
                                .glyphs
                                .iter()
                                .find(|(size, g, _)| *size == cell.font_size && *g == cell.glyph)
                            else {
                                continue;
                            };
                            let color = cell.color.map_or_else(
                                || {
                                    if cache.media.options().preset.spec().color == Color::Accent {
                                        theme.accent()
                                    } else {
                                        theme.text()
                                    }
                                },
                                |[r, g, b]| {
                                    Rgba {
                                        r: f32::from(r) / 255.,
                                        g: f32::from(g) / 255.,
                                        b: f32::from(b) / 255.,
                                        a: 1.,
                                    }
                                    .into()
                                },
                            );
                            let origin = bounds.origin
                                + point(
                                    px(cell.x as f32) - glyph.width() / 2.,
                                    px((cell.y - frame.cell_height() / 2.) as f32),
                                );
                            paint(
                                glyph,
                                origin,
                                px(frame.cell_height() as f32),
                                color.opacity(frame.glyph_alpha as f32),
                                window,
                                cx,
                            );
                        }
                    },
                )
                .size_full(),
            )
            .into_any_element()
    }
}
