//! Atomic local preferences; project persistence lives in voxelite-format.
use crate::editor::{EditorError, EditorResult};
use std::{fs, path::Path};

/// Replace app preferences atomically, using the same durable staging as project saves.
pub(crate) fn write_local(path: &Path, bytes: &[u8]) -> EditorResult<()> {
    use std::io::Write;
    let parent = path
        .parent()
        .ok_or_else(|| EditorError::Gesture("Invalid settings path".into()))?;
    fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|e| e.error)?;
    Ok(())
}
