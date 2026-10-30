#!/usr/bin/env python3
"""Enforce plugin and MCP adapter dependency boundaries."""

import json
from pathlib import Path, PurePath
import subprocess
import sys


def dependency_violations(metadata):
    """Return violations from Cargo metadata without running Cargo or reading files."""
    root = PurePath(metadata["workspace_root"])
    plugins = root / "crates/plugins"
    members = set(metadata["workspace_members"])
    workspace_crates = {
        PurePath(package["manifest_path"]).parent: package["name"]
        for package in metadata["packages"] if package["id"] in members
    }
    workspace_names = {
        package["id"]: package["name"]
        for package in metadata["packages"] if package["id"] in members
    }
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    violations = []
    for package in metadata["packages"]:
        manifest = PurePath(package["manifest_path"])
        is_mcp = manifest.parent == root / "crates/voxelite-mcp"
        if manifest.is_relative_to(plugins):
            allowed = {"voxelite-plugin-api"}
            rule = "only voxelite-plugin-api is allowed outside dev-dependencies"
        elif is_mcp:
            # Cargo shares dependencies between library and binary targets. The
            # binary may compose built-ins; the adapter must not import them.
            allowed = {
                "voxelite-core", "voxelite-format", "voxelite-render",
                "voxelite-plugin-api", "voxelite-plugin-host",
            }
            rule = "MCP must receive plugins from its composition root"
        else:
            continue
        allowed_edges = {(kind, name) for kind in [None, "build"] for name in allowed}
        if is_mcp and any(
            dependency["name"] == "voxelite-plugins-builtin"
            and dependency["kind"] is None and dependency["optional"]
            for dependency in package["dependencies"]
        ):
            allowed_edges.add((None, "voxelite-plugins-builtin"))
        forbidden = set()
        # Inspect declarations, not just resolved edges: disabled optional and
        # target-specific dependencies must obey the same rule. Cargo's path is
        # the package location even when a dependency is renamed.
        for dependency in package["dependencies"]:
            if dependency["kind"] == "dev" or "path" not in dependency:
                continue
            name = workspace_crates.get(PurePath(dependency["path"]))
            if name is not None and (
                (dependency["kind"], name) not in allowed_edges
                or (is_mcp and name == "voxelite-plugins-builtin" and not dependency["optional"])
            ):
                forbidden.add((dependency["kind"] or "normal", name))
        # A registry dependency can also resolve to a workspace crate via a
        # Cargo patch, without a path in its declaration.
        for dependency in nodes.get(package["id"], {}).get("deps", []):
            name = workspace_names.get(dependency["pkg"])
            if name is not None:
                for kind in dependency["dep_kinds"]:
                    if kind["kind"] != "dev" and (kind["kind"], name) not in allowed_edges:
                        forbidden.add((kind["kind"] or "normal", name))
        for kind, name in sorted(forbidden):
            violations.append(
                f"{manifest.relative_to(root)}: {kind} dependency on {name}; {rule}"
            )
    return violations


def main():
    root = Path(__file__).resolve().parents[1]
    command = ["cargo", "metadata", "--format-version", "1", "--locked"]
    try:
        result = subprocess.run(command + ["--offline"], cwd=root, capture_output=True, text=True)
        if result.returncode:
            result = subprocess.run(command, cwd=root, capture_output=True, text=True)
        if result.returncode:
            print(result.stderr, file=sys.stderr, end="")
            return 1
        violations = dependency_violations(json.loads(result.stdout))
    except (OSError, json.JSONDecodeError) as error:
        print(f"Could not read Cargo metadata: {error}", file=sys.stderr)
        return 1
    if violations:
        print("Plugin dependency rule failed:\n" + "\n".join(violations), file=sys.stderr)
        return 1
    print("Plugin dependency rule passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
