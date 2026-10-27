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
        normal_dependencies("voxelite-render")
            .intersection(&workspace_names)
            .copied()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["voxelite-core", "voxelite-mesh"])
    );
    for name in [
        "voxelite-export",
        "voxelite-plugin-host",
        "voxelite-plugin-exporters",
        "voxelite-plugin-api",
        "voxelite-format",
    ] {
        assert!(
            !normal_dependencies(name).contains("voxelite-render"),
            "{name} must not depend on voxelite-render"
        );
    }
}
