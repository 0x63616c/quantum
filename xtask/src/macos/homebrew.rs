//! Deterministic Homebrew casks for the renamed release bundle.
use super::{invalid, stable_version};
use crate::Result;
use std::{fs, path::Path};

const TEMPLATE: &str = include_str!("../../../packaging/homebrew/quantum.rb");

fn render(version: &str, sha256: &str) -> Result<String> {
    if stable_version(version)? < stable_version("0.4.0")? {
        return Err(invalid("Homebrew template requires version 0.4.0 or later"));
    }
    if sha256.len() != 64 || !sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid("Expected a 64-digit hexadecimal SHA-256"));
    }
    Ok(TEMPLATE
        .replace("@@VERSION@@", version)
        .replace("@@SHA256@@", &sha256.to_ascii_lowercase()))
}

pub(super) fn write(version: &str, sha256: &str, output: &Path) -> Result<()> {
    let cask = render(version, sha256)?;
    fs::write(output, cask)
        .map_err(|error| invalid(&format!("Cannot write Homebrew cask: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendered_cask_matches_golden() {
        assert_eq!(
            render("0.4.0", &"AB".repeat(32)).unwrap(),
            include_str!("tests/fixtures/homebrew.rb")
        );
    }

    #[test]
    fn rejects_invalid_release_metadata() {
        for version in ["0.3.1", "v0.4.0", "0.4.0-beta", "01.4.0", "1.2", "1.2.3\n"] {
            assert!(render(version, &"a".repeat(64)).is_err());
        }
        for hash in [
            "a".repeat(63),
            "g".repeat(64),
            format!("{}\n", "a".repeat(63)),
        ] {
            assert!(render("0.4.0", &hash).is_err());
        }
        assert!(render("1.2.3", &"0".repeat(64)).is_ok());
    }
}
