//! Bounded, versioned recent-project history, independent of the window toolkit.
use crate::{editor::EditorResult, local_settings};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::SystemTime,
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
impl Recents {
    /// Use an explicit settings directory, including isolated test directories.
    pub fn new(directory: &Path) -> Self {
        Self {
            path: directory.join("recents.json"),
        }
    }

    /// Read newest first, tolerating corrupt/unknown versions and dropping missing files.
    pub fn load(&self) -> EditorResult<Vec<RecentProject>> {
        let Some(bytes) = local_settings::read_local(&self.path)? else {
            return Ok(Vec::new());
        };
        let Ok(history) = serde_json::from_slice::<History>(&bytes) else {
            return Ok(Vec::new());
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
        let mut projects = projects.to_vec();
        // The v1 timestamp encoding is unsigned. Modification times are always
        // refreshed from disk on load, so persist an epoch placeholder for old
        // files instead of failing to record them or changing the file format.
        for project in &mut projects {
            if project.modified < SystemTime::UNIX_EPOCH {
                project.modified = SystemTime::UNIX_EPOCH;
            }
        }
        let bytes = serde_json::to_vec_pretty(&History {
            version: 1,
            projects,
        })
        .map_err(|error| crate::editor::EditorError::Gesture(error.to_string()))?;
        local_settings::write_local(&self.path, &bytes)
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

/// Display the containing folder, abbreviating only the actual home directory.
pub fn folder_label(path: &Path, home: Option<&Path>) -> String {
    display_path(path.parent().unwrap_or(path), home)
}

/// Calendar-style modification time in the user's local timezone.
pub fn modified_label(modified: SystemTime, now: SystemTime) -> String {
    let local = |time: SystemTime| {
        let nanos = match time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => i128::try_from(duration.as_nanos()).ok()?,
            Err(error) => -i128::try_from(error.duration().as_nanos()).ok()?,
        };
        chrono::DateTime::from_timestamp(
            i64::try_from(nanos.div_euclid(1_000_000_000)).ok()?,
            nanos.rem_euclid(1_000_000_000) as u32,
        )
        .map(|date| date.with_timezone(&chrono::Local))
    };
    match (local(modified), local(now)) {
        (Some(modified), Some(now)) => calendar_label(modified, now),
        _ => "Unknown date".into(),
    }
}

fn calendar_label<T: chrono::TimeZone>(
    modified: chrono::DateTime<T>,
    now: chrono::DateTime<T>,
) -> String
where
    T::Offset: std::fmt::Display,
{
    let seconds = (now.timestamp() - modified.timestamp()).max(0);
    if seconds < 60 {
        return "Just now".into();
    }
    let days = now
        .date_naive()
        .signed_duration_since(modified.date_naive())
        .num_days();
    if days == 1 {
        return "Yesterday".into();
    }
    if days == 0 {
        let (count, unit) = if seconds < 3600 {
            (seconds / 60, "minute")
        } else {
            (seconds / 3600, "hour")
        };
        return format!("{count} {unit}{} ago", if count == 1 { "" } else { "s" });
    }
    use chrono::Datelike;
    modified
        .format(if modified.year() == now.year() {
            "%b %-d"
        } else {
            "%b %-d, %Y"
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pre_epoch_files_round_trip_without_changing_the_v1_encoding() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("old.voxl");
        std::fs::write(&file, b"project").unwrap();
        let modified = SystemTime::UNIX_EPOCH - std::time::Duration::from_secs(86400);
        std::fs::File::options()
            .write(true)
            .open(&file)
            .unwrap()
            .set_modified(modified)
            .unwrap();
        let recents = Recents::new(dir.path());
        assert_eq!(
            recents.record(&file, "Old project").unwrap()[0].modified,
            modified
        );
        assert_eq!(recents.load().unwrap()[0].modified, modified);
    }
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
        assert_eq!(
            folder_label(Path::new("/home/calum/Quantum/a.voxl"), home),
            "~/Quantum"
        );
        assert_eq!(folder_label(Path::new("/home/calum/a.voxl"), home), "~");
        assert_eq!(
            folder_label(Path::new("/home/calum-other/a.voxl"), home),
            "/home/calum-other"
        );
    }

    #[test]
    fn calendar_times_handle_midnight_years_and_clock_skew() {
        use chrono::{Duration, TimeZone, Utc};
        let now = Utc.with_ymd_and_hms(2026, 10, 27, 12, 0, 0).unwrap();
        for (seconds, label) in [
            (0, "Just now"),
            (60, "1 minute ago"),
            (120, "2 minutes ago"),
            (3600, "1 hour ago"),
            (7200, "2 hours ago"),
            (86400, "Yesterday"),
            (3 * 86400, "Oct 24"),
        ] {
            assert_eq!(calendar_label(now - Duration::seconds(seconds), now), label);
        }
        assert_eq!(calendar_label(now + Duration::hours(1), now), "Just now");
        let midnight = Utc.with_ymd_and_hms(2026, 10, 27, 0, 5, 0).unwrap();
        assert_eq!(
            calendar_label(midnight - Duration::minutes(10), midnight),
            "Yesterday"
        );
        assert_eq!(
            calendar_label(Utc.with_ymd_and_hms(2025, 10, 27, 12, 0, 0).unwrap(), now),
            "Oct 27, 2025"
        );
    }

    #[test]
    fn out_of_calendar_range_timestamps_do_not_panic() {
        if let Some(time) =
            SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(i64::MAX as u64))
        {
            assert_eq!(modified_label(time, SystemTime::UNIX_EPOCH), "Unknown date");
        }
        assert!(
            !modified_label(
                SystemTime::UNIX_EPOCH - std::time::Duration::from_secs(86400),
                SystemTime::now()
            )
            .is_empty()
        );
    }
}
