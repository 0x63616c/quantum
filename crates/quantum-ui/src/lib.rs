//! Shared GPUI controls and presentation primitives for the Quantum shell.
//! Components own field editing and appearance; the app owns domain state.
//! See `crates/quantum-ui/AGENTS.md` for tokens, components and migration/verification rules.
#![doc = include_str!("../README.md")]

pub mod ascii_reveal;
mod color;
pub use ascii_reveal::{AsciiImageReveal, AsciiReveal, AsciiSkeleton};
mod button;
mod color_picker;
mod components;
pub use button::*;
mod feedback;
mod field_style;
pub use field_style::FocusGroup;
mod interaction;
mod overlays;
mod presentation;
mod rows;
pub use feedback::*;
pub use overlays::*;
pub use rows::*;
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
pub use tokens::{ColorToken, ControlHeight, Palette, Radius, Space, TextSize, Theme};

/// Apply a complete typography token in a GPUI builder chain.
pub trait TextStyle: gpui::Styled + Sized {
    /// Set the family, size, weight and line height together.
    fn text_style(self, size: TextSize) -> Self {
        size.apply(self)
    }
}
impl<T: gpui::Styled> TextStyle for T {}
