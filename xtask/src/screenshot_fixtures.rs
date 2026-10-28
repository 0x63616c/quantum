//! Disposable project fixtures shared by both revisions of a screenshot comparison.

use std::{
    fs::{self, File, FileTimes},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use serde::Serialize;

use crate::{Error, Result};

#[derive(Serialize)]
struct Project {
    name: &'static str,
    path: PathBuf,
    modified: SystemTime,
}

#[derive(Serialize)]
struct History {
    version: u32,
    projects: Vec<Project>,
}

/// Seed five real projects beneath an explicit disposable home, never the user's home.
///
/// Reuse the stored clock when preparing another revision so file bytes, paths and
/// modification times agree on both sides. `data/recents.json` can be copied into
/// each screen's fresh application data directory without sharing preferences.
pub fn prepare_welcome(root: &Path, capture_home: &Path, data: &Path) -> Result<()> {
    io(capture_home, fs::create_dir_all(capture_home))?;
    let capture_home = io(capture_home, capture_home.canonicalize())?;
    let clock = capture_home.join("clock.txt");
    let seconds = match fs::read_to_string(&clock) {
        Ok(value) => value.parse::<u64>().map_err(|error| {
            Error::Invalid(format!(
                "invalid capture clock {}: {error}",
                clock.display()
            ))
        })?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let seconds = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|error| Error::Invalid(format!("capture clock: {error}")))?
                .as_secs();
            io(&clock, fs::write(&clock, seconds.to_string()))?;
            seconds
        }
        Err(source) => {
            return Err(Error::Io {
                path: clock,
                source,
            });
        }
    };
    let mut projects = Vec::new();
    for (filename, name, folder, age) in [
        ("autumn-workshop", "Autumn Workshop", "Quantum", 2 * 3600),
        ("snow-cabin", "Snow Cabin", "Quantum/client", 86400),
        ("cherry-blossom", "Cherry Blossom", "Desktop", 3 * 86400),
        ("lighthouse", "Lighthouse", "Quantum", 5 * 86400),
        (
            "autumn-workshop",
            "Workshop Study",
            "Quantum/samples",
            8 * 86400,
        ),
    ] {
        let folder = capture_home.join(folder);
        io(&folder, fs::create_dir_all(&folder))?;
        let path = folder.join(format!("{name}.voxl"));
        let source = root.join("examples").join(format!("{filename}.voxl"));
        let bytes = io(&source, fs::read(&source))?;
        io(&path, fs::write(&path, bytes))?;
        let modified = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(seconds.checked_sub(age).ok_or_else(
                || Error::Invalid("capture clock predates the fixture history".into()),
            )?))
            .ok_or_else(|| Error::Invalid("capture clock is out of range".into()))?;
        let file = io(&path, File::options().write(true).open(&path))?;
        io(
            &path,
            file.set_times(
                FileTimes::new()
                    .set_accessed(modified)
                    .set_modified(modified),
            ),
        )?;
        projects.push(Project {
            name,
            path,
            modified,
        });
    }
    io(data, fs::create_dir_all(data))?;
    let path = data.join("recents.json");
    let bytes = serde_json::to_vec(&History {
        version: 1,
        projects,
    })
    .map_err(|source| Error::Json {
        context: path.display().to_string(),
        source,
    })?;
    io(&path, fs::write(&path, bytes))
}

fn io<T>(path: &Path, result: std::io::Result<T>) -> Result<T> {
    result.map_err(|source| Error::Io {
        path: path.into(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_revisions_use_real_projects_under_the_same_isolated_home() {
        let root = crate::workspace::Workspace::discover().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let capture_home = directory.path().join("home");
        let mut histories = Vec::new();
        for side in ["before", "after"] {
            let data = directory.path().join(side);
            prepare_welcome(root.root(), &capture_home, &data).unwrap();
            let bytes = fs::read(data.join("recents.json")).unwrap();
            let history: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(history["version"], 1);
            let projects = history["projects"].as_array().unwrap();
            assert_eq!(projects.len(), 5);
            let mut folders = Vec::new();
            for project in projects {
                let path = Path::new(project["path"].as_str().unwrap());
                folders.push(
                    path.parent()
                        .unwrap()
                        .strip_prefix(capture_home.canonicalize().unwrap())
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned(),
                );
                let modified: SystemTime =
                    serde_json::from_value(project["modified"].clone()).unwrap();
                assert_eq!(path.metadata().unwrap().modified().unwrap(), modified);
                let name = project["name"].as_str().unwrap();
                let name = if name == "Workshop Study" {
                    "Autumn Workshop"
                } else {
                    name
                };
                let source = root.path(format!(
                    "examples/{}.voxl",
                    name.to_lowercase().replace(' ', "-")
                ));
                assert_eq!(fs::read(path).unwrap(), fs::read(source).unwrap());
            }
            assert_eq!(
                folders,
                [
                    "Quantum",
                    "Quantum/client",
                    "Desktop",
                    "Quantum",
                    "Quantum/samples"
                ]
            );
            histories.push(bytes);
        }
        assert_eq!(histories[0], histories[1]);
    }
}
