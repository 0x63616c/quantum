//! Deterministic headless geometry and software rendering; no GPU/display required.
mod mesh;
pub use mesh::*;
mod camera;
mod occupancy;
mod raster;
pub use camera::*;
pub use occupancy::*;
pub use raster::*;

mod chunk_mesh;
pub use chunk_mesh::*;
#[cfg(any(test, feature = "gpu"))]
mod shadow;
#[cfg(any(test, feature = "gpu"))]
mod surface_size;
#[cfg(feature = "gpu")]
use surface_size::surface_size;
// Type-check GPU resources on Linux too; only the Metal worker instantiates them.
#[cfg(feature = "gpu")]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub mod gpu;
