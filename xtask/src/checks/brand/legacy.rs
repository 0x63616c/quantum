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
}

pub(super) fn check(context: &Context<'_>) -> Result<Vec<String>> {
    let inventory: Inventory = super::read_toml(context.workspace, "brand.toml")?;
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
    Ok(scanner.finish())
}

struct Rule {
    name: String,
    paths: Vec<glob::Pattern>,
    literals: Vec<String>,
    used: bool,
}

struct Scanner {
    legacy: Regex,
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
                paths,
                literals: entry.current,
                used: false,
            });
        }
        Ok(Self {
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
                if !rule.paths.iter().any(|pattern| pattern.matches(path)) {
                    continue;
                }
                for literal in &rule.literals {
                    // Only the actual legacy occurrence inside a permitted literal is exempt.
                    if text.match_indices(literal).any(|(start, value)| {
                        start <= hit.start() && start + value.len() >= hit.end()
                    }) {
                        rule.used = true;
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
        for rule in self.rules {
            if !rule.used {
                self.errors.push(format!(
                    "brand.toml: stale compatibility.{} (no legacy occurrence matched)",
                    rule.name
                ));
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
        assert_eq!(errors.len(), 2);
        assert!(errors[1].contains("stale compatibility.format"));
    }
}
