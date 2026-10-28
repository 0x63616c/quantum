//! Exercise real std::process adaptation using this Rust test executable as a child.

use std::{ffi::OsString, fs};
use xtask::cmd::{Command, SystemRunner};

#[test]
fn rust_child() {
    let Some(log) = std::env::var_os("XTASK_TEST_LOG") else {
        return;
    };
    let record = serde_json::json!({
        "cwd": std::env::current_dir().unwrap(),
        "value": std::env::var("XTASK_TEST_VALUE").unwrap(),
        "removed": std::env::var_os("XTASK_TEST_REMOVED"),
    });
    fs::write(log, serde_json::to_vec(&record).unwrap()).unwrap();
    println!("child stdout");
    eprintln!("child stderr");
    std::process::exit(std::env::var("XTASK_TEST_EXIT").unwrap().parse().unwrap());
}

#[expect(
    clippy::unwrap_used,
    reason = "test fixture or assertion helper reports invalid setup by panicking"
)]
fn child(directory: &std::path::Path, code: i32) -> Command {
    Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "rust_child", "--nocapture"])
        .env("XTASK_TEST_LOG", directory.join("child.json"))
        .env("XTASK_TEST_VALUE", "literal $(value) with spaces")
        .env("XTASK_TEST_EXIT", code.to_string())
        .env("XTASK_TEST_REMOVED", "discard")
        .env_remove("XTASK_TEST_REMOVED")
        .current_dir(directory)
}

#[test]
fn real_process_applies_environment_directory_capture_and_status() {
    let directory = tempfile::tempdir().unwrap();
    let runner = SystemRunner::default();
    let output = child(directory.path(), 0).output(&runner).unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("child stdout"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("child stderr"));
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.path().join("child.json")).unwrap()).unwrap();
    assert_eq!(
        record["cwd"],
        directory.path().canonicalize().unwrap().to_str().unwrap()
    );
    assert_eq!(record["value"], "literal $(value) with spaces");
    assert!(record["removed"].is_null());
    assert_eq!(
        child(directory.path(), 19).status(&runner).unwrap().code(),
        Some(19)
    );
    let error = child(directory.path(), 19)
        .output(&runner)
        .unwrap_err()
        .to_string();
    assert!(error.contains("19") && error.contains("child stderr"));
    child(directory.path(), 0)
        .allow_in_dry_run()
        .run(&SystemRunner::dry_run())
        .unwrap();
    // Keep non-UTF-8 support in the builder's type contract on every platform.
    let _: Vec<OsString> = child(directory.path(), 0)
        .arguments()
        .map(OsString::from)
        .collect();
}
