//! Plugins must not acquire encoders, a renderer or a GPU stack through the API.
use std::process::Command;

#[test]
fn transitive_dependencies_exclude_encoders_and_renderers() {
    let output = Command::new(env!("CARGO"))
        .args([
            "tree",
            "-p",
            "voxelite-plugin-api",
            "--locked",
            "--offline",
            "--edges",
            "normal,build",
            "--prefix",
            "none",
            "--format",
            "{p}",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cargo tree");
    assert!(
        output.status.success(),
        "cargo tree failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for line in String::from_utf8(output.stdout).unwrap().lines() {
        let name = line.split_whitespace().next().unwrap();
        assert!(
            !matches!(
                name,
                "voxelite-export" | "voxelite-plugin-exporters" | "voxelite-render"
            ) && !name.starts_with("wgpu"),
            "plugin API must not depend on {name}"
        );
    }
}
