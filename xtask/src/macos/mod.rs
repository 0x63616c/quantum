//! macOS release tasks. Commands use argument vectors, strip signing credentials
//! from every child, and accept an injected runner for offline qualification.

use crate::{
    Error, Result,
    cmd::{Command, Runner, SystemRunner},
    workspace::Workspace,
};
use clap::Subcommand;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicUsize},
};

pub mod appcast;
mod feed;
mod files;
pub mod package;
pub mod release_notes;
pub mod sign;

pub use feed::{Update, validate, validate_release};
pub use files::sha256;

/// Variables which must never be inherited by release subprocesses.
pub const SECRETS: &[&str] = &[
    "MACOS_CERTIFICATE_P12_BASE64",
    "MACOS_CERTIFICATE_PASSWORD",
    "MACOS_SIGNING_IDENTITY",
    "APPLE_API_KEY_P8_BASE64",
    "APPLE_API_KEY_ID",
    "APPLE_API_ISSUER_ID",
    "SPARKLE_ED_PRIVATE_KEY",
];
/// The only repository permitted in the stable update feed.
pub const REPOSITORY: &str = "0x63616c/voxelite";
/// Stable download URL prefix; preserve published asset names.
pub const DOWNLOAD: &str = "https://github.com/0x63616c/voxelite/releases/download/";

/// Explicit configuration, without global environment mutation or secret Debug output.
#[derive(Default)]
pub struct Environment(BTreeMap<String, String>);
impl Environment {
    /// Read only release configuration from the process environment.
    pub fn from_process() -> Self {
        let mut env = Self::default();
        for name in SECRETS.iter().copied().chain([
            "CARGO_TARGET_DIR",
            "VERSION",
            "MACOS_SIGNING_MODE",
            "SPARKLE_PUBLIC_ED_KEY",
            "TAG",
            "GITHUB_REPOSITORY",
            "GITHUB_OUTPUT",
            "TMPDIR",
        ]) {
            if let Ok(value) = std::env::var(name) {
                env.0.insert(name.into(), value);
            }
        }
        env
    }
    /// Set a configuration value, useful for callers and tests.
    pub fn set(&mut self, name: &str, value: impl Into<String>) {
        self.0.insert(name.into(), value.into());
    }
    /// Read a value, treating missing variables as empty as the original scripts did.
    pub fn get(&self, name: &str) -> &str {
        self.0.get(name).map_or("", String::as_str)
    }
    /// Read a nonempty value or its default.
    pub fn or<'a>(&'a self, name: &str, default: &'a str) -> &'a str {
        let value = self.get(name);
        if value.is_empty() { default } else { value }
    }
}

/// Services for a release operation. The root and environment are explicit so
/// packaging logic can run with fake tools on Linux without global state.
pub struct Context<'a> {
    /// Repository root.
    pub root: &'a Path,
    /// Real, recording, or dry-run process executor.
    pub runner: &'a dyn Runner,
    /// Configuration retained in-process; credentials are never inherited by tools.
    pub environment: &'a Environment,
}
impl Context<'_> {
    /// Resolve a repository-relative path.
    pub fn path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.root.join(path)
    }
    /// Build a credential-free command in the repository root.
    pub fn command(&self, program: impl AsRef<std::ffi::OsStr>) -> Command {
        clean(Command::new(program).current_dir(self.root))
    }
    /// Apply release environment and directory policy to an xshell command vector.
    pub fn script(&self, command: xshell::Cmd<'_>) -> Command {
        clean(Command::from(command).current_dir(self.root))
    }
    pub(crate) fn private(&self, operation: &str, command: Command) -> Result<()> {
        command.output(self.runner).map(|_| ()).map_err(|_| Error::Invalid(format!(
            "{operation} failed; raw signing diagnostics suppressed. Reproduce locally to inspect them."
        )))
    }
    pub(crate) fn plan(&self, message: impl std::fmt::Display) {
        if self.runner.is_dry_run() {
            eprintln!("[dry-run] {message}");
        }
    }
}
/// Remove release credentials without changing the parent's environment.
pub fn clean(mut command: Command) -> Command {
    for name in SECRETS {
        command = command.env_remove(name);
    }
    command
}

/// Numerically ordered stable version. Decimal components are not restricted to
/// machine integers: the release scripts accepted arbitrarily large components.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct StableVersion([(usize, String); 3]);

/// Validate exactly three canonical decimal components, without suffixes.
pub fn stable_version(value: &str) -> Result<StableVersion> {
    let parts: Vec<_> = value.split('.').collect();
    if parts.len() != 3
        || !parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|b| b.is_ascii_digit())
                && (*part == "0" || !part.starts_with('0'))
        })
    {
        return Err(invalid("Expected a stable MAJOR.MINOR.PATCH version"));
    }
    Ok(StableVersion(std::array::from_fn(|i| {
        (parts[i].len(), parts[i].into())
    })))
}
/// Stable vMAJOR.MINOR.PATCH release tag validation.
pub fn stable_tag(value: &str) -> bool {
    value
        .strip_prefix('v')
        .is_some_and(|value| stable_version(value).is_ok())
}
/// Read the app version and optionally enforce the release tag before building.
pub fn version(workspace: &Workspace, runner: &dyn Runner, tag: Option<&str>) -> Result<String> {
    let output = clean(workspace.cargo())
        .args(["metadata", "--no-deps", "--locked", "--format-version", "1"])
        .allow_in_dry_run()
        .output(runner)?;
    let metadata: crate::workspace::Metadata =
        serde_json::from_slice(&output.stdout).map_err(|source| Error::Json {
            context: "cargo metadata".into(),
            source,
        })?;
    let version = metadata.package_version("voxelite-app")?.to_string();
    if tag.is_some_and(|tag| tag != format!("v{version}")) {
        return Err(Error::Invalid(
            "Release tag must match the voxelite-app Cargo version.".into(),
        ));
    }
    Ok(version)
}

/// Grouped CLI for release preparation; none of these commands publish assets.
#[derive(Subcommand)]
pub enum Task {
    /// Build, sign, optionally notarize, and create dist/Voxelite.dmg
    Package {
        /// Use existing target/release executables
        #[arg(long)]
        skip_build: bool,
        /// Print the full redacted plan without executing commands or writing files
        #[arg(long)]
        dry_run: bool,
    },
    /// Generate a signed feed from the notarized DMG and downloaded history
    Appcast {
        /// Print the full redacted plan without executing commands or writing files
        #[arg(long)]
        dry_run: bool,
    },
    /// Download the highest stable release's feed, preserving its exact bytes
    FetchAppcast {
        /// Destination XML file
        out: PathBuf,
    },
    /// Validate an appcast offline, optionally against a packaged release
    CheckAppcast {
        /// Feed XML file
        file: PathBuf,
        /// Accept a feed with no installable items
        #[arg(long)]
        allow_empty: bool,
        /// Check the newest item against this distribution directory
        #[arg(long, requires = "previous")]
        release_dir: Option<PathBuf>,
        /// Previous feed; required with --release-dir
        #[arg(long, requires = "release_dir")]
        previous: Option<PathBuf>,
    },
    /// Write installation notes and non-merge commit subjects for TAG
    ReleaseNotes,
}

/// Register signal flags while the CLI owns cleanup guards. No signal handler
/// performs I/O; the runner kills/reaps the active child, then Rust unwinds normally.
struct Signals(Vec<signal_hook::SigId>);
impl Signals {
    fn install(flag: &Arc<AtomicUsize>) -> Result<Self> {
        let mut signals = Self(Vec::new());
        for (signal, code) in [
            (signal_hook::consts::SIGINT, 130),
            (signal_hook::consts::SIGTERM, 143),
        ] {
            signals.0.push(
                signal_hook::flag::register_usize(signal, flag.clone(), code).map_err(|e| {
                    Error::Invalid(format!("cannot install release cleanup handler: {e}"))
                })?,
            );
        }
        Ok(signals)
    }
}
impl Drop for Signals {
    fn drop(&mut self) {
        for id in self.0.drain(..) {
            signal_hook::low_level::unregister(id);
        }
    }
}

/// Execute the grouped CLI, returning script-compatible interruption exit codes.
pub fn execute(task: Task, workspace: &Workspace) -> Result<u8> {
    let flag = Arc::new(AtomicUsize::new(0));
    let _signals = Signals::install(&flag)?;
    let dry = matches!(
        task,
        Task::Package { dry_run: true, .. } | Task::Appcast { dry_run: true }
    );
    let runner = if dry {
        SystemRunner::dry_run()
    } else {
        SystemRunner::default()
    }
    .with_interruption(flag);
    let environment = Environment::from_process();
    let context = Context {
        root: workspace.root(),
        runner: &runner,
        environment: &environment,
    };
    let result = match task {
        Task::Package { skip_build, .. } => package::package(
            &context,
            &package::Options {
                skip_build,
                ..Default::default()
            },
        ),
        Task::Appcast { .. } => appcast::generate(&context),
        Task::FetchAppcast { out } => appcast::fetch(&context, &out),
        Task::CheckAppcast {
            file,
            allow_empty,
            release_dir,
            previous,
        } => check_appcast(
            &file,
            allow_empty,
            release_dir.as_deref(),
            previous.as_deref(),
        ),
        Task::ReleaseNotes => release_notes::generate(&context),
    };
    if runner.interruption_code() != 0 && !matches!(result, Err(Error::Cleanup(_))) {
        return Ok(runner.interruption_code() as u8);
    }
    result.map(|()| 0)
}

/// Offline CLI validation, including the original diagnostic prefix.
pub fn check_appcast(
    file: &Path,
    allow_empty: bool,
    release_dir: Option<&Path>,
    previous: Option<&Path>,
) -> Result<()> {
    let result = match (release_dir, previous) {
        (Some(dist), Some(previous)) => validate_release(file, dist, previous),
        (None, None) => validate(file, allow_empty, None).map(|_| ()),
        _ => Err(Error::Invalid(
            "--release-dir and --previous must be supplied together".into(),
        )),
    };
    result.map_err(|e| Error::Invalid(format!("Invalid appcast: {e}")))?;
    println!("Appcast validation passed.");
    Ok(())
}

pub(crate) fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|source| Error::Io {
        path: path.into(),
        source,
    })
}
pub(crate) fn write(path: &Path, bytes: impl AsRef<[u8]>) -> Result<()> {
    fs::write(path, bytes).map_err(|source| Error::Io {
        path: path.into(),
        source,
    })
}
pub(crate) fn mkdir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| Error::Io {
        path: path.into(),
        source,
    })
}
pub(crate) fn copy(from: &Path, to: &Path) -> Result<()> {
    fs::copy(from, to).map(|_| ()).map_err(|source| Error::Io {
        path: to.into(),
        source,
    })
}
pub(crate) fn invalid(message: &str) -> Error {
    Error::Invalid(message.into())
}

#[cfg(test)]
mod tests;
