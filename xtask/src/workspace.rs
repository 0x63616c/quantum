//! Repository paths, tracked filenames and Cargo package metadata.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{
    Error, Result,
    cmd::{Command, Runner},
};

/// A checked repository root, independent of the caller's working directory.
#[derive(Clone, Debug)]
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// Locate the repository containing this xtask's manifest.
    pub fn discover() -> Result<Self> {
        Self::at(Path::new(env!("CARGO_MANIFEST_DIR")).join(".."))
    }

    /// Check an explicit root, useful for temporary repositories in tests.
    pub fn at(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let root = root.canonicalize().map_err(|source| Error::Io {
            path: root.into(),
            source,
        })?;
        let workspace = Self { root };
        if !workspace.path("xtask/Cargo.toml").is_file()
            || !workspace
                .read("Cargo.toml")?
                .lines()
                .any(|line| line.split('#').next().unwrap_or_default().trim() == "[workspace]")
        {
            return Err(Error::Invalid(format!(
                "{} is not a repository root (expected [workspace] in Cargo.toml and xtask/Cargo.toml)",
                workspace.root.display()
            )));
        }
        Ok(workspace)
    }

    /// Absolute repository root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolve a repository-relative path.
    pub fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root.join(relative)
    }

    /// Repository report/output directory, independent of `CARGO_TARGET_DIR`.
    pub fn target(&self) -> PathBuf {
        self.path("target")
    }

    /// Distribution output directory.
    pub fn dist(&self) -> PathBuf {
        self.path("dist")
    }

    /// Site sources and generated pages.
    pub fn site(&self) -> PathBuf {
        self.path("site")
    }

    /// Existing scripts, retained until their commands are ported.
    pub fn scripts(&self) -> PathBuf {
        self.path("scripts")
    }

    /// Start Cargo in this workspace, honoring Cargo's own executable override.
    pub fn cargo(&self) -> Command {
        Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .current_dir(&self.root)
            // `cargo run` gives xtask a package identity. cargo-machete uses its
            // absence to distinguish `cargo machete` from `cargo run -- ...`.
            .env_remove("CARGO_PKG_NAME")
    }

    /// List tracked UTF-8 paths without splitting whitespace or walking build outputs.
    /// Git may run during dry runs because the index is only read.
    pub fn tracked_files(&self, runner: &dyn Runner) -> Result<Vec<String>> {
        let sh = xshell::Shell::new().map_err(|error| Error::Invalid(error.to_string()))?;
        let command = Command::from(xshell::cmd!(sh, "git ls-files -z"))
            .current_dir(&self.root)
            .allow_in_dry_run();
        // Keep the standalone tidy command's established Git diagnostics.
        // This command has no credentials or secret arguments.
        let output = runner
            .execute(&command, true)
            .map_err(|error| Error::Invalid(format!("cannot list tracked files: {error}")))?;
        if !output.status.success() {
            return Err(Error::Invalid(format!(
                "git ls-files failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        tracked_paths(&output.stdout)
            .map(|paths| paths.into_iter().map(str::to_owned).collect())
            .map_err(|error| Error::Invalid(format!("tracked filename is not UTF-8: {error}")))
    }

    /// Read locked Cargo metadata without resolving dependencies into a graph.
    pub fn metadata(&self, runner: &dyn Runner) -> Result<Metadata> {
        let output = self
            .cargo()
            .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
            .allow_in_dry_run()
            .output(runner)?;
        serde_json::from_slice(&output.stdout).map_err(|source| Error::Json {
            context: "cargo metadata".into(),
            source,
        })
    }

    /// Read a repository-relative UTF-8 file with a path-bearing error.
    pub fn read(&self, relative: impl AsRef<Path>) -> Result<String> {
        let path = self.path(relative);
        std::fs::read_to_string(&path).map_err(|source| Error::Io { path, source })
    }
}

pub(crate) fn tracked_paths(output: &[u8]) -> std::result::Result<Vec<&str>, std::str::Utf8Error> {
    Ok(std::str::from_utf8(output)?
        .split('\0')
        .filter(|path| !path.is_empty())
        .collect())
}

/// The subset of Cargo metadata needed by maintenance commands.
#[derive(Debug, Deserialize)]
pub struct Metadata {
    packages: Vec<Package>,
    workspace_members: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Package {
    id: String,
    name: String,
    version: String,
}

impl Metadata {
    /// Look up a workspace member's semantic version by package name.
    pub fn package_version(&self, name: &str) -> Result<semver::Version> {
        let package = self
            .packages
            .iter()
            .find(|package| package.name == name && self.workspace_members.contains(&package.id))
            .ok_or_else(|| {
                Error::Invalid(format!(
                    "workspace package {name:?} was not found in cargo metadata"
                ))
            })?;
        semver::Version::parse(&package.version).map_err(|error| {
            Error::Invalid(format!("invalid version for package {name:?}: {error}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::RecordingRunner;

    #[test]
    fn paths_and_root_sanity() {
        let workspace = Workspace::discover().unwrap();
        assert_eq!(workspace.target(), workspace.root().join("target"));
        assert_eq!(workspace.dist(), workspace.path("dist"));
        assert_eq!(workspace.site(), workspace.path("site"));
        assert_eq!(workspace.scripts(), workspace.path("scripts"));
        assert!(
            workspace
                .cargo()
                .environment()
                .any(|(key, value)| key == "CARGO_PKG_NAME" && value.is_none())
        );
        assert!(Workspace::at(tempfile::tempdir().unwrap().path()).is_err());
    }

    #[test]
    fn tracked_files_preserve_whitespace_and_reject_invalid_utf8() {
        let workspace = Workspace::discover().unwrap();
        let runner = RecordingRunner::stdout(b"docs/a b\0docs/c\nd\0");
        assert_eq!(
            workspace.tracked_files(&runner).unwrap(),
            ["docs/a b", "docs/c\nd"]
        );
        runner.assert_args(0, &["ls-files", "-z"]);
        assert_eq!(
            runner.commands.borrow()[0].0.directory(),
            Some(workspace.root())
        );
        assert!(
            workspace
                .tracked_files(&RecordingRunner::stdout(b"\xff\0"))
                .unwrap_err()
                .to_string()
                .starts_with("tracked filename is not UTF-8:")
        );
    }

    #[test]
    fn tracked_file_errors_preserve_tidy_diagnostics() {
        use crate::cmd::{Output, Status};
        let workspace = Workspace::discover().unwrap();
        let runner = RecordingRunner::default();
        runner
            .results
            .borrow_mut()
            .push_back(Err(std::io::Error::other("missing git")));
        assert_eq!(
            workspace.tracked_files(&runner).unwrap_err().to_string(),
            "cannot list tracked files: missing git"
        );
        runner.results.borrow_mut().push_back(Ok(Output {
            status: Status::from_code(128),
            stderr: b"fatal: not a git repository\n".to_vec(),
            ..Output::default()
        }));
        assert_eq!(
            workspace.tracked_files(&runner).unwrap_err().to_string(),
            "git ls-files failed: fatal: not a git repository"
        );
    }

    #[test]
    fn metadata_selects_workspace_members_and_reports_missing_or_invalid_data() {
        let workspace = Workspace::discover().unwrap();
        let runner = RecordingRunner::stdout(br#"{"packages":[{"id":"dep","name":"app","version":"9.0.0"},{"id":"local","name":"app","version":"1.2.3"}],"workspace_members":["local"]}"#);
        let metadata = workspace.metadata(&runner).unwrap();
        assert_eq!(
            metadata.package_version("app").unwrap().to_string(),
            "1.2.3"
        );
        assert!(metadata.package_version("missing").is_err());
        runner.assert_args(
            0,
            &["metadata", "--no-deps", "--format-version", "1", "--locked"],
        );
        assert!(
            workspace
                .metadata(&RecordingRunner::stdout(b"bad json"))
                .is_err()
        );
    }
}
