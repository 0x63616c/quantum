//! Validate workflow pins, permissions, runner isolation and PR cancellation.

use serde_json::Value;

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

fn audit(source: &str) -> std::result::Result<Vec<String>, serde_saphyr::Error> {
    let workflow: Value = serde_saphyr::from_str(source)?;
    let mut errors = Vec::new();
    if !workflow
        .as_object()
        .is_some_and(|map| map.contains_key("permissions"))
    {
        errors.push("missing top-level permissions key".into());
    }
    // Inspect semantic YAML, including flow maps and quoted keys, while ignoring
    // shell scripts and comments containing the word 'uses'.
    inspect(&workflow, &mut errors);
    audit_jobs(&workflow, &mut errors);
    audit_concurrency(&workflow, &mut errors);
    Ok(errors)
}

const OWNER_GUARD: &str = "github.actor == '0x63616c' && (github.event_name != 'pull_request' || github.event.pull_request.head.repo.full_name == github.repository)";

fn audit_jobs(workflow: &Value, errors: &mut Vec<String>) {
    if has_event(workflow, "pull_request_target") {
        errors.push("pull_request_target is forbidden; use isolated pull_request jobs".into());
    }
    let Some(jobs) = workflow["jobs"].as_object() else {
        return;
    };
    for (name, job) in jobs {
        // Includes the fromJSON expression used to choose a rendering runner.
        if !job["runs-on"].to_string().contains("self-hosted") {
            continue;
        }
        if !job["if"].as_str().is_some_and(owner_guarded) {
            errors.push(format!("{name}: self-hosted job must end its condition with the owner/same-repository guard as a top-level conjunction"));
        }
        if job["env"]["CARGO_BUILD_JOBS"] != 4 {
            errors.push(format!(
                "{name}: self-hosted job must set CARGO_BUILD_JOBS: 4"
            ));
        }
    }
}

fn owner_guarded(condition: &str) -> bool {
    let condition = condition.trim();
    let condition = condition
        .strip_prefix("${{")
        .and_then(|s| s.strip_suffix("}}"))
        .map_or(condition, str::trim);
    if condition == OWNER_GUARD {
        return true;
    }
    let Some(prefix) = condition.strip_suffix(&format!(" && {OWNER_GUARD}")) else {
        return false;
    };
    // A suffix alone is insufficient: `allowed || condition && guard` bypasses it.
    // Require balanced parentheses/quotes and no OR outside parentheses in the prefix.
    let mut depth = 0_u32;
    let mut quoted = false;
    for ch in prefix.chars() {
        if ch == '\'' {
            quoted = !quoted;
        } else if !quoted {
            match ch {
                '(' => depth += 1,
                ')' => match depth.checked_sub(1) {
                    Some(value) => depth = value,
                    None => return false,
                },
                '|' if depth == 0 => return false,
                _ => {}
            }
        }
    }
    !prefix.trim().is_empty() && depth == 0 && !quoted
}

fn has_event(workflow: &Value, event: &str) -> bool {
    let trigger = &workflow["on"];
    trigger.get(event).is_some()
        || trigger.as_str() == Some(event)
        || trigger
            .as_array()
            .is_some_and(|events| events.iter().any(|value| value == event))
}

fn audit_concurrency(workflow: &Value, errors: &mut Vec<String>) {
    let concurrency = &workflow["concurrency"];
    if has_event(workflow, "pull_request") {
        let group = concurrency["group"].as_str().unwrap_or_default();
        if !group.contains("github.ref") && !group.contains("github.event.pull_request.number") {
            errors.push("PR concurrency group must identify the PR ref or number".into());
        }
        let cancel = &concurrency["cancel-in-progress"];
        let cancels_prs = cancel == true
            || matches!(
                cancel.as_str(),
                Some("${{ github.event_name == 'pull_request' }}")
                    | Some(
                        "${{ github.event_name != 'schedule' && github.event_name != 'workflow_dispatch' }}"
                    )
            );
        if !cancels_prs {
            errors.push("PR workflows must cancel superseded runs for the same PR".into());
        }
    }
    if has_event(workflow, "schedule") {
        if concurrency["cancel-in-progress"] == true {
            errors.push("scheduled runs must not be cancelled by newer runs".into());
        }
        if !concurrency["group"]
            .as_str()
            .is_some_and(|group| group.contains("github.run_id"))
        {
            errors.push(
                "scheduled runs need a run-specific concurrency group to preserve pending coverage"
                    .into(),
            );
        }
    }
}

fn inspect(value: &Value, errors: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key == "uses" && !value.as_str().is_some_and(pinned) {
                    let reference = value
                        .as_str()
                        .map_or_else(|| value.to_string(), str::to_owned);
                    errors.push(format!(
                        "uses: {reference} must reference a full 40-hex commit SHA (or a local action / digest-pinned Docker image)"
                    ));
                }
                inspect(value, errors);
            }
        }
        Value::Array(values) => {
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
mod tests;

#[cfg(test)]
mod protection_tests {
    use super::*;

    #[test]
    fn main_ruleset_requires_existing_unfiltered_check_contexts() {
        let payload: Value =
            serde_json::from_str(include_str!("../../../.github/rulesets/protect-main.json"))
                .unwrap();
        assert_eq!(payload["bypass_actors"], serde_json::json!([]));
        assert_eq!(payload["enforcement"], "active");
        assert_eq!(payload["target"], "branch");
        assert_eq!(
            payload["conditions"]["ref_name"],
            serde_json::json!({"include": ["~DEFAULT_BRANCH"], "exclude": []})
        );
        let rules = payload["rules"].as_array().unwrap();
        for kind in ["deletion", "non_fast_forward", "pull_request"] {
            assert!(rules.iter().any(|rule| rule["type"] == kind));
        }
        let checks = rules
            .iter()
            .find(|rule| rule["type"] == "required_status_checks")
            .unwrap();
        assert_eq!(
            checks["parameters"]["strict_required_status_checks_policy"],
            true
        );
        let required = checks["parameters"]["required_status_checks"]
            .as_array()
            .unwrap();
        let mut contexts: Vec<_> = required
            .iter()
            .map(|check| check["context"].as_str().unwrap())
            .collect();
        contexts.sort_unstable();
        assert_eq!(
            contexts,
            [
                "Conventional Commit title",
                "Linked issue",
                "linux",
                "macos"
            ]
        );
        let mut actual = Vec::new();
        for source in [
            include_str!("../../../.github/workflows/ci-linux.yml"),
            include_str!("../../../.github/workflows/ci-macos.yml"),
            include_str!("../../../.github/workflows/pr-conventions.yml"),
        ] {
            let workflow: Value = serde_saphyr::from_str(source).unwrap();
            let trigger = &workflow["on"]["pull_request"];
            assert!(!workflow["on"]["pull_request"].is_boolean());
            assert!(workflow["on"].get("pull_request").is_some());
            assert!(trigger.get("paths").is_none());
            assert!(trigger.get("paths-ignore").is_none());
            for (id, job) in workflow["jobs"].as_object().unwrap() {
                actual.push(job["name"].as_str().unwrap_or(id).to_owned());
            }
        }
        for check in checks["parameters"]["required_status_checks"]
            .as_array()
            .unwrap()
        {
            assert_eq!(check["integration_id"], 15368);
            assert!(actual.iter().any(|name| check["context"] == *name));
        }
        let pr = rules
            .iter()
            .find(|rule| rule["type"] == "pull_request")
            .unwrap();
        assert_eq!(pr["parameters"]["required_approving_review_count"], 0);
    }

}
