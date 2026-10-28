//! Stable workspace version changes, planned before writing and rolled back on failure.

use std::{fs, ops::Range};

use semver::Version;

use crate::{Error, Result, cmd::Runner, workspace::Workspace};

/// Read the inherited workspace version without invoking Cargo or build scripts.
/// Useful for a dry-run plan; release tag checks use the app's Cargo metadata.
pub fn workspace_version(workspace: &Workspace) -> Result<Version> {
    manifest_version(&workspace.read("Cargo.toml")?).map(|(_, version)| version)
}

/// Choose a stable version. Explicit equal versions are harmless no-ops; downgrades,
/// prereleases, build metadata and overflowing increments are rejected.
pub fn next_version(current: &Version, request: &str) -> Result<Version> {
    stable(current)?;
    let increment = |value: u64| {
        value
            .checked_add(1)
            .ok_or_else(|| Error::Invalid("version component overflow".into()))
    };
    let next = match request {
        "major" => Version::new(increment(current.major)?, 0, 0),
        "minor" => Version::new(current.major, increment(current.minor)?, 0),
        "patch" => Version::new(current.major, current.minor, increment(current.patch)?),
        value => Version::parse(value).map_err(|error| {
            Error::Invalid(format!(
                "invalid version {value:?}: {error}; expected major, minor, patch or X.Y.Z"
            ))
        })?,
    };
    stable(&next)?;
    if next < *current {
        return Err(Error::Invalid(format!(
            "cannot downgrade workspace version from {current} to {next}"
        )));
    }
    Ok(next)
}

fn stable(version: &Version) -> Result<()> {
    if version.pre.is_empty() && version.build.is_empty() {
        Ok(())
    } else {
        Err(Error::Invalid(format!(
            "version {version} is not a stable X.Y.Z version"
        )))
    }
}

// Locate a single quoted value without reformatting the enclosing document.
fn quoted_value(text: &str, offset: usize) -> Result<Range<usize>> {
    let value = text[offset..].trim_start();
    let start = text.len() - value.len();
    let quote = value
        .chars()
        .next()
        .filter(|c| matches!(c, '\'' | '"'))
        .ok_or_else(|| Error::Invalid("expected a quoted version string".into()))?;
    let end = value[1..]
        .find(quote)
        .ok_or_else(|| Error::Invalid("unterminated version string".into()))?;
    Ok(start + 1..start + 1 + end)
}

fn manifest_version(source: &str) -> Result<(Range<usize>, Version)> {
    let mut in_package = false;
    let mut found = None;
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package =
                trimmed.split('#').next().unwrap_or_default().trim() == "[workspace.package]";
        } else if in_package
            && let Some((key, _)) = line.split_once('=')
            && key.trim() == "version"
        {
            if found.is_some() {
                return Err(Error::Invalid(
                    "duplicate [workspace.package] version in Cargo.toml".into(),
                ));
            }
            // Restrict parsing to this line: malformed input must never consume a later key.
            let local = quoted_value(line, key.len() + 1)?;
            let trailing = &line[local.end + 1..];
            if !trailing.trim().is_empty() && !trailing.trim().starts_with('#') {
                return Err(Error::Invalid(
                    "unexpected text after [workspace.package] version".into(),
                ));
            }
            let version = Version::parse(&line[local.clone()]).map_err(|error| {
                Error::Invalid(format!("invalid [workspace.package] version: {error}"))
            })?;
            stable(&version)?;
            found = Some((offset + local.start..offset + local.end, version));
        }
        offset += line.len();
    }
    found.ok_or_else(|| {
        Error::Invalid("missing quoted [workspace.package] version in Cargo.toml".into())
    })
}

fn replace_reference(
    source: &str,
    prefix: &str,
    suffix: &str,
    old: &Version,
    new: &Version,
) -> Result<String> {
    let mut positions = source.match_indices(prefix);
    let (start, _) = positions
        .next()
        .ok_or_else(|| Error::Invalid(format!("missing version marker {prefix:?}")))?;
    if positions.next().is_some() {
        return Err(Error::Invalid(format!(
            "ambiguous version marker {prefix:?}"
        )));
    }
    let start = start + prefix.len();
    let end = source[start..]
        .find(suffix)
        .map(|end| start + end)
        .ok_or_else(|| Error::Invalid(format!("missing version suffix {suffix:?}")))?;
    replace_range(source, start..end, old, new)
}

fn replace_range(
    source: &str,
    range: Range<usize>,
    old: &Version,
    new: &Version,
) -> Result<String> {
    if source[range.clone()] != old.to_string() {
        return Err(Error::Invalid(format!(
            "version reference {:?} does not match workspace version {old}",
            &source[range]
        )));
    }
    let mut updated = source.to_owned();
    updated.replace_range(range, &new.to_string());
    Ok(updated)
}

fn compatibility_version(source: &str, old: &Version, new: &Version) -> Result<String> {
    let json: serde_json::Value = serde_json::from_str(source).map_err(|source| Error::Json {
        context: "MCP compatibility JSON".into(),
        source,
    })?;
    if json
        .get("application_version")
        .and_then(serde_json::Value::as_str)
        != Some(old.to_string().as_str())
    {
        return Err(Error::Invalid(format!(
            "application_version does not match workspace version {old}"
        )));
    }
    let marker = "\"application_version\"";
    let mut matches = source.match_indices(marker);
    let (start, _) = matches
        .next()
        .ok_or_else(|| Error::Invalid("missing application_version key".into()))?;
    if matches.next().is_some() {
        return Err(Error::Invalid("ambiguous application_version key".into()));
    }
    let tail = &source[start + marker.len()..];
    let tail = tail
        .trim_start()
        .strip_prefix(':')
        .ok_or_else(|| Error::Invalid("expected colon after application_version".into()))?;
    let range = quoted_value(source, source.len() - tail.len())?;
    replace_range(source, range, old, new)
}

struct Edit {
    path: &'static str,
    before: String,
    after: String,
}

/// A validated set of edits. Only current version references are included, never
/// historical appcasts, release notes or generated site output.
pub struct Plan {
    workspace: Workspace,
    from: Version,
    to: Version,
    edits: Vec<Edit>,
    lockfile: Vec<u8>,
}

impl Plan {
    /// Original workspace version.
    pub fn from(&self) -> &Version {
        &self.from
    }

    /// Requested workspace version.
    pub fn to(&self) -> &Version {
        &self.to
    }

    /// Read and validate every reference and the lockfile before allowing any writes.
    pub fn prepare(workspace: &Workspace, runner: &dyn Runner, request: &str) -> Result<Self> {
        let tracked = workspace.tracked_files(runner)?;
        let paths = [
            "Cargo.toml",
            "README.md",
            "crates/voxelite-site/templates/download.html",
            "spec/mcp/v1/compatibility.json",
            "Cargo.lock",
        ];
        for path in paths {
            if !tracked.iter().any(|tracked| tracked == path) {
                return Err(Error::Invalid(format!(
                    "required version file {path} is not tracked"
                )));
            }
        }
        let manifest = workspace.read("Cargo.toml")?;
        let (range, from) = manifest_version(&manifest)?;
        let to = next_version(&from, request)?;
        let mut edits = vec![Edit {
            path: "Cargo.toml",
            after: replace_range(&manifest, range, &from, &to)?,
            before: manifest,
        }];
        for (path, prefix, suffix) in [
            ("README.md", "The current workspace version is **", "**"),
            (
                "crates/voxelite-site/templates/download.html",
                "Version ",
                " for macOS",
            ),
        ] {
            let before = workspace.read(path)?;
            let after = replace_reference(&before, prefix, suffix, &from, &to)
                .map_err(|error| Error::Invalid(format!("{path}: {error}")))?;
            edits.push(Edit {
                path,
                before,
                after,
            });
        }
        let path = "spec/mcp/v1/compatibility.json";
        let before = workspace.read(path)?;
        let after = compatibility_version(&before, &from, &to)
            .map_err(|error| Error::Invalid(format!("{path}: {error}")))?;
        edits.push(Edit {
            path,
            before,
            after,
        });
        let path = workspace.path("Cargo.lock");
        let lockfile = fs::read(&path).map_err(|source| Error::Io { path, source })?;
        Ok(Self {
            workspace: workspace.clone(),
            from,
            to,
            edits,
            lockfile,
        })
    }

    /// Files that will change, including the Cargo-managed lockfile.
    pub fn changed_paths(&self) -> Vec<&'static str> {
        if self.from == self.to {
            Vec::new()
        } else {
            self.edits
                .iter()
                .map(|edit| edit.path)
                .chain(["Cargo.lock"])
                .collect()
        }
    }

    /// Apply the plan and refresh workspace members offline. On a write/command
    /// failure, restore all original bytes. Dry runs invoke only the runner's fake
    /// lockfile command and do not write. Concurrent file changes invalidate a plan.
    pub fn apply(&self, runner: &dyn Runner) -> Result<()> {
        if self.from == self.to {
            return Ok(());
        }
        for edit in &self.edits {
            if self.workspace.read(edit.path)? != edit.before {
                return Err(Error::Invalid(format!(
                    "{} changed after version planning; run bump-version again",
                    edit.path
                )));
            }
        }
        let path = self.workspace.path("Cargo.lock");
        if fs::read(&path).map_err(|source| Error::Io { path, source })? != self.lockfile {
            return Err(Error::Invalid(
                "Cargo.lock changed after version planning; run bump-version again".into(),
            ));
        }
        let update = self
            .workspace
            .cargo()
            .args(["update", "--workspace", "--offline"]);
        if runner.is_dry_run() {
            return update.run(runner).map_err(Error::from);
        }
        let result: Result<()> = (|| {
            for edit in &self.edits {
                self.write(edit.path, edit.after.as_bytes())?;
            }
            update.run(runner)?;
            Ok(())
        })();
        if let Err(error) = result {
            let mut failures = vec![format!("version update failed: {error}")];
            for (path, bytes) in self
                .edits
                .iter()
                .map(|edit| (edit.path, edit.before.as_bytes()))
                .chain([("Cargo.lock", self.lockfile.as_slice())])
            {
                if let Err(error) = self.write(path, bytes) {
                    failures.push(format!("could not restore {path}: {error}"));
                }
            }
            return Err(Error::Invalid(failures.join("\n")));
        }
        Ok(())
    }

    fn write(&self, relative: &str, bytes: &[u8]) -> Result<()> {
        let path = self.workspace.path(relative);
        fs::write(&path, bytes).map_err(|source| Error::Io { path, source })
    }
}

#[cfg(test)]
mod tests;
