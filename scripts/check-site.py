#!/usr/bin/env python3
"""Check static site's local navigation, downloads, fragments and source parity."""
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlsplit
from zipfile import ZipFile
import json
import re

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / 'site'

class Page(HTMLParser):
    def __init__(self, text):
        super().__init__()
        self.links, self.ids = [], set()
        self.feed(text)

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if 'id' in attrs:
            assert attrs['id'] not in self.ids, f'Duplicate ID: {attrs["id"]}'
            self.ids.add(attrs['id'])
        for key in ('href', 'src', 'data-dark', 'data-light'):
            if key in attrs:
                self.links.append(attrs[key])

pages = {p: Page(p.read_text()) for p in SITE.rglob('*.html') if 'app' not in p.relative_to(SITE).parts and 'content' not in p.relative_to(SITE).parts}
checked = 0

def check(source, link):
    global checked
    url = urlsplit(link)
    if url.scheme or url.netloc:
        return
    target = (SITE / unquote(url.path).lstrip('/') if url.path.startswith('/')
              else source.parent / unquote(url.path)) if url.path else source
    if target.is_dir():
        target /= 'index.html'
    target = target.resolve()
    assert target.is_relative_to(SITE), f'{source.relative_to(SITE)}: escaping link {link}'
    assert target.is_file(), f'{source.relative_to(SITE)}: missing {link}'
    if url.fragment and target in pages:
        assert unquote(url.fragment) in pages[target].ids, f'{source.relative_to(SITE)}: missing fragment {link}'
    checked += 1

for path, page in pages.items():
    for link in page.links:
        check(path, link)
for path in (SITE / 'assets').rglob('*.md'):
    for link in re.findall(r'\[[^\]]+\]\(([^ )]+)\)', path.read_text()):
        check(path, link)
for link in re.findall(r'url\([\'"]?([^\)\'\"]+)', (SITE / 'assets/site.css').read_text()):
    check(SITE / 'assets/site.css', link)
with ZipFile(SITE / 'assets/voxelite-skills.zip') as archive:
    expected = {str(p.relative_to(ROOT)): p for folder in ('skills', 'integrations') for p in (ROOT / folder).rglob('*') if p.is_file()}
    assert set(archive.namelist()) == set(expected)
    for name, source in expected.items():
        assert archive.read(name) == source.read_bytes(), f'Stale archive: {name}'
        assert (SITE / 'assets' / name).read_bytes() == source.read_bytes(), f'Stale site skill: {name}'
assert json.loads((SITE / 'assets/mcp-tools.json').read_text()) == json.loads((ROOT / 'spec/mcp/v1/tools.json').read_text())
for name in ('voxelite_app.js', 'voxelite_app_bg.wasm'):
    assert (SITE / 'app' / name).read_bytes() == (ROOT / 'web/dist' / name).read_bytes(), f'Stale app: {name}'
def without_seo(text):
    return re.sub(r'<!-- Generated SEO: start -->.*?<!-- Generated SEO: end -->', '', text, flags=re.S)
assert without_seo((SITE / 'app/index.html').read_text()) == without_seo((ROOT / 'web/index.html').read_text()), 'Stale editor shell'
print(f'Checked {len(pages)} pages, {checked} local links/assets, skill archive, MCP catalog and bundled app.')
