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
            return Err(crate::editor::EditorError::ProjectNotFile);
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
        Ok(
            local_settings::json_store::<History>(&self.path)?.save(&History {
                version: 1,
                projects,
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

/// Display the containing folder, abbreviating only the actual home directory.
pub fn folder_label(path: &Path, home: Option<&Path>) -> String {
    display_path(path.parent().unwrap_or(path), home)
}

/// Calendar-style modification time in the user's local timezone.
pub fn modified_label(modified: SystemTime, now: SystemTime) -> String {
    use chrono::TimeZone;
    modified_label_with_offset(modified, now, |utc| {
        chrono::Local.offset_from_utc_datetime(&utc.naive_utc())
    })
}

// Resolve each instant's offset separately: applying today's offset to an old
// mtime gets calendar boundaries wrong across daylight-saving transitions.
fn modified_label_with_offset(
    modified: SystemTime,
    now: SystemTime,
    offset: impl Fn(chrono::DateTime<chrono::Utc>) -> chrono::FixedOffset,
) -> String {
    let utc = |time: SystemTime| {
        let nanos = match time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => i128::try_from(duration.as_nanos()).ok()?,
            Err(error) => -i128::try_from(error.duration().as_nanos()).ok()?,
        };
        chrono::DateTime::from_timestamp(
            i64::try_from(nanos.div_euclid(1_000_000_000)).ok()?,
            nanos.rem_euclid(1_000_000_000) as u32,
        )
    };
    let label = || {
        let modified = utc(modified)?;
        let now = utc(now)?;
        // A UTC instant can be representable while its local date overflows.
        // Never call DateTime::date_naive/format before checking that boundary.
        let modified_date = modified
            .naive_utc()
            .checked_add_offset(offset(modified))?
            .date();
        let now_date = now.naive_utc().checked_add_offset(offset(now))?.date();
        let seconds = now.signed_duration_since(modified).num_seconds().max(0);
        if now < modified {
            return Some("Just now".into());
        }
        let days = now_date.signed_duration_since(modified_date).num_days();
        if days == 1 {
            return Some("Yesterday".into());
        }
        if days == 0 {
            if seconds < 60 {
                return Some("Just now".into());
            }
            let (count, unit) = if seconds < 3600 {
                (seconds / 60, "minute")
            } else {
                (seconds / 3600, "hour")
            };
            return Some(format!(
                "{count} {unit}{} ago",
                if count == 1 { "" } else { "s" }
            ));
        }
        use chrono::Datelike;
        Some(
            modified_date
                .format(if modified_date.year() == now_date.year() {
                    "%b %-d"
                } else {
                    "%b %-d, %Y"
                })
                .to_string(),
        )
    };
    label().unwrap_or_else(|| "Unknown date".into())
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

    fn instant(value: &str) -> SystemTime {
        chrono::DateTime::parse_from_rfc3339(value).unwrap().into()
    }

    #[test]
    fn system_timezone_handles_local_midnight_and_calendar_overflow() {
        use chrono::{DateTime, Local, TimeZone, Utc};
        use std::time::Duration;

        // Exercise the public formatter in each TZ test run, without mutating
        // the process environment while other tests are running.
        let midnight: SystemTime = Local
            .with_ymd_and_hms(2026, 10, 27, 0, 5, 0)
            .single()
            .unwrap()
            .into();
        assert_eq!(
            modified_label(midnight - Duration::from_secs(600), midnight),
            "Yesterday"
        );
        let midnight = midnight - Duration::from_secs(5 * 60 - 15);
        assert_eq!(
            modified_label(midnight - Duration::from_secs(30), midnight),
            "Yesterday"
        );
        for boundary in [DateTime::<Utc>::MIN_UTC, DateTime::<Utc>::MAX_UTC] {
            let label = modified_label(boundary.into(), midnight);
            let reverse = modified_label(midnight, boundary.into());
            let offset = Local.offset_from_utc_datetime(&boundary.naive_utc());
            if (boundary == DateTime::<Utc>::MAX_UTC && offset.local_minus_utc() > 0)
                || (boundary == DateTime::<Utc>::MIN_UTC && offset.local_minus_utc() < 0)
            {
                assert_eq!(label, "Unknown date");
                assert_eq!(reverse, "Unknown date");
            }
        }
    }

    #[test]
    fn system_timezone_buckets_calendar_days_across_both_dst_transitions() {
        use chrono::{Local, TimeZone};
        use std::time::Duration;

        // These are the 2026 transition days in Los Angeles and Auckland.
        // Run the public Local-based formatter under each TZ without changing
        // environment variables inside the multithreaded test process.
        for (month, day) in [(3, 8), (11, 1), (4, 5), (9, 27)] {
            let date = chrono::NaiveDate::from_ymd_opt(2026, month, day).unwrap();
            let local = |days, hour, minute| -> SystemTime {
                let date = date + chrono::Duration::days(days);
                Local
                    .from_local_datetime(&date.and_hms_opt(hour, minute, 0).unwrap())
                    .single()
                    .unwrap()
                    .into()
            };
            let now = local(0, 4, 0);
            assert_eq!(modified_label(local(-1, 23, 30), now), "Yesterday");
            assert_eq!(
                modified_label(now - Duration::from_secs(7200), now),
                "2 hours ago"
            );
            // The previous calendar day can be more than 24 elapsed hours away.
            assert_eq!(modified_label(local(0, 0, 30), local(1, 0, 5)), "Yesterday");
        }
        let now: SystemTime = Local
            .with_ymd_and_hms(2026, 10, 27, 12, 0, 0)
            .single()
            .unwrap()
            .into();
        let modified: SystemTime = Local
            .with_ymd_and_hms(2026, 10, 25, 23, 59, 0)
            .single()
            .unwrap()
            .into();
        assert_eq!(modified_label(modified, now), "Oct 25");
    }

    fn label(modified: SystemTime, now: SystemTime, east: i32) -> String {
        modified_label_with_offset(modified, now, |_| {
            chrono::FixedOffset::east_opt(east).unwrap()
        })
    }

    #[test]
    fn calendar_times_handle_midnight_years_and_clock_skew_without_machine_timezone() {
        use std::time::Duration;
        let now = instant("2026-10-27T12:00:00Z");
        for (seconds, expected) in [
            (0, "Just now"),
            (60, "1 minute ago"),
            (120, "2 minutes ago"),
            (3600, "1 hour ago"),
            (7200, "2 hours ago"),
            (86400, "Yesterday"),
            (3 * 86400, "Oct 24"),
        ] {
            assert_eq!(label(now - Duration::from_secs(seconds), now, 0), expected);
        }
        for seconds in [1, 3600, 86400, 365 * 86400] {
            assert_eq!(
                label(now + Duration::from_secs(seconds), now, 0),
                "Just now"
            );
        }
        assert_eq!(
            label(instant("2025-10-27T12:00:00Z"), now, 0),
            "Oct 27, 2025"
        );
        for offset in [-12 * 3600, -7 * 3600, 0, 5 * 3600 + 1800, 13 * 3600] {
            // The same local midnight in each injected timezone.
            let midnight = instant("2026-10-27T00:05:00Z");
            let midnight = if offset >= 0 {
                midnight - Duration::from_secs(offset as u64)
            } else {
                midnight + Duration::from_secs((-offset) as u64)
            };
            assert_eq!(
                label(midnight - Duration::from_secs(600), midnight, offset),
                "Yesterday"
            );
        }
        let modified = instant("2026-10-26T23:30:00Z");
        let now = instant("2026-10-27T01:30:00Z");
        assert_eq!(label(modified, now, 0), "Yesterday");
        assert_eq!(label(modified, now, 13 * 3600), "2 hours ago");
        assert_eq!(label(modified, now, -7 * 3600), "2 hours ago");
    }

    #[test]
    fn daylight_saving_uses_each_instants_offset_and_elapsed_utc_time() {
        for (modified, now, transition, before, after, expected) in [
            (
                "2026-03-08T09:30:00Z",
                "2026-03-08T10:30:00Z",
                "2026-03-08T10:00:00Z",
                -8,
                -7,
                "1 hour ago",
            ),
            (
                "2026-11-01T08:30:00Z",
                "2026-11-01T09:30:00Z",
                "2026-11-01T09:00:00Z",
                -7,
                -8,
                "1 hour ago",
            ),
            // Applying the new offset to the old instant would change its date.
            (
                "2026-03-08T07:30:00Z",
                "2026-03-08T11:00:00Z",
                "2026-03-08T10:00:00Z",
                -8,
                -7,
                "Yesterday",
            ),
            (
                "2026-11-01T07:30:00Z",
                "2026-11-01T11:00:00Z",
                "2026-11-01T09:00:00Z",
                -7,
                -8,
                "3 hours ago",
            ),
            (
                "2026-09-26T11:30:00Z",
                "2026-09-26T15:00:00Z",
                "2026-09-26T14:00:00Z",
                12,
                13,
                "Yesterday",
            ),
        ] {
            let transition = instant(transition);
            assert_eq!(
                modified_label_with_offset(instant(modified), instant(now), |utc| {
                    let east = if SystemTime::from(utc) < transition {
                        before
                    } else {
                        after
                    };
                    chrono::FixedOffset::east_opt(east * 3600).unwrap()
                }),
                expected
            );
        }
    }

    #[test]
    fn pre_epoch_and_out_of_calendar_range_timestamps_do_not_panic() {
        use chrono::{DateTime, Utc};
        use std::time::Duration;
        let epoch = SystemTime::UNIX_EPOCH;
        assert_eq!(
            label(
                epoch - Duration::from_nanos(1),
                epoch + Duration::from_secs(120),
                0
            ),
            "Yesterday"
        );
        assert_eq!(
            label(
                epoch - Duration::from_secs(86400),
                instant("2026-10-27T12:00:00Z"),
                0
            ),
            "Dec 31, 1969"
        );
        for (time, offset) in [
            (DateTime::<Utc>::MAX_UTC, 3600),
            (DateTime::<Utc>::MIN_UTC, -3600),
        ] {
            assert_eq!(label(time.into(), epoch, offset), "Unknown date");
            assert_eq!(label(epoch, time.into(), offset), "Unknown date");
        }
        for time in [
            epoch.checked_add(Duration::from_secs(i64::MAX as u64)),
            epoch.checked_sub(Duration::from_secs(i64::MAX as u64)),
        ]
        .into_iter()
        .flatten()
        {
            assert_eq!(label(time, epoch, 0), "Unknown date");
            assert_eq!(label(epoch, time, 0), "Unknown date");
        }
    }
}
