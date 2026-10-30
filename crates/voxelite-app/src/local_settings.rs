//! Host-owned bounded local settings storage.
use crate::editor::{EditorError, EditorResult};
use std::path::{Path, PathBuf};

/// App-local settings directory, overridable for isolated tests and captures.
pub fn data_dir() -> PathBuf {
    std::env::var_os("VOXELITE_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
                })
                .join("voxelite")
        })
}
/// Replace a local settings file atomically.
#[cfg(not(target_family = "wasm"))]
pub fn write_local(path: &Path, bytes: &[u8]) -> EditorResult<()> {
    crate::persistence::write_local(path, bytes)
}
/// Persist browser settings in origin-local storage.
#[cfg(all(feature = "web", target_family = "wasm"))]
pub fn write_local(path: &Path, bytes: &[u8]) -> EditorResult<()> {
    crate::web::store(&path.to_string_lossy(), bytes)
}

/// Read bounded settings bytes, or `None` if no settings have been saved.
pub fn read_local(path: &Path) -> EditorResult<Option<Vec<u8>>> {
    #[cfg(not(target_family = "wasm"))]
    let bytes = {
        if !path.exists() {
            return Ok(None);
        }
        if std::fs::metadata(path)?.len() > 4 * 1024 * 1024 {
            return Err(EditorError::Gesture("Settings file too large".into()));
        }
        Some(std::fs::read(path)?)
    };
    #[cfg(target_family = "wasm")]
    let bytes = crate::web::load(&path.to_string_lossy())?;
    if bytes.as_ref().is_some_and(|b| b.len() > 4 * 1024 * 1024) {
        return Err(EditorError::Gesture("Settings file too large".into()));
    }
    Ok(bytes)
}
