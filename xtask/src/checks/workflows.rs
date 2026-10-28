//! Validate workflow action references and explicit default permissions.

use serde_yaml_ng::Value;

use crate::{
    Error, Result,
    check::{Context, Outcome},
    workspace::Workspace,
};

pub(crate) fn run(context: &Context<'_>) -> Outcome {
    super::outcome(check(context.workspace))
}

fn check(workspace: &Workspace) -> Result<Vec<String>> {
    let mut errors = Vec::new();
    for path in super::paths(workspace, ".github/workflows/*")? {
        if !matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("yml" | "yaml")
        ) {
            continue;
        }
        let label = path
            .strip_prefix(workspace.root())
            .unwrap_or(&path)
            .display()
            .to_string();
        let source = workspace.read(&path)?;
        errors.extend(
            audit(&source)
                .map_err(|error| Error::Invalid(format!("{label}: {error}")))?
                .into_iter()
                .map(|error| format!("{label}: {error}")),
        );
    }
    Ok(errors)
}

fn audit(source: &str) -> std::result::Result<Vec<String>, serde_yaml_ng::Error> {
    let workflow: Value = serde_yaml_ng::from_str(source)?;
    let mut errors = Vec::new();
    if !workflow
        .as_mapping()
        .is_some_and(|map| map.contains_key(Value::from("permissions")))
    {
        errors.push("missing top-level permissions key".into());
    }
    // Inspect semantic YAML, including flow maps and quoted keys, while ignoring
    // shell scripts and comments containing the word 'uses'.
    inspect(&workflow, &mut errors);
    Ok(errors)
}

fn inspect(value: &Value, errors: &mut Vec<String>) {
    match value {
        Value::Mapping(map) => {
            for (key, value) in map {
                if key.as_str() == Some("uses") && !value.as_str().is_some_and(pinned) {
                    errors.push("uses must reference a full 40-hex commit SHA (or a local action / digest-pinned Docker image)".into());
                }
                inspect(value, errors);
            }
        }
        Value::Sequence(values) => {
            for value in values {
                inspect(value, errors);
            }
        }
        _ => {}
    }
}

fn pinned(reference: &str) -> bool {
    if reference.starts_with("./") {
        return true;
    }
    if let Some(image) = reference.strip_prefix("docker://") {
        return image
            .rsplit_once("@sha256:")
            .is_some_and(|(name, digest)| !name.is_empty() && hex(digest, 64));
    }
    reference
        .rsplit_once('@')
        .is_some_and(|(name, sha)| !name.is_empty() && hex(sha, 40))
}

fn hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references_require_complete_immutable_pins() {
        for reference in [
            format!("actions/checkout@{}", "a".repeat(40)),
            format!("owner/repo/path@{}", "A".repeat(40)),
            "./.github/actions/local".into(),
            format!("docker://alpine@sha256:{}", "1".repeat(64)),
        ] {
            assert!(pinned(&reference), "{reference}");
        }
        for reference in [
            "actions/checkout@v4".into(),
            format!("actions/checkout@{}", "a".repeat(39)),
            format!("actions/checkout@{}", "g".repeat(40)),
            "docker://alpine:latest".into(),
            "docker://alpine@sha256:123".into(),
            "${{ inputs.action }}".into(),
        ] {
            assert!(!pinned(&reference), "{reference}");
        }
    }

    #[test]
    fn yaml_structure_covers_reusable_workflows_and_flow_steps() {
        let source = "permissions: {}\njobs:\n  reusable:\n    uses: owner/repo/.github/workflows/test.yml@main\n  test:\n    steps:\n      - {'uses': 'actions/checkout@v4'}\n      - run: |\n          uses: shell text\n";
        assert_eq!(audit(source).unwrap().len(), 2);
        assert!(
            audit("permissions: {contents: read}\njobs: {}")
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            audit("jobs: {test: {permissions: {}}}").unwrap(),
            ["missing top-level permissions key"]
        );
        assert!(audit("jobs: [").is_err());
    }

    #[test]
    fn repository_workflows_pass() {
        assert!(check(&Workspace::discover().unwrap()).unwrap().is_empty());
    }
}
