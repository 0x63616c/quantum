#!/usr/bin/env python3
"""Check static site's local navigation, downloads, fragments and source parity."""
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlsplit
from zipfile import ZipFile
import json
import hashlib
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
        if 'srcset' in attrs:
            self.links.extend(item.strip().split()[0] for item in attrs['srcset'].split(','))
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
    assert '/gallery/' in page.links, f'{path}: missing Gallery navigation'
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
for name in ('index.html', 'voxelite_app.js', 'voxelite_app_bg.wasm'):
    assert (SITE / 'app' / name).read_bytes() == (ROOT / 'web/dist' / name).read_bytes(), f'Stale app: {name}'
print(f'Checked {len(pages)} pages, {checked} local links/assets, skill archive, MCP catalog and bundled app.')

manifest = json.loads((SITE / 'gallery/manifest.json').read_text())
assert 17 <= len(manifest) <= 25
assert len({model['id'] for model in manifest}) == len(manifest)
assert {model['category'] for model in manifest} == {'Characters', 'Buildings', 'Vehicles', 'Objects', 'Nature'}
for model in manifest:
    source = SITE / model['source'].lstrip('/')
    assert hashlib.sha256(source.read_bytes()).hexdigest() == model['sha256']
    project = json.loads(source.read_text())
    voxels = [v for layer in project['layers'] if layer['visible'] for v in layer['voxels']]
    assert len(voxels) == model['voxel_count']
    assert [max(v[i] for v in voxels) - min(v[i] for v in voxels) + 1 for i in range(3)] == model['dims']
    used = {v[3] for v in voxels}
    colors = [dict(name=s.get('name', ''), hex='#' + ''.join(f'{c:02x}' for c in s['rgba'][:3])) for s in project['palette']['slots'] if s['index'] in used]
    assert colors == model['palette']
    for theme in ('dark', 'light'):
        for size in ('tile', 'large', 'turn'):
            for fmt in model['formats']:
                assert (SITE / 'gallery/images' / f"{model['id']}-{theme}-{size}.{fmt}").is_file()
print(f'Gallery: {len(manifest)} MCP models, verified source hashes, counts, dimensions, palettes and both themes.')
