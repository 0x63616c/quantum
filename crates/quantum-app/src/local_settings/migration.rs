//! One-time directory migration. Publish only complete copies, without replacing
//! an existing destination, and leave the source intact on any failed operation.
use std::{fs, io, path::Path};

pub(super) fn initialize(home: &Path, xdg: Option<&Path>, macos: bool) -> io::Result<()> {
    let new = super::settings_dir(home, xdg, macos);
    let old = new.with_file_name("voxelite");
    migrate(&old, &new)?;
    // Older macOS builds used the platform-independent location.
    if macos {
        let old = super::settings_dir(home, xdg, false).with_file_name("voxelite");
        migrate(&old, &new)?;
    }
    Ok(())
}

pub(super) fn exists(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

pub(super) fn migrate(old: &Path, new: &Path) -> io::Result<()> {
    migrate_with(old, new, rename_new, copy_directory)
}

fn migrate_with(
    old: &Path,
    new: &Path,
    rename: impl FnOnce(&Path, &Path) -> io::Result<()>,
    copy: impl FnOnce(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
    if exists(new)? || !exists(old)? {
        return Ok(());
    }
    if !fs::symlink_metadata(old)?.is_dir() {
        return Err(io::Error::other(
            "Previous settings path is not a directory",
        ));
    }
    let parent = new
        .parent()
        .ok_or_else(|| io::Error::other("Missing data directory parent"))?;
    fs::create_dir_all(parent)?;
    match rename(old, new) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => return Ok(()),
        Err(error) if error.kind() == io::ErrorKind::CrossesDevices => {}
        Err(error) => return Err(error),
    }
    // Stage on the destination filesystem. Failed copies never leave a partial
    // destination that could suppress a later migration attempt.
    let staging = tempfile::Builder::new()
        .prefix(".quantum-migration-")
        .tempdir_in(parent)?;
    copy(old, staging.path())?;
    match rename_new(staging.path(), new) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error),
    }
}

fn rename_new(old: &Path, new: &Path) -> io::Result<()> {
    #[cfg(any(target_os = "linux", target_vendor = "apple"))]
    {
        use rustix::fs::{CWD, RenameFlags, renameat_with};
        renameat_with(CWD, old, CWD, new, RenameFlags::NOREPLACE).map_err(Into::into)
    }
    #[cfg(not(any(target_os = "linux", target_vendor = "apple")))]
    {
        if exists(new)? {
            return Err(io::ErrorKind::AlreadyExists.into());
        }
        fs::rename(old, new)
    }
}

fn copy_directory(old: &Path, new: &Path) -> io::Result<()> {
    for entry in fs::read_dir(old)? {
        let entry = entry?;
        let source = entry.path();
        let destination = new.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            fs::create_dir(&destination)?;
            copy_directory(&source, &destination)?;
        } else if kind.is_file() {
            fs::copy(&source, &destination)?;
        } else {
            copy_special(&source, &destination, kind)?;
        }
    }
    fs::set_permissions(new, fs::metadata(old)?.permissions())
}

fn copy_special(old: &Path, new: &Path, kind: fs::FileType) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{FileTypeExt, symlink};
        if kind.is_symlink() {
            return symlink(fs::read_link(old)?, new);
        }
        if kind.is_socket()
            && old
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("session-") && n.ends_with(".sock"))
        {
            // Sockets are process endpoints, never durable settings. New sessions
            // use unique names, so moved sockets cannot prevent a new bind either.
            return Ok(());
        }
    }
    let _ = (new, kind);
    Err(io::Error::other(format!(
        "Cannot migrate settings entry {}",
        old.display()
    )))
}

#[cfg(test)]
mod tests;
