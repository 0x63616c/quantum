//! Display-independent editor state, picking, camera, and file persistence.
#[cfg(unix)]
mod attachment;
pub mod comments;
pub mod editor;
pub mod frame_meter;
pub mod gizmo;
pub mod history;
pub mod palettes;
#[cfg(not(target_family = "wasm"))]
mod persistence;
pub mod preferences;

#[cfg(all(feature = "web", target_family = "wasm"))]
extern crate self as voxelite_app;
#[cfg(all(feature = "web", target_family = "wasm"))]
mod actions;
#[cfg(all(feature = "web", target_family = "wasm"))]
mod assets;
#[cfg(all(feature = "web", target_family = "wasm"))]
mod shortcuts;
#[cfg(all(feature = "web", target_family = "wasm"))]
mod text_field;
#[cfg(all(feature = "web", target_family = "wasm"))]
mod theme;
#[cfg(all(feature = "web", target_family = "wasm"))]
mod ui;
#[cfg(all(feature = "web", target_family = "wasm"))]
mod web;
