#!/usr/bin/env python3
"""Check that site validation explains missing local build output."""
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class SiteTests(unittest.TestCase):
    def test_checker_requires_built_pages(self):
        for pages in ((), ('index.html',), ('index.html', 'docs/index.html')):
            with self.subTest(pages=pages), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                (root / 'scripts').mkdir()
                checker = root / 'scripts/check-site.py'
                shutil.copyfile(ROOT / 'scripts/check-site.py', checker)
                for name in pages:
                    path = root / 'site' / name
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text('<!doctype html>')
                result = subprocess.run([sys.executable, '-B', str(checker)],
                                        capture_output=True, text=True, check=False)
                self.assertNotEqual(result.returncode, 0)
                missing = ('index.html', 'docs/index.html', 'download/index.html')[len(pages)]
                self.assertIn(f'Missing site/{missing}', result.stderr)
                self.assertIn('run scripts/build-site.sh first', result.stderr)
                self.assertNotIn('Traceback', result.stderr)


if __name__ == '__main__':
    unittest.main()
