//! Execute the workflow filters with paginated GitHub file-list fixtures.

use super::*;
use std::{collections::BTreeMap, fs, os::unix::fs::PermissionsExt, process::Command};

fn filter(
    source: &str,
    event: &str,
    reference: &str,
    files: &str,
    api_fails: bool,
) -> Option<BTreeMap<String, String>> {
    let workflow = workflow(source);
    let script = workflow["jobs"]["changes"]["steps"][0]["run"]
        .as_str()
        .unwrap();
    let root = tempfile::tempdir().unwrap();
    let gh = root.path().join("gh");
    // The fixture receives the real invocation. Require rename-aware pagination,
    // then supply filenames as gh's jq projection would, including previous names.
    fs::write(&gh, "#!/bin/bash\n[[ \"$*\" == *--paginate* && \"$*\" == *previous_filename* ]] || exit 97\n[[ \"$API_FAILS\" == false ]] || exit 1\nprintf '%s\\n' \"$CHANGED_FILES\"\n").unwrap();
    fs::set_permissions(&gh, fs::Permissions::from_mode(0o755)).unwrap();
    let output_file = root.path().join("output");
    let output = Command::new("bash")
        .args(["-euo", "pipefail", "-c", script])
        .env(
            "PATH",
            format!(
                "{}:{}",
                root.path().display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .env("GITHUB_EVENT_NAME", event)
        .env("GITHUB_REF", reference)
        .env("GITHUB_REPOSITORY", "fixture/quantum")
        .env("PR_NUMBER", "235")
        .env("GITHUB_OUTPUT", &output_file)
        .env("CHANGED_FILES", files)
        .env("API_FAILS", api_fails.to_string())
        .output()
        .unwrap();
    if api_fails {
        assert!(!output.status.success());
        assert!(
            !output_file.exists(),
            "API failure must not become a successful skip"
        );
        return None;
    }
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Some(
        fs::read_to_string(output_file)
            .unwrap()
            .lines()
            .map(|line| {
                let (key, value) = line.split_once('=').unwrap();
                (key.to_owned(), value.to_owned())
            })
            .collect(),
    )
}

fn macos(files: &str) -> [bool; 3] {
    let result = filter(MACOS, "pull_request", "refs/pull/235/merge", files, false).unwrap();
    ["code", "package", "metal"].map(|key| result[key] == "true")
}

#[test]
fn macos_pr_scope_covers_sources_dependencies_packaging_and_renames() {
    for (files, expected) in [
        ("docs/ci-runners.md", [false, false, false]),
        ("README.md\nCONTRIBUTING.md", [false, false, false]),
        ("site/content/index.md", [true, false, false]),
        ("crates/quantum-site/src/main.rs", [true, false, false]),
        ("spec/mcp/v1/tools.json", [true, false, false]),
        ("crates/quantum-format/src/lib.rs", [true, false, true]),
        ("crates/quantum-render-plan/src/lib.rs", [true, false, true]),
        (".github/workflows/pr-conventions.yml", [true, false, false]),
        ("crates/quantum-app/src/ui.rs", [true, false, true]),
        (
            "crates/quantum-render/src/gpu/tests/profiling.rs",
            [true, false, true],
        ),
        ("crates/quantum-core/src/lib.rs", [true, false, true]),
        ("crates/quantum-mesh/src/lib.rs", [true, false, true]),
        ("crates/quantum-ui/src/text_field.rs", [true, false, false]),
        ("crates/quantum-mcp/src/lib.rs", [true, false, false]),
        (
            "crates/plugins/quantum-plugin-tools/src/lib.rs",
            [true, false, false],
        ),
        (
            "vendor/gpui-apple/src/metal_renderer.rs",
            [true, false, false],
        ),
        ("examples/autumn-workshop.voxl", [true, false, false]),
        ("Cargo.toml", [true, true, true]),
        ("Cargo.lock", [true, true, true]),
        ("rust-toolchain.toml", [true, true, true]),
        (".cargo/config.toml", [true, true, true]),
        (".config/nextest.toml", [true, false, true]),
        ("packaging/macos/Info.plist", [true, true, false]),
        ("crates/quantum-app/Cargo.toml", [true, true, true]),
        ("xtask/src/macos/package.rs", [true, true, false]),
        (".github/workflows/ci-macos.yml", [true, true, true]),
        (".github/workflows/release-macos.yml", [true, true, false]),
        (
            "docs/moved.rs\ncrates/quantum-app/src/old.rs",
            [true, false, true],
        ),
        ("", [true, true, true]),
    ] {
        assert_eq!(macos(files), expected, "{files}");
    }
}

#[test]
fn every_main_push_runs_rust_and_metal_and_nightly_also_packages() {
    for (event, reference, package) in [
        ("push", "refs/heads/main", "false"),
        ("schedule", "refs/heads/main", "true"),
        ("workflow_dispatch", "refs/heads/main", "true"),
        (
            "workflow_dispatch",
            "refs/heads/ci/235-fast-pr-feedback",
            "false",
        ),
    ] {
        // Docs-only input must not narrow push, scheduled or manual coverage.
        let result = filter(MACOS, event, reference, "docs/ci-runners.md", false).unwrap();
        assert_eq!(result["code"], "true");
        assert_eq!(result["metal"], "true");
        assert_eq!(result["package"], package);
    }
}

#[test]
fn rendering_keeps_its_existing_scope_and_always_runs_nightly() {
    for (files, expected) in [
        ("docs/ci-runners.md", "false"),
        ("Cargo.lock", "false"),
        ("Cargo.toml", "false"),
        ("crates/quantum-render/src/lib.rs", "true"),
        ("crates/quantum-core/src/lib.rs", "true"),
        ("crates/quantum-bench/src/lib.rs", "true"),
        ("xtask/src/main.rs", "true"),
        ("bench/pinned-machine.toml", "true"),
        ("examples/autumn-workshop.voxl", "true"),
        (".cargo/config.toml", "true"),
        (".github/workflows/render-checks.yml", "true"),
        ("docs/moved.rs\ncrates/quantum-render/src/old.rs", "true"),
        ("", "true"),
    ] {
        let result = filter(RENDER, "pull_request", "refs/pull/235/merge", files, false).unwrap();
        assert_eq!(result["render"], expected, "{files}");
    }
    for event in ["schedule", "workflow_dispatch"] {
        let result = filter(RENDER, event, "refs/heads/main", "Cargo.lock", false).unwrap();
        assert_eq!(result["render"], "true");
    }
}

#[test]
fn failed_file_lookup_fails_both_filters() {
    for source in [MACOS, RENDER] {
        assert!(filter(source, "pull_request", "refs/pull/235/merge", "", true).is_none());
    }
}
