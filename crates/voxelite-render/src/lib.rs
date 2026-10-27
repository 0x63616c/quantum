//! Reusable voxel renderer over immutable core snapshots and shared mesh geometry.
//!
//! Public API: cameras ([`Camera`], [`PresetView`], [`Projection`], [`visible_bounds`]),
//! CPU rasterization ([`View`], [`RenderOptions`], [`RenderedImage`], [`RasterCache`],
//! [`render`]), [`RenderError`]/[`RenderResult`], and the optional `gpu` module.
//! Occupancy, chunk meshes, shadow traversal and GPU resources are implementation details.
mod ambient_occlusion;
mod camera;
mod camera_transition;
pub mod diagnostics;
pub use camera_transition::*;
mod occupancy;
mod raster;
pub use ambient_occlusion::AmbientOcclusionStrength;
pub use camera::{Camera, PresetView, Projection, visible_bounds};
pub use raster::{RasterCache, RenderOptions, RenderedImage, View, render};
pub use voxelite_render_plan::*;

/// Shared geometry/render validation error. This alias preserves existing error
/// variants, display strings and caller conversions after geometry moved to mesh.
pub type RenderError = voxelite_mesh::MeshError;
/// Result from rendering operations, retaining the shared geometry error type.
pub type RenderResult<T> = Result<T, RenderError>;

#[cfg(any(test, feature = "gpu"))]
// Linux type-checks GPU resources, but only the Metal worker drives meshing.
#[cfg_attr(all(feature = "gpu", not(target_os = "macos")), allow(dead_code))]
mod chunk_mesh;
#[cfg(any(test, feature = "gpu"))]
mod shadow;
#[cfg(any(test, feature = "gpu"))]
mod surface_size;
#[cfg(feature = "gpu")]
use surface_size::surface_size;
// Shared production viewport and portable GPU test harness.
#[cfg(feature = "gpu")]
pub mod gpu;

#[cfg(test)]
mod tests;
