#!/usr/bin/env python3
"""Check that the scanner reports redacted findings without writing files."""

import contextlib
import io
from pathlib import Path
import runpy
import tempfile
import unittest
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[2]


class ScanPiiTests(unittest.TestCase):
    def run_scanner(self, content, with_history=False):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            script = root / "scripts/scan-pii.py"
            script.parent.mkdir()
            script.write_bytes((ROOT / "scripts/scan-pii.py").read_bytes())
            (root / "sample.txt").write_bytes(content)
            commit = "a" * 40
            blob = "b" * 40
            responses = {
                ("rev-list", "--all", "--reflog"): (
                    f"{commit}\n".encode() if with_history else b""
                ),
                ("ls-files", "-z", "--cached", "--others", "--exclude-standard"): (
                    b"sample.txt\0"
                ),
                ("rev-list", "--all", "--reflog", "--objects"): (
                    f"{blob} historical.txt\n".encode() if with_history else b""
                ),
                ("cat-file", "commit", commit): (
                    b"author Fixture Author <fixture@example.invalid> 0 +0000\n"
                    b"committer Fixture Author <fixture@example.invalid> 0 +0000\n\n"
                    + content
                ),
            }

            def git_output(args, cwd):
                self.assertEqual(cwd, root)
                self.assertEqual(args[0], "git")
                return responses[tuple(args[1:])]

            batch = f"{blob} blob {len(content)}\n".encode() + content + b"\n"
            process = Mock(stdin=io.BytesIO(), stdout=io.BytesIO(batch))
            process.wait.return_value = 0
            before = {p.relative_to(root): p.read_bytes()
                      for p in root.rglob("*") if p.is_file()}
            output = io.StringIO()
            with patch("subprocess.check_output", side_effect=git_output), \
                    patch("subprocess.Popen", return_value=process), \
                    contextlib.redirect_stdout(output):
                runpy.run_path(str(script), run_name="__main__")
            after = {p.relative_to(root): p.read_bytes()
                     for p in root.rglob("*") if p.is_file()}
            self.assertEqual(after, before, "Scanning must not write a report file")
            return output.getvalue()

    def test_full_report_is_printed_with_redacted_current_and_history_findings(self):
        email = "private@example.invalid"
        credential = "ghp_" + "x" * 24
        output = self.run_scanner(f"{email}\n{credential}\n".encode(), with_history=True)
        self.assertTrue(output.startswith("# Repository and Git history personal-info scan\n"))
        for section in ["Git metadata", "Working tree findings", "Historical blob findings",
                        "Commit messages", "Outcome"]:
            self.assertIn(f"## {section}\n", output)
        self.assertIn("`sample.txt:1` | email |", output)
        self.assertIn("`sample.txt:2` | credential-shaped string |", output)
        self.assertIn("`historical.txt:1` | email |", output)
        self.assertIn("`aaaaaaaaaaaa:message:2` — credential-shaped string.", output)
        for value in [email, credential, "Fixture Author", "fixture@example.invalid"]:
            self.assertNotIn(value, output)

    def test_empty_findings_still_print_a_complete_report(self):
        output = self.run_scanner(b"Ordinary project documentation.\n")
        self.assertIn("| — | No matches | — |", output)
        self.assertIn("| — | No blob matches |", output)
        self.assertIn("No pattern matches in commit messages.", output)
        self.assertIn("## Outcome\n", output)


if __name__ == "__main__":
    unittest.main()
