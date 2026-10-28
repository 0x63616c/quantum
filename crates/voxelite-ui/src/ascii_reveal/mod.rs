// Ported from ascii-reveal by Quan Hoang (MIT), https://github.com/quanhoangindex/ascii-reveal — see crates/voxelite-ui/NOTICE
//! Deterministic ASCII placeholders and content reveals from ascii-reveal v0.3.0
//! (`f602e204232d2d31de2b67c66a166bcdbde51602`). The four core modules have no GPUI
//! imports. Cache a [`core::Decode`] or [`media::Media`] and evaluate arbitrary times;
//! the GPUI builders cache these plans in keyed state automatically.
//!
//! # Deviations from the original
//! - Explicit GPUI text/image builders replace DOM traversal, CSS selectors, SSR,
//!   hydration, surfaces, pointer-origin discovery, lazy sections and mascots.
//!   Use [`core::Timing::wave_delay`] when coordinating separate targets. Local
//!   clocks omit the browser's 16 ms mount delay and random controller/run seeds.
//! - The app passes reduced motion. Skeletons are still; reveals use the original
//!   220 ms CSS ease-out fade; reduced-motion hides are instant.
//! - Geist Mono and library theme tokens replace CSS/custom fonts and colors.
//!   Text preserves explicit newlines but does not soft-wrap. Final shaped text
//!   reserves its exact dimensions, including non-ASCII graphemes. Unicode
//!   segmentation uses the crate's pinned Unicode version instead of the host ICU.
//!   GPUI line metrics replace Canvas text-baseline adjustments. Presets are a
//!   typed built-in enum, without arbitrary runtime CSS/preset objects. Widths
//!   use GPUI lengths; callers compose block height and shape rather than CSS.
//! - Skeletons cache three shaped runs per row (base and two wave alternatives)
//!   and paint the original 16 clipped strips, using GPUI's glyph atlas instead
//!   of rasterizing offscreen canvases. Block corners use the library radius;
//!   clipping is rectangular, as elsewhere in GPUI. Seed zero is respected for
//!   both base and wave (upstream's wave alone substitutes seed one).
//! - One-shot 33 ms timers replace the shared browser rAF. They stop on completion,
//!   removal, explicit elapsed-time snapshots, static/reduced skeletons, clipping,
//!   and inactive/hidden windows. There is no browser watchdog; delayed frames
//!   evaluate current time without replaying missed ticks.
//! - Images take bounded, already fitted RGBA thumbnails at DPR 1; loading,
//!   decoding, object-fit, CORS fallback and device-pixel resampling belong to the
//!   caller. All five grids are built once up front. The sampling, gates,
//!   luminance ramp and coarse-to-fine crossfade match `mediaTarget`.
//! - A loading text builder switches directly to the skeleton requested by this
//!   API. Explicit [`core::Direction::Hide`] provides upstream scramble-out.
//!   Changing text/options restarts the local reveal; callers can supply a stable
//!   start instant to synchronize or replay. Non-finite speed falls back to one.
//! - The accessible label is always the real text or supplied image description;
//!   decorative canvas glyphs never enter the accessibility tree. GPUI has no
//!   `aria-busy` builder. There are no interactive controls or focus stops here.

mod clock;
pub mod core;
pub mod media;
mod paint;
pub mod presets;
pub mod skeleton;
mod view;
pub use view::{AsciiImageReveal, AsciiReveal, AsciiSkeleton};

#[cfg(test)]
mod tests;
