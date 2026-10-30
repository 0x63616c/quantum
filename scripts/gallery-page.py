#!/usr/bin/env python3
"""Generate progressive-enhancement gallery markup from the saved manifest."""
import html,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'site/gallery'

def picture(model,size='tile',eager=False):
    slug=model['id'];sources=[]
    for theme in ['light','dark']:
        media='(prefers-color-scheme: light)' if theme=='light' else 'all'
        for fmt in reversed(model['formats']):
            sources.append(f'<source type="image/{fmt}" media="{media}" data-theme-source="{theme}" srcset="/gallery/images/{slug}-{theme}-{size}.{fmt}">')
    return '<picture>'+''.join(sources)+f'<img src="/gallery/images/{slug}-dark-{size}.webp" alt="{html.escape(model["title"])} — an original voxel {html.escape(model["category"].lower())} model" width="640" height="640" loading="{"eager" if eager else "lazy"}" decoding="async"></picture>'

def main():
    entries=json.loads((OUT/'manifest.json').read_text())
    cards=[]
    for i,m in enumerate(entries):
        cards.append(f'''<a class="gallery-card" data-id="{m['id']}" data-category="{m['category']}" href="{m['source']}" download aria-label="Download {html.escape(m['title'])}.voxl"><div class="tile-art">{picture(m,eager=i<4)}</div><div class="tile-caption"><h2>{html.escape(m['title'])}</h2><span>{m['category']}</span><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 17 17 7M7 7h10v10"/></svg></div></a>''')
    filters=''.join(f'<button type="button" class="filter-chip" data-filter="{c}" aria-pressed="{str(c=="All").lower()}">{c}</button>' for c in ['All','Characters','Buildings','Vehicles','Objects','Nature'])
    template=(OUT/'template.html.in').read_text().replace('<!-- CARDS -->','\n'.join(cards)).replace('<!-- FILTERS -->',filters).replace('<!-- DATA -->',json.dumps(entries,separators=(',',':')).replace('<','\\u003c'))
    (OUT/'index.html').write_text(template)
if __name__=='__main__':main()
