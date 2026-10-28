//! Repository consistency rules, separate from command selection and reporting.

pub(crate) mod brand;
pub(crate) mod docs;
pub(crate) mod plugin_deps;

use std::path::PathBuf;

use crate::{Error, Result, check::Outcome, workspace::Workspace};

fn outcome(result: Result<Vec<String>>) -> Outcome {
    match result {
        Ok(errors) if errors.is_empty() => Outcome::Pass,
        Ok(errors) => Outcome::Fail(errors),
        Err(error) => Outcome::Fail(vec![error.to_string()]),
    }
}

/// Expand repository-relative patterns, preserving sorted paths and I/O errors.
fn paths(workspace: &Workspace, pattern: &str) -> Result<Vec<PathBuf>> {
    let root = workspace.root().to_str().ok_or_else(|| {
        Error::Invalid(format!(
            "repository path is not UTF-8: {}",
            workspace.root().display()
        ))
    })?;
    let pattern = format!("{}/{pattern}", glob::Pattern::escape(root));
    let mut paths = glob::glob(&pattern)
        .map_err(|error| Error::Invalid(format!("invalid path pattern {pattern:?}: {error}")))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| Error::Invalid(format!("cannot inspect {pattern:?}: {error}")))?;
    paths.sort();
    Ok(paths)
}

/// Match descendant filenames without following directory symlinks, like
/// pathlib's recursive glob. A symlink loop must not widen or stall a docs check.
fn descendants(workspace: &Workspace, directory: &str, pattern: &str) -> Result<Vec<PathBuf>> {
    let pattern = glob::Pattern::new(pattern).map_err(|error| {
        Error::Invalid(format!("invalid filename pattern {pattern:?}: {error}"))
    })?;
    let mut pending = vec![workspace.path(directory)];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        let entries = std::fs::read_dir(&directory).map_err(|source| Error::Io {
            path: directory.clone(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| Error::Io {
                path: directory.clone(),
                source,
            })?;
            let path = entry.path();
            if pattern.matches_path(std::path::Path::new(&entry.file_name())) {
                paths.push(path.clone());
            }
            if entry
                .file_type()
                .map_err(|source| Error::Io {
                    path: path.clone(),
                    source,
                })?
                .is_dir()
            {
                pending.push(path);
            }
        }
    }
    paths.sort();
    Ok(paths)
}

/// Capture the first group of each match, including duplicates and source order.
fn captures(source: &str, pattern: &str) -> Result<Vec<String>> {
    let regex = regex::Regex::new(pattern)
        .map_err(|error| Error::Invalid(format!("invalid check pattern {pattern:?}: {error}")))?;
    Ok(regex
        .captures_iter(source)
        .map(|capture| capture[1].to_owned())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_preserve_order_duplicates_and_whitespace_rules() {
        let source = "const ID : &str = \"one\"; const ID:&str=\"two\"; const ID:&str=\"two\";";
        assert_eq!(
            captures(source, r#"const\s+ID\s*:\s*&str\s*=\s*"([^"]+)""#).unwrap(),
            ["one", "two", "two"]
        );
        assert!(captures(source, r"missing (value)").unwrap().is_empty());
        assert!(captures(source, "[").is_err());
    }
}
