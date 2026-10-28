#!/usr/bin/env python3
"""Subset the app's variable fonts for the site. Requires fonttools and brotli."""
from html.parser import HTMLParser
from pathlib import Path
from fontTools import subset
from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parents[1]


class Text(HTMLParser):
    def __init__(self):
        super().__init__()
        self.characters = set()
        self.ignored = False

    def handle_starttag(self, tag, attrs):
        if tag in ('script', 'style'):
            self.ignored = True
        for key, value in attrs:
            if key in ('alt', 'title', 'aria-label') and value:
                self.characters.update(map(ord, value))

    def handle_endtag(self, tag):
        if tag in ('script', 'style'):
            self.ignored = False

    def handle_data(self, data):
        if not self.ignored:
            self.characters.update(map(ord, data))


text = Text()
for path in sorted((ROOT / 'site').rglob('*.html')):
    if not path.is_relative_to(ROOT / 'site/app'):
        text.feed(path.read_text())
# Basic Latin and Latin-1, plus every punctuation/symbol used in the HTML.
characters = set(range(0x20, 0x100)) | text.characters
for name in ('Geist', 'GeistMono'):
    source = ROOT / f'crates/quantum-app/assets/fonts/{name}.ttf'
    font = TTFont(source, recalcTimestamp=False)
    axes = [(a.axisTag, a.minValue, a.defaultValue, a.maxValue) for a in font['fvar'].axes]
    options = subset.Options()
    options.flavor = 'woff2'
    options.recalc_timestamp = False
    options.layout_features = ['*']
    subsetter = subset.Subsetter(options=options)
    subsetter.populate(unicodes=characters)
    subsetter.subset(font)
    font.flavor = 'woff2'
    destination = ROOT / f'site/assets/fonts/{name}.woff2'
    font.save(destination)
    result = TTFont(destination)
    assert [(a.axisTag, a.minValue, a.defaultValue, a.maxValue) for a in result['fvar'].axes] == axes
    assert characters & TTFont(source).getBestCmap().keys() <= result.getBestCmap().keys()
    print(f'{name}: {source.stat().st_size:,} → {destination.stat().st_size:,} bytes; axes {axes}')
