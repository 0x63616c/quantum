//! Display-independent editor state, picking, camera, and file persistence.

/// Application identifier shared by native windows and the macOS bundle.
pub const APP_ID: &str = "co.worldwidewebb.quantum";

#[cfg(unix)]
mod attachment;
pub mod comments;
pub mod core_plugin;
pub use core_plugin::app_host;
pub mod debug_overlay;
pub mod diagnostics;
pub mod editor;
pub mod frame_meter;
pub mod history;
pub mod local_settings;
pub mod palettes;
mod persistence;
pub mod preferences;
pub mod recents;
pub mod reduce_motion;
pub mod samples;
#[path = "ui/save_flow/format.rs"]
pub mod save_format;
pub mod save_picker;
pub mod startup;
pub mod thumbnails;

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
