#!/usr/bin/env python3
"""Parse every site HTML file and validate SEO, JSON-LD and crawl files offline."""
import argparse
from collections import defaultdict
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import sys
from urllib.parse import unquote, urlsplit
from urllib.robotparser import RobotFileParser
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
NS = '{http://www.sitemaps.org/schemas/sitemap/0.9}'


def require(condition, message):
    if not condition:
        raise ValueError(message)


class Page(HTMLParser):
    def __init__(self, path, text):
        super().__init__(convert_charrefs=True)
        self.path = path
        self.heads = self.bodies = 0
        self.in_head = self.in_title = self.in_json = False
        self.titles, self.canonicals, self.json_ld = [], [], []
        self.meta = defaultdict(list)
        self.feed(text)
        self.close()
        require(not (self.in_head or self.in_title or self.in_json), f'{path}: unclosed metadata')

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        if tag == 'head':
            self.heads += 1
            self.in_head = True
        elif tag == 'body':
            self.bodies += 1
        canonical = tag == 'link' and 'canonical' in attrs.get('rel', '').lower().split()
        json_ld = tag == 'script' and attrs.get('type', '').lower() == 'application/ld+json'
        if tag in ('title', 'meta') or canonical or json_ld:
            require(self.in_head, f'{self.path}: {tag} metadata outside head')
            require(len(attributes) == len(attrs), f'{self.path}: duplicate {tag} attributes')
        if tag == 'title':
            self.titles.append('')
            self.in_title = True
        elif tag == 'meta':
            for kind in ('name', 'property'):
                if kind in attrs:
                    self.meta[kind, attrs[kind].lower()].append(attrs.get('content', ''))
        elif canonical:
            self.canonicals.append(attrs.get('href', ''))
        elif json_ld:
            self.json_ld.append('')
            self.in_json = True

    def handle_endtag(self, tag):
        if tag == 'head':
            self.in_head = False
        elif tag == 'title':
            self.in_title = False
        elif tag == 'script':
            self.in_json = False

    def handle_data(self, data):
        if self.in_title:
            self.titles[-1] += data
        if self.in_json:
            self.json_ld[-1] += data

    def one(self, values, label):
        require(len(values) == 1 and values[0].strip(), f'{self.path}: expected one nonempty {label}')
        return values[0]

    def tag(self, name, kind='name'):
        return self.one(self.meta[kind, name], name)


def absolute_url(value, label, base=None):
    require(isinstance(value, str) and value and not re.search(r'\s|\\', value), f'{label}: invalid URL')
    url = urlsplit(value)
    require(url.scheme in ('https', 'http') and url.hostname, f'{label}: URL must be absolute HTTP(S)')
    require(url.username is None and url.password is None, f'{label}: URL contains credentials')
    require(not url.query and not url.fragment, f'{label}: URL must not contain query or fragment')
    require(url.port is None or 0 < url.port < 65536, f'{label}: invalid port')
    if base is not None:
        require(f'{url.scheme}://{url.netloc}' == base, f'{label}: URL differs from BASE_URL')
    return url


def local_asset(site, value, label, base):
    url = absolute_url(value, label, base)
    asset = (site / unquote(url.path).lstrip('/')).resolve()
    require(asset.is_relative_to(site.resolve()) and asset.is_file(), f'{label}: missing local asset')
    return asset


def unique_json_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f'JSON-LD: duplicate key {key}')
        result[key] = value
    return result


def reject_json_constant(value):
    raise ValueError(f'JSON-LD: invalid JSON constant {value}')


def check_identity(value, label):
    # Product names are allowed; personal attribution/contact fields are not.
    if isinstance(value, dict):
        forbidden = {'author', 'creator', 'contributor', 'founder', 'email', 'telephone',
                     'address', 'contactpoint', 'sameas'}
        require(not forbidden.intersection(key.lower() for key in value), f'{label}: personal attribution/contact field')
        types = value.get('@type', [])
        require(isinstance(types, (str, list)), f'{label}: invalid JSON-LD type')
        require('Person' not in ([types] if isinstance(types, str) else types), f'{label}: Person schema')
        for child in value.values():
            check_identity(child, label)
    elif isinstance(value, list):
        for child in value:
            check_identity(child, label)
    elif isinstance(value, str):
        require(not re.search(r'[\w.+%-]+@[\w.-]+\.[A-Za-z]{2,}', value), f'{label}: email address')


def check_application(data, page, site, base, description, image):
    label = f'{page.path}: SoftwareApplication'
    require(isinstance(data, dict), f'{label}: expected JSON object')
    for key, expected in {'@context': 'https://schema.org', '@type': 'SoftwareApplication',
                          'name': 'Voxelite', 'applicationCategory': 'DesignApplication',
                          'url': base + '/', 'image': image, 'description': description}.items():
        require(data.get(key) == expected, f'{label}: incorrect {key}')
    require('open-source' in description.lower(), f'{label}: missing open-source description')
    require(data.get('isAccessibleForFree') is True, f'{label}: must be free')
    offer = data.get('offers', {})
    require(isinstance(offer, dict) and offer.get('@type') == 'Offer', f'{label}: missing Offer')
    require(type(offer.get('price')) in (int, float) and offer['price'] == 0, f'{label}: price must be zero')
    require(offer.get('priceCurrency') == 'USD', f'{label}: missing price currency')
    license_file = local_asset(site, data.get('license'), label + ' license', base)
    require('MIT License' in license_file.read_text(), f'{label}: missing open-source MIT license')


def page_route(path):
    if path.name == 'index.html':
        parent = path.parent.as_posix()
        return '/' if parent == '.' else f'/{parent}/'
    return '/' + path.as_posix()


def check_site(site, base):
    origin = absolute_url(base, 'BASE_URL')
    require(not origin.path, 'BASE_URL must be an origin without a trailing slash or path')
    pages, fragments = {}, 0
    for path in sorted(site.rglob('*.html')):
        relative = path.relative_to(site)
        page = Page(relative, path.read_text())
        if relative.parts[0] == 'content':
            require(page.heads == page.bodies == 0, f'{relative}: full page hidden among source fragments')
            fragments += 1
        else:
            require(page.heads == page.bodies == 1, f'{relative}: expected a complete HTML page')
            pages[relative] = page
    for required in ('index.html', 'app/index.html', '404.html'):
        require(Path(required) in pages, f'Missing required page: {required}; regenerate the site')

    seen_titles, seen_descriptions, public_urls = {}, {}, set()
    structured_count = 0
    for path, page in pages.items():
        title, description = page.one(page.titles, 'title'), page.tag('description')
        for text, seen, label in ((title, seen_titles, 'title'), (description, seen_descriptions, 'description')):
            normalized = ' '.join(text.split()).casefold()
            require(normalized not in seen, f'{path}: duplicate {label} with {seen.get(normalized)}')
            seen[normalized] = path
        canonical = page.one(page.canonicals, 'canonical')
        absolute_url(canonical, f'{path}: canonical', base)
        require(canonical == base + page_route(path), f'{path}: canonical does not match page route')
        image = page.tag('og:image', 'property')
        local_asset(site, image, f'{path}: og:image', base)
        for key, expected in {'og:url': canonical, 'og:title': title,
                              'og:description': description, 'og:type': 'website'}.items():
            require(page.tag(key, 'property') == expected, f'{path}: {key} mismatch')
        for key, expected in {'twitter:card': 'summary_large_image', 'twitter:title': title,
                              'twitter:description': description, 'twitter:image': image}.items():
            require(page.tag(key) == expected, f'{path}: {key} mismatch')
        require(not page.meta['name', 'twitter:site'] and not page.meta['name', 'twitter:creator'], f'{path}: social attribution')
        check_identity([title, description, *[value for values in page.meta.values() for value in values]], str(path))
        directives = set(re.split(r'[,\s]+', ','.join(page.meta['name', 'robots']).lower()))
        if path == Path('404.html'):
            require('noindex' in directives, f'{path}: expected noindex')
        else:
            require(not {'noindex', 'none'} & directives, f'{path}: public page cannot be noindex')
            public_urls.add(canonical)
        data = []
        for raw in page.json_ld:
            try:
                item = json.loads(raw, object_pairs_hook=unique_json_object, parse_constant=reject_json_constant)
            except ValueError as error:
                raise ValueError(f'{path}: invalid JSON-LD: {error}') from error
            require(isinstance(item, (dict, list)) and item, f'{path}: JSON-LD must be a nonempty object or array')
            check_identity(item, str(path))
            data.append(item)
            structured_count += 1
        if path == Path('index.html'):
            require(len(data) == 1, f'{path}: expected one SoftwareApplication JSON-LD block')
            check_application(data[0], page, site, base, description, image)

    sitemap = ET.parse(site / 'sitemap.xml').getroot()
    require(sitemap.tag == NS + 'urlset', 'sitemap.xml: invalid root or namespace')
    locations = []
    for node in sitemap:
        require(node.tag == NS + 'url', 'sitemap.xml: unexpected element')
        locs = node.findall(NS + 'loc')
        require(len(locs) == 1 and locs[0].text, 'sitemap.xml: expected one nonempty loc per URL')
        absolute_url(locs[0].text, 'sitemap.xml: loc', base)
        locations.append(locs[0].text)
    require(len(locations) == len(set(locations)), 'sitemap.xml: duplicate URLs')
    require(set(locations) == public_urls,
            f'sitemap.xml: page mismatch; missing={sorted(public_urls - set(locations))}, extra={sorted(set(locations) - public_urls)}')

    robots = RobotFileParser()
    robots.parse((site / 'robots.txt').read_text().splitlines())
    require(robots.site_maps() == [base + '/sitemap.xml'], 'robots.txt: expected absolute sitemap URL')
    for url in public_urls:
        require(robots.can_fetch('*', url), f'robots.txt: public page blocked: {url}')
    require(not robots.can_fetch('*', base + '/content/overview.html'), 'robots.txt: source fragments must be disallowed')
    return (f'SEO OK: parsed {len(pages) + fragments} HTML files; {len(pages)} pages, '
            f'{fragments} source fragments, {len(locations)} sitemap URLs, {structured_count} valid JSON-LD block(s). '
            'Unique titles/descriptions, canonical/Open Graph/Twitter tags, local images, free open-source '
            'SoftwareApplication, 404 exclusion and robots all pass.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('site', nargs='?', type=Path, default=ROOT / 'site')
    args = parser.parse_args()
    try:
        source = (ROOT / 'crates/voxelite-site/src/seo.rs').read_text()
        constants = re.findall(r'^pub const BASE_URL: &str = "([^"\n]+)";', source, re.M)
        require(len(constants) == 1, 'Expected one Rust BASE_URL constant')
        print(check_site(args.site, constants[0]))
    except (ValueError, OSError, ET.ParseError) as error:
        print(f'SEO FAIL: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
