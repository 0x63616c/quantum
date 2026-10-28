//! Pure document invariants and atomic, shared editing for Quantum.
//!
//! Coordinates are right handed, Y up; all regions have an exclusive maximum.
//! Palette slot zero represents air. Use [`Engine`] to mutate validated documents.
/// Canonical JSON utilities shared with the portable format and version hashing.
pub mod canonical;
mod chunk;
mod command;
mod engine;
mod entities;
mod extensions;
mod model;
mod palette;
pub use chunk::*;
pub use command::*;
pub use engine::*;
pub use entities::*;
pub use extensions::*;
pub use model::*;
pub use palette::*;
pub use uuid::Uuid;

/// A model operation failed before changing live state.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A value violates a model invariant.
    #[error("invalid data: {0}")]
    Invalid(String),
    /// A stable identifier was not found.
    #[error("not found: {0}")]
    NotFound(String),
    /// A coordinate lies outside the finite document or operation region.
    #[error("out of bounds: {0:?}")]
    OutOfBounds(Pos),
    /// Voxel mutation requires a visible, unlocked layer.
    #[error("layer is locked or hidden: {0}")]
    LayerBlocked(Uuid),
    /// Work, storage, or history admission was exceeded.
    #[error("limit exceeded: {0}")]
    Limit(String),
    /// The caller captured an obsolete revision or history head.
    #[error("conflict: {0}")]
    Conflict(String),
    /// A command in an atomic batch failed.
    #[error("batch command {index}: {source}")]
    Batch {
        /// Zero-based command index.
        index: usize,
        /// Underlying error.
        source: Box<Error>,
    },
}
/// The result of a checked core operation.
pub type Result<T> = std::result::Result<T, Error>;
