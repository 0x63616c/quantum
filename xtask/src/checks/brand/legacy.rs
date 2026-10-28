//! Reject legacy branding outside justified, live compatibility exceptions.

use std::collections::BTreeMap;

use regex::Regex;
use serde::Deserialize;

use crate::{Error, Result, check::Context};

#[derive(Deserialize)]
struct Inventory {
    compatibility: BTreeMap<String, Exception>,
}

#[derive(Deserialize)]
struct Exception {
    current: Vec<String>,
    paths: Vec<String>,
    reason: String,
    #[serde(default)]
    generated_paths: Vec<String>,
}

pub(super) fn check(context: &Context<'_>) -> Result<Vec<String>> {
    let inventory: Inventory = super::read_toml(context.workspace, "brand.toml")?;
    let generated = std::env::var_os("QUANTUM_BRAND_SITE");
    let generated_rules = inventory
        .compatibility
        .iter()
        .filter(|(_, entry)| !entry.generated_paths.is_empty())
        .map(|(name, entry)| {
            (
                name.clone(),
                Exception {
                    current: entry.current.clone(),
                    paths: entry.generated_paths.clone(),
                    reason: entry.reason.clone(),
                    generated_paths: Vec::new(),
                },
            )
        })
        .collect();
    let mut scanner = Scanner::new(inventory.compatibility)?;
    for path in context.workspace.tracked_files(context.runner)? {
        scanner.inspect(&path, &path);
        let absolute = context.workspace.path(&path);
        let bytes = std::fs::read(&absolute).map_err(|source| Error::Io {
            path: absolute,
            source,
        })?;
        if !bytes.contains(&0)
            && let Ok(text) = std::str::from_utf8(&bytes)
        {
            scanner.inspect(&path, text);
        }
    }
    let mut errors = scanner.finish();
    if let Some(root) = generated {
        let mut generated = Scanner::new(generated_rules)?;
        generated.audit_literals = false;
        inspect_directory(
            std::path::Path::new(&root),
            std::path::Path::new(&root),
            &mut generated,
        )?;
        errors.extend(generated.finish());
    }
    Ok(errors)
}

fn inspect_directory(
    root: &std::path::Path,
    directory: &std::path::Path,
    scanner: &mut Scanner,
) -> Result<()> {
    for entry in std::fs::read_dir(directory).map_err(|source| Error::Io {
        path: directory.into(),
        source,
    })? {
        let entry = entry.map_err(|source| Error::Io {
            path: directory.into(),
            source,
        })?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|error| Error::Invalid(error.to_string()))?
            .to_string_lossy();
        // Authored inputs are checked as tracked source; only scan published files.
        if matches!(relative.as_ref(), "content" | "README.md") {
            continue;
        }
        if entry
            .file_type()
            .map_err(|source| Error::Io {
                path: path.clone(),
                source,
            })?
            .is_dir()
        {
            inspect_directory(root, &path, scanner)?;
        } else {
            scanner.inspect(&relative, &relative);
            let bytes = std::fs::read(&path).map_err(|source| Error::Io {
                path: path.clone(),
                source,
            })?;
            if !bytes.contains(&0)
                && let Ok(text) = std::str::from_utf8(&bytes)
            {
                scanner.inspect(&relative, text);
            }
        }
    }
    Ok(())
}

struct Rule {
    name: String,
    paths: Vec<glob::Pattern>,
    literals: Vec<String>,
    used_paths: Vec<bool>,
    used_literals: Vec<bool>,
}

struct Scanner {
    legacy: Regex,
    audit_literals: bool,
    rules: Vec<Rule>,
    errors: Vec<String>,
}

impl Scanner {
    fn new(exceptions: BTreeMap<String, Exception>) -> Result<Self> {
        let mut rules = Vec::new();
        for (name, entry) in exceptions {
            if entry.reason.trim().is_empty() || entry.current.is_empty() || entry.paths.is_empty()
            {
                return Err(Error::Invalid(format!(
                    "brand.toml: {name} needs literals, paths and a reason"
                )));
            }
            let paths = entry
                .paths
                .iter()
                .map(|path| glob::Pattern::new(path))
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|error| Error::Invalid(format!("brand.toml: {name}: {error}")))?;
            rules.push(Rule {
                name,
                used_paths: vec![false; paths.len()],
                used_literals: vec![false; entry.current.len()],
                paths,
                literals: entry.current,
            });
        }
        Ok(Self {
            audit_literals: true,
            legacy: Regex::new(concat!("(?i)", "voxe", "lite"))
                .map_err(|error| Error::Invalid(error.to_string()))?,
            rules,
            errors: Vec::new(),
        })
    }

    fn inspect(&mut self, path: &str, text: &str) {
        for hit in self.legacy.find_iter(text) {
            let mut allowed = false;
            for rule in &mut self.rules {
                let Some(path_index) = rule.paths.iter().position(|pattern| pattern.matches(path))
                else {
                    continue;
                };
                for (literal_index, literal) in rule.literals.iter().enumerate() {
                    // Only the actual legacy occurrence inside a permitted literal is exempt.
                    if text.match_indices(literal.as_str()).any(|(start, value)| {
                        start <= hit.start() && start + value.len() >= hit.end()
                    }) {
                        rule.used_paths[path_index] = true;
                        rule.used_literals[literal_index] = true;
                        allowed = true;
                    }
                }
            }
            if !allowed {
                let before = &text[..hit.start()];
                let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
                let col = before
                    .rsplit('\n')
                    .next()
                    .unwrap_or_default()
                    .chars()
                    .count()
                    + 1;
                self.errors.push(format!(
                    "{path}:{line}:{col}: legacy brand {:?}; see brand.toml compatibility",
                    hit.as_str()
                ));
            }
        }
    }

    fn finish(mut self) -> Vec<String> {
        // Every listed path and literal must still be needed, so retired
        // compatibility cannot linger inside an otherwise live group.
        for rule in self.rules {
            for (pattern, used) in rule.paths.iter().zip(&rule.used_paths) {
                if !used {
                    self.errors.push(format!(
                        "brand.toml: stale compatibility.{} path {:?} (no legacy occurrence matched)",
                        rule.name,
                        pattern.as_str()
                    ));
                }
            }
            for (literal, used) in rule.literals.iter().zip(&rule.used_literals) {
                if self.audit_literals && !used {
                    self.errors.push(format!(
                        "brand.toml: stale compatibility.{} literal {:?} (no legacy occurrence matched)",
                        rule.name, literal
                    ));
                }
            }
        }
        self.errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scanner() -> Scanner {
        Scanner::new(BTreeMap::new()).unwrap()
    }

    #[test]
    fn built_output_is_scanned_with_the_same_rules() {
        let output = tempfile::tempdir().unwrap();
        let nested = output.path().join("blog");
        std::fs::create_dir_all(&nested).unwrap();
        let path = nested.join("index.html");
        let legacy = concat!("Voxe", "lite");
        std::fs::write(&path, legacy).unwrap();
        let mut rejected = scanner();
        inspect_directory(output.path(), output.path(), &mut rejected).unwrap();
        assert!(rejected.finish()[0].contains("blog/index.html:1:1"));
        let mut allowed = Scanner::new(BTreeMap::from([(
            "history".into(),
            Exception {
                current: vec![legacy.into()],
                paths: vec!["blog/index.html".into()],
                generated_paths: Vec::new(),
                reason: "Historical announcement".into(),
            },
        )]))
        .unwrap();
        inspect_directory(output.path(), output.path(), &mut allowed).unwrap();
        assert!(allowed.finish().is_empty());
    }

    #[test]
    fn stray_names_and_case_variants_fail() {
        for name in [
            concat!("Voxe", "lite"),
            concat!("VOXE", "LITE_"),
            concat!("voxe", "lite_"),
        ] {
            let mut scanner = scanner();
            scanner.inspect("README.md", &format!("é\n {name}"));
            assert!(scanner.finish()[0].starts_with("README.md:2:2:"));
        }
    }

    #[test]
    fn filenames_and_directories_fail_even_without_text() {
        let mut scanner = scanner();
        let path = concat!("old/", "Voxe", "lite.app/icon.png");
        scanner.inspect(path, path);
        assert_eq!(scanner.finish().len(), 1);
    }

    #[test]
    fn allowed_path_passes_and_unused_entry_is_stale() {
        let literal = concat!("voxe", "lite");
        let make = || {
            Scanner::new(BTreeMap::from([(
                "format".into(),
                Exception {
                    current: vec![literal.into()],
                    paths: vec!["schema.json".into()],
                    reason: "Frozen format discriminator".into(),
                    generated_paths: Vec::new(),
                },
            )]))
            .unwrap()
        };
        let mut allowed = make();
        allowed.inspect("schema.json", literal);
        assert!(allowed.finish().is_empty());
        let mut wrong = make();
        wrong.inspect("README.md", literal);
        let errors = wrong.finish();
        assert_eq!(errors.len(), 3, "{errors:?}");
        assert!(errors[0].starts_with("README.md:1:1:"));
        assert!(errors[1].contains("stale compatibility.format path"));
        assert!(errors[2].contains("stale compatibility.format literal"));
    }

    #[test]
    fn unused_path_or_literal_inside_live_group_is_stale() {
        let literal = concat!("voxe", "lite");
        let mime = concat!("application/vnd.", "voxe", "lite.project+json");
        let mut scanner = Scanner::new(BTreeMap::from([(
            "format".into(),
            Exception {
                current: vec![literal.into(), mime.into()],
                paths: vec!["schema.json".into(), "retired.json".into()],
                reason: "Frozen format discriminator".into(),
                generated_paths: Vec::new(),
            },
        )]))
        .unwrap();
        scanner.inspect("schema.json", literal);
        let errors = scanner.finish();
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(errors[0].contains("path \"retired.json\""));
        assert!(errors[1].contains("literal"));
    }
}

#[cfg(test)]
mod migration_tests {
    use super::*;

    #[test]
    fn compatibility_literal_does_not_exempt_other_occurrences_in_the_same_file() {
        let legacy = concat!("voxe", "lite");
        let bundle = format!("co.worldwidewebb.{legacy}");
        let mut scanner = Scanner::new(BTreeMap::from([(
            "bundle".into(),
            Exception {
                current: vec![bundle.clone()],
                paths: vec!["migration.rs".into()],
                generated_paths: Vec::new(),
                reason: "Read the old defaults domain".into(),
            },
        )]))
        .unwrap();
        scanner.inspect("migration.rs", &format!("{bundle}\n{legacy}.v1\n{legacy}"));
        scanner.inspect("new.rs", &bundle);
        let errors = scanner.finish();
        assert_eq!(errors.len(), 3, "{errors:?}");
        assert!(errors[0].starts_with("migration.rs:2:1:"));
        assert!(errors[1].starts_with("migration.rs:3:1:"));
        assert!(errors[2].starts_with("new.rs:1:18:"));
    }

    #[test]
    fn removing_migration_leaves_stale_path_and_literal_even_with_live_history() {
        let legacy = concat!("Voxe", "lite");
        let mut scanner = Scanner::new(BTreeMap::from([
            (
                "history".into(),
                Exception {
                    current: vec![legacy.into()],
                    paths: vec!["CHANGELOG.md".into()],
                    generated_paths: Vec::new(),
                    reason: "Historical announcement".into(),
                },
            ),
            (
                "removed".into(),
                Exception {
                    current: vec![legacy.into()],
                    paths: vec!["migration.rs".into()],
                    generated_paths: Vec::new(),
                    reason: "Retired migration".into(),
                },
            ),
        ]))
        .unwrap();
        scanner.inspect("CHANGELOG.md", legacy);
        let errors = scanner.finish();
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(errors[0].contains("stale compatibility.removed path"));
        assert!(errors[1].contains("stale compatibility.removed literal"));
    }
}
