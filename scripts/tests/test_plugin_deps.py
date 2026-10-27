#!/usr/bin/env python3
"""Test plugin dependency checks with fixture Cargo metadata."""

import contextlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("plugin_deps", ROOT / "scripts/check-plugin-deps.py")
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


def package(name, directory):
    return {
        "id": f"path+file://{directory}#{name}@0.2.0",
        "name": name,
        "manifest_path": f"{directory}/Cargo.toml",
        "dependencies": [],
    }


def dependency(name, directory=None, **overrides):
    result = {
        "name": name, "kind": None, "rename": None, "optional": False,
        "target": None, "source": None if directory else "registry+https://example.invalid/index",
    }
    if directory:
        result["path"] = directory
    return result | overrides


class PluginDependencyTests(unittest.TestCase):
    def setUp(self):
        self.plugin = package("voxelite-plugin-example", "/repo/crates/plugins/voxelite-plugin-example")
        self.api = package("voxelite-plugin-api", "/repo/crates/voxelite-plugin-api")
        self.core = package("voxelite-core", "/repo/crates/voxelite-core")
        self.host = package("voxelite-plugin-host", "/repo/crates/voxelite-plugin-host")
        self.other = package("voxelite-plugin-other", "/repo/crates/plugins/voxelite-plugin-other")
        packages = [self.plugin, self.api, self.core, self.host, self.other]
        self.metadata = {
            "workspace_root": "/repo",
            "workspace_members": [p["id"] for p in packages],
            "packages": packages,
            "resolve": {"nodes": []},
        }

    def workspace_dependency(self, package, **overrides):
        return dependency(package["name"], str(Path(package["manifest_path"]).parent), **overrides)

    def test_allows_api_external_crates_and_dev_dependencies(self):
        self.plugin["dependencies"] = [
            self.workspace_dependency(self.api),
            dependency("serde"),
            dependency("external-helper", "/repo/vendor/external-helper"),
            self.workspace_dependency(self.host, kind="dev"),
            self.workspace_dependency(self.core, kind="dev"),
        ]
        self.assertEqual(CHECKER.dependency_violations(self.metadata), [])

    def test_rejects_other_workspace_dependencies(self):
        for target in [self.core, self.host, self.other]:
            with self.subTest(target=target["name"]):
                self.plugin["dependencies"] = [self.workspace_dependency(target)]
                self.assertEqual(CHECKER.dependency_violations(self.metadata), [
                    "crates/plugins/voxelite-plugin-example/Cargo.toml: "
                    f"normal dependency on {target['name']}; "
                    "only voxelite-plugin-api is allowed outside dev-dependencies",
                ])

    def test_checks_renamed_optional_target_and_build_dependencies(self):
        for overrides in [
            {"rename": "engine"},
            {"optional": True},
            {"target": 'cfg(target_os = "macos")'},
            {"kind": "build"},
        ]:
            with self.subTest(overrides=overrides):
                self.plugin["dependencies"] = [self.workspace_dependency(self.core, **overrides)]
                violations = CHECKER.dependency_violations(self.metadata)
                self.assertEqual(len(violations), 1)
                self.assertIn(f"{overrides.get('kind', 'normal')} dependency on voxelite-core",
                              violations[0])

    def test_uses_plugin_directory_not_package_name_or_path_prefix(self):
        for directory in ["/repo/crates/plugins-helper", "/outside/crates/plugins/helper"]:
            outside = package("voxelite-plugin-outside", directory)
            outside["dependencies"] = [self.workspace_dependency(self.core)]
            self.metadata["packages"].append(outside)
        self.assertEqual(CHECKER.dependency_violations(self.metadata), [])
        self.plugin["name"] = "ordinary-name"
        self.plugin["dependencies"] = [self.workspace_dependency(self.core)]
        self.assertEqual(len(CHECKER.dependency_violations(self.metadata)), 1)

    def test_checks_all_packages_under_plugin_directory(self):
        self.metadata["workspace_members"].remove(self.plugin["id"])
        self.plugin["dependencies"] = [self.workspace_dependency(self.core)]
        self.assertEqual(len(CHECKER.dependency_violations(self.metadata)), 1)

    def test_external_packages_with_workspace_names_are_allowed(self):
        self.plugin["dependencies"] = [
            dependency("voxelite-core"),
            dependency("voxelite-core", "/outside/voxelite-core"),
        ]
        self.assertEqual(CHECKER.dependency_violations(self.metadata), [])

    def test_reports_all_violations(self):
        self.plugin["dependencies"] = [self.workspace_dependency(self.core)]
        self.other["dependencies"] = [self.workspace_dependency(self.host)]
        violations = CHECKER.dependency_violations(self.metadata)
        self.assertEqual(len(violations), 2)
        self.assertIn("voxelite-plugin-example/Cargo.toml", violations[0])
        self.assertIn("voxelite-plugin-other/Cargo.toml", violations[1])

    def test_checks_patched_workspace_dependencies_by_resolved_identity(self):
        self.plugin["dependencies"] = [dependency("voxelite-core", rename="engine")]
        for kind in [None, "dev", "build"]:
            with self.subTest(kind=kind):
                self.plugin["dependencies"][0]["kind"] = kind
                self.metadata["resolve"]["nodes"] = [{
                    "id": self.plugin["id"],
                    "deps": [{
                        "name": "engine", "pkg": self.core["id"],
                        "dep_kinds": [{"kind": kind, "target": None}],
                    }],
                }]
                violations = CHECKER.dependency_violations(self.metadata)
                self.assertEqual(len(violations), 0 if kind == "dev" else 1)

    def test_reports_declared_and_resolved_dependency_once(self):
        self.plugin["dependencies"] = [self.workspace_dependency(self.core)]
        self.metadata["resolve"]["nodes"] = [{
            "id": self.plugin["id"],
            "deps": [{
                "name": "voxelite_core", "pkg": self.core["id"],
                "dep_kinds": [{"kind": None, "target": None}, {"kind": "dev", "target": None}],
            }],
        }]
        self.assertEqual(len(CHECKER.dependency_violations(self.metadata)), 1)

    def test_offline_metadata_fallback_stays_locked(self):
        failed = subprocess.CompletedProcess([], 1, "", "offline cache miss")
        success = subprocess.CompletedProcess([], 0, json.dumps(self.metadata), "")
        with patch.object(CHECKER.subprocess, "run", side_effect=[failed, success]) as run:
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(CHECKER.main(), 0)
        self.assertEqual([call.args[0] for call in run.call_args_list], [
            ["cargo", "metadata", "--format-version", "1", "--locked", "--offline"],
            ["cargo", "metadata", "--format-version", "1", "--locked"],
        ])
        self.assertTrue(all(call.kwargs["cwd"] == ROOT for call in run.call_args_list))

    def test_cli_fails_on_violations_without_retrying_successful_metadata(self):
        self.plugin["dependencies"] = [self.workspace_dependency(self.core)]
        success = subprocess.CompletedProcess([], 0, json.dumps(self.metadata), "")
        with patch.object(CHECKER.subprocess, "run", return_value=success) as run:
            with contextlib.redirect_stderr(io.StringIO()) as stderr:
                self.assertEqual(CHECKER.main(), 1)
        run.assert_called_once()
        self.assertIn("normal dependency on voxelite-core", stderr.getvalue())

    def test_cli_fails_when_both_metadata_attempts_fail(self):
        failed = subprocess.CompletedProcess([], 1, "", "Cargo metadata failed\n")
        with patch.object(CHECKER.subprocess, "run", return_value=failed) as run:
            with contextlib.redirect_stderr(io.StringIO()) as stderr:
                self.assertEqual(CHECKER.main(), 1)
        self.assertEqual(run.call_count, 2)
        self.assertIn("Cargo metadata failed", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
