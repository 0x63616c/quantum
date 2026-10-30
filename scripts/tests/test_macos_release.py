#!/usr/bin/env python3
"""Exercise release workflow shell steps without contacting GitHub."""

import json
import os
from pathlib import Path
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
        self.assertIn("scripts/package-macos.sh --skip-build", workflow_step("Sign and package"))

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
                calls = [json.loads(line) for line in (root / "calls").read_text().splitlines()]
                self.assertEqual(result.returncode == 0, not failure)
                self.assertIn("--draft", calls[0])
                self.assertIn("--verify-tag", calls[0])
                if failure == "create":
                    self.assertEqual(len(calls), 1)
                else:
                    self.assertEqual(calls[1][:3], ["release", "upload", "v1.2.3"])
                    self.assertNotIn("--clobber", calls[1])
                    self.assertIn("dist/Voxelite.dmg", calls[1])
                    self.assertIn("dist/Voxelite.dmg.sha256", calls[1])
                    self.assertEqual("site/appcast.xml" in calls[1], generated)
                    if failure == "upload":
                        self.assertEqual(len(calls), 2)
                    else:
                        self.assertEqual(calls[2], ["release", "edit", "v1.2.3", "--draft=false"])


if __name__ == "__main__":
    unittest.main()
