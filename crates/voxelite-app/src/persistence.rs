//! Bounded disk snapshots and optimistic, atomic project saves.
use crate::editor::{EditorError, EditorResult};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use voxelite_format::Project;

pub(crate) type Fingerprint = [u8; 32];

pub(crate) fn read(path: &Path) -> EditorResult<Vec<u8>> {
    // Reject devices and FIFOs before open, which could otherwise block forever.
    if !fs::metadata(path)?.is_file() {
        return Err(EditorError::Gesture("Open a regular .voxl file".into()));
    }
    let file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(EditorError::Gesture("Open a regular .voxl file".into()));
    }
    let mut bytes = Vec::new();
    file.take(voxelite_format::MAX_FILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > voxelite_format::MAX_FILE_BYTES {
        return Err(EditorError::Gesture(
            "Project exceeds the 128 MiB file limit".into(),
        ));
    }
    Ok(bytes)
}

pub(crate) fn fingerprint(bytes: &[u8]) -> Fingerprint {
    Sha256::digest(bytes).into()
}

pub(crate) fn destination(path: &Path) -> EditorResult<PathBuf> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .ok_or_else(|| EditorError::Gesture("Choose a project filename".into()))?;
    let path = parent.canonicalize()?.join(name);
    if let Ok(metadata) = fs::symlink_metadata(&path)
        && !metadata.file_type().is_file()
    {
        return Err(EditorError::Gesture(
            "Save to a regular file, not a directory or symbolic link".into(),
        ));
    }
    Ok(path)
}

/// An immutable save snapshot, safe to execute on a background worker.
pub struct SaveTask {
    pub(crate) project: Project,
    pub(crate) path: PathBuf,
    pub(crate) baseline: Option<(PathBuf, Fingerprint)>,
    pub(crate) revision: u64,
}

/// A durable save receipt. Install it only into the editor that requested it.
pub struct SavedProject {
    pub(crate) project: Project,
    pub(crate) path: PathBuf,
    pub(crate) fingerprint: Fingerprint,
    pub(crate) revision: u64,
}

impl SaveTask {
    /// Serialize and persist, rejecting external changes to an open destination.
    /// Save As may replace a selected regular file; changes during writing still reject.
    pub fn write(self) -> EditorResult<SavedProject> {
        let path = destination(&self.path)?;
        let current = match read(&path) {
            Ok(bytes) => Some(fingerprint(&bytes)),
            Err(EditorError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e),
        };
        let conflict = || {
            EditorError::Gesture(
                "File changed outside Voxelite. Save As to a new file, or reopen it.".into(),
            )
        };
        if let Some((original, expected)) = &self.baseline
            && *original == path
            && current != Some(*expected)
        {
            return Err(conflict());
        }
        let parent = path.parent().expect("canonical parent");
        let bytes = self.project.to_bytes()?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(&bytes)?;
        if let Ok(metadata) = fs::metadata(&path) {
            temporary
                .as_file()
                .set_permissions(metadata.permissions())?;
        }
        temporary.as_file().sync_all()?;
        // Recheck after serialization. This is optimistic conflict detection, not a lock
        // against unrelated processes racing the final filesystem rename.
        if let Some(expected) = current {
            destination(&path)?;
            if fingerprint(&read(&path).map_err(|_| conflict())?) != expected {
                return Err(conflict());
            }
            temporary.persist(&path).map_err(|e| e.error)?;
        } else {
            temporary.persist_noclobber(&path).map_err(|e| e.error)?;
        }
        #[cfg(unix)]
        File::open(parent)?.sync_all().map_err(|error| {
            EditorError::Gesture(format!("File was written, but directory sync failed: {error}. Verify the saved file before retrying."))
        })?;
        Ok(SavedProject {
            project: self.project,
            path,
            fingerprint: fingerprint(&bytes),
            revision: self.revision,
        })
    }
}
