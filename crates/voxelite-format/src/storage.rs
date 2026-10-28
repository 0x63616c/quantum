//! Durable file replacement and bounded, recoverable JSON storage.
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs,
    io::{self, Read, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
};

pub(crate) fn parent(path: &Path) -> &Path {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
}

pub(crate) fn stage(path: &Path, bytes: &[u8]) -> io::Result<tempfile::NamedTempFile> {
    stage_with_builder(path, bytes, &tempfile::Builder::new())
}

fn stage_with_builder(
    path: &Path,
    bytes: &[u8],
    builder: &tempfile::Builder<'_, '_>,
) -> io::Result<tempfile::NamedTempFile> {
    let mut temporary = builder.tempfile_in(parent(path))?;
    temporary.write_all(bytes)?;
    temporary.flush()?;
    Ok(temporary)
}

pub(crate) fn sync_directory(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    fs::File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Replace a file using a same-directory temporary, file sync, rename and directory sync.
/// Existing target permissions are preserved. On Unix, new files use mode 0o666
/// filtered by the process umask, matching `std::fs::write`.
/// The parent directory must exist. Failures before rename leave the target intact;
/// a directory-sync error means replacement occurred but durability is uncertain.
pub fn safe_write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    replace(path.as_ref(), bytes.as_ref(), || Ok(()))
}

fn replace(
    path: &Path,
    bytes: &[u8],
    before_rename: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    let metadata = fs::metadata(path).ok();
    let builder = tempfile::Builder::new();
    #[cfg(unix)]
    let builder = {
        let mut builder = builder;
        use std::os::unix::fs::PermissionsExt;
        if metadata.is_none() {
            builder.permissions(fs::Permissions::from_mode(0o666));
        }
        builder
    };
    let temporary = stage_with_builder(path, bytes, &builder)?;
    if let Some(metadata) = metadata {
        temporary
            .as_file()
            .set_permissions(metadata.permissions())?;
    }
    temporary.as_file().sync_all()?;
    before_rename()?;
    temporary.persist(path).map_err(|e| e.error)?;
    sync_directory(parent(path))
}

/// A bounded JSON file. Missing files load defaults; malformed files are archived
/// as `filename.corrupt` (or a numbered sibling) and reported as `InvalidData`.
/// The caller may show that error or fall back to defaults without losing evidence.
pub struct JsonStore<T: Serialize + DeserializeOwned + Default> {
    path: PathBuf,
    marker: PhantomData<T>,
}
impl<T: Serialize + DeserializeOwned + Default> JsonStore<T> {
    /// Bind storage to an explicit path, without accessing a home directory.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            marker: PhantomData,
        }
    }
    /// Load at most 4 MiB, preserving corrupt data before returning an error.
    pub fn load(&self) -> io::Result<T> {
        let file = match fs::File::open(&self.path) {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(T::default()),
            Err(e) => return Err(e),
        };
        let mut bytes = Vec::new();
        file.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
        let parsed = if bytes.len() > 4 * 1024 * 1024 {
            Err("settings file exceeds 4 MiB".to_owned())
        } else {
            serde_json::from_slice(&bytes).map_err(|e| e.to_string())
        };
        match parsed {
            Ok(value) => Ok(value),
            Err(error) => {
                // Reserve a unique backup without replacing an earlier corrupt file.
                for index in 0_u64.. {
                    let mut name = self.path.as_os_str().to_os_string();
                    name.push(if index == 0 {
                        ".corrupt".to_owned()
                    } else {
                        format!(".{index}.corrupt")
                    });
                    let backup = PathBuf::from(name);
                    match fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&backup)
                    {
                        Ok(reserved) => {
                            drop(reserved);
                            if let Err(error) = fs::rename(&self.path, &backup) {
                                let _ = fs::remove_file(&backup);
                                return Err(error);
                            }
                            fs::File::open(&backup)?.sync_all()?;
                            sync_directory(parent(&self.path))?;
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("{error}; preserved in {}", backup.display()),
                            ));
                        }
                        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                        Err(e) => return Err(e),
                    }
                }
                unreachable!()
            }
        }
    }
    /// Serialize before replacing the previous file. Create missing parent directories.
    pub fn save(&self, value: &T) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "settings file exceeds 4 MiB",
            ));
        }
        fs::create_dir_all(parent(&self.path))?;
        safe_write(&self.path, bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    #[test]
    fn safe_write_matches_creation_mode_and_preserves_existing_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let reference = dir.path().join("reference");
        let path = dir.path().join("output");
        fs::write(&reference, b"reference").unwrap();
        safe_write(&path, b"new").unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o7777,
            fs::metadata(&reference).unwrap().permissions().mode() & 0o7777,
        );

        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        safe_write(&path, b"replacement").unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o7777,
            0o640,
        );
    }

    #[test]
    fn failure_before_rename_keeps_target_and_cleans_staging() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("output");
        for existing in [false, true] {
            if existing {
                safe_write(&path, b"old").unwrap();
            }
            assert!(replace(&path, b"new", || Err(io::Error::other("injected failure"))).is_err());
            if existing {
                assert_eq!(fs::read(&path).unwrap(), b"old");
            } else {
                assert!(!path.exists());
            }
            assert_eq!(
                fs::read_dir(dir.path()).unwrap().count(),
                usize::from(existing)
            );
        }
        safe_write(&path, b"new").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"new");
    }
    #[test]
    fn json_defaults_round_trip_and_corrupt_backups() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let store = JsonStore::<Vec<String>>::new(&path);
        assert!(store.load().unwrap().is_empty());
        let value = vec!["value".to_owned()];
        store.save(&value).unwrap();
        assert_eq!(store.load().unwrap(), value);
        for (bytes, backup) in [
            (b"broken".as_slice(), "settings.json.corrupt"),
            (b"invalid".as_slice(), "settings.json.1.corrupt"),
        ] {
            safe_write(&path, bytes).unwrap();
            assert_eq!(store.load().unwrap_err().kind(), io::ErrorKind::InvalidData);
            assert_eq!(fs::read(dir.path().join(backup)).unwrap(), bytes);
            assert!(!path.exists());
        }
        store.save(&value).unwrap();
        assert_eq!(store.load().unwrap(), value);
        assert_eq!(
            fs::read(dir.path().join("settings.json.corrupt")).unwrap(),
            b"broken"
        );
    }
}
