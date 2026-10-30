use super::*;
use crate::{
    cmd::{Command, Output, Runner, Status, SystemRunner},
    testing::RecordingRunner,
};
use std::io;

#[test]
fn stable_bumps_reset_lower_components_and_reject_invalid_requests() {
    let current = Version::new(1, 2, 3);
    for (request, expected) in [
        ("major", "2.0.0"),
        ("minor", "1.3.0"),
        ("patch", "1.2.4"),
        ("5.6.7", "5.6.7"),
        ("1.2.3", "1.2.3"),
    ] {
        assert_eq!(
            next_version(&current, request).unwrap().to_string(),
            expected
        );
    }
    for request in [
        "1.2.2",
        "0.9.9",
        "v2.0.0",
        "2.0",
        "2.0.0-rc.1",
        "2.0.0+build",
        "02.0.0",
        "",
        "major ",
    ] {
        assert!(next_version(&current, request).is_err(), "{request}");
    }
    for (current, request) in [
        (Version::new(u64::MAX, 0, 0), "major"),
        (Version::new(1, u64::MAX, 0), "minor"),
        (Version::new(1, 0, u64::MAX), "patch"),
    ] {
        assert!(next_version(&current, request).is_err());
    }
    assert!(next_version(&Version::parse("1.0.0-rc.1").unwrap(), "patch").is_err());
}

#[test]
fn manifest_edits_only_the_workspace_version_preserving_formatting() {
    let source = "[package]\r\nversion = \"9.8.7\"\r\n[workspace.package] # note\r\n  version  =  '1.2.3' # keep\r\nedition = \"2024\"\r\n[dependencies]\r\nversion = \"4.5.6\"\r\n";
    let (range, current) = manifest_version(source).unwrap();
    let updated = replace_range(source, range, &current, &Version::new(2, 0, 0)).unwrap();
    assert_eq!(updated, source.replace("'1.2.3'", "'2.0.0'"));
    for source in [
        "[package]\nversion = \"1.2.3\"",
        "[workspace.package]\nversion = 123",
        "[workspace.package]\nversion = \"1.2.3",
        "[workspace.package]\nversion = \nother = \"1.2.3\"",
        "[workspace.package]\nversion = \"1.2.3\" junk",
        "[workspace.package]\nversion = \"1.2.3\"\nversion = \"1.2.3\"",
    ] {
        assert!(manifest_version(source).is_err(), "{source}");
    }
}

#[test]
fn references_fail_closed_and_leave_history_unchanged() {
    let old = Version::new(1, 2, 3);
    let new = Version::new(1, 2, 4);
    let prefix = "The current workspace version is **";
    let source = format!("{prefix}1.2.3**.\nHistorical release 1.2.3.\n");
    assert_eq!(
        replace_reference(&source, prefix, "**", &old, &new).unwrap(),
        format!("{prefix}1.2.4**.\nHistorical release 1.2.3.\n")
    );
    for source in [
        "no marker".into(),
        format!("{prefix}1.2.2**"),
        format!("{prefix}1.2.3"),
        source.repeat(2),
    ] {
        assert!(replace_reference(&source, prefix, "**", &old, &new).is_err());
    }
    let json = "{\n  \"application_version\" : \"1.2.3\", \"schema_version\": \"1.2.3\"\n}\n";
    assert_eq!(
        compatibility_version(json, &old, &new).unwrap(),
        json.replacen("1.2.3", "1.2.4", 1)
    );
    for source in [
        "not json",
        "{}",
        "{\"application_version\":123}",
        "{\"application_version\":\"1.2.2\"}",
        "{\"application_version\":\"1.2.3\",\"application_version\":\"1.2.3\"}",
    ] {
        assert!(compatibility_version(source, &old, &new).is_err());
    }
}

fn fixture() -> (tempfile::TempDir, Workspace, RecordingRunner) {
    let directory = tempfile::tempdir().unwrap();
    let files = [
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"xtask\", \"crates/member\"]\nexclude = [\"external\"]\nresolver = \"2\"\n[workspace.package]\nversion = \"1.2.3\"\n",
        ),
        (
            "xtask/Cargo.toml",
            "[package]\nname = \"fixture\"\nversion.workspace = true\nedition = \"2024\"\n[dependencies]\nexternal = { path = \"../external\" }\n",
        ),
        ("xtask/src/lib.rs", "//! Fixture.\n"),
        (
            "crates/member/Cargo.toml",
            "[package]\nname = \"member\"\nversion.workspace = true\nedition = \"2024\"\n",
        ),
        (
            "crates/member/src/lib.rs",
            "//! Another workspace member.\n",
        ),
        (
            "external/Cargo.toml",
            "[package]\nname = \"external\"\nversion = \"9.8.7\"\nedition = \"2024\"\n",
        ),
        ("external/src/lib.rs", "//! Non-workspace dependency.\n"),
        ("README.md", "The current workspace version is **1.2.3**.\n"),
        (
            "crates/voxelite-site/templates/download.html",
            "Version 1.2.3 for macOS 13+\n",
        ),
        (
            "spec/mcp/v1/compatibility.json",
            "{\"application_version\":\"1.2.3\"}\n",
        ),
        (
            "Cargo.lock",
            "# Test lockfile\nversion = 4\n\n[[package]]\nname = \"external\"\nversion = \"9.8.7\"\n\n[[package]]\nname = \"fixture\"\nversion = \"1.2.3\"\ndependencies = [\n \"external\",\n]\n\n[[package]]\nname = \"member\"\nversion = \"1.2.3\"\n",
        ),
        ("site/appcast.xml", "historical 1.2.3"),
    ];
    for (relative, contents) in files {
        let path = directory.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
    let tracked = files
        .iter()
        .map(|(path, _)| *path)
        .collect::<Vec<_>>()
        .join("\0");
    let runner = RecordingRunner::stdout(tracked.as_bytes());
    let workspace = Workspace::at(directory.path()).unwrap();
    (directory, workspace, runner)
}

fn assert_originals(plan: &Plan) {
    for edit in &plan.edits {
        assert_eq!(plan.workspace.read(edit.path).unwrap(), edit.before);
    }
    assert_eq!(
        fs::read(plan.workspace.path("Cargo.lock")).unwrap(),
        plan.lockfile
    );
}

#[test]
fn dry_run_and_equal_versions_never_write() {
    let (_directory, workspace, runner) = fixture();
    let plan = Plan::prepare(&workspace, &runner, "minor").unwrap();
    assert_eq!(plan.changed_paths().len(), 5);
    plan.apply(&SystemRunner::dry_run()).unwrap();
    assert_originals(&plan);
    let (_directory, workspace, runner) = fixture();
    let plan = Plan::prepare(&workspace, &runner, "1.2.3").unwrap();
    assert!(plan.changed_paths().is_empty());
    plan.apply(&runner).unwrap(); // No canned result: any command would fail this test.
    assert_originals(&plan);
}

#[test]
fn update_failure_restores_every_file_including_a_modified_lockfile() {
    struct FailingUpdate;
    impl Runner for FailingUpdate {
        fn execute(&self, command: &Command, _: bool) -> io::Result<Output> {
            assert_eq!(
                command.arguments().collect::<Vec<_>>(),
                ["update", "--workspace", "--offline"]
            );
            let root = command.directory().unwrap();
            assert!(
                fs::read_to_string(root.join("Cargo.toml"))
                    .unwrap()
                    .contains("1.2.4")
            );
            fs::write(root.join("Cargo.lock"), "partially changed")?;
            Ok(Output {
                status: Status::from_code(42),
                ..Output::default()
            })
        }
    }
    let (_directory, workspace, runner) = fixture();
    let plan = Plan::prepare(&workspace, &runner, "patch").unwrap();
    assert!(
        plan.apply(&FailingUpdate)
            .unwrap_err()
            .to_string()
            .contains("42")
    );
    assert_originals(&plan);
}

#[test]
fn stale_references_and_concurrent_changes_fail_before_writing() {
    let (_directory, workspace, runner) = fixture();
    let plan = Plan::prepare(&workspace, &runner, "patch").unwrap();
    fs::write(workspace.path("README.md"), "concurrent edit").unwrap();
    assert!(
        plan.apply(&runner)
            .unwrap_err()
            .to_string()
            .contains("changed after version planning")
    );
    assert_eq!(workspace.read("Cargo.toml").unwrap(), plan.edits[0].before);
    let (_directory, workspace, runner) = fixture();
    fs::write(
        workspace.path("README.md"),
        "The current workspace version is **1.0.0**",
    )
    .unwrap();
    assert!(Plan::prepare(&workspace, &runner, "patch").is_err());
    assert!(workspace.read("Cargo.toml").unwrap().contains("1.2.3"));
    let runner = RecordingRunner::stdout(b"Cargo.toml\0");
    assert!(Plan::prepare(&workspace, &runner, "patch").is_err());
}

#[test]
fn real_offline_cargo_refreshes_workspace_member_versions() {
    let (_directory, workspace, runner) = fixture();
    let plan = Plan::prepare(&workspace, &runner, "patch").unwrap();
    plan.apply(&SystemRunner::default()).unwrap();
    for edit in &plan.edits {
        assert_eq!(workspace.read(edit.path).unwrap(), edit.after);
    }
    assert!(
        workspace
            .read("Cargo.lock")
            .unwrap()
            .contains("version = \"1.2.4\"")
    );
    assert_eq!(
        workspace.read("site/appcast.xml").unwrap(),
        "historical 1.2.3"
    );
    assert_eq!(
        workspace
            .metadata(&SystemRunner::default())
            .unwrap()
            .package_version("fixture")
            .unwrap(),
        Version::new(1, 2, 4)
    );
    let lockfile = workspace.read("Cargo.lock").unwrap();
    assert!(lockfile.contains("name = \"member\"\nversion = \"1.2.4\""));
    assert!(lockfile.contains("name = \"external\"\nversion = \"9.8.7\""));
    assert!(!lockfile.contains("1.2.3"));
}

#[test]
fn diff_includes_lockfile_members_and_preserves_external_versions() {
    let (_directory, workspace, runner) = fixture();
    let plan = Plan::prepare(&workspace, &runner, "patch").unwrap();
    let diff = plan.diff();
    assert!(diff.contains(
        "--- a/Cargo.toml\n+++ b/Cargo.toml\n-version = \"1.2.3\"\n+version = \"1.2.4\""
    ));
    let lock = diff.split("+++ b/Cargo.lock\n").nth(1).unwrap();
    assert_eq!(lock.matches("+version = \"1.2.4\"").count(), 2);
    assert!(!lock.contains("9.8.7"));
    assert_originals(&plan);
}
