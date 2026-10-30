#!/usr/bin/env python3
"""Exercise release workflow shell steps without contacting GitHub."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/release-macos.yml"


def workflow_step(name):
    """Extract a literal run block so tests execute the workflow's actual shell."""
    lines = WORKFLOW.read_text().splitlines()
    start = lines.index(f"      - name: {name}")
    start = lines.index("        run: |", start) + 1
    body = []
    for line in lines[start:]:
        if line and not line.startswith("          "):
            break
        body.append(line[10:])
    return "\n".join(body)


class ReleaseTests(unittest.TestCase):
    def test_release_tags(self):
        for tag in ["v0.1.0", "v12.34.56", "main", "v1", "v01.2.3",
                    "v1.2.3-rc.1", "v1.2.3\nmain", "--help", "$(false)"]:
            with self.subTest(tag=tag):
                result = subprocess.run(
                    ["bash", "-euo", "pipefail", "-c", workflow_step("Validate release tag")],
                    env=os.environ | {"TAG": tag}, capture_output=True,
                )
                self.assertEqual(result.returncode == 0, tag in ["v0.1.0", "v12.34.56"])
        self.assertIn("ref: refs/tags/${{ env.TAG }}", WORKFLOW.read_text())
        self.assertIn("          fetch-depth: 0", WORKFLOW.read_text())

    def test_self_hosted_guards_and_pins(self):
        guard = ("    if: github.actor == '0x63616c' && (github.event_name != 'pull_request' || "
                 "github.event.pull_request.head.repo.full_name == github.repository)")
        for path in (ROOT / ".github/workflows").glob("*.yml"):
            workflow = path.read_text()
            self.assertNotIn("pull_request_target:", workflow)
            if "self-hosted" in workflow:
                self.assertIn(guard, workflow.splitlines())
            for line in workflow.splitlines():
                if "uses: actions/" in line:
                    self.assertRegex(line, r"uses: actions/[a-z-]+@[0-9a-f]{40}(?: |$)")

    def test_release_build_precedes_credentials(self):
        workflow = WORKFLOW.read_text()
        build = workflow.index("      - name: Build app without signing credentials")
        package = workflow.index("      - name: Sign and package")
        self.assertLess(build, package)
        self.assertNotIn("secrets.", workflow[:package])
        self.assertIn('cargo build --release --locked -p voxelite-mcp',
                      workflow_step('Build app without signing credentials'))
        self.assertIn("scripts/package-macos.sh --skip-build", workflow_step("Sign and package"))
        notes = workflow.index("      - name: Generate release notes")
        appcast = workflow.index("      - name: Generate signed update feed")
        self.assertLess(package, notes)  # Packaging clears dist/ and writes .notarized.
        self.assertLess(notes, appcast)

    def test_tag_must_match_cargo_before_building(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cargo = root / "cargo"
            cargo.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
if sys.argv[1] == 'metadata':
    print(json.dumps({'packages': [{'name':'voxelite-app', 'version':'1.2.3'}]}))
else:
    pathlib.Path(os.environ['BUILT']).touch()
''')
            cargo.chmod(0o755)
            for tag, success in (("v1.2.4", False), ("v1.2.3", True)):
                result = subprocess.run(
                    ["bash", "-euo", "pipefail", "-c", workflow_step("Build app without signing credentials")],
                    env=os.environ | {"PATH": str(root) + os.pathsep + os.environ["PATH"],
                                      "TAG": tag, "BUILT": str(root / "built")},
                    capture_output=True,
                )
                self.assertEqual(result.returncode == 0, success)
                self.assertEqual((root / "built").exists(), success)

    def test_site_release_trigger_and_shared_concurrency(self):
        workflow = (ROOT / ".github/workflows/deploy-site.yml").read_text()
        self.assertIn('workflows: ["Release macOS"]', workflow)
        self.assertIn("github.event.workflow_run.conclusion == 'success'", workflow)
        self.assertIn("github.event.workflow_run.actor.login == '0x63616c'", workflow)
        self.assertIn("github.event.workflow_run.head_repository.full_name == github.repository", workflow)
        self.assertIn("group: deploy-site-${{ github.ref }}", workflow)
        self.assertIn("python3 scripts/fetch-macos-appcast.py site/appcast.xml", workflow)

    def test_release_requires_all_apple_credentials_before_packaging(self):
        names = ("MACOS_CERTIFICATE_P12_BASE64", "MACOS_CERTIFICATE_PASSWORD",
                 "APPLE_API_KEY_P8_BASE64", "APPLE_API_KEY_ID", "APPLE_API_ISSUER_ID")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "scripts").mkdir()
            package = root / "scripts/package-macos.sh"
            package.write_text("#!/bin/sh\ntouch packaged\n")
            package.chmod(0o755)
            for missing in (*names, None):
                credentials = {name: "fixture value" if name != missing else "" for name in names}
                result = subprocess.run(
                    ["bash", "-euo", "pipefail", "-c", workflow_step("Sign and package")],
                    cwd=root, env=os.environ | credentials | {"TAG": "v1.2.3"},
                    capture_output=True, text=True,
                )
                self.assertEqual(result.returncode == 0, missing is None)
                self.assertEqual((root / "packaged").exists(), missing is None)
                self.assertNotIn("fixture value", result.stdout + result.stderr)

    def test_publication_is_staged_and_immutable(self):
        for generated, failure in [(True, ""), (False, ""), (True, "create"), (True, "upload")]:
            with self.subTest(generated=generated, failure=failure), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                gh = root / "gh"
                gh.write_text('''#!/usr/bin/env python3
import json, os, sys
with open(os.environ["CALLS"], "a") as log:
    log.write(json.dumps(sys.argv[1:]) + "\\n")
if sys.argv[2] == os.environ["FAIL_AT"]:
    sys.exit(1)
''')
                gh.chmod(0o755)
                result = subprocess.run(
                    ["bash", "-euo", "pipefail", "-c", workflow_step("Publish release")],
                    env=os.environ | {
                        "PATH": str(root) + os.pathsep + os.environ["PATH"],
                        "TAG": "v1.2.3", "APPCAST_GENERATED": str(generated).lower(),
                        "CALLS": str(root / "calls"), "FAIL_AT": failure,
                    }, capture_output=True,
                )
                calls = [json.loads(line) for line in (root / "calls").read_text().splitlines()] if (root / "calls").exists() else []
                self.assertEqual(result.returncode == 0, generated and not failure)
                if not generated:
                    self.assertEqual(calls, [])
                    continue
                self.assertIn("--draft", calls[0])
                self.assertIn("--verify-tag", calls[0])
                self.assertEqual(calls[0][calls[0].index("--notes-file") + 1],
                                 "dist/RELEASE_NOTES.md")
                if failure == "create":
                    self.assertEqual(len(calls), 1)
                else:
                    self.assertEqual(calls[1][:3], ["release", "upload", "v1.2.3"])
                    self.assertNotIn("--clobber", calls[1])
                    self.assertIn("dist/Voxelite.dmg", calls[1])
                    self.assertIn("dist/Voxelite.dmg.sha256", calls[1])
                    self.assertIn("dist/Voxelite.html", calls[1])
                    self.assertEqual("site/appcast.xml" in calls[1], generated)
                    if failure == "upload":
                        self.assertEqual(len(calls), 2)
                    else:
                        self.assertEqual(calls[2], ["release", "edit", "v1.2.3", "--draft=false"])


class ReleaseNotesTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory(prefix="voxelite-release-test-")
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        for name in ("scripts/generate-macos-release-notes.py", "packaging/macos/RELEASE_NOTES.md",
                     "packaging/macos/RELEASE_NOTES.unnotarized.md"):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, path)
        self.env = os.environ | {
            "GIT_AUTHOR_NAME": "Private Author", "GIT_COMMITTER_NAME": "Private Committer",
            "GIT_AUTHOR_EMAIL": "author@example.invalid", "GIT_COMMITTER_EMAIL": "committer@example.invalid",
        }
        self.git("init", "--initial-branch=main")
        self.git("config", "commit.gpgsign", "false")
        self.git("config", "tag.gpgsign", "false")
        self.initial = self.commit("Initial version")

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, env=self.env,
                                       stderr=subprocess.PIPE, text=True).strip()

    def commit(self, subject):
        self.git("commit", "--allow-empty", "-m", subject, "-m", "Private commit body")
        return self.git("rev-parse", "--short", "HEAD")

    def notes(self, tag, notarized=False):
        result = subprocess.run(
            ["bash", "-euo", "pipefail", "-c", workflow_step("Generate release notes")],
            cwd=self.root, env=self.env | {"TAG": tag}, capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        notes = (self.root / "dist/RELEASE_NOTES.md").read_text()
        template = "RELEASE_NOTES.md" if notarized else "RELEASE_NOTES.unnotarized.md"
        self.assertTrue(notes.startswith((ROOT / "packaging/macos" / template).read_text().rstrip() + "\n\n"))
        for private in ("Private Author", "Private Committer", "@example.invalid", "Private commit body"):
            self.assertNotIn(private, notes)
        return notes

    def test_installation_notes_follow_current_notarization_marker(self):
        self.git("tag", "v0.3.0")
        # A fresh checkout, a notarized package, then an unnotarized rebuild.
        unnotarized = self.notes("v0.3.0")
        self.assertIn("not notarized", unnotarized)
        self.assertIn("right-click", unnotarized)
        self.assertIn("Open Anyway", unnotarized)
        self.assertNotIn("notarized by Apple", unnotarized)
        marker = self.root / "dist/.notarized"
        marker.touch()
        notarized = self.notes("v0.3.0", notarized=True)
        self.assertIn("Developer ID signed and notarized by Apple", notarized)
        self.assertIn("0.3.0", notarized)
        self.assertIn("updates automatically through Sparkle", notarized)
        for workaround in ("not notarized", "right-click", "Open Anyway"):
            self.assertNotIn(workaround, notarized)
        self.assertEqual(notarized.split("## Changes since ")[1],
                         unnotarized.split("## Changes since ")[1])
        marker.unlink()
        self.assertEqual(self.notes("v0.3.0"), unnotarized)

    def test_previous_stable_tag_and_release_boundary(self):
        self.git("tag", "-a", "v0.1.0", "-m", "First release")
        self.commit("Previous stable version")
        self.git("tag", "v0.2.0")
        first = self.commit('Fix <script>alert("x")</script> & café')
        for name in ("v0.3.0-rc.1", "v0.3.0+build", "v0.3.0.1", "v00.3.0", "nightly"):
            self.git("tag", name)
        last = self.commit("Prepare release")
        self.git("tag", "-a", "v0.3.0", "-m", "Release")
        self.git("tag", "v0.2.1")  # Same-commit tags are not previous releases.
        self.commit("Future change")
        self.git("tag", "v0.4.0")
        self.git("branch", "v0.3.0")  # A colliding branch must not supply commits.
        self.git("checkout", "-b", "unmerged", self.initial)
        self.commit("Unmerged change")
        self.git("tag", "v9.0.0")
        notes = self.notes("v0.3.0")
        self.assertEqual(notes.split("## Changes since ")[1],
                         f'v0.2.0\n\n- Prepare release ({last})\n'
                         f'- Fix <script>alert("x")</script> & café ({first})\n')

    def test_first_release_on_root_commit(self):
        self.git("tag", "v1.0.0")
        self.git("tag", "v0.9.0")
        notes = self.notes("v1.0.0")
        self.assertIn(f"## Changes since the beginning\n\n- Initial version ({self.initial})\n", notes)

    def test_no_previous_stable_tag_includes_all_commits(self):
        self.git("tag", "v0.1.0-rc.1")
        last = self.commit("First stable version")
        self.git("tag", "v0.1.0")
        notes = self.notes("v0.1.0")
        self.assertIn(f"## Changes since the beginning\n\n- First stable version ({last})\n"
                      f"- Initial version ({self.initial})\n", notes)

    def test_merged_history_and_annotated_previous_tag(self):
        self.git("checkout", "-b", "feature")
        self.commit("Previous release on merged branch")
        self.git("tag", "-a", "v0.1.0", "-m", "Previous release")
        feature = self.commit("Feature change")
        self.git("checkout", "main")
        main = self.commit("Main change")
        self.git("merge", "--no-ff", "feature", "-m", "Merge subject must be omitted")
        self.git("tag", "v0.2.0")
        notes = self.notes("v0.2.0")
        self.assertIn("## Changes since v0.1.0", notes)
        self.assertIn(f"- Feature change ({feature})", notes)
        self.assertIn(f"- Main change ({main})", notes)
        self.assertNotIn("Merge subject", notes)
        self.assertNotIn("Previous release on merged branch", notes)
        self.assertNotIn("Initial version", notes)


if __name__ == "__main__":
    unittest.main()
