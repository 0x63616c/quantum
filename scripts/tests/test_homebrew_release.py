"""Exercise the actual tap workflow shell with local fake release and git tools."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


def step():
    text = (ROOT / '.github/workflows/release-macos.yml').read_text()
    return '\n'.join(line[10:] for line in text.split(
        '      - name: Update Homebrew tap\n', 1)[1].split('        run: |\n', 1)[1].splitlines())



class HomebrewReleaseTests(unittest.TestCase):
    def run_shell(self, secret='', tag='v0.4.0', mismatch=False, unchanged=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            log = root / 'log'
            for name, body in {
                'gh': '''#!/bin/bash
while [[ "$1" != --dir ]]; do shift; done
printf dmg > "$2/Quantum.dmg"
sha256sum "$2/Quantum.dmg" > "$2/Quantum.dmg.sha256"
if [[ "$MISMATCH" = yes ]]; then printf '%064d' 0 > "$2/Quantum.dmg.sha256"; fi
''',
                'cargo': '''#!/bin/bash
[[ -z "${HOMEBREW_TAP_DEPLOY_KEY+x}" && -z "${GH_TOKEN+x}" ]] || exit 2
echo cargo >> "$LOG"
while [[ "$1" != --output ]]; do shift; done
printf cask > "$2"
''',
                'git': '''#!/bin/bash
echo "$*" >> "$LOG"
if [[ "$1" = clone ]]; then mkdir -p "${@: -1}/Casks"; fi
if [[ "$*" = *'diff --cached --quiet'* ]]; then [[ "$UNCHANGED" = yes ]]; fi
''',
            }.items():
                tool = root / name
                tool.write_text(body)
                tool.chmod(0o755)
            result = subprocess.run(['bash', '-euo', 'pipefail', '-c', step()], cwd=ROOT,
                env={**os.environ, 'PATH': f'{root}:' + os.environ['PATH'],
                     'HOMEBREW_TAP_DEPLOY_KEY': secret, 'GH_TOKEN': 'test-token', 'TAG': tag,
                     'LOG': str(log), 'MISMATCH': 'yes' if mismatch else 'no',
                     'UNCHANGED': 'yes' if unchanged else 'no'}, capture_output=True, text=True)
            if secret:
                self.assertNotIn(secret, result.stdout + result.stderr)
            return result, log.read_text() if log.exists() else ''

    def test_missing_key_skips_successfully(self):
        result, log = self.run_shell()
        self.assertEqual(result.returncode, 0)
        self.assertIn('::warning::', result.stdout)
        self.assertEqual(log, '')

    def test_checksum_mismatch_and_invalid_tag_never_write_tap(self):
        for options in ({'mismatch': True}, {'tag': 'v0.4.0;false'}):
            result, log = self.run_shell(secret='private-test-key', **options)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(log, '')

    def test_changed_cask_pushes_and_rerun_is_noop(self):
        for unchanged in (False, True):
            result, log = self.run_shell(secret='private-test-key', unchanged=unchanged)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual('push origin HEAD:main' in log, not unchanged)
            self.assertNotIn('--force', log)

    def test_release_job_is_isolated(self):
        workflow = (ROOT / '.github/workflows/release-macos.yml').read_text().split('  homebrew-tap:', 1)[1]
        for text in ('needs: dmg', 'runs-on: ubuntu-24.04', 'continue-on-error: true',
                     'permissions: { contents: read }', 'persist-credentials: false',
                     'ref: refs/tags/${{ env.TAG }}'):
            self.assertIn(text, workflow)
        self.assertIn('StrictHostKeyChecking=yes', step())
        self.assertNotIn('ssh-keyscan', step())
