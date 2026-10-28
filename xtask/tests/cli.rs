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
