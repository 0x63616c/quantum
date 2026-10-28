//! Host-owned bounded local settings storage.
use crate::editor::{EditorError, EditorResult};

pub mod bundle_migration;
mod migration;
use std::path::{Path, PathBuf};

/// App-local settings directory, overridable for isolated tests and captures.
pub fn data_dir() -> PathBuf {
    if let Some(directory) = std::env::var_os("QUANTUM_DATA_DIR").or_else(legacy_override) {
        return directory.into();
    }
    settings_dir(
        Path::new(&std::env::var_os("HOME").unwrap_or_default()),
        std::env::var_os("XDG_DATA_HOME").as_deref().map(Path::new),
        cfg!(target_os = "macos"),
    )
}

fn settings_dir(home: &Path, xdg: Option<&Path>, macos: bool) -> PathBuf {
    if macos {
        home.join("Library/Application Support/quantum")
    } else {
        xdg.map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(".local/share"))
            .join("quantum")
    }
}

// Kept only as an explicit override for existing launch scripts.
fn legacy_override() -> Option<std::ffi::OsString> {
    std::env::var_os("VOXELITE_DATA_DIR")
}

/// Whether startup uses an explicitly isolated data directory.
pub fn has_data_override() -> bool {
    std::env::var_os("QUANTUM_DATA_DIR")
        .or_else(legacy_override)
        .is_some()
}

/// Move previous installations' data before any settings, recents or palettes load.
/// An explicit directory override is used as-is. Migration failures stop startup
/// rather than silently replacing the user's settings with defaults.
pub fn initialize_data_dir() -> EditorResult<()> {
    if std::env::var_os("QUANTUM_DATA_DIR")
        .or_else(legacy_override)
        .is_some()
    {
        return Ok(());
    }
    let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let xdg = std::env::var_os("XDG_DATA_HOME");
    migration::initialize(
        &home,
        xdg.as_deref().map(Path::new),
        cfg!(target_os = "macos"),
    )?;
    Ok(())
}

/// Bind bounded JSON storage after startup has migrated the data directory.
pub fn json_store<T: serde::Serialize + serde::de::DeserializeOwned + Default>(
    path: &Path,
) -> EditorResult<quantum_format::storage::JsonStore<T>> {
    Ok(quantum_format::storage::JsonStore::new(path))
}

/// Replace a local settings file atomically.
pub fn write_local(path: &Path, bytes: &[u8]) -> EditorResult<()> {
    crate::persistence::write_local(path, bytes)
}

/// Read bounded settings bytes, or `None` if no settings have been saved.
pub fn read_local(path: &Path) -> EditorResult<Option<Vec<u8>>> {
    use std::io::Read;
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let mut bytes = Vec::new();
    file.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(EditorError::Gesture("Settings file too large".into()));
    }
    Ok(Some(bytes))
}
