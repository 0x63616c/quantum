//! One-time migration of macOS data whose namespace is the bundle identifier.
//! Defaults go through the preferences service, never a raw plist copy. New
//! keys and existing destinations win; a failed migration is safe to retry.
use std::{collections::BTreeMap, io, path::Path};

const LEGACY_ID: &str = "co.worldwidewebb.voxelite";
const MARKER: &str = ".bundle-migration-v1";
const DIRECTORIES: &[(&str, &str)] = &[
    ("Saved Application State", ".savedState"),
    ("Caches", ""),
    ("HTTPStorages", ""),
    ("WebKit", ""),
    ("Application Support", ""),
];

/// Preferences service abstraction; values retain their native property-list types.
pub trait Defaults {
    /// A native property-list value, or a test representation.
    type Value;
    /// Read only the persistent domain, excluding global/registered defaults.
    fn read(&mut self, domain: &str) -> io::Result<BTreeMap<String, Self::Value>>;
    /// Persist the merged domain before the completion marker is written.
    fn write(&mut self, domain: &str, values: BTreeMap<String, Self::Value>) -> io::Result<()>;
}

trait Storage {
    fn complete(&self) -> io::Result<bool>;
    fn migrate(&mut self, directory: &str, suffix: &str) -> io::Result<()>;
    fn copy_cookies(&mut self, directory: &str) -> io::Result<()>;
    fn finish(&mut self) -> io::Result<()>;
}

struct Files<'a>(&'a Path);
impl Storage for Files<'_> {
    fn complete(&self) -> io::Result<bool> {
        super::migration::exists(
            &self
                .0
                .join("Library/Application Support/quantum")
                .join(MARKER),
        )
    }

    fn migrate(&mut self, directory: &str, suffix: &str) -> io::Result<()> {
        let parent = self.0.join("Library").join(directory);
        let old = parent.join(format!("{LEGACY_ID}{suffix}"));
        let new = parent.join(format!("{}{suffix}", crate::APP_ID));
        super::migration::migrate(&old, &new)
    }

    fn copy_cookies(&mut self, directory: &str) -> io::Result<()> {
        let parent = self.0.join("Library").join(directory);
        migrate_file(
            &parent.join(format!("{LEGACY_ID}.binarycookies")),
            &parent.join(format!("{}.binarycookies", crate::APP_ID)),
        )
    }

    fn finish(&mut self) -> io::Result<()> {
        let parent = self.0.join("Library/Application Support/quantum");
        std::fs::create_dir_all(&parent)?;
        let marker = tempfile::NamedTempFile::new_in(parent)?;
        marker.as_file().sync_all()?;
        match marker.persist_noclobber(
            self.0
                .join("Library/Application Support/quantum")
                .join(MARKER),
        ) {
            Ok(_) => Ok(()),
            Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
            Err(error) => Err(error.error),
        }
    }
}

fn migrate_file(old: &Path, new: &Path) -> io::Result<()> {
    if super::migration::exists(new)? || !super::migration::exists(old)? {
        return Ok(());
    }
    if !std::fs::symlink_metadata(old)?.is_file() {
        return Err(io::Error::other(
            "Previous cookie store is not a regular file",
        ));
    }
    let parent = new
        .parent()
        .ok_or_else(|| io::Error::other("Missing store parent"))?;
    let staging = tempfile::NamedTempFile::new_in(parent)?;
    std::fs::copy(old, staging.path())?;
    staging.as_file().sync_all()?;
    match staging.persist_noclobber(new) {
        Ok(_) => Ok(()),
        Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error.error),
    }
}

/// Run before GPUI or Sparkle initializes, after the Application Support migration.
/// The caller suppresses this for explicit data-directory overrides.
pub fn initialize(home: &Path, defaults: &mut impl Defaults) -> io::Result<()> {
    migrate(&mut Files(home), defaults)
}

fn migrate(storage: &mut impl Storage, defaults: &mut impl Defaults) -> io::Result<()> {
    if storage.complete()? {
        return Ok(());
    }
    let old = defaults.read(LEGACY_ID)?;
    let mut new = defaults.read(crate::APP_ID)?;
    for (key, value) in old {
        new.entry(key).or_insert(value);
    }
    defaults.write(crate::APP_ID, new)?;
    for (directory, suffix) in DIRECTORIES {
        storage.migrate(directory, suffix)?;
    }
    for directory in ["HTTPStorages", "Cookies"] {
        storage.copy_cookies(directory)?;
    }
    storage.finish()
}

#[cfg(test)]
mod tests;
