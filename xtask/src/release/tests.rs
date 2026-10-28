use super::*;
use crate::{
    cmd::{Output, SystemRunner},
    testing::RecordingRunner,
};
use std::fs;

fn fixture() -> (tempfile::TempDir, Workspace) {
    let dir = tempfile::tempdir().unwrap();
    let source = Workspace::discover().unwrap();
    for path in [
        "Cargo.toml",
        "packaging/macos/RELEASE_NOTES.md",
        "Cargo.lock",
        "README.md",
        "xtask/Cargo.toml",
        "spec/mcp/v1/compatibility.json",
    ] {
        let to = dir.path().join(path);
        fs::create_dir_all(to.parent().unwrap()).unwrap();
        fs::copy(source.path(path), to).unwrap();
    }
    let ws = Workspace::at(dir.path()).unwrap();
    (dir, ws)
}

fn replies(values: &[&str]) -> RecordingRunner {
    let runner = RecordingRunner::default();
    for value in values {
        runner.results.borrow_mut().push_back(Ok(Output {
            stdout: value.as_bytes().to_vec(),
            ..Output::default()
        }));
    }
    runner
}
const TRACKED: &str = "Cargo.toml\0Cargo.lock\0README.md\0spec/mcp/v1/compatibility.json\0";
const GREEN: &str = r#"{"total_count":1,"check_runs":[{"head_sha":"abc","status":"completed","conclusion":"success"}]}"#;

#[test]
fn fixture_commit_logs_cover_groups_breaking_links_and_empty_history() {
    for (log, expected) in [
        ("", "No changes."),
        (
            "feat(ui)!: new tools (#12)\0BREAKING CHANGE: old tools removed\0\nfix: café\0\0\nperf(mesh): faster\0\0\nrefactor: simplify\0\0\nchore: maintenance\0\0\nlegacy subject\0\0",
            "old tools removed",
        ),
        ("fix(core): bounds (#x)\0\0", "bounds (#x)"),
        ("feat: new\0BREAKING-CHANGE: migration\0", "migration"),
    ] {
        let out = changelog(log, "1.2.3");
        assert!(out.contains(expected), "{out}");
        if log.contains("#12") {
            for text in [
                "### Added",
                "### Fixed",
                "### Performance",
                "### Refactored",
                "### Other",
                "### Breaking changes",
                "https://github.com/0x63616c/quantum/pull/12",
                "legacy subject",
            ] {
                assert!(out.contains(text));
            }
        }
    }
}

#[test]
fn inserts_before_history_preserving_unreleased_and_preamble() {
    let old = "# Changelog\n\n## [Unreleased]\n\nPending\n\n## [1.0.0]\n\nOld\n";
    let updated = insert_changelog(old, "## [2.0.0]\n\nNew\n");
    assert!(updated.find("[2.0.0]").unwrap() < updated.find("[1.0.0]").unwrap());
    assert!(updated.starts_with(old.split_once("## [1.0.0]").unwrap().0));
    assert!(updated.ends_with("## [1.0.0]\n\nOld\n"));
}

#[test]
fn dry_run_is_byte_identical_with_no_git_mutations_or_network() {
    let (_dir, ws) = fixture();
    let before: Vec<_> = TRACKED
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|p| (p, fs::read(ws.path(p)).unwrap()))
        .collect();
    let runner = replies(&[TRACKED, "v0.1.0", "feat: new (#1)\0\0"]);
    let report = run(&ws, &runner, "patch", false).unwrap();
    for text in [
        "Dry run",
        "--- a/Cargo.toml",
        "### Added",
        "git switch -c",
        "gh pr create",
        "release tag --execute",
    ] {
        assert!(report.contains(text));
    }
    for (path, bytes) in before {
        assert_eq!(fs::read(ws.path(path)).unwrap(), bytes);
    }
    assert!(!ws.path("CHANGELOG.md").exists());
    runner.assert_args(0, &["ls-files", "-z"]);
    runner.assert_args(
        1,
        &[
            "tag",
            "--merged",
            "HEAD",
            "--list",
            "v*",
            "--sort=-version:refname",
        ],
    );
    runner.assert_args(
        2,
        &[
            "log",
            "--no-merges",
            "--no-show-signature",
            "--no-color",
            "--format=%s%x00%b%x00",
            "refs/tags/v0.1.0..HEAD",
            "--",
        ],
    );
    // A real repository's tracked bytes and refs also survive a real offline preview.
    let real = Workspace::discover().unwrap();
    let system = SystemRunner::default();
    let refs = git(&real, &system, &["show-ref"]).unwrap();
    let files = real.tracked_files(&system).unwrap();
    let bytes: Vec<_> = files
        .iter()
        .map(|p| fs::read(real.path(p)).unwrap())
        .collect();
    run(&real, &system, "patch", false).unwrap();
    assert_eq!(git(&real, &system, &["show-ref"]).unwrap(), refs);
    for (path, bytes) in files.iter().zip(bytes) {
        assert_eq!(fs::read(real.path(path)).unwrap(), bytes);
    }
}

#[test]
fn execute_preparation_only_uses_expected_fake_commands() {
    let (_dir, ws) = fixture();
    let runner = replies(&[
        "",
        "main",
        "abc",
        "abc\trefs/heads/main",
        "",
        "",
        GREEN,
        TRACKED,
        "",
        "fix: safe\0\0",
        "",
        "",
        "",
        "",
    ]);
    run(&ws, &runner, "patch", true).unwrap();
    let next = version::next_version(
        &version::workspace_version(&Workspace::discover().unwrap()).unwrap(),
        "patch",
    )
    .unwrap();
    runner.assert_args(10, &["switch", "-c", &format!("chore/release-v{next}")]);
    runner.assert_args(11, &["update", "--workspace", "--offline"]);
    runner.assert_args(
        12,
        &[
            "add",
            "--",
            "Cargo.toml",
            "README.md",
            "spec/mcp/v1/compatibility.json",
            "Cargo.lock",
            "CHANGELOG.md",
        ],
    );
    runner.assert_args(13, &["commit", "-m", &format!("chore(release): v{next}")]);
    assert!(ws.read("CHANGELOG.md").unwrap().contains("safe"));
}

#[test]
fn tag_execution_only_uses_fake_runner_and_checks_merged_commit() {
    let (_dir, ws) = fixture();
    let version = version::workspace_version(&ws).unwrap();
    fs::write(
        ws.path("CHANGELOG.md"),
        format!("# Changelog\n\n## [Unreleased]\n\n## [{version}]\n"),
    )
    .unwrap();
    let title = format!("chore(release): v{version}");
    let runner = replies(&[
        "",
        "main",
        "abc",
        "abc\trefs/heads/main",
        "",
        "",
        GREEN,
        &title,
        "abc parent",
        "",
        "",
    ]);
    run(&ws, &runner, "tag", true).unwrap();
    runner.assert_args(
        9,
        &[
            "tag",
            "-a",
            &format!("v{version}"),
            "-m",
            &format!("Release v{version}"),
            "abc",
        ],
    );
    runner.assert_args(
        10,
        &[
            "push",
            "origin",
            &format!("refs/tags/v{version}:refs/tags/v{version}"),
        ],
    );
    let runner = replies(&[]);
    run(&ws, &runner, "tag", false).unwrap();
    assert!(runner.commands.borrow().is_empty());
}

#[test]
fn preflight_fails_closed_before_any_mutation() {
    let (_dir, ws) = fixture();
    for values in [
        vec!["dirty"],
        vec!["", "feature"],
        vec!["", "main", "abc", "def\trefs/heads/main"],
        vec!["", "main", "abc", "abc", "v1.0.0"],
        vec!["", "main", "abc", "abc", "", "remote-tag"],
        vec![
            "",
            "main",
            "abc",
            "abc",
            "",
            "",
            "{\"total_count\":0,\"check_runs\":[]}",
        ],
    ] {
        let runner = replies(&values);
        assert!(run(&ws, &runner, "patch", true).is_err());
        assert!(!ws.path("CHANGELOG.md").exists());
    }
}

#[test]
fn ci_and_tag_identity_failures_never_reach_mutation_commands() {
    let (_dir, ws) = fixture();
    for checks in [
        r#"{"total_count":1,"check_runs":[{"head_sha":"abc","status":"completed","conclusion":"failure"}]}"#,
        r#"{"total_count":1,"check_runs":[{"head_sha":"abc","status":"queued","conclusion":null}]}"#,
        r#"{"total_count":1,"check_runs":[{"head_sha":"abc","status":"completed","conclusion":"skipped"}]}"#,
        r#"{"total_count":1,"check_runs":[{"head_sha":"other","status":"completed","conclusion":"success"}]}"#,
        r#"{"total_count":101,"check_runs":[]}"#,
        "invalid json",
    ] {
        let runner = replies(&["", "main", "abc", "abc", "", "", checks]);
        assert!(run(&ws, &runner, "tag", true).is_err());
        assert_eq!(runner.commands.borrow().len(), 7);
    }
    let title = format!(
        "chore(release): v{}",
        version::workspace_version(&ws).unwrap()
    );
    for suffix in [
        vec!["unrelated subject"],
        vec![&title, "abc parent1 parent2"],
        vec![&title, "abc parent"],
    ] {
        let mut values = vec!["", "main", "abc", "abc", "", "", GREEN];
        values.extend(suffix);
        let runner = replies(&values);
        assert!(run(&ws, &runner, "tag", true).is_err());
    }
}
