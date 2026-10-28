//! Resolved dependency isolation for portable document crates.
use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use crate::{
    Error, Result,
    check::{Context, Outcome},
};

#[derive(Deserialize)]
struct Metadata {
    workspace_members: BTreeSet<String>,
    packages: Vec<Package>,
    resolve: Resolve,
}
#[derive(Deserialize)]
struct Package {
    id: String,
    name: String,
}
#[derive(Deserialize)]
struct Resolve {
    nodes: Vec<Node>,
}
#[derive(Deserialize)]
struct Node {
    id: String,
    dependencies: Vec<String>,
}

pub(crate) fn run(context: &Context<'_>) -> Outcome {
    super::outcome((|| {
        let json = context
            .workspace
            .cargo()
            .args([
                "metadata",
                "--format-version",
                "1",
                "--all-features",
                "--locked",
            ])
            .read(context.runner)?;
        violations(&json)
    })())
}

fn violations(json: &str) -> Result<Vec<String>> {
    let metadata: Metadata = serde_json::from_str(json).map_err(|source| Error::Json {
        context: "dependency boundary metadata".into(),
        source,
    })?;
    let names: BTreeMap<_, _> = metadata
        .packages
        .iter()
        .map(|p| (p.id.as_str(), p.name.as_str()))
        .collect();
    let edges: BTreeMap<_, _> = metadata
        .resolve
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), &n.dependencies))
        .collect();
    let mut errors = Vec::new();
    for root in ["voxelite-core", "voxelite-format", "voxelite-mesh"] {
        let package = metadata
            .packages
            .iter()
            .find(|p| p.name == root && metadata.workspace_members.contains(&p.id))
            .ok_or_else(|| Error::Invalid(format!("boundary metadata missing {root}")))?;
        let mut pending = vec![(package.id.as_str(), vec![root.to_owned()])];
        let mut visited = BTreeSet::new();
        while let Some((id, path)) = pending.pop() {
            if !visited.insert(id) {
                continue;
            }
            let dependencies = edges.get(id).ok_or_else(|| {
                Error::Invalid(format!("boundary metadata missing resolve node {id}"))
            })?;
            for dependency in *dependencies {
                let name = names.get(dependency.as_str()).ok_or_else(|| {
                    Error::Invalid(format!("boundary metadata missing package {dependency}"))
                })?;
                let mut path = path.clone();
                path.push((*name).to_owned());
                if matches!(*name, "gpui" | "gpui-unofficial" | "voxelite-app") {
                    errors.push(format!(
                        "portable crate dependency forbidden: {}",
                        path.join(" → ")
                    ));
                } else {
                    pending.push((dependency, path));
                }
            }
        }
    }
    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture(extra: &[(&str, &str, Vec<&str>)]) -> String {
        let mut packages = Vec::new();
        let mut nodes = Vec::new();
        for (id, name, dependencies) in [
            ("core", "voxelite-core", vec![]),
            ("format", "voxelite-format", vec!["core"]),
            ("mesh", "voxelite-mesh", vec!["core"]),
        ]
        .into_iter()
        .chain(extra.iter().cloned())
        {
            packages.push(json!({"id":id,"name":name}));
            nodes.push(json!({"id":id,"dependencies":dependencies}));
        }
        json!({"workspace_members":["core", "format", "mesh"],"packages":packages,"resolve":{"nodes":nodes}}).to_string()
    }
    #[test]
    fn runner_uses_locked_complete_metadata_and_propagates_bad_output() {
        use crate::{testing::RecordingRunner, workspace::Workspace};
        let workspace = Workspace::discover().unwrap();
        for (json, passes) in [(fixture(&[]), true), ("{}".into(), false)] {
            let runner = RecordingRunner::stdout(json.as_bytes());
            let context = Context {
                workspace: &workspace,
                runner: &runner,
                strict: true,
            };
            assert_eq!(matches!(run(&context), Outcome::Pass), passes);
            runner.assert_args(
                0,
                &[
                    "metadata",
                    "--format-version",
                    "1",
                    "--all-features",
                    "--locked",
                ],
            );
        }
    }
    #[test]
    fn portable_graph_passes() {
        assert!(violations(&fixture(&[])).unwrap().is_empty());
    }
    #[test]
    fn direct_and_transitive_package_identity_are_checked() {
        for forbidden in ["gpui", "gpui-unofficial", "voxelite-app"] {
            for through in [false, true] {
                let mut value: serde_json::Value = serde_json::from_str(&fixture(&[
                    ("alias", forbidden, vec![]),
                    ("bridge", "bridge", vec!["alias"]),
                ]))
                .unwrap();
                value["resolve"]["nodes"][0]["dependencies"] =
                    json!([if through { "bridge" } else { "alias" }]);
                assert_eq!(violations(&value.to_string()).unwrap().len(), 3);
            }
        }
    }
    #[test]
    fn workspace_identity_wins_over_same_named_registry_package() {
        let mut value: serde_json::Value = serde_json::from_str(&fixture(&[
            ("external-core", "voxelite-core", vec!["forbidden"]),
            ("forbidden", "gpui-unofficial", vec![]),
        ]))
        .unwrap();
        value["packages"].as_array_mut().unwrap().reverse();
        assert!(violations(&value.to_string()).unwrap().is_empty());
        value["resolve"]["nodes"][0]["dependencies"] = json!(["forbidden"]);
        assert_eq!(violations(&value.to_string()).unwrap().len(), 3);
        value["workspace_members"] = json!(["format", "mesh"]);
        assert!(violations(&value.to_string()).is_err());
    }
    #[test]
    fn cycles_terminate_and_incomplete_metadata_fails() {
        let mut value: serde_json::Value = serde_json::from_str(&fixture(&[])).unwrap();
        value["resolve"]["nodes"][0]["dependencies"] = json!(["mesh"]);
        assert!(violations(&value.to_string()).unwrap().is_empty());
        value["resolve"]["nodes"][0]["dependencies"] = json!(["missing"]);
        assert!(violations(&value.to_string()).is_err());
        assert!(violations("{}").is_err());
    }
}
