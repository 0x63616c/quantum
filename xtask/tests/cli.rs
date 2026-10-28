//! Public CLI smoke tests do not invoke the GPU harness or modify the repository.

use std::process::Command;

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
