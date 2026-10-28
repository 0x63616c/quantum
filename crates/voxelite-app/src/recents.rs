//! Bounded, versioned recent-project history, independent of the window toolkit.
use crate::{editor::EditorResult, local_settings};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

/// Maximum number of projects retained in the history.
pub const CAPACITY: usize = 12;

/// A project that still exists on disk.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentProject {
    /// Canonical file identity, shared by aliases and symbolic links.
    pub path: PathBuf,
    /// Document name captured at the last successful open or save.
    pub name: String,
    /// Current filesystem modification time (refreshed on read).
    pub modified: SystemTime,
}

#[derive(Serialize, Deserialize)]
struct History {
    version: u32,
    projects: Vec<RecentProject>,
}

/// Recent-project storage beside the other application preferences.
#[derive(Clone)]
pub struct Recents {
    path: PathBuf,
}
impl Default for History {
    fn default() -> Self {
        Self {
            version: 1,
            projects: Vec::new(),
        }
    }
}
impl Recents {
    /// Use an explicit settings directory, including isolated test directories.
    pub fn new(directory: &Path) -> Self {
        Self {
            path: directory.join("recents.json"),
        }
    }

    /// Read newest first, tolerating corrupt/unknown versions and dropping missing files.
    pub fn load(&self) -> EditorResult<Vec<RecentProject>> {
        let history = match local_settings::json_store::<History>(&self.path)?.load() {
            Ok(history) => history,
            Err(error) if error.kind() == std::io::ErrorKind::InvalidData => History::default(),
            Err(error) => return Err(error.into()),
        };
        if history.version != 1 {
            return Ok(Vec::new());
        }
        let mut seen = HashSet::new();
        Ok(history
            .projects
            .into_iter()
            .filter_map(|mut project| {
                project.path = project.path.canonicalize().ok()?;
                let metadata = project.path.metadata().ok()?;
                if !metadata.is_file() || !seen.insert(project.path.clone()) {
                    return None;
                }
                project.modified = metadata.modified().ok()?;
                Some(project)
            })
            .take(CAPACITY)
            .collect())
    }

    /// Move a successfully opened/saved file to the front, deduplicating its canonical path.
    /// The desktop controller serializes mutations from its windows.
    pub fn record(&self, path: &Path, name: &str) -> EditorResult<Vec<RecentProject>> {
        let path = path.canonicalize()?;
        let metadata = path.metadata()?;
        if !metadata.is_file() {
            return Err(crate::editor::EditorError::Gesture(
                "Project is not a file".into(),
            ));
        }
        let mut projects = self.load()?;
        projects.retain(|project| project.path != path);
        projects.insert(
            0,
            RecentProject {
                path,
                name: name.to_owned(),
                modified: metadata.modified()?,
            },
        );
        projects.truncate(CAPACITY);
        self.write(&projects)?;
        Ok(projects)
    }

    /// Clear only the history; project files are never removed.
    pub fn clear(&self) -> EditorResult<()> {
        self.write(&[])
    }

    fn write(&self, projects: &[RecentProject]) -> EditorResult<()> {
        Ok(
            local_settings::json_store::<History>(&self.path)?.save(&History {
                version: 1,
                projects: projects.to_vec(),
            })?,
        )
    }
}

/// Abbreviate only a complete home-directory prefix, preserving other paths.
pub fn display_path(path: &Path, home: Option<&Path>) -> String {
    if let Some(home) = home.filter(|home| !home.as_os_str().is_empty())
        && let Ok(relative) = path.strip_prefix(home)
    {
        return if relative.as_os_str().is_empty() {
            "~".into()
        } else {
            format!("~/{}", relative.display())
        };
    }
    path.display().to_string()
}

/// A compact relative file-modification label, including future clock skew.
pub fn modified_label(modified: SystemTime, now: SystemTime) -> String {
    let seconds = now
        .duration_since(modified)
        .unwrap_or(Duration::ZERO)
        .as_secs();
    match seconds {
        0..60 => "Just now".into(),
        60..3600 => format!("{}m ago", seconds / 60),
        3600..86400 => format!("{}h ago", seconds / 3600),
        _ => format!("{}d ago", seconds / 86400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn history_survives_restart_deduplicates_and_moves_to_front() {
        let dir = tempfile::tempdir().unwrap();
        let store = Recents::new(dir.path());
        let a = dir.path().join("a.voxl");
        let b = dir.path().join("b.voxl");
        fs::write(&a, b"a").unwrap();
        fs::write(&b, b"b").unwrap();
        store.record(&a, "A").unwrap();
        store.record(&b, "B").unwrap();
        store
            .record(&dir.path().join("./a.voxl"), "Renamed")
            .unwrap();
        let entries = Recents::new(dir.path()).load().unwrap();
        assert_eq!(
            entries.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
            ["Renamed", "B"]
        );
        assert_eq!(entries[0].path, a.canonicalize().unwrap());
        assert_eq!(
            entries[0].modified,
            a.metadata().unwrap().modified().unwrap()
        );
        store.clear().unwrap();
        assert!(store.load().unwrap().is_empty());
        assert!(a.exists() && b.exists());
    }

    #[test]
    #[cfg(unix)]
    fn symlink_aliases_share_one_identity() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("project.voxl");
        let alias = dir.path().join("alias.voxl");
        fs::write(&path, b"project").unwrap();
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        let store = Recents::new(dir.path());
        store.record(&path, "Project").unwrap();
        store.record(&alias, "Alias").unwrap();
        assert_eq!(store.load().unwrap().len(), 1);
        assert_eq!(store.load().unwrap()[0].path, path.canonicalize().unwrap());
    }

    #[test]
    fn caps_history_and_drops_deleted_files_and_directories() {
        let dir = tempfile::tempdir().unwrap();
        let store = Recents::new(dir.path());
        for i in 0..20 {
            let path = dir.path().join(format!("{i}.voxl"));
            fs::write(&path, b"project").unwrap();
            store.record(&path, &i.to_string()).unwrap();
        }
        let entries = store.load().unwrap();
        assert_eq!(entries.len(), CAPACITY);
        assert_eq!(entries[0].name, "19");
        assert_eq!(entries[11].name, "8");
        fs::remove_file(&entries[0].path).unwrap();
        fs::remove_file(&entries[1].path).unwrap();
        fs::create_dir(&entries[1].path).unwrap();
        assert_eq!(store.load().unwrap().len(), CAPACITY - 2);
        assert!(
            store
                .record(&dir.path().join("missing"), "Missing")
                .is_err()
        );
        assert!(store.record(dir.path(), "Directory").is_err());
    }

    #[test]
    fn corrupt_and_unknown_history_recovers_on_next_record() {
        let dir = tempfile::tempdir().unwrap();
        let store = Recents::new(dir.path());
        assert!(store.load().unwrap().is_empty());
        let project = dir.path().join("p.voxl");
        fs::write(&project, b"project").unwrap();
        for bytes in [b"{broken".as_slice(), br#"{"version":99,"projects":[]}"#] {
            fs::write(&store.path, bytes).unwrap();
            assert!(store.load().unwrap().is_empty());
            store.record(&project, "Recovered").unwrap();
            assert_eq!(store.load().unwrap()[0].name, "Recovered");
        }
    }

    #[test]
    fn failed_atomic_replacement_keeps_previous_history() {
        let dir = tempfile::tempdir().unwrap();
        let store = Recents::new(dir.path());
        fs::create_dir(&store.path).unwrap();
        assert!(store.clear().is_err());
        assert!(store.path.is_dir());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn presentation_handles_home_boundaries_and_time_skew() {
        let home = Some(Path::new("/home/calum"));
        assert_eq!(
            display_path(Path::new("/home/calum/project.voxl"), home),
            "~/project.voxl"
        );
        assert_eq!(
            display_path(Path::new("/home/calum-other/project.voxl"), home),
            "/home/calum-other/project.voxl"
        );
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100_000);
        for (seconds, label) in [
            (0, "Just now"),
            (60, "1m ago"),
            (3600, "1h ago"),
            (86400, "1d ago"),
        ] {
            assert_eq!(
                modified_label(now - Duration::from_secs(seconds), now),
                label
            );
        }
        assert_eq!(
            modified_label(now + Duration::from_secs(100), now),
            "Just now"
        );
    }
}
