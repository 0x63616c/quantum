#!/usr/bin/env python3
"""Exercise CI routing and failure reports without contacting GitHub."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

WORKFLOW = Path(__file__).resolve().parents[2] / '.github/workflows/ci-macos.yml'


def workflow_step(name):
    """Read the actual shell body from a workflow step."""
    lines = WORKFLOW.read_text().splitlines()
    start = lines.index(f'      - name: {name}')
    start = lines.index('        run: |', start) + 1
    body = []
    for line in lines[start:]:
        if line and not line.startswith('          '):
            break
        body.append(line[10:])
    return '\n'.join(body)


class MacosCiTests(unittest.TestCase):
    def run_step(self, name, **env):
        return subprocess.run(
            ['bash', '-euo', 'pipefail', '-c', workflow_step(name)],
            env={**os.environ, **env}, capture_output=True, text=True, check=True,
        )

    def test_rust_package_scope(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / 'output'
            for event in ('push', 'pull_request', 'schedule', 'workflow_dispatch'):
                with self.subTest(event=event):
                    output.write_text('')
                    self.run_step('Select Rust packages', GITHUB_EVENT_NAME=event,
                                  GITHUB_OUTPUT=str(output))
                    packages = output.read_text().strip().removeprefix('packages=')
                    if event in ('schedule', 'workflow_dispatch'):
                        self.assertEqual(packages, '--workspace')
                    else:
                        self.assertEqual(packages.split(), [
                            '-p', 'quantum-app', '-p', 'quantum-ui',
                            '-p', 'quantum-render', '-p', 'quantum-mcp',
                            '-p', 'quantum-bench',
                        ])
        workflow = WORKFLOW.read_text()
        self.assertIn('cargo clippy ${{ steps.rust-packages.outputs.packages }} '
                      '--all-targets --locked --features quantum-app/test-support -- -D warnings', workflow)
        self.assertIn('cargo nextest run ${{ steps.rust-packages.outputs.packages }} '
                      '--locked --features quantum-app/test-support --profile ci', workflow)
        self.assertIn('cargo test --doc ${{ steps.rust-packages.outputs.packages }} '
                      '--locked --features quantum-app/test-support', workflow)

    def test_release_requires_full_checks_before_building(self):
        workflow = (WORKFLOW.parent / 'release-macos.yml').read_text()
        build = workflow.index('      - name: Build app without signing credentials')
        for command in (
            'cargo clippy --workspace --all-targets --locked --features quantum-app/test-support -- -D warnings',
            'cargo test --workspace --locked --features quantum-app/test-support --no-fail-fast',
        ):
            self.assertLess(workflow.index(command), build)
        self.assertNotIn('continue-on-error:', workflow)

    def test_package_routing(self):
        cases = [
            ('push', 'refs/heads/main', '', 'false'),
            ('schedule', 'refs/heads/main', '', 'true'),
            ('workflow_dispatch', 'refs/heads/main', '', 'true'),
            ('workflow_dispatch', 'refs/heads/topic', '', 'false'),
            ('pull_request', 'refs/pull/1/merge', 'packaging/macos/Info.plist', 'true'),
            ('pull_request', 'refs/pull/1/merge', 'crates/quantum-core/src/lib.rs', 'false'),
        ]
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            gh = root / 'gh'
            gh.write_text('#!/bin/bash\nprintf "%s\\n" "$PR_FILES"\n')
            gh.chmod(0o755)
            for event, ref, files, package in cases:
                with self.subTest(event=event, ref=ref, files=files):
                    output = root / 'output'
                    output.write_text('')
                    self.run_step(
                        'Detect code changes', PATH=f'{tmp}:{os.environ["PATH"]}',
                        GITHUB_EVENT_NAME=event, GITHUB_REF=ref,
                        GITHUB_REPOSITORY='example/repo', PR_NUMBER='1',
                        PR_FILES=files, GITHUB_OUTPUT=str(output),
                    )
                    self.assertIn(f'package={package}\n', output.read_text())
                    self.assertIn('code=true\n', output.read_text())

    def test_failure_issue_is_created_or_commented(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            gh = root / 'gh'
            gh.write_text('''#!/usr/bin/env python3
import json, os, sys
with open(os.environ['CALLS'], 'a') as out:
    out.write(json.dumps(sys.argv[1:]) + '\\n')
if sys.argv[1:3] == ['issue', 'list']:
    print(os.environ['EXISTING_ISSUE'])
''')
            gh.chmod(0o755)
            for existing in ('', '42'):
                with self.subTest(existing=existing):
                    calls = root / 'calls'
                    calls.write_text('')
                    self.run_step(
                        'Open or update nightly failure issue',
                        PATH=f'{tmp}:{os.environ["PATH"]}', CALLS=str(calls),
                        EXISTING_ISSUE=existing, RUN_URL='https://example.com/run/1',
                        CHANGES_RESULT='success', SCRIPTS_RESULT='success',
                        MACOS_RESULT='failure',
                    )
                    commands = [json.loads(line) for line in calls.read_text().splitlines()]
                    last = commands[-1]
                    self.assertEqual(last[:2], ['issue', 'comment' if existing else 'create'])
                    self.assertIn('https://example.com/run/1', last[last.index('--body') + 1])
                    labels = commands[-2] if existing else last
                    self.assertIn('type:ci,area:ci,priority:p0', labels)
