//! Read-only export callbacks and the shared, byte-stable core encoders.

pub use voxelite_export::{ExportError, ExportResult};

/// Shared encoding operations; built-ins access core exporters only through the API.
pub mod encoding {
    pub use voxelite_export::{glb, png_rgba};
}

/// Captured RGBA8 pixels for a host-selected view or contact sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportImage {
    /// Output width.
    pub width: u32,
    /// Output height.
    pub height: u32,
    /// Row-major, top-to-bottom straight-alpha sRGB pixels.
    pub rgba: Vec<u8>,
}

/// Stateless export implementation. Returns bytes; only the host can write them.
/// Function pointers can run on a worker without sending plugin instances or GUI state.
pub type ExportCallback = fn(&dyn crate::ExportCx) -> ExportResult<Vec<u8>>;
