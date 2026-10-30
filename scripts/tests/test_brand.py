#!/usr/bin/env python3
"""Keep the branding inventory aligned with source, manifests and packaging."""

import json
from pathlib import Path
import plistlib
import re
import tomllib
import unittest
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[2]
BRAND = tomllib.loads((ROOT / "brand.toml").read_text())


class BrandTests(unittest.TestCase):
    def current(self, name, group="identifiers"):
        return BRAND[group][name]["current"]

    def source(self, path):
        return (ROOT / path).read_text()

    def assert_capture(self, path, pattern, expected):
        self.assertEqual(re.findall(pattern, self.source(path)), [expected], path)

    def test_inventory_marks_stability_and_only_plans_safe_renames(self):
        for group, entries in BRAND.items():
            for name, entry in entries.items():
                with self.subTest(group=group, name=name):
                    self.assertIn("current", entry)
                    self.assertIsInstance(entry["stable"], bool)
                    if entry["stable"]:
                        self.assertEqual(entry.get("next", entry["current"]), entry["current"])

    def test_bundle_names_identifiers_and_document_types(self):
        info = plistlib.loads((ROOT / "packaging/macos/Info.plist").read_bytes())
        for key, name in [
            ("CFBundleIdentifier", "bundle_id"),
            ("CFBundleName", "bundle_name"),
            ("CFBundleDisplayName", "bundle_display_name"),
            ("CFBundleExecutable", "bundle_executable"),
            ("CFBundleIconFile", "bundle_icon"),
        ]:
            with self.subTest(key=key):
                self.assertEqual(info[key], self.current(name))
        self.assertEqual(info["CFBundleDisplayName"], self.current("display_name", "product"))
        for key, name in [
            ("UTExportedTypeDeclarations", "exported_utis"),
            ("UTImportedTypeDeclarations", "imported_utis"),
        ]:
            self.assertEqual(
                [item["UTTypeIdentifier"] for item in info.get(key, [])], self.current(name)
            )
        self.assertEqual(
            [uti for item in info.get("CFBundleDocumentTypes", [])
             for uti in item.get("LSItemContentTypes", [])],
            self.current("document_type_ids"),
        )

    def test_window_app_ids(self):
        self.assert_capture(
            "crates/voxelite-app/src/lib.rs", r'pub\s+const\s+APP_ID\s*:\s*&str\s*=\s*"([^"]+)"',
            self.current("window_app_id"),
        )
        self.assertEqual(self.current("window_app_id"), self.current("bundle_id"))
        for path in ["ui/welcome.rs", "ui/settings.rs"]:
            source = self.source("crates/voxelite-app/src/" + path)
            with self.subTest(path=path):
                self.assertIn("app_id: Some(voxelite_app::APP_ID.into())", source)
                self.assertNotRegex(source, r'app_id\s*:\s*Some\(\s*"')

    def test_package_prefix_and_binaries(self):
        workspace = tomllib.loads(self.source("Cargo.toml"))["workspace"]
        manifests = set()
        for member in workspace["members"]:
            manifests.update(ROOT.glob(f"{member}/Cargo.toml"))
        manifests.difference_update(ROOT / path for path in workspace["exclude"])
        self.assertTrue(manifests)
        for path in sorted(manifests):
            package = tomllib.loads(path.read_text())["package"]["name"]
            if path == ROOT / "xtask/Cargo.toml":
                continue  # Repository tooling has no product prefix.
            with self.subTest(manifest=str(path.relative_to(ROOT))):
                self.assertTrue(package.startswith(self.current("crate_name_prefix")), package)
        app = tomllib.loads(self.source("crates/voxelite-app/Cargo.toml"))
        self.assertIn(self.current("app_binary"), [binary["name"] for binary in app["bin"]])
        mcp = tomllib.loads(self.source("crates/voxelite-mcp/Cargo.toml"))
        self.assertEqual(mcp["package"]["name"], self.current("mcp_binary"))
        self.assertTrue((ROOT / "crates/voxelite-mcp/src/main.rs").is_file())

    def test_mcp_prefix_and_version(self):
        # Check both advertised names and dispatch; there is no prefix constant yet.
        self.assert_capture(
            "crates/voxelite-mcp/src/catalog.rs", r'format!\(\s*"([^"]+)\{name\}"\s*\)',
            self.current("mcp_tool_prefix"),
        )
        self.assert_capture(
            "crates/voxelite-mcp/src/session.rs", r'\.strip_prefix\(\s*"([^"]+)"\s*\)',
            self.current("mcp_tool_prefix"),
        )
        self.assert_capture(
            "crates/voxelite-mcp/src/lib.rs", r'const\s+API_VERSION\s*:\s*&str\s*=\s*"([^"]+)"',
            self.current("mcp_api_version"),
        )

    def test_site_repository_and_tagline(self):
        self.assert_capture(
            "crates/voxelite-site/src/main.rs", r'const\s+SITE_BASE_URL\s*:\s*&str\s*=\s*"([^"]+)"',
            "https://" + self.current("site_domain"),
        )
        self.assert_capture(
            "crates/voxelite-site/templates/github-link.html", r'href="([^"]+)"',
            self.current("github_repo_url"),
        )
        self.assertEqual(
            urlsplit(self.current("sparkle_appcast_url")).hostname, self.current("site_domain")
        )
        self.assertIn(self.current("tagline", "product"), self.source("README.md").splitlines())

    def test_project_format(self):
        schema = json.loads(self.source(self.current("format_spec") + "/schema.json"))
        self.assertEqual(schema["properties"]["format"]["const"], self.current("format_magic"))
        self.assert_capture(
            "crates/voxelite-format/src/lib.rs", r'format:\s*"([^"]+)"\.into\(\)',
            self.current("format_magic"),
        )
        self.assert_capture(
            "crates/voxelite-format/src/lib.rs", r'wire\.format\s*!=\s*"([^"]+)"',
            self.current("format_magic"),
        )
        self.assert_capture(
            "crates/voxelite-format/src/lib.rs", r'const\s+MIME_TYPE\s*:\s*&str\s*=\s*"([^"]+)"',
            self.current("format_mime_type"),
        )
        labels = re.findall(
            r'Self::Project\s*=>\s*"([^"]+)"',
            self.source("crates/voxelite-app/src/ui/save_flow/format.rs"),
        )
        self.assertEqual(labels, [self.current("file_extension").removeprefix("."),
                                  self.current("document_label")])

    def test_storage_paths(self):
        path = "crates/voxelite-app/src/local_settings.rs"
        self.assertEqual(
            re.findall(r'var_os\(\s*"([^"]+)"\s*\)', self.source(path)),
            [self.current("data_directory_env", "storage"), "XDG_DATA_HOME", "HOME"],
        )
        parts = re.findall(r'\.join\(\s*"([^"]+)"\s*\)', self.source(path))
        directory = self.current("data_directory_name", "storage")
        self.assertEqual(parts, [".local/share", directory])
        self.assertEqual(self.current("fallback_data_directory", "storage"), "~/" + "/".join(parts))
        self.assertEqual(self.current("xdg_data_directory", "storage"), "$XDG_DATA_HOME/" + directory)
        self.assert_capture(
            "crates/voxelite-app/src/attachment.rs", r'home\.join\(\s*"([^"]+)"\s*\)',
            self.current("documents_directory", "storage").removeprefix("~/"),
        )

    def test_skill_names(self):
        skills = sorted((ROOT / "skills").glob("*/SKILL.md"))
        self.assertTrue(skills)
        for path in skills:
            with self.subTest(skill=path.parent.name):
                self.assertTrue(path.parent.name.startswith(self.current("skills_name_prefix")))
                self.assertEqual(re.findall(r'^name:\s*(\S+)\s*$', path.read_text(), re.M),
                                 [path.parent.name])


if __name__ == "__main__":
    unittest.main()
