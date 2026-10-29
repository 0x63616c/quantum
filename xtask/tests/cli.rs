//! Public CLI smoke tests do not invoke the GPU harness or modify the repository.

use std::process::Command;

#[cfg(target_os = "linux")]
#[test]
fn screenshot_body_validation_precedes_output_cleanup_and_builds() {
    let temp = tempfile::tempdir().unwrap();
    let body = temp.path().join("pr body.md");
    let out = temp.path().join("screenshots");
    std::fs::create_dir(&out).unwrap();
    let previous = out.join("keep.txt");
    std::fs::write(&previous, "previous capture").unwrap();
    for (directive, diagnostic) in [
        ("Screens: settings-typo-*", "unknown screen selector"),
        (
            "Screens: welcome; $(false) --out ../escape",
            "invalid screen selector",
        ),
        (
            "Screens: welcome\nScreens: editor-dark",
            "use only one Screens: line",
        ),
    ] {
        std::fs::write(&body, directive).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["screenshots", "--pr-body"])
            .arg(&body)
            .arg("--out")
            .arg(&out)
            .env("CARGO", temp.path().join("must-not-build"))
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(diagnostic), "{stderr}");
        assert_eq!(
            std::fs::read_to_string(&previous).unwrap(),
            "previous capture"
        );
    }
}

#[test]
fn help_list_and_usage_errors() {
    let binary = env!("CARGO_BIN_EXE_xtask");
    for args in [
        vec![],
        vec!["--help"],
        vec!["help"],
        vec!["check", "--list"],
    ] {
        let output = Command::new(binary).args(&args).output().unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("tidy"));
    }
    for args in [
        vec!["unknown"],
        vec!["tidy", "extra"],
        vec!["check"],
        vec!["bump-version"],
    ] {
        assert_eq!(
            Command::new(binary)
                .args(args)
                .output()
                .unwrap()
                .status
                .code(),
            Some(2)
        );
    }
    let output = Command::new(binary)
        .args(["validate", "--only", "typo"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown check"));
}

#[test]
fn plugin_check_keeps_exit_codes_and_diagnostic_streams() {
    let binary = env!("CARGO_BIN_EXE_xtask");
    let success = Command::new(binary)
        .args(["check", "plugin-deps"])
        .output()
        .unwrap();
    assert_eq!(success.status.code(), Some(0), "{success:?}");
    assert!(
        success
            .stdout
            .starts_with(b"Plugin dependency rule passed.\n")
    );
    assert!(success.stderr.is_empty());

    // An explicit missing executable exercises the real CLI failure path without
    // changing process-global environment, PATH, or any workspace manifest.
    let temp = tempfile::tempdir().unwrap();
    let failure = Command::new(binary)
        .args(["check", "plugin-deps"])
        .env("CARGO", temp.path().join("missing-cargo"))
        .output()
        .unwrap();
    assert_eq!(failure.status.code(), Some(1));
    assert!(
        failure
            .stderr
            .starts_with(b"Could not read Cargo metadata:")
    );
    assert!(
        String::from_utf8(failure.stdout)
            .unwrap()
            .contains("0 passed, 1 failed, 0 skipped")
    );
}

#[test]
fn dispatch_failure_renders_a_diagnostic_and_exits_unsuccessfully() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["check", "not-a-check"])
        .env("NO_COLOR", "1")
        .env("TERM", "dumb")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("not-a-check"), "{stderr}");
    assert!(!stderr.contains('\u{1b}'));
}
