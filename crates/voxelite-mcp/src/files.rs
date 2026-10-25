//! Root-scoped, bounded file access and crash-resistant atomic replacement.
use crate::error::{Failure, Result};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

pub(crate) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn path(root: &Path, relative: &str) -> Result<PathBuf> {
    let requested = Path::new(relative);
    if requested.as_os_str().is_empty()
        || requested
            .components()
            .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(
            Failure::invalid("path must be relative to the project root, without '..'").at("/path"),
        );
    }
    let joined = root.join(requested);
    let parent = joined
        .parent()
        .ok_or_else(|| Failure::invalid("path needs a filename"))?
        .canonicalize()?;
    if !parent.starts_with(root) {
        return Err(Failure::new(
            "path_outside_root",
            "path traverses a symlink outside the project root",
        ));
    }
    let target = parent.join(
        joined
            .file_name()
            .ok_or_else(|| Failure::invalid("path needs a filename"))?,
    );
    match fs::symlink_metadata(&target) {
        Ok(m) if !m.file_type().is_file() => {
            return Err(Failure::invalid(
                "target must be a regular file, not a directory or symlink",
            ));
        }
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.into()),
        _ => {}
    }
    Ok(target)
}
pub(crate) fn read(path: &Path) -> Result<Vec<u8>> {
    let f = File::open(path)?;
    if !f.metadata()?.is_file() {
        return Err(Failure::invalid("input must be a regular file"));
    }
    let mut bytes = Vec::new();
    f.take(voxelite_format::MAX_FILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > voxelite_format::MAX_FILE_BYTES {
        return Err(Failure::new("limit_exceeded", "file exceeds 128 MiB"));
    }
    Ok(bytes)
}
/// A missing expected hash means create-only; otherwise compare before rename.
pub(crate) fn write(path: &Path, bytes: &[u8], expected_hash: Option<&str>) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| Failure::invalid("missing parent directory"))?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.flush()?;
    temp.as_file().sync_all()?;
    if let Some(expected) = expected_hash {
        if hash(&read(path)?) != expected {
            return Err(Failure::new(
                "external_file_conflict",
                "destination changed since open/save; save to a new path or reopen",
            ));
        }
        temp.persist(path).map_err(|e| Failure::from(e.error))?;
    } else {
        temp.persist_noclobber(path).map_err(|e| {
            if e.error.kind() == std::io::ErrorKind::AlreadyExists {
                Failure::new("file_exists", "destination exists; choose a new path")
            } else {
                Failure::from(e.error)
            }
        })?;
    }
    #[cfg(unix)]
    File::open(parent)?.sync_all().map_err(|e|Failure::new("outcome_unknown",format!("file was renamed but directory sync failed: {e}; verify destination before retrying")))?;
    Ok(())
}
