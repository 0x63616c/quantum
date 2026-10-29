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

#[test]
fn every_self_hosted_job_requires_a_guard_and_build_limit() {
    let good = serde_json::json!({
        "runs-on": ["self-hosted", "macOS", "ARM64", "xcode"],
        "if": OWNER_GUARD,
        "env": {"CARGO_BUILD_JOBS": 4},
    });
    let mut bad = good.clone();
    bad["if"] = "true".into();
    bad["env"] = serde_json::json!({});
    let mut errors = Vec::new();
    audit_jobs(
        &serde_json::json!({"jobs": {"safe": good, "unsafe": bad}}),
        &mut errors,
    );
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().all(|error| error.starts_with("unsafe:")));
}

#[test]
fn guard_cannot_be_bypassed_by_or_or_an_unbalanced_prefix() {
    for prefix in [
        "",
        "needs.changes.outputs.code == 'true' && ",
        "(a || b) && ",
    ] {
        assert!(owner_guarded(&format!("{prefix}{OWNER_GUARD}")));
    }
    for prefix in ["true || ", "true || false && ", "(true && ", "' && "] {
        assert!(!owner_guarded(&format!("{prefix}{OWNER_GUARD}")));
    }
    assert!(owner_guarded(&format!("${{{{ {OWNER_GUARD} }}}}")));
    assert!(!owner_guarded(&format!("{OWNER_GUARD} || true")));
}

#[test]
fn dynamic_self_hosted_runner_and_privileged_trigger_are_audited() {
    let mut errors = Vec::new();
    audit_jobs(
        &serde_json::json!({
            "on": ["pull_request_target"],
            "jobs": {"render": {"runs-on": "${{ fromJSON('[\"self-hosted\",\"macOS\"]') }}"}}
        }),
        &mut errors,
    );
    assert_eq!(errors.len(), 3);
}

#[test]
fn concurrency_cancels_prs_but_preserves_nightly_runs() {
    let mut workflow = serde_json::json!({"on": {"pull_request": null, "schedule": []}});
    let mut errors = Vec::new();
    audit_concurrency(&workflow, &mut errors);
    assert_eq!(errors.len(), 3);
    workflow["concurrency"] = serde_json::json!({
        "group": "ci-${{ github.ref }}",
        "cancel-in-progress": true,
    });
    errors.clear();
    audit_concurrency(&workflow, &mut errors);
    assert_eq!(
        errors.len(),
        2,
        "nightly cancellation and pending replacement"
    );
    workflow["concurrency"] = serde_json::json!({
        "group": "ci-${{ github.ref }}-${{ github.event_name == 'pull_request' && 'pr' || github.run_id }}",
        "cancel-in-progress": "${{ github.event_name == 'pull_request' }}",
    });
    errors.clear();
    audit_concurrency(&workflow, &mut errors);
    assert!(errors.is_empty());
}

const MACOS: &str = include_str!("../../../../.github/workflows/ci-macos.yml");
const RENDER: &str = include_str!("../../../../.github/workflows/render-checks.yml");

fn workflow(source: &str) -> Value {
    serde_saphyr::from_str(source).unwrap()
}

fn step<'a>(job: &'a Value, name: &str) -> &'a Value {
    job["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|step| step["name"] == name)
        .unwrap_or_else(|| panic!("missing step {name}"))
}

#[test]
fn hosted_macos_keeps_the_required_context_toolchain_and_checks_without_secrets() {
    let workflow = workflow(MACOS);
    assert!(has_event(&workflow, "pull_request"));
    assert!(workflow["on"]["pull_request"].is_null());
    let job = &workflow["jobs"]["macos"];
    assert_eq!(job["runs-on"], "xcode-27");
    assert!(job["name"].is_null(), "preserve the macos check context");
    assert_eq!(
        job["if"],
        "needs.changes.outputs.code == 'true' && (github.event_name == 'pull_request' || github.event_name == 'push')"
    );
    for forbidden in ["secrets.", "vars.", "github.actor", "head.repo"] {
        assert!(!job.to_string().contains(forbidden));
    }
    assert_eq!(
        job["env"]["DEVELOPER_DIR"],
        "/Applications/Xcode_27.0.app/Contents/Developer"
    );
    assert_eq!(
        step(job, "Install pinned Rust toolchain")["run"],
        "rustup show"
    );
    let steps = job["steps"].as_array().unwrap();
    let cache = steps
        .iter()
        .position(|step| {
            step["uses"]
                .as_str()
                .is_some_and(|value| value.starts_with("Swatinem/rust-cache@"))
        })
        .unwrap();
    let rustup = steps
        .iter()
        .position(|step| step["name"] == "Install pinned Rust toolchain")
        .unwrap();
    assert!(rustup < cache);
    assert_eq!(steps[0]["with"]["persist-credentials"], false);
    assert_eq!(
        step(job, "Install nextest")["with"]["tool"],
        "cargo-nextest@0.9.150"
    );
    assert_eq!(
        step(job, "Select Rust packages")["run"]
            .as_str()
            .unwrap()
            .lines()
            .last()
            .unwrap(),
        "echo 'packages=-p quantum-app -p quantum-ui -p quantum-render -p quantum-mcp -p quantum-bench' >> \"$GITHUB_OUTPUT\""
    );
    assert_eq!(
        step(job, "Test Rust packages")["run"],
        "cargo nextest run ${{ steps.rust-packages.outputs.packages }} --locked --features quantum-app/test-support --profile ci"
    );
    assert_eq!(
        step(job, "Test doctests")["run"],
        "cargo test --doc ${{ steps.rust-packages.outputs.packages }} --locked --features quantum-app/test-support"
    );
    assert_eq!(
        step(job, "Build app and unsigned DMG")["env"]["PACKAGE_CARGO_PROFILE"],
        "ci-dist"
    );
    assert_eq!(
        step(job, "Build app and unsigned DMG")["if"],
        "needs.changes.outputs.package == 'true'"
    );
    assert_eq!(
        step(job, "Build app and unsigned DMG")["run"],
        "cargo xtask macos package"
    );
}

#[test]
fn hardware_checks_and_nightly_reporting_preserve_every_moved_check() {
    let workflow = workflow(MACOS);
    let job = &workflow["jobs"]["macos-hardware"];
    assert!(owner_guarded(job["if"].as_str().unwrap()));
    assert!(job["if"].as_str().unwrap().starts_with(
        "(github.event_name != 'pull_request' || needs.changes.outputs.metal == 'true') && "
    ));
    assert_eq!(
        step(job, "Test opt-in profiling on Metal")["run"],
        "cargo nextest run -p quantum-render --lib --locked --features profiling,gpu --profile ci --run-ignored only -E 'test(profiling_gpu_zones)'"
    );
    assert!(step(job, "Test opt-in profiling on Metal")["if"].is_null());
    assert_eq!(
        step(job, "Test workspace")["run"],
        "cargo nextest run --workspace --locked --features quantum-app/test-support --profile ci"
    );
    for name in ["Clippy workspace", "Test workspace"] {
        assert_eq!(
            step(job, name)["if"],
            "github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'"
        );
    }
    let hosted = &workflow["jobs"]["macos"];
    let hosted_clippy = step(hosted, "Clippy")["run"].as_str().unwrap();
    assert_eq!(
        step(job, "Clippy workspace")["run"],
        hosted_clippy.replace("${{ steps.rust-packages.outputs.packages }}", "--workspace")
    );
    assert_eq!(
        step(job, "Test doctests")["run"],
        "cargo test --doc --workspace --locked --features quantum-app/test-support"
    );
    assert_eq!(
        step(job, "Build app and unsigned DMG")["env"]["PACKAGE_CARGO_PROFILE"],
        "release"
    );
    let reporter = &workflow["jobs"]["report-nightly-failure"];
    assert_eq!(
        reporter["needs"],
        serde_json::json!(["changes", "scripts", "macos-hardware"])
    );
    for job in ["changes", "scripts", "macos-hardware"] {
        assert!(
            reporter["if"]
                .as_str()
                .unwrap()
                .contains(&format!("needs.{job}.result != 'success'"))
        );
    }
    assert!(
        step(reporter, "Open or update nightly failure issue")["run"]
            .as_str()
            .unwrap()
            .contains("type:ci,area:ci,priority:p0")
    );
}

#[test]
fn rendering_remains_a_hardware_gate_with_job_level_filters_and_nightly_reporting() {
    let workflow = workflow(RENDER);
    assert!(workflow["on"]["pull_request"].is_null());
    let job = &workflow["jobs"]["render"];
    assert_eq!(job["needs"], "changes");
    assert!(owner_guarded(job["if"].as_str().unwrap()));
    assert!(job["runs-on"].as_str().unwrap().contains("self-hosted"));
    let gate = step(job, "Visual gate and determinism");
    assert!(
        gate["run"]
            .as_str()
            .unwrap()
            .contains("args=(visual --determinism)")
    );
    assert!(gate["continue-on-error"].is_null());
    let reporter = &workflow["jobs"]["report-nightly-failure"];
    assert_eq!(reporter["needs"], serde_json::json!(["changes", "render"]));
    assert!(
        reporter["if"]
            .as_str()
            .unwrap()
            .contains("needs.render.result != 'success'")
    );
}

#[cfg(unix)]
#[path = "filters.rs"]
mod filters;
