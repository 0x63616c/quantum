//! Portable application preferences, separate from project documents.
use crate::{
    editor::{EditorError, EditorResult},
    local_settings,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use voxelite_render::AmbientOcclusionStrength;

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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    /// User-selected color scheme.
    pub appearance: Appearance,
    /// Show the ground grid in the viewport.
    pub grid: bool,
    /// Supersample settled viewport renders.
    pub antialiasing: bool,
    /// Enable local voxel contact shading in every viewport.
    pub ambient_occlusion: bool,
    /// AO multiplier, retained while the toggle is off; one is the original look.
    pub ambient_occlusion_strength: AmbientOcclusionStrength,
    /// Display viewport FPS, average frame time and renderer in the status bar.
    pub show_performance: bool,
    /// Display mouse navigation hints over the viewport.
    pub navigation_hints: bool,
    /// Animate preset and programmatic camera changes; off snaps. macOS Reduce Motion always snaps.
    pub animate_camera: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            appearance: Appearance::System,
            grid: true,
            antialiasing: true,
            ambient_occlusion: true,
            ambient_occlusion_strength: AmbientOcclusionStrength::default(),
            show_performance: true,
            navigation_hints: true,
            animate_camera: true,
        }
    }
}
impl Preferences {
    /// Viewport strength with the enable toggle applied. Exports use their own defaults.
    pub fn viewport_ambient_occlusion(&self) -> AmbientOcclusionStrength {
        if self.ambient_occlusion {
            self.ambient_occlusion_strength
        } else {
            AmbientOcclusionStrength::OFF
        }
    }

    /// Load preferences; missing and newly introduced fields use defaults.
    pub fn load(path: &Path) -> EditorResult<Self> {
        let Some(bytes) = local_settings::read_local(path)? else {
            return Ok(Self::default());
        };
        serde_json::from_slice(&bytes).map_err(|e| EditorError::Gesture(e.to_string()))
    }
    /// Persist atomically on desktop or in origin-local storage on the web.
    pub fn save(&self, path: &Path) -> EditorResult<()> {
        let bytes =
            serde_json::to_vec_pretty(self).map_err(|e| EditorError::Gesture(e.to_string()))?;
        local_settings::write_local(path, &bytes)
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
        assert!(upgraded.ambient_occlusion);
        assert_eq!(upgraded.viewport_ambient_occlusion().get(), 1.0);
    }

    #[test]
    fn grid_toggle_preserves_the_existing_persistence_key_and_default() {
        let mut preferences = Preferences::default();
        assert!(preferences.grid);
        preferences.grid = !preferences.grid;
        assert_eq!(
            serde_json::to_string(&preferences).unwrap(),
            r#"{"appearance":"system","grid":false,"antialiasing":true,"ambient_occlusion":true,"ambient_occlusion_strength":1.0,"show_performance":true,"navigation_hints":true,"animate_camera":true}"#
        );
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        preferences.save(&path).unwrap();
        assert!(!Preferences::load(&path).unwrap().grid);
    }

    #[test]
    fn old_settings_enable_camera_animation_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(
            &path,
            br#"{"appearance":"dark","grid":false,"antialiasing":true,"show_performance":true,"navigation_hints":false}"#,
        ).unwrap();
        let mut preferences = Preferences::load(&path).unwrap();
        assert!(preferences.animate_camera);
        preferences.animate_camera = false;
        preferences.save(&path).unwrap();
        assert!(!Preferences::load(&path).unwrap().animate_camera);
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

    #[test]
    fn ambient_occlusion_survives_restart_and_toggle_retains_strength() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        for value in [0.0, 0.5, 1.0, 2.0] {
            let strength = AmbientOcclusionStrength::new(value).unwrap();
            let mut preferences = Preferences {
                ambient_occlusion_strength: strength,
                ..Default::default()
            };
            for enabled in [true, false, true] {
                preferences.ambient_occlusion = enabled;
                preferences.save(&path).unwrap();
                let loaded = Preferences::load(&path).unwrap();
                assert_eq!(loaded, preferences);
                assert_eq!(loaded.ambient_occlusion_strength, strength);
                assert_eq!(
                    loaded.viewport_ambient_occlusion(),
                    if enabled {
                        strength
                    } else {
                        AmbientOcclusionStrength::OFF
                    }
                );
            }
        }
        for value in ["-0.1", "2.1", "null", "\"NaN\""] {
            std::fs::write(&path, format!("{{\"ambient_occlusion_strength\":{value}}}")).unwrap();
            assert!(Preferences::load(&path).is_err());
        }
    }
}
