//! Deterministic headless geometry and software rendering; no GPU/display required.
mod mesh;
pub use mesh::*;
mod camera;
mod raster;
pub use camera::*;
pub use raster::*;

/// Opt-in browser benchmark instrumentation (compiled out of ordinary builds).
pub mod perf;

mod chunk_mesh;
pub use chunk_mesh::*;
#[cfg(feature = "gpu")]
pub mod gpu;
