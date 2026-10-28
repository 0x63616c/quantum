//! Enforce plugin and MCP boundaries from declarations and resolved package identity.

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    io::{self, Write},
    path::PathBuf,
};

use serde::Deserialize;

use crate::{
    Error, Result,
    check::{Context, Outcome},
};

pub(crate) fn run(context: &Context<'_>) -> Outcome {
    run_with_output(context, &mut io::stdout().lock(), &mut io::stderr().lock())
}

fn run_with_output(
    context: &Context<'_>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Outcome {
    let diagnostic = match check(context) {
        Ok(diagnostic) => diagnostic,
        Err(error) => Some(format!("{error}\n")),
    };
    let (written, outcome) = match diagnostic {
        Some(message) => (
            stderr.write_all(message.as_bytes()),
            Outcome::Fail(vec![message]),
        ),
        None => (
            writeln!(stdout, "Plugin dependency rule passed."),
            Outcome::Pass,
        ),
    };
    match written {
        Ok(()) => outcome,
        Err(error) => Outcome::Fail(vec![format!(
            "Could not report plugin dependency rule: {error}"
        )]),
    }
}

/// Return the complete legacy stderr diagnostic, or no diagnostic on success.
/// The registry adds its timed summary separately, without replacing these bytes.
fn check(context: &Context<'_>) -> Result<Option<String>> {
    let command = context
        .workspace
        .cargo()
        .args(["metadata", "--format-version", "1", "--locked"])
        .allow_in_dry_run();
    // This command has no secrets. Keep Cargo's full diagnostic and retry only
    // a failed offline invocation, never a successful result with rule violations.
    let read = |command: &crate::cmd::Command| {
        context
            .runner
            .execute(command, true)
            .map_err(|error| Error::Invalid(format!("Could not read Cargo metadata: {error}")))
    };
    let mut output = read(&command.clone().arg("--offline"))?;
    if !output.status.success() {
        output = read(&command)?;
    }
    if !output.status.success() {
        return Ok(Some(String::from_utf8_lossy(&output.stderr).into_owned()));
    }
    let metadata = serde_json::from_slice(&output.stdout)
        .map_err(|error| Error::Invalid(format!("Could not read Cargo metadata: {error}")))?;
    let errors = dependency_violations(&metadata);
    Ok((!errors.is_empty())
        .then(|| format!("Plugin dependency rule failed:\n{}\n", errors.join("\n"))))
}

#[derive(Deserialize)]
struct Metadata {
    workspace_root: PathBuf,
    workspace_members: HashSet<String>,
    packages: Vec<Package>,
    resolve: Resolve,
}

#[derive(Deserialize)]
struct Package {
    id: String,
    name: String,
    manifest_path: PathBuf,
    dependencies: Vec<Dependency>,
}

#[derive(Deserialize)]
struct Dependency {
    name: String,
    kind: Option<String>,
    optional: bool,
    path: Option<PathBuf>,
}

#[derive(Deserialize)]
struct Resolve {
    nodes: Vec<Node>,
}

#[derive(Deserialize)]
struct Node {
    id: String,
    deps: Vec<Edge>,
}

#[derive(Deserialize)]
struct Edge {
    pkg: String,
    dep_kinds: Vec<DependencyKind>,
}

#[derive(Deserialize)]
struct DependencyKind {
    kind: Option<String>,
}

/// Inspect declarations as well as the graph: optional and target-specific edges
/// may be absent from resolution, while patched registry edges have no path.
fn dependency_violations(metadata: &Metadata) -> Vec<String> {
    let root = &metadata.workspace_root;
    let plugins = root.join("crates/plugins");
    let members: Vec<_> = metadata
        .packages
        .iter()
        .filter(|package| metadata.workspace_members.contains(&package.id))
        .collect();
    let workspace_crates: HashMap<_, _> = members
        .iter()
        .filter_map(|package| {
            package
                .manifest_path
                .parent()
                .map(|path| (path, package.name.as_str()))
        })
        .collect();
    let workspace_names: HashMap<_, _> = members
        .iter()
        .map(|package| (package.id.as_str(), package.name.as_str()))
        .collect();
    let nodes: HashMap<_, _> = metadata
        .resolve
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect();
    let mut violations = Vec::new();
    for package in &metadata.packages {
        let manifest = &package.manifest_path;
        let is_mcp = manifest.parent() == Some(root.join("crates/quantum-mcp").as_path());
        let (allowed, rule): (&[&str], _) = if manifest.starts_with(&plugins) {
            (
                &["quantum-plugin-api"],
                "only quantum-plugin-api is allowed outside dev-dependencies",
            )
        } else if is_mcp {
            (
                &[
                    "quantum-core",
                    "quantum-format",
                    "quantum-render",
                    "quantum-plugin-api",
                    "quantum-plugin-host",
                ],
                "MCP must receive plugins from its composition root",
            )
        } else {
            continue;
        };
        let optional_builtin = is_mcp
            && package.dependencies.iter().any(|dependency| {
                dependency.name == "quantum-plugins-builtin"
                    && dependency.kind.is_none()
                    && dependency.optional
            });
        let allowed_edge = |kind: Option<&str>, name: &str| {
            (matches!(kind, None | Some("build")) && allowed.contains(&name))
                || (kind.is_none() && name == "quantum-plugins-builtin" && optional_builtin)
        };
        let mut forbidden = BTreeSet::new();
        for dependency in &package.dependencies {
            let kind = dependency.kind.as_deref();
            if kind == Some("dev") {
                continue;
            }
            if let Some(name) = dependency
                .path
                .as_deref()
                .and_then(|path| workspace_crates.get(path))
                && (!allowed_edge(kind, name)
                    || (is_mcp && *name == "quantum-plugins-builtin" && !dependency.optional))
            {
                forbidden.insert((kind.unwrap_or("normal"), *name));
            }
        }
        if let Some(node) = nodes.get(package.id.as_str()) {
            for edge in &node.deps {
                if let Some(name) = workspace_names.get(edge.pkg.as_str()) {
                    for kind in &edge.dep_kinds {
                        let kind = kind.kind.as_deref();
                        if kind != Some("dev") && !allowed_edge(kind, name) {
                            forbidden.insert((kind.unwrap_or("normal"), *name));
                        }
                    }
                }
            }
        }
        for (kind, name) in forbidden {
            let relative = manifest.strip_prefix(root).unwrap_or(manifest);
            violations.push(format!(
                "{}: {kind} dependency on {name}; {rule}",
                relative.display()
            ));
        }
    }
    violations
}

#[cfg(test)]
mod tests;
