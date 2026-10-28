//! Keep the reusable renderer independent of hosts and encoding crates.
use serde_json::Value;
use std::{collections::BTreeSet, process::Command};

#[test]
fn normal_workspace_dependencies_preserve_renderer_boundary() {
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--locked",
            "--offline",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cargo metadata");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Value = serde_json::from_slice(&output.stdout).expect("cargo metadata JSON");
    let members = metadata["workspace_members"].as_array().unwrap();
    let packages: Vec<_> = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|package| members.contains(&package["id"]))
        .collect();
    let workspace_names: BTreeSet<_> = packages
        .iter()
        .map(|package| package["name"].as_str().unwrap())
        .collect();
    assert!(!workspace_names.contains("quantum-export"));
    let normal_dependencies = |name: &str| -> BTreeSet<&str> {
        packages
            .iter()
            .find(|package| package["name"] == name)
            .expect("workspace package")["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|dependency| dependency["kind"].is_null())
            .map(|dependency| dependency["name"].as_str().unwrap())
            .collect()
    };
    assert_eq!(
        normal_dependencies("quantum-render")
            .intersection(&workspace_names)
            .copied()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["quantum-core", "quantum-mesh", "quantum-render-plan"])
    );
    for name in [
        "quantum-plugin-host",
        "quantum-plugin-exporters",
        "quantum-plugin-api",
        "quantum-format",
    ] {
        assert!(
            !normal_dependencies(name).contains("quantum-render"),
            "{name} must not depend on quantum-render"
        );
    }
}

#[test]
fn default_builds_exclude_profiling_features_and_runtime_dependencies() {
    let output = Command::new(env!("CARGO"))
        .args([
            "tree",
            "--locked",
            "--offline",
            "-p",
            "quantum-app",
            "-p",
            "quantum-render",
            "-p",
            "quantum-mesh",
            "--edges",
            "normal",
            "--prefix",
            "none",
            "--format",
            "{p}|{f}",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("inspect default dependency trees");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tree = String::from_utf8(output.stdout).unwrap();
    for line in tree.lines().filter(|line| !line.is_empty()) {
        let (package, features) = line.split_once('|').expect("package and feature list");
        let name = package.split_whitespace().next().unwrap();
        match name {
            "quantum-app" | "quantum-render" | "quantum-mesh" => assert!(
                !features.split(',').any(|feature| feature == "profiling"),
                "default build unexpectedly enables {name}/profiling",
            ),
            "tracing-tracy" | "tracy-client" | "tracy-client-sys" | "tracing-subscriber" => {
                panic!("default build unexpectedly links {name}");
            }
            _ => {}
        }
    }
}
