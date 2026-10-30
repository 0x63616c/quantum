#!/usr/bin/env python3
"""Validate local links in the domain glossary and product direction."""

from pathlib import Path
import re
import unittest
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[2]


class DocumentationTests(unittest.TestCase):
    def test_domain_and_direction_links_resolve_from_their_document(self):
        for relative in ["docs/glossary.md", "docs/adr/0003-product-direction.md"]:
            document = ROOT / relative
            links = re.findall(r"\[[^\]]+\]\(([^\s)]+)\)", document.read_text())
            self.assertTrue(links, relative)
            for link in links:
                url = urlsplit(link)
                if url.scheme or url.netloc or not url.path:
                    continue
                with self.subTest(document=relative, link=link):
                    self.assertTrue((document.parent / unquote(url.path)).exists())


if __name__ == "__main__":
    unittest.main()
