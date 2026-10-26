#!/usr/bin/env python3
"""Check static site navigation, SEO metadata, crawl files and source parity."""
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlsplit
from zipfile import ZipFile
from xml.etree import ElementTree
import json
import re

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / 'site'
BASE_URL = 'https://voxelite.worldwidewebb.co'

class Page(HTMLParser):
    def __init__(self, text):
        super().__init__()
        self.links, self.ids = [], set()
        self.head, self.titles, self.structured_data = [], [], []
        self.in_head, self.head_count, self.capture = False, 0, None
        self.feed(text)
        self.close()

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == 'head':
            self.in_head = True
            self.head_count += 1
        elif self.in_head:
            self.head.append((tag, attrs))
            if tag == 'title' or (tag == 'script' and attrs.get('type') == 'application/ld+json'):
                self.capture = []
        if 'id' in attrs:
            assert attrs['id'] not in self.ids, f'Duplicate ID: {attrs["id"]}'
            self.ids.add(attrs['id'])
        for key in ('href', 'src', 'data-dark', 'data-light'):
            if key in attrs:
                self.links.append(attrs[key])

    def handle_data(self, data):
        if self.capture is not None:
            self.capture.append(data)

    def handle_endtag(self, tag):
        if self.in_head and self.capture is not None and tag in ('title', 'script'):
            target = self.titles if tag == 'title' else self.structured_data
            target.append(''.join(self.capture).strip())
            self.capture = None
        if tag == 'head':
            self.in_head = False

    def tags(self, tag, **attributes):
        return [attrs for name, attrs in self.head if name == tag
                and all(attrs.get(key) == value for key, value in attributes.items())]

    def meta(self, key, attribute='name'):
        matches = self.tags('meta', **{attribute: key})
        assert len(matches) == 1, f'Expected one {attribute}="{key}", found {len(matches)}'
        value = matches[0].get('content', '')
        assert value.strip(), f'Empty metadata: {key}'
        return value

# content/ contains build-time article fragments, not standalone pages.
pages = {p: Page(p.read_text()) for p in sorted(SITE.rglob('*.html'))
         if 'content' not in p.relative_to(SITE).parts}
checked = 0

def check(source, link):
    global checked
    url = urlsplit(link)
    if (url.scheme or url.netloc) and (url.scheme, url.netloc) != ('https', urlsplit(BASE_URL).netloc):
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

def page_url(path):
    relative = path.relative_to(SITE).as_posix()
    if relative.endswith('index.html'):
        relative = relative.removesuffix('index.html')
    return f'{BASE_URL}/{relative}'


def check_seo(path, page):
    assert page.head_count == 1 and not page.in_head, 'Expected one complete head'
    assert len(page.titles) == 1 and page.titles[0], 'Expected one nonempty title'
    title = page.titles[0]
    description = page.meta('description')
    assert 120 <= len(description) <= 160, f'Description length: {len(description)}'
    if path.is_relative_to(SITE / 'docs'):
        assert title.endswith(' — Voxelite docs'), f'Docs title: {title}'
        assert 'documentation: documentation' not in description.lower(), description

    canonical = [attrs for attrs in page.tags('link') if 'canonical' in attrs.get('rel', '').split()]
    if path == SITE / '404.html':
        assert not canonical, '404 must not have a canonical URL'
        assert 'noindex' in re.split(r'[\s,]+', page.meta('robots').lower()), '404 must be noindex'
    else:
        assert len(canonical) == 1, 'Expected exactly one canonical URL'
        assert canonical[0].get('href') == page_url(path), 'Incorrect canonical URL'
        for attrs in page.tags('meta'):
            if attrs.get('name', '').lower() in ('robots', 'googlebot', 'bingbot'):
                directives = re.split(r'[\s,]+', attrs.get('content', '').lower())
                assert not {'noindex', 'none'} & set(directives), 'Public page must be indexable'

    for key, expected in {
        'og:title': title,
        'og:description': description,
        'og:url': page_url(path),
        'og:type': 'website',
        'og:site_name': 'Voxelite',
    }.items():
        assert page.meta(key, 'property') == expected, f'Incorrect {key}'
    image = page.meta('og:image', 'property')
    assert image.startswith(f'{BASE_URL}/assets/'), 'Social image must use the absolute production URL'
    assert not urlsplit(image).query and not urlsplit(image).fragment, 'Social image must have a stable URL'
    assert urlsplit(image).path.endswith(('.png', '.jpg', '.jpeg')), 'Expected a raster social image'
    check(path, image)
    assert page.meta('twitter:card') == 'summary_large_image', 'Incorrect Twitter card'
    assert page.meta('twitter:title') == title, 'Incorrect Twitter title'
    assert page.meta('twitter:description') == description, 'Incorrect Twitter description'
    assert page.meta('twitter:image') == image, 'Twitter and OG images differ'
    assert page.meta('twitter:image:alt') == page.meta('og:image:alt', 'property'), 'Image alt text differs'
    icons = [attrs for attrs in page.tags('link') if 'icon' in attrs.get('rel', '').split()]
    assert len(icons) == 1, 'Expected one favicon'
    assert icons[0].get('type') == 'image/svg+xml', 'Expected an SVG favicon'
    assert icons[0].get('href') in ('/assets/mark.svg', f'{BASE_URL}/assets/mark.svg'), 'Incorrect favicon'


for path, page in pages.items():
    try:
        check_seo(path, page)
    except AssertionError as error:
        raise AssertionError(f'{path.relative_to(SITE)}: {error}') from error
    for link in page.links:
        check(path, link)

titles = [page.titles[0] for page in pages.values()]
descriptions = [page.meta('description') for page in pages.values()]
assert len(titles) == len(set(titles)), 'Page titles must be unique'
assert len(descriptions) == len(set(descriptions)), 'Page descriptions must be unique'

home = pages[SITE / 'index.html']
assert len(home.structured_data) == 1, 'Expected one homepage JSON-LD block'
application = json.loads(home.structured_data[0])
for key, expected in {
    '@context': 'https://schema.org',
    '@type': 'SoftwareApplication',
    'name': 'Voxelite',
    'description': home.meta('description'),
    'applicationCategory': 'DesignApplication',
    'operatingSystem': 'Web, macOS',
    'url': f'{BASE_URL}/',
    'image': home.meta('og:image', 'property'),
    'license': f'{BASE_URL}/assets/LICENSE',
    'author': {'@type': 'Organization', 'name': 'World Wide Webb'},
    'publisher': {'@type': 'Organization', 'name': 'World Wide Webb'},
}.items():
    assert application.get(key) == expected, f'Incorrect SoftwareApplication {key}'
assert application.get('isAccessibleForFree') is True, 'Application must be free'
assert 'MIT License' in (SITE / 'assets/LICENSE').read_text(), 'Expected MIT license'
offer = application['offers']
assert offer['@type'] == 'Offer' and offer['price'] == 0 and offer['priceCurrency'] == 'USD', 'Expected a free offer'

namespace = '{http://www.sitemaps.org/schemas/sitemap/0.9}'
sitemap = ElementTree.parse(SITE / 'sitemap.xml').getroot()
assert sitemap.tag == f'{namespace}urlset', 'Incorrect sitemap namespace'
locations = [loc.text for loc in sitemap.findall(f'{namespace}url/{namespace}loc')]
expected_urls = {page_url(path) for path in pages if path != SITE / '404.html'}
assert len(locations) == len(set(locations)), 'Duplicate sitemap URLs'
assert set(locations) == expected_urls, 'Sitemap must list every public page and exclude the 404'
robots = [line.strip() for line in (SITE / 'robots.txt').read_text().splitlines() if line.strip()]
assert robots == ['User-agent: *', 'Allow: /', f'Sitemap: {BASE_URL}/sitemap.xml'], 'Robots must allow all and link the sitemap'

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
assert (SITE / 'app/index.html').read_bytes() == (ROOT / 'web/index.html').read_bytes(), 'Stale app template'
print(f'Checked {len(pages)} pages with SEO metadata, {len(locations)} sitemap URLs, robots.txt, '
      f'homepage JSON-LD, {checked} local links/assets, skill archive, MCP catalog and bundled app.')
