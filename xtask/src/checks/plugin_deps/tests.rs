use super::*;
use crate::{
    cmd::{Output, Status},
    testing::RecordingRunner,
    workspace::Workspace,
};
use serde_json::{Value, json};

const PLUGIN: usize = 0;
const API: usize = 1;
const CORE: usize = 2;
const HOST: usize = 3;
const MESH: usize = 4;
const OTHER: usize = 5;
const MCP: usize = 6;
const BUILTIN: usize = 7;

fn package(name: &str, directory: &str) -> Value {
    json!({"id": format!("path+file://{directory}#{name}@0.2.0"), "name": name,
        "manifest_path": format!("{directory}/Cargo.toml"), "dependencies": []})
}

fn fixture() -> Value {
    let packages: Vec<_> = [
        ("voxelite-plugin-example", "plugins/voxelite-plugin-example"),
        ("voxelite-plugin-api", "voxelite-plugin-api"),
        ("voxelite-core", "voxelite-core"),
        ("voxelite-plugin-host", "voxelite-plugin-host"),
        ("voxelite-mesh", "voxelite-mesh"),
        ("voxelite-plugin-other", "plugins/voxelite-plugin-other"),
        ("voxelite-mcp", "voxelite-mcp"),
        ("voxelite-plugins-builtin", "voxelite-plugins-builtin"),
    ]
    .iter()
    .map(|(name, path)| package(name, &format!("/repo/crates/{path}")))
    .collect();
    json!({"workspace_root": "/repo", "workspace_members": packages.iter().map(|p| &p["id"]).collect::<Vec<_>>(),
        "packages": packages, "resolve": {"nodes": []}})
}

fn dependency(name: &str, directory: Option<&str>, overrides: Value) -> Value {
    let mut result = json!({"name": name, "kind": null, "rename": null, "optional": false, "target": null,
        "source": if directory.is_some() { Value::Null } else { json!("registry+https://example.invalid/index") }});
    if let Some(path) = directory {
        result["path"] = json!(path);
    }
    result
        .as_object_mut()
        .unwrap()
        .extend(overrides.as_object().unwrap().clone());
    result
}

fn workspace_dependency(metadata: &Value, index: usize, overrides: Value) -> Value {
    let package = &metadata["packages"][index];
    let directory = std::path::Path::new(package["manifest_path"].as_str().unwrap())
        .parent()
        .unwrap();
    dependency(
        package["name"].as_str().unwrap(),
        directory.to_str(),
        overrides,
    )
}

fn violations(metadata: &Value) -> Vec<String> {
    dependency_violations(&serde_json::from_value(metadata.clone()).unwrap())
}

fn resolved_edge(metadata: &mut Value, owner: usize, target: usize, kinds: Value) {
    metadata["resolve"]["nodes"] = json!([{"id": metadata["packages"][owner]["id"], "deps": [{
        "name": "renamed", "pkg": metadata["packages"][target]["id"], "dep_kinds": kinds}]}]);
}

#[test]
fn allows_api_external_crates_and_dev_dependencies() {
    let mut m = fixture();
    m["packages"][PLUGIN]["dependencies"] = json!([
        workspace_dependency(&m, API, json!({})),
        dependency("serde", None, json!({})),
        dependency(
            "external-helper",
            Some("/repo/vendor/external-helper"),
            json!({})
        ),
        workspace_dependency(&m, HOST, json!({"kind":"dev"})),
        workspace_dependency(&m, CORE, json!({"kind":"dev"}))
    ]);
    assert!(violations(&m).is_empty());
}

#[test]
fn geometry_must_be_reached_through_api() {
    let mut m = fixture();
    m["packages"][API]["dependencies"] = json!([workspace_dependency(&m, MESH, json!({}))]);
    m["packages"][PLUGIN]["dependencies"] = json!([workspace_dependency(&m, API, json!({}))]);
    assert!(violations(&m).is_empty());
    let mesh = workspace_dependency(&m, MESH, json!({}));
    m["packages"][PLUGIN]["dependencies"]
        .as_array_mut()
        .unwrap()
        .push(mesh);
    assert_eq!(violations(&m).len(), 1);
}

#[test]
fn mcp_allows_api_host_binary_composition_and_test_plugins() {
    let mut m = fixture();
    m["packages"][MCP]["dependencies"] = json!([
        workspace_dependency(&m, API, json!({})),
        workspace_dependency(&m, HOST, json!({})),
        workspace_dependency(&m, CORE, json!({})),
        workspace_dependency(&m, BUILTIN, json!({"optional":true})),
        workspace_dependency(&m, PLUGIN, json!({"kind":"dev"})),
        dependency("png", None, json!({}))
    ]);
    m["packages"][BUILTIN]["dependencies"] = json!([
        workspace_dependency(&m, HOST, json!({})),
        workspace_dependency(&m, PLUGIN, json!({}))
    ]);
    assert!(violations(&m).is_empty());
}

#[test]
fn rejects_all_production_dependency_forms() {
    for (owner, target, manifest, rule) in [
        (
            PLUGIN,
            CORE,
            "crates/plugins/voxelite-plugin-example/Cargo.toml",
            "only voxelite-plugin-api is allowed outside dev-dependencies",
        ),
        (
            MCP,
            PLUGIN,
            "crates/voxelite-mcp/Cargo.toml",
            "MCP must receive plugins from its composition root",
        ),
    ] {
        for overrides in [
            json!({}),
            json!({"rename":"engine"}),
            json!({"optional":true}),
            json!({"target":"cfg(target_os = \"macos\")"}),
            json!({"kind":"build"}),
        ] {
            let mut m = fixture();
            m["packages"][owner]["dependencies"] =
                json!([workspace_dependency(&m, target, overrides.clone())]);
            let kind = overrides["kind"].as_str().unwrap_or("normal");
            let name = m["packages"][target]["name"].as_str().unwrap();
            assert_eq!(
                violations(&m),
                [format!("{manifest}: {kind} dependency on {name}; {rule}")]
            );
        }
    }
}

#[test]
fn mcp_composition_must_be_optional_and_not_a_build_dependency() {
    for overrides in [
        json!({}),
        json!({"kind":"build"}),
        json!({"kind":"build", "optional":true}),
    ] {
        let mut m = fixture();
        m["packages"][MCP]["dependencies"] = json!([
            workspace_dependency(&m, BUILTIN, json!({"optional":true})),
            workspace_dependency(&m, BUILTIN, overrides)
        ]);
        assert_eq!(violations(&m).len(), 1);
    }
}

#[test]
fn composition_is_not_an_escape_hatch_for_plugins() {
    let mut m = fixture();
    m["packages"][PLUGIN]["dependencies"] = json!([workspace_dependency(&m, BUILTIN, json!({}))]);
    assert_eq!(violations(&m).len(), 1);
}

#[test]
fn rejects_other_workspace_dependencies() {
    for target in [CORE, HOST, MESH, OTHER] {
        let mut m = fixture();
        m["packages"][PLUGIN]["dependencies"] =
            json!([workspace_dependency(&m, target, json!({}))]);
        let name = m["packages"][target]["name"].as_str().unwrap();
        assert_eq!(
            violations(&m),
            [format!(
                "crates/plugins/voxelite-plugin-example/Cargo.toml: normal dependency on {name}; only voxelite-plugin-api is allowed outside dev-dependencies"
            )]
        );
    }
}

#[test]
fn uses_plugin_directory_not_package_name_or_path_prefix() {
    let mut m = fixture();
    for directory in [
        "/repo/crates/plugins-helper",
        "/outside/crates/plugins/helper",
    ] {
        let mut outside = package("voxelite-plugin-outside", directory);
        outside["dependencies"] = json!([workspace_dependency(&m, CORE, json!({}))]);
        m["packages"].as_array_mut().unwrap().push(outside);
    }
    assert!(violations(&m).is_empty());
    m["packages"][PLUGIN]["name"] = json!("ordinary-name");
    m["packages"][PLUGIN]["dependencies"] = json!([workspace_dependency(&m, CORE, json!({}))]);
    assert_eq!(violations(&m).len(), 1);
}

#[test]
fn checks_all_packages_under_plugin_directory() {
    let mut m = fixture();
    m["workspace_members"]
        .as_array_mut()
        .unwrap()
        .remove(PLUGIN);
    m["packages"][PLUGIN]["dependencies"] = json!([workspace_dependency(&m, CORE, json!({}))]);
    assert_eq!(violations(&m).len(), 1);
}

#[test]
fn external_packages_with_workspace_names_are_allowed() {
    let mut m = fixture();
    m["packages"][PLUGIN]["dependencies"] = json!([
        dependency("voxelite-core", None, json!({})),
        dependency("voxelite-core", Some("/outside/voxelite-core"), json!({}))
    ]);
    assert!(violations(&m).is_empty());
}

#[test]
fn reports_all_violations_in_package_order() {
    let mut m = fixture();
    m["packages"][PLUGIN]["dependencies"] = json!([workspace_dependency(&m, CORE, json!({}))]);
    m["packages"][OTHER]["dependencies"] = json!([workspace_dependency(&m, HOST, json!({}))]);
    let errors = violations(&m);
    assert_eq!(errors.len(), 2);
    assert!(errors[0].contains("voxelite-plugin-example/Cargo.toml"));
    assert!(errors[1].contains("voxelite-plugin-other/Cargo.toml"));
}

#[test]
fn checks_patched_workspace_dependencies_by_resolved_identity() {
    for (owner, target) in [(PLUGIN, CORE), (MCP, PLUGIN)] {
        for kind in [Value::Null, json!("dev"), json!("build")] {
            let mut m = fixture();
            m["packages"][owner]["dependencies"] = json!([dependency(
                m["packages"][target]["name"].as_str().unwrap(),
                None,
                json!({"rename":"engine", "kind":kind})
            )]);
            resolved_edge(&mut m, owner, target, json!([{"kind":kind, "target":null}]));
            assert_eq!(violations(&m).len(), usize::from(kind != "dev"));
        }
    }
}

#[test]
fn reports_declared_and_resolved_dependency_once() {
    let mut m = fixture();
    m["packages"][PLUGIN]["dependencies"] = json!([workspace_dependency(&m, CORE, json!({}))]);
    resolved_edge(&mut m, PLUGIN, CORE, json!([{"kind":null}, {"kind":"dev"}]));
    assert_eq!(violations(&m).len(), 1);
}

fn run_check(runner: &RecordingRunner) -> Outcome {
    run(&Context {
        workspace: &Workspace::discover().unwrap(),
        runner,
        strict: false,
    })
}

fn failed() -> Output {
    Output {
        status: Status::from_code(1),
        stderr: b"Cargo metadata failed\n".to_vec(),
        ..Output::default()
    }
}

#[test]
fn offline_metadata_fallback_stays_locked() {
    let runner = RecordingRunner::default();
    runner.results.borrow_mut().extend([
        Ok(failed()),
        Ok(Output {
            stdout: serde_json::to_vec(&fixture()).unwrap(),
            ..Output::default()
        }),
    ]);
    assert_eq!(run_check(&runner), Outcome::Pass);
    runner.assert_args(
        0,
        &["metadata", "--format-version", "1", "--locked", "--offline"],
    );
    runner.assert_args(1, &["metadata", "--format-version", "1", "--locked"]);
    let workspace = Workspace::discover().unwrap();
    assert!(
        runner
            .commands
            .borrow()
            .iter()
            .all(|(command, capture)| command.directory() == Some(workspace.root()) && *capture)
    );
}

#[test]
fn fails_on_violations_without_retrying_successful_metadata() {
    let mut m = fixture();
    m["packages"][PLUGIN]["dependencies"] = json!([workspace_dependency(&m, CORE, json!({}))]);
    let runner = RecordingRunner::stdout(&serde_json::to_vec(&m).unwrap());
    let Outcome::Fail(messages) = run_check(&runner) else {
        panic!("expected failure")
    };
    assert_eq!(runner.commands.borrow().len(), 1);
    assert_eq!(messages[0], "Plugin dependency rule failed:");
    assert!(messages[1].contains("normal dependency on voxelite-core"));
}

#[test]
fn fails_when_both_metadata_attempts_fail() {
    let runner = RecordingRunner::default();
    runner
        .results
        .borrow_mut()
        .extend([Ok(failed()), Ok(failed())]);
    assert_eq!(
        run_check(&runner),
        Outcome::Fail(vec!["Cargo metadata failed\n".into()])
    );
    assert_eq!(runner.commands.borrow().len(), 2);
}

#[test]
fn invalid_metadata_and_spawn_errors_are_not_retried() {
    for output in [
        Ok(Output {
            stdout: b"{".to_vec(),
            ..Output::default()
        }),
        Err(std::io::Error::other("cargo unavailable")),
    ] {
        let runner = RecordingRunner::default();
        runner.results.borrow_mut().push_back(output);
        assert!(
            matches!(run_check(&runner), Outcome::Fail(errors) if errors[0].starts_with("Could not read Cargo metadata:"))
        );
        assert_eq!(runner.commands.borrow().len(), 1);
    }
}
