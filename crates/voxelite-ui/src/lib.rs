//! Shared GPUI controls and presentation primitives for the Voxelite shell.
//! Components own field editing and appearance; the app owns domain state.
//! See `crates/voxelite-ui/AGENTS.md` for tokens, components and migration/verification rules.
pub mod ascii_reveal;
mod color;
pub use ascii_reveal::{AsciiImageReveal, AsciiReveal, AsciiSkeleton};
mod color_picker;
mod components;
mod navigation;
mod surfaces;
pub use navigation::*;
pub use surfaces::*;
mod menu;
mod numeric_field;
mod text_field;
pub use numeric_field::*;
pub use text_field::*;
pub mod editing;
mod slider;
mod status;
pub mod tokens;
pub use color::*;
pub use color_picker::*;
pub use components::*;
pub use menu::*;
pub use slider::*;
pub use status::*;
pub use tokens::Theme;

mod spinner;
pub use spinner::Spinner;
