//! MCP root-scoped path policy; persistence is shared with the editor.
use crate::error::{Failure, Result};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

pub(crate) use quantum_format::session::fingerprint as hash;

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
/// Exports are create-only, using the shared atomic persistence contract.
pub(crate) fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    quantum_format::session::write_new(path, bytes).map_err(Into::into)
}
