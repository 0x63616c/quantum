//! Typed edits; batch preparation runs against private copy-on-write state.
mod apply;
mod cells;
mod shapes;
mod structure;
mod types;

pub(crate) use apply::{Work, apply, validate_restore};
pub use types::{
    Axis, Collision, Command, LayerEdit, Mirror, Mode, PaletteEdit, Shape, Transform, Voxel,
};

#[cfg(test)]
mod tests;
