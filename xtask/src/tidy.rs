//! Check tracked files without walking build outputs or other untracked files.

use std::{collections::BTreeSet, path::Path, process::Command};

const ROOT_ENTRIES: &[&str] = &[
    ".cargo",
    ".gitattributes",
    ".github",
    ".gitignore",
    "AGENTS.md",
    "Cargo.lock",
    "Cargo.toml",
    "LICENSE",
    "README.md",
    "bench",
    "brand.toml",
    "crates",
    "deny.toml",
    "design",
    "docs",
    "examples",
    "integrations",
    "package-lock.json",
    "package.json",
    "packaging",
    "rust-toolchain.toml",
    "scripts",
    "site",
    "skills",
    "spec",
    "vendor",
    "xtask",
];
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;
// No tracked files currently exceed 5 MiB. Add only reviewed, exact paths here.
const LARGE_FILES: &[&str] = &[];

pub(super) fn run() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(&root)
        .output()
        .map_err(|error| format!("cannot list tracked files: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let paths = tracked_paths(&output.stdout)
        .map_err(|error| format!("tracked filename is not UTF-8: {error}"))?;
    let mut errors = BTreeSet::new();
    for path in paths {
        // Measure the tracked symlink itself, not an untracked target.
        let metadata = root
            .join(path)
            .symlink_metadata()
            .map_err(|error| format!("cannot inspect tracked file {path:?}: {error}"))?;
        errors.extend(check_file(path, metadata.len(), LARGE_FILES));
    }
    if errors.is_empty() {
        println!("Repository hygiene checks passed.");
        Ok(())
    } else {
        Err(errors.into_iter().collect::<Vec<_>>().join("\n"))
    }
}

fn tracked_paths(output: &[u8]) -> Result<Vec<&str>, std::str::Utf8Error> {
    Ok(std::str::from_utf8(output)?
        .split('\0')
        .filter(|path| !path.is_empty())
        .collect())
}

fn check_file(path: &str, size: u64, large_files: &[&str]) -> Vec<String> {
    let mut errors = Vec::new();
    let root = path.split('/').next().unwrap_or_default();
    if !ROOT_ENTRIES.contains(&root) {
        errors.push(format!("unexpected tracked root entry: {root:?}"));
    }
    if size > MAX_FILE_SIZE && !large_files.contains(&path) {
        errors.push(format!(
            "tracked file exceeds 5 MiB ({size} bytes): {path:?}"
        ));
    }
    if path.split('/').any(|name| {
        name == ".DS_Store"
            || name.ends_with(".orig")
            || name.ends_with(".rej")
            || name.ends_with('~')
    }) {
        errors.push(format!("tracked junk file: {path:?}"));
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permits_reviewed_root_entries_and_nested_files() {
        for root in ROOT_ENTRIES {
            assert!(check_file(root, 0, LARGE_FILES).is_empty(), "{root}");
        }
        for path in ["crates/app/src/main.rs", "docs/screenshots/example.png"] {
            assert!(check_file(path, MAX_FILE_SIZE, LARGE_FILES).is_empty());
        }
    }

    #[test]
    fn rejects_removed_and_unknown_root_entries() {
        for path in [
            "PLAN.md",
            "GLOSSARY.md",
            "before-after.png",
            "screenshots/example.png",
            "new-directory/file.rs",
            ".new-dotfile",
            "crates-backup/file.rs",
        ] {
            let errors = check_file(path, 0, LARGE_FILES);
            assert_eq!(errors.len(), 1, "{path}");
            assert!(errors[0].starts_with("unexpected tracked root entry:"));
        }
    }

    #[test]
    fn enforces_size_boundary_and_exact_exceptions() {
        let path = "bench/scenes/large.voxl";
        assert!(check_file(path, MAX_FILE_SIZE, &[]).is_empty());
        assert_eq!(check_file(path, MAX_FILE_SIZE + 1, &[]).len(), 1);
        assert!(check_file(path, MAX_FILE_SIZE + 1, &[path]).is_empty());
        for exception in ["bench", "bench/scenes", "large.voxl", "bench/scenes/large"] {
            assert_eq!(check_file(path, MAX_FILE_SIZE + 1, &[exception]).len(), 1);
        }
    }

    #[test]
    fn rejects_junk_even_in_allowed_directories_or_size_exceptions() {
        for path in [
            ".DS_Store",
            "crates/.DS_Store",
            "crates/file.rs.orig",
            "crates/file.rs.rej",
            "crates/file.rs~",
            "crates/backup~/file.rs",
        ] {
            assert!(
                check_file(path, MAX_FILE_SIZE + 1, &[path])
                    .iter()
                    .any(|error| error.starts_with("tracked junk file:")),
                "{path}"
            );
        }
        for path in ["docs/original.md", "docs/file.orig.md", "docs/rejects.md"] {
            assert!(check_file(path, 0, LARGE_FILES).is_empty());
        }
    }

    #[test]
    fn reports_all_violations() {
        assert_eq!(
            check_file("backup/dump.orig", MAX_FILE_SIZE + 1, &[]).len(),
            3
        );
    }

    #[test]
    fn parses_nul_separated_names_without_splitting_whitespace() {
        assert_eq!(
            tracked_paths(b"docs/has spaces.md\0docs/has\nnewline.md\0").unwrap(),
            ["docs/has spaces.md", "docs/has\nnewline.md"]
        );
        assert!(tracked_paths(b"").unwrap().is_empty());
        assert!(tracked_paths(b"docs/\xff\0").is_err());
    }
}
