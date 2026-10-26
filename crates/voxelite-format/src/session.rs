//! Shared live document authority, bounded file reads, and optimistic atomic saves.
//!
//! Hosts choose their destination policy; snapshots and successful save receipts
//! keep filesystem work off the owner thread without clearing intervening edits.
use crate::{FormatError, Project};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf};
#[cfg(not(target_family = "wasm"))]
use std::{path::Path, sync::Arc};
use voxelite_core::{Engine, Limits};

/// Failure to open or durably save a document.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// Portable project validation failed.
    #[error(transparent)]
    Format(#[from] FormatError),
    /// Filesystem operation failed before a confirmed save.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Opening special files could block or read unbounded data.
    #[error("Open a regular .voxl file")]
    NotRegular,
    /// Input exceeded the portable format limit.
    #[error("Project exceeds the 128 MiB file limit")]
    TooLarge,
    /// A destination must have a filename.
    #[error("Choose a project filename")]
    MissingFilename,
    /// Save destinations cannot be directories or symlinks.
    #[error("Save to a regular file, not a directory or symbolic link")]
    InvalidDestination,
    /// The destination no longer matches the observed baseline.
    #[error("File changed outside Voxelite. Save As to a new file, or reopen it.")]
    ExternalChange,
    /// Create-only saves cannot replace an existing destination.
    #[error("destination exists; choose a new path")]
    FileExists,
    /// Rename succeeded, but durability could not be confirmed.
    #[error(
        "File was written, but directory sync failed: {0}. Verify the saved file before retrying."
    )]
    DirectorySync(std::io::Error),
}
/// Result of a document-session operation.
pub type SessionResult<T> = Result<T, SessionError>;

/// Host rules for saving to a selected destination.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SavePolicy {
    /// App Save As may replace a selected regular file; track only the last path.
    ReplaceSelected,
    /// Agent saves may replace only paths opened or saved by this session.
    CreateOrTracked,
}

/// One live engine and its portable metadata, path, and disk fingerprints.
pub struct DocumentSession {
    /// Shared validated command authority, including revision and dirty state.
    pub engine: Engine,
    /// Most recent successful open/save path; in-memory projects have none.
    pub path: Option<PathBuf>,
    project: Project,
    baselines: BTreeMap<PathBuf, String>,
    #[cfg(not(target_family = "wasm"))]
    identity: Arc<()>,
}
impl DocumentSession {
    /// Restore a project and its build history with host-selected history limits.
    /// An optional existing path establishes the initial disk fingerprint.
    pub fn new(project: Project, path: Option<PathBuf>, limits: Limits) -> SessionResult<Self> {
        #[cfg(not(target_family = "wasm"))]
        let baselines = path
            .as_deref()
            .map(|path| {
                let canonical = path.canonicalize()?;
                let hash = fingerprint(&read(&canonical)?);
                Ok::<_, SessionError>((canonical, hash))
            })
            .transpose()?
            .into_iter()
            .collect();
        #[cfg(target_family = "wasm")]
        let baselines = BTreeMap::new();
        let engine = project.open_engine(limits)?;
        Ok(Self {
            engine,
            path,
            project,
            baselines,
            #[cfg(not(target_family = "wasm"))]
            identity: Arc::new(()),
        })
    }

    /// Keep a content-only placeholder for an externally owned live session.
    /// Attachment polling replaces it with that authority before serving calls.
    pub fn placeholder(project: Project) -> SessionResult<Self> {
        let engine = Engine::new(project.document.clone()).map_err(FormatError::from)?;
        Ok(Self {
            engine,
            project,
            path: None,
            baselines: BTreeMap::new(),
            #[cfg(not(target_family = "wasm"))]
            identity: Arc::new(()),
        })
    }

    /// Open a bounded disk snapshot, fingerprinting exactly the bytes parsed.
    #[cfg(not(target_family = "wasm"))]
    pub fn open(path: &Path, limits: Limits) -> SessionResult<Self> {
        let path = path.canonicalize()?;
        let bytes = read(&path)?;
        let mut session = Self::new(crate::from_bytes(&bytes)?, None, limits)?;
        session.baselines.insert(path.clone(), fingerprint(&bytes));
        session.path = Some(path);
        Ok(session)
    }

    /// Whether live content differs from the last saved engine state.
    pub fn is_dirty(&self) -> bool {
        self.engine.is_dirty()
    }

    /// Whether persistence is needed, including a never-saved agent document.
    pub fn needs_save(&self) -> bool {
        self.is_dirty() || self.baselines.is_empty()
    }

    /// Capture live content while preserving unknown portable metadata.
    pub fn project(&self) -> Project {
        let mut project = self.project.clone();
        project.document = self.engine.document().clone();
        project
    }

    /// Capture an immutable save job without performing filesystem I/O.
    #[cfg(not(target_family = "wasm"))]
    pub fn prepare_save(&self, path: &Path, policy: SavePolicy) -> SaveTask {
        SaveTask {
            project: self.project(),
            path: path.to_path_buf(),
            baselines: self.baselines.clone(),
            snapshot: self.engine.snapshot(),
            identity: self.identity.clone(),
            policy,
        }
    }

    /// Adopt a durable receipt from this session instance; later edits remain dirty.
    #[cfg(not(target_family = "wasm"))]
    pub fn finish_save(&mut self, saved: SavedProject) {
        if !Arc::ptr_eq(&saved.identity, &self.identity)
            || saved.project.document.id != self.engine.document().id
        {
            return;
        }
        self.project = saved.project;
        self.path = Some(saved.path.clone());
        if saved.policy == SavePolicy::ReplaceSelected {
            self.baselines.clear();
        }
        self.baselines.insert(saved.path, saved.fingerprint);
        self.engine.mark_saved_snapshot(saved.snapshot);
    }
}

/// Immutable project snapshot that can be written on a background worker.
#[cfg(not(target_family = "wasm"))]
pub struct SaveTask {
    project: Project,
    path: PathBuf,
    baselines: BTreeMap<PathBuf, String>,
    snapshot: Arc<voxelite_core::Document>,
    identity: Arc<()>,
    policy: SavePolicy,
}
/// Receipt produced only after a successful durable save.
#[cfg(not(target_family = "wasm"))]
pub struct SavedProject {
    project: Project,
    path: PathBuf,
    fingerprint: String,
    bytes: usize,
    snapshot: Arc<voxelite_core::Document>,
    identity: Arc<()>,
    policy: SavePolicy,
}
#[cfg(not(target_family = "wasm"))]
impl SavedProject {
    /// SHA-256 of the exact bytes written, for protocol responses.
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
    /// Number of encoded bytes written.
    pub fn bytes(&self) -> usize {
        self.bytes
    }
}
#[cfg(not(target_family = "wasm"))]
impl SaveTask {
    /// Serialize, stage, recheck the destination, and atomically rename.
    pub fn write(self) -> SessionResult<SavedProject> {
        let path = destination(&self.path)?;
        let expected = if self.policy == SavePolicy::ReplaceSelected {
            let current = match read(&path) {
                Ok(bytes) => Some(fingerprint(&bytes)),
                Err(SessionError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(e),
            };
            if let Some(baseline) = self.baselines.get(&path)
                && current.as_ref() != Some(baseline)
            {
                return Err(SessionError::ExternalChange);
            }
            current
        } else {
            self.baselines.get(&path).cloned()
        };
        let bytes = self.project.to_bytes()?;
        atomic_write(&path, &bytes, expected.as_deref(), self.policy)?;
        Ok(SavedProject {
            project: self.project,
            path,
            fingerprint: fingerprint(&bytes),
            bytes: bytes.len(),
            snapshot: self.snapshot,
            identity: self.identity,
            policy: self.policy,
        })
    }
}

/// SHA-256 fingerprint of an exact byte snapshot, encoded as lowercase hex.
pub fn fingerprint(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Read only a regular file, bounded by the portable project size limit.
#[cfg(not(target_family = "wasm"))]
pub fn read(path: &Path) -> SessionResult<Vec<u8>> {
    use std::{
        fs::{self, File},
        io::Read,
    };
    // Reject devices and FIFOs before opening, which could otherwise block.
    if !fs::metadata(path)?.is_file() {
        return Err(SessionError::NotRegular);
    }
    let file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(SessionError::NotRegular);
    }
    let mut bytes = Vec::new();
    file.take(crate::MAX_FILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > crate::MAX_FILE_BYTES {
        return Err(SessionError::TooLarge);
    }
    Ok(bytes)
}

#[cfg(not(target_family = "wasm"))]
fn destination(path: &Path) -> SessionResult<PathBuf> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path.file_name().ok_or(SessionError::MissingFilename)?;
    let path = parent.canonicalize()?.join(name);
    if let Ok(metadata) = std::fs::symlink_metadata(&path)
        && !metadata.file_type().is_file()
    {
        return Err(SessionError::InvalidDestination);
    }
    Ok(path)
}

/// Persist an exported artifact without replacing an existing file.
#[cfg(not(target_family = "wasm"))]
pub fn write_new(path: &Path, bytes: &[u8]) -> SessionResult<()> {
    atomic_write(path, bytes, None, SavePolicy::CreateOrTracked)
}

#[cfg(not(target_family = "wasm"))]
fn atomic_write(
    path: &Path,
    bytes: &[u8],
    expected: Option<&str>,
    policy: SavePolicy,
) -> SessionResult<()> {
    use std::{
        fs::{self, File},
        io::Write,
    };
    let parent = path.parent().ok_or(SessionError::MissingFilename)?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(bytes)?;
    temporary.flush()?;
    if policy == SavePolicy::ReplaceSelected
        && let Ok(metadata) = fs::metadata(path)
    {
        temporary
            .as_file()
            .set_permissions(metadata.permissions())?;
    }
    temporary.as_file().sync_all()?;
    // Optimistic conflict detection, not a lock against an unrelated process
    // racing the final rename. A new destination uses a no-clobber persist.
    if let Some(expected) = expected {
        destination(path)?;
        let current = if policy == SavePolicy::ReplaceSelected {
            read(path).map_err(|_| SessionError::ExternalChange)?
        } else {
            read(path)?
        };
        if fingerprint(&current) != expected {
            return Err(SessionError::ExternalChange);
        }
        temporary.persist(path).map_err(|e| e.error)?;
    } else {
        temporary.persist_noclobber(path).map_err(|e| {
            if e.error.kind() == std::io::ErrorKind::AlreadyExists {
                SessionError::FileExists
            } else {
                e.error.into()
            }
        })?;
    }
    #[cfg(unix)]
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(SessionError::DirectorySync)?;
    Ok(())
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests;
