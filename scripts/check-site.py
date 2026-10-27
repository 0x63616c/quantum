#!/usr/bin/env python3
"""Check a built site's navigation, Markdown, SEO, crawl files and source parity."""
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

for name in ('index.html', 'docs/index.html', 'app/index.html',
             'app/voxelite_app.js', 'app/voxelite_app_bg.wasm'):
    if not (SITE / name).is_file():
        raise SystemExit(f'Missing site/{name}; run scripts/build-site.sh first.')

class Page(HTMLParser):
    def __init__(self, text):
        super().__init__()
        self.links, self.ids = [], set()
        self.head, self.titles, self.structured_data = [], [], []
        self.in_head, self.head_count, self.capture = False, 0, None
        self.elements, self.examples, self.example = [], [], None
        self.feed(text)
        self.close()

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        self.elements.append((tag, attrs))
        if tag == 'pre':
            self.example = []
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
        if self.example is not None:
            self.example.append(data)
        if self.capture is not None:
            self.capture.append(data)

    def handle_endtag(self, tag):
        if tag == 'pre' and self.example is not None:
            self.examples.append(''.join(self.example))
            self.example = None
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
    if url.path.startswith('/_vercel/'):
        return  # Served by Vercel at the edge, not part of the static tree.
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
        assert title.endswith(': Quantum docs'), f'Docs title: {title}'
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
        'og:site_name': 'Quantum',
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


ANALYTICS_SCRIPTS = ('/_vercel/insights/script.js', '/_vercel/speed-insights/script.js')
ANALYTICS_QUEUES = ('window.va=window.va||', 'window.si=window.si||')

def check_analytics(path, page):
    sources = [attrs.get('src') for name, attrs in page.head if name == 'script']
    for src in ANALYTICS_SCRIPTS:
        assert sources.count(src) == 1, f'Expected one {src} in head'
        assert 'defer' in next(a for n, a in page.head if n == 'script' and a.get('src') == src), f'{src} must be deferred'
    text = path.read_text()
    for queue in ANALYTICS_QUEUES:
        assert queue in text, f'Missing analytics queue {queue}'


def check_fonts(page):
    # The wasm canvas embeds its native TTFs; its loading screen uses system-ui.
    if not page.tags('link', rel='stylesheet', href='/assets/site.css'):
        return
    for name in ('Geist', 'GeistMono'):
        preload = page.tags('link', rel='preload', href=f'/assets/fonts/{name}.woff2')
        assert len(preload) == 1, f'Missing or duplicated {name} preload'
        assert preload[0].get('as') == 'font' and preload[0].get('type') == 'font/woff2'
        assert 'crossorigin' in preload[0], 'Fonts must preload with CORS'


for path, page in pages.items():
    try:
        check_analytics(path, page)
        check_seo(path, page)
        check_fonts(page)
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
    'name': 'Quantum',
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
doc_pages = {path: page for path, page in pages.items() if path.is_relative_to(SITE / 'docs')}
markdown_urls = {page_url(path).rstrip('/') + '.md' for path in doc_pages}
expected_urls.update(markdown_urls | {f'{BASE_URL}/llms.txt', f'{BASE_URL}/llms-full.txt'})
assert len(locations) == len(set(locations)), 'Duplicate sitemap URLs'
assert set(locations) == expected_urls, 'Sitemap must list public HTML, canonical Markdown and LLM indexes, excluding the 404 and raw aliases'
for location in locations:
    check(SITE / 'sitemap.xml', location)
robots_text = (SITE / 'robots.txt').read_text()
robots = [line.strip() for line in robots_text.splitlines() if line.strip() and not line.startswith('#')]
assert robots == ['User-agent: *', 'Allow: /', f'Sitemap: {BASE_URL}/sitemap.xml'], 'Robots must allow all and link the sitemap'
for name in ('llms.txt', 'llms-full.txt'):
    assert f'{BASE_URL}/{name}' in robots_text, f'Robots must advertise {name} in a comment'

def markdown_links(text):
    # The exports use inline links; ignore fenced examples when checking URLs.
    prose = re.sub(r'^(`{3,}|~{3,})[^\n]*\n.*?^\1\s*$', '', text, flags=re.M | re.S)
    return re.findall(r'\[[^\]]+\]\(([^ )]+)\)', prose)


llms = (SITE / 'llms.txt').read_text()
full = (SITE / 'llms-full.txt').read_text()
assert llms.startswith('# Quantum\n\n> '), 'llms.txt must start with an H1 and summary blockquote'
assert '\n## Documentation\n\n' in llms, 'Missing llms.txt documentation section'
entries = re.findall(r'^- \[([^\]]+)\]\(([^)]+)\): (.+)$', llms, flags=re.M)
assert len(entries) == len(doc_pages), 'llms.txt must list each doc exactly once'
assert {url for _, url, _ in entries} == markdown_urls, 'llms.txt must link to every canonical .md page'
assert full.startswith('# Quantum documentation\n\n> '), 'Missing full documentation heading'
assert full.count('\nSource: [') == len(doc_pages), 'llms-full.txt must contain each doc exactly once'
for html, page in doc_pages.items():
    url = page_url(html).rstrip('/') + '.md'
    path = SITE / urlsplit(url).path.lstrip('/')
    markdown = path.read_text()
    alias = html.with_suffix('.md')
    assert alias.read_bytes() == path.read_bytes(), f'Markdown aliases differ: {path}'
    title = page.titles[0].removesuffix(': Quantum docs')
    assert markdown.startswith(f'# {title}\n\n'), f'Incorrect Markdown heading: {path}'
    assert (title, url, page.meta('description')) in entries, f'Stale llms.txt entry: {path}'
    assert f'\nSource: [{url}]({url})\n\n{markdown}' in full, f'Stale llms-full.txt content: {path}'
    alternate = page.tags('link', rel='alternate', type='text/markdown')
    assert len(alternate) == 1 and alternate[0].get('href') == urlsplit(url).path, f'Incorrect Markdown alternate: {html}'
    describedby = page.tags('link', rel='describedby', type='text/markdown')
    assert len(describedby) == 1 and describedby[0].get('href') == '/llms.txt', f'Missing LLM discovery link: {html}'
    controls = [attrs for tag, attrs in page.elements if tag == 'a' and 'data-markdown' in attrs]
    assert len(controls) == 1 and controls[0].get('href') == urlsplit(url).path, f'Missing Markdown view/copy affordance: {html}'
    examples = re.findall(r'^(`{3,}|~{3,})[^\n]*\n(.*?)\n\1\s*$', markdown, flags=re.M | re.S)
    assert [code for _, code in examples] == page.examples, f'Lost or changed code/schema examples: {path}'
    assert not re.search(r'<(?:script|style|nav|header|footer|table|details)\b', markdown), f'HTML chrome in Markdown: {path}'
    for source in (path, alias):
        for link in markdown_links(markdown):
            check(source, link)
for path in (SITE / 'llms.txt', SITE / 'llms-full.txt'):
    for link in markdown_links(path.read_text()):
        check(path, link)

catalog = json.loads((SITE / 'assets/mcp-tools.json').read_text())
mcp = (SITE / 'docs/mcp.md').read_text()
# Installation now contains JSON too; schemas start at the live tool catalog.
catalog_markdown = re.split(r'^## All \d+ tools\s*$', mcp, maxsplit=1, flags=re.M)[1]
schemas = [json.loads(block) for block in re.findall(r'^```json\n(.*?)\n```', catalog_markdown, flags=re.M | re.S)]
expected_schemas = [tool[key] for tool in catalog for key in ('inputSchema', 'outputSchema') if key in tool]
assert schemas == expected_schemas, 'Markdown MCP schemas must exactly match the live catalog'

for path in (SITE / 'assets').rglob('*.md'):
    for link in re.findall(r'\[[^\]]+\]\(([^ )]+)\)', path.read_text()):
        check(path, link)
for link in re.findall(r'url\([\'"]?([^\)\'\"]+)', (SITE / 'assets/site.css').read_text()):
    check(SITE / 'assets/site.css', link)
css = (SITE / 'assets/site.css').read_text()
assert '.ttf' not in css and 'font-display:swap' not in css, 'Site fonts must not swap late'
for name in ('Geist', 'GeistMono'):
    assert (SITE / f'assets/fonts/{name}.woff2').read_bytes()[:4] == b'wOF2'
    assert (SITE / f'assets/fonts/OFL-{name}.txt').is_file()
hosting = json.loads((SITE / 'vercel.json').read_text())
assert 'outputDirectory' not in hosting, 'site/ is published as the site-dist root'
assert any(rule['source'] == '/assets/fonts/(.*)' and
           {'key': 'Cache-Control', 'value': 'public, max-age=31536000, immutable'} in rule['headers']
           for rule in hosting['headers']), 'Missing immutable font cache headers'
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
print(f'Checked {len(pages)} pages with SEO metadata, {len(doc_pages) * 2} Markdown exports, LLM indexes, {len(locations)} sitemap URLs, robots.txt, '
      f'homepage JSON-LD, {checked} local links/assets, skill archive, MCP catalog and bundled app.')
