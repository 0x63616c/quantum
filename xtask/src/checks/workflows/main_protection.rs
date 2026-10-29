//! Contracts for the ruleset and its required workflow results.

use serde_json::{Value, json};

const MACOS: &str = include_str!("../../../../.github/workflows/ci-macos.yml");

fn ruleset() -> Value {
    serde_json::from_str(include_str!(
        "../../../../.github/rulesets/protect-main.json"
    ))
    .unwrap()
}

#[test]
fn main_ruleset_requires_complete_policy() {
    let payload = ruleset();
    assert_eq!(payload["bypass_actors"], json!([]));
    assert_eq!(payload["enforcement"], "active");
    assert_eq!(payload["target"], "branch");
    assert_eq!(
        payload["conditions"]["ref_name"],
        json!({"include": ["~DEFAULT_BRANCH"], "exclude": []})
    );
    let rules = payload["rules"].as_array().unwrap();
    for kind in [
        "deletion",
        "non_fast_forward",
        "pull_request",
        "required_status_checks",
    ] {
        assert_eq!(rules.iter().filter(|rule| rule["type"] == kind).count(), 1);
    }
    assert_eq!(rules.len(), 4);
    let checks = rules
        .iter()
        .find(|rule| rule["type"] == "required_status_checks")
        .unwrap();
    assert_eq!(
        checks["parameters"]["strict_required_status_checks_policy"],
        true
    );
    assert_eq!(checks["parameters"]["do_not_enforce_on_create"], false);
    let required = checks["parameters"]["required_status_checks"]
        .as_array()
        .unwrap();
    let mut contexts: Vec<_> = required
        .iter()
        .map(|check| {
            assert_eq!(check["integration_id"], 15368);
            check["context"].as_str().unwrap()
        })
        .collect();
    contexts.sort_unstable();
    assert_eq!(
        contexts,
        ["Conventional Commit title", "Ticket key", "linux", "macos"]
    );
    let pr = rules
        .iter()
        .find(|rule| rule["type"] == "pull_request")
        .unwrap();
    assert_eq!(pr["parameters"]["required_approving_review_count"], 0);
    assert_eq!(pr["parameters"]["require_code_owner_review"], false);
    assert_eq!(pr["parameters"]["require_last_push_approval"], false);
}

#[test]
fn required_contexts_are_reported_for_every_pr() {
    for (source, jobs) in [
        (
            include_str!("../../../../.github/workflows/ci-linux.yml"),
            vec![("linux", "linux")],
        ),
        (MACOS, vec![("macos-required", "macos")]),
        (
            include_str!("../../../../.github/workflows/pr-conventions.yml"),
            vec![
                ("title", "Conventional Commit title"),
                ("ticket-key", "Ticket key"),
            ],
        ),
    ] {
        let workflow: Value = serde_saphyr::from_str(source).unwrap();
        let trigger = workflow["on"].get("pull_request").unwrap();
        assert!(trigger.is_null() || trigger.is_object());
        for filter in ["paths", "paths-ignore", "branches", "branches-ignore"] {
            assert!(trigger.get(filter).is_none(), "{filter} can omit checks");
        }
        if let Some(types) = trigger.get("types") {
            for event in ["opened", "reopened", "synchronize"] {
                assert!(types.as_array().unwrap().contains(&json!(event)));
            }
        }
        if jobs.iter().any(|(id, _)| *id == "ticket-key") {
            // Editing a title or removing an issue link must invalidate its check.
            assert!(
                trigger["types"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("edited"))
            );
        }
        for (id, context) in jobs {
            let job = &workflow["jobs"][id];
            assert!(job.is_object());
            assert_eq!(job["name"].as_str().unwrap_or(id), context);
            assert!(job.get("strategy").is_none(), "matrix changes check names");
            assert!(job.get("continue-on-error").is_none());
        }
    }
}

#[test]
fn macos_required_check_observes_every_prerequisite() {
    let workflow: Value = serde_saphyr::from_str(MACOS).unwrap();
    let job = &workflow["jobs"]["macos-required"];
    assert_eq!(job["if"], "always()");
    assert_eq!(job["needs"], json!(["changes", "scripts", "macos"]));
    assert_eq!(job["runs-on"], "ubuntu-24.04");
    let steps = job["steps"].as_array().unwrap();
    assert_eq!(steps.len(), 1);
    let step = &steps[0];
    assert!(step.get("if").is_none());
    assert!(step.get("continue-on-error").is_none());
    assert_eq!(step["shell"], "bash");
    for (name, expression) in [
        ("CHANGES_RESULT", "needs.changes.result"),
        ("CODE_CHANGED", "needs.changes.outputs.code"),
        ("SCRIPTS_RESULT", "needs.scripts.result"),
        ("MACOS_RESULT", "needs.macos.result"),
    ] {
        assert_eq!(step["env"][name], format!("${{{{ {expression} }}}}"));
    }
    let native = &workflow["jobs"]["macos"];
    assert_eq!(native["name"], "native-macos");
    assert_eq!(native["env"]["CARGO_BUILD_JOBS"], 3);
    assert_eq!(
        native["if"],
        "needs.changes.outputs.code == 'true' && (github.event_name == 'pull_request' || github.event_name == 'push')"
    );
}

#[test]
fn macos_required_check_rejects_failed_or_missing_results() {
    let workflow: Value = serde_saphyr::from_str(MACOS).unwrap();
    let script = workflow["jobs"]["macos-required"]["steps"][0]["run"]
        .as_str()
        .unwrap();
    let results = ["success", "failure", "cancelled", "skipped", ""];
    for changes in results {
        for scripts in results {
            for native in results {
                for code in ["true", "false", ""] {
                    let output = std::process::Command::new("bash")
                        .args(["--noprofile", "--norc", "-c", script])
                        .env_clear()
                        .env("CHANGES_RESULT", changes)
                        .env("CODE_CHANGED", code)
                        .env("SCRIPTS_RESULT", scripts)
                        .env("MACOS_RESULT", native)
                        .output()
                        .unwrap();
                    let expected = changes == "success"
                        && matches!(
                            (code, scripts, native),
                            ("true", "success", "success") | ("false", "skipped", "skipped")
                        );
                    assert_eq!(
                        output.status.success(),
                        expected,
                        "changes={changes}, code={code}, scripts={scripts}, native={native}: {output:?}"
                    );
                }
            }
        }
    }
}
