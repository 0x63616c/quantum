//! Atomic local preferences; project persistence lives in voxelite-format.
use crate::editor::EditorResult;
use std::{fs, path::Path};

/// Replace local bytes with the shared durable writer.
pub(crate) fn write_local(path: &Path, bytes: &[u8]) -> EditorResult<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    Ok(voxelite_format::storage::safe_write(path, bytes)?)
}
