//! Portable application preferences, separate from project documents.
use crate::{
    editor::{EditorError, EditorResult},
    palettes,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Application color scheme. System tracks live operating-system changes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Appearance {
    /// Follow the operating system.
    #[default]
    System,
    /// Always use the light palette.
    Light,
    /// Always use the dark palette.
    Dark,
}

/// Settings shared by every editor and settings window.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    /// User-selected color scheme.
    pub appearance: Appearance,
    /// Show the ground grid in the viewport.
    pub grid: bool,
    /// Supersample settled viewport renders.
    pub antialiasing: bool,
    /// Display measured CPU render performance in the status bar.
    pub show_performance: bool,
    /// Display mouse navigation hints over the viewport.
    pub navigation_hints: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            appearance: Appearance::System,
            grid: true,
            antialiasing: true,
            show_performance: true,
            navigation_hints: true,
        }
    }
}
impl Preferences {
    /// Load preferences; missing and newly introduced fields use defaults.
    pub fn load(path: &Path) -> EditorResult<Self> {
        let Some(bytes) = palettes::read_local(path)? else {
            return Ok(Self::default());
        };
        serde_json::from_slice(&bytes).map_err(|e| EditorError::Gesture(e.to_string()))
    }
    /// Persist atomically on desktop or in origin-local storage on the web.
    pub fn save(&self, path: &Path) -> EditorResult<()> {
        let bytes =
            serde_json::to_vec_pretty(self).map_err(|e| EditorError::Gesture(e.to_string()))?;
        palettes::write_local(path, &bytes)
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;

    #[test]
    fn preferences_survive_restart_and_accept_new_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        assert_eq!(Preferences::load(&path).unwrap(), Preferences::default());
        let preferences = Preferences {
            appearance: Appearance::Light,
            grid: false,
            ..Default::default()
        };
        preferences.save(&path).unwrap();
        assert_eq!(Preferences::load(&path).unwrap(), preferences);
        std::fs::write(&path, br#"{"appearance":"dark","future_setting":true}"#).unwrap();
        let upgraded = Preferences::load(&path).unwrap();
        assert_eq!(upgraded.appearance, Appearance::Dark);
        assert!(upgraded.grid && upgraded.antialiasing);
    }

    #[test]
    fn corrupt_settings_and_failed_writes_are_reported() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, b"not json").unwrap();
        assert!(Preferences::load(&path).is_err());
        assert!(Preferences::default().save(dir.path()).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"not json");
    }
}
