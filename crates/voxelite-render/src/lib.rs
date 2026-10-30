//! Deterministic headless geometry and software rendering; no GPU/display required.
mod mesh;
pub use mesh::*;
mod camera;
mod raster;
pub use camera::*;
pub use raster::*;
