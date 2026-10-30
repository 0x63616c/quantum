#!/usr/bin/env python3
"""Keep documentation images available independently of local capture output."""

from pathlib import Path
import re
import subprocess
import unittest
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[2]


class DocumentationAssetTests(unittest.TestCase):
    def test_local_markdown_image_targets_exist(self):
        documents = [ROOT / name for name in (
            'README.md', 'AGENTS.md', 'examples/README.md',
            'site/README.md', 'design/ART-CHECKLIST.md',
        )]
        documents.extend((ROOT / 'docs').rglob('*.md'))
        documents.extend((ROOT / 'crates').rglob('README.md'))
        checked = 0
        for document in documents:
            for target in re.findall(r'\[[^\]\n]*\]\(([^\s)]+)\)', document.read_text()):
                url = urlsplit(target)
                if url.scheme or url.netloc:
                    continue
                path = document.parent / unquote(url.path)
                if path.suffix.lower() not in {'.png', '.jpg', '.jpeg', '.gif', '.svg', '.webp'}:
                    continue
                with self.subTest(document=document.relative_to(ROOT), target=target):
                    self.assertTrue(path.is_file(), f'Missing documentation image: {path}')
                    self.assertFalse(path.resolve().is_relative_to(ROOT / 'screenshots'))
                checked += 1
        self.assertGreater(checked, 0)

    def test_captures_are_ignored_but_documentation_images_are_not(self):
        captures = [str(Path('screenshots') / name) for name in (
            'app-dark.png', 'site/final/verification.json', 'review-1/tools.png',
        )]
        images = [str(path.relative_to(ROOT)) for path in (ROOT / 'docs/images').glob('*.png')]
        self.assertTrue(images)
        result = subprocess.run(
            ['git', 'check-ignore', '--no-index', '--stdin'], cwd=ROOT,
            input='\n'.join(captures + images) + '\n', text=True,
            capture_output=True, check=True,
        )
        self.assertEqual(result.stdout.splitlines(), captures)


if __name__ == '__main__':
    unittest.main()
