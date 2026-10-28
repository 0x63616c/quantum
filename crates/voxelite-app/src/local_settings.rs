//! Host-owned bounded local settings storage.
use crate::editor::{EditorError, EditorResult};
use std::path::{Path, PathBuf};

/// App-local settings directory, overridable for isolated tests and captures.
pub fn data_dir() -> PathBuf {
    if let Some(directory) = std::env::var_os("VOXELITE_DATA_DIR") {
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
        home.join("Library/Application Support/voxelite")
    } else {
        xdg.map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(".local/share"))
            .join("voxelite")
    }
}

/// Bind JSON storage, migrating a legacy macOS file before its first read.
pub fn json_store<T: serde::Serialize + serde::de::DeserializeOwned + Default>(
    path: &Path,
) -> EditorResult<voxelite_format::storage::JsonStore<T>> {
    if cfg!(target_os = "macos")
        && std::env::var_os("VOXELITE_DATA_DIR").is_none()
        && path.parent() == Some(data_dir().as_path())
    {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
        let old = settings_dir(
            &home,
            std::env::var_os("XDG_DATA_HOME").as_deref().map(Path::new),
            false,
        );
        if let Some(name) = path.file_name() {
            migrate_file(&old.join(name), path)?;
        }
    }
    Ok(voxelite_format::storage::JsonStore::new(path))
}

fn migrate_file(old: &Path, new: &Path) -> EditorResult<()> {
    let mut marker = new.as_os_str().to_os_string();
    marker.push(".migrated");
    let marker = PathBuf::from(marker);
    if marker.try_exists()? {
        return Ok(());
    }
    // Even an empty or corrupt new file is authoritative. Never overwrite it.
    match std::fs::symlink_metadata(new) {
        Ok(_) => {
            write_local(&marker, b"migrated\n")?;
            return Ok(());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let Some(bytes) = read_local(old)? else {
        return Ok(());
    };
    if let Some(parent) = new.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match voxelite_format::session::write_new(new, &bytes) {
        Ok(()) => {
            write_local(&marker, b"migrated\n")?;
            std::fs::remove_file(old)?;
        }
        Err(voxelite_format::session::SessionError::FileExists) => {}
        Err(error) => return Err(EditorError::Gesture(error.to_string())),
    }
    Ok(())
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
    let bytes = Some(bytes);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn platform_paths_and_migration_preserve_new_data() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path();
        let old_dir = settings_dir(home, None, false);
        let new_dir = settings_dir(home, None, true);
        assert_eq!(new_dir, home.join("Library/Application Support/voxelite"));
        assert_eq!(old_dir, home.join(".local/share/voxelite"));
        assert_eq!(
            settings_dir(home, Some(&home.join("xdg")), false),
            home.join("xdg/voxelite")
        );
        for (old_exists, new_exists) in [(true, false), (false, true), (true, true), (false, false)]
        {
            let name = format!("{old_exists}-{new_exists}.json");
            let old = old_dir.join(&name);
            let new = new_dir.join(&name);
            if old_exists {
                write_local(&old, b"old").unwrap();
            }
            if new_exists {
                write_local(&new, b"new").unwrap();
            }
            migrate_file(&old, &new).unwrap();
            if new_exists {
                assert_eq!(std::fs::read(&new).unwrap(), b"new");
            } else if old_exists {
                assert_eq!(std::fs::read(&new).unwrap(), b"old");
                assert!(!old.exists());
            } else {
                assert!(!new.exists());
            }
            if old_exists && new_exists {
                assert_eq!(std::fs::read(&old).unwrap(), b"old");
            }
            migrate_file(&old, &new).unwrap();
            if old_exists || new_exists {
                std::fs::remove_file(&new).unwrap();
                migrate_file(&old, &new).unwrap();
                assert!(!new.exists(), "migration must not resurrect legacy data");
            }
        }
    }
}
