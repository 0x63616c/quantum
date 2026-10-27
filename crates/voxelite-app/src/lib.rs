//! Display-independent editor state, picking, camera, and file persistence.

/// Application identifier shared by native windows and the macOS bundle.
pub const APP_ID: &str = "co.worldwidewebb.voxelite";

#[cfg(unix)]
mod attachment;
pub mod comments;
pub mod core_plugin;
pub use core_plugin::app_host;
pub mod editor;
pub mod frame_meter;
pub mod history;
pub mod local_settings;
pub mod palettes;
#[cfg(not(target_family = "wasm"))]
mod persistence;
pub mod preferences;
#[cfg(not(target_family = "wasm"))]
pub mod recents;
pub mod reduce_motion;
#[path = "ui/save_flow/format.rs"]
pub mod save_format;
#[cfg(not(target_family = "wasm"))]
pub mod thumbnails;

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

#[cfg(test)]
mod tests {
    #[test]
    fn app_id_matches_macos_bundle_identifier() {
        let plist = include_str!("../../../packaging/macos/Info.plist");
        let (_, value) = plist.split_once("<key>CFBundleIdentifier</key>").unwrap();
        let bundle_id = value
            .trim_start()
            .strip_prefix("<string>")
            .unwrap()
            .split_once("</string>")
            .unwrap()
            .0;
        assert_eq!(super::APP_ID, bundle_id);
    }
}
