//! Deterministic headless geometry and software rendering; no GPU/display required.
mod mesh;
pub use mesh::*;
mod camera;
mod raster;
pub use camera::*;
pub use raster::*;

mod chunk_mesh;
pub use chunk_mesh::*;
#[cfg(any(test, all(feature = "gpu", target_os = "macos")))]
mod shadow;
#[cfg(any(test, all(feature = "gpu", target_os = "macos")))]
mod surface_size;
#[cfg(all(feature = "gpu", target_os = "macos"))]
use surface_size::surface_size;
#[cfg(all(feature = "gpu", target_os = "macos"))]
pub mod gpu;
