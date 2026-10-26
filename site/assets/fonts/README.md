# Site fonts

Geist and Geist Mono retain the variable `wght` axis, 100–900. The site uses
WOFF2 subsets; the native/wasm app keeps its separate TTF sources in
`crates/voxelite-app/assets/fonts/`. Both OFL license files remain here.

After regenerating the HTML, rebuild subsets with fontTools and Brotli:

```sh
python3 -m venv /tmp/voxelite-fonts
/tmp/voxelite-fonts/bin/pip install fonttools brotli
/tmp/voxelite-fonts/bin/python scripts/build-site-fonts.py
```

The script keeps basic Latin/Latin-1 and the punctuation and symbols used in site
HTML, retaining all layout features and variable weights. It verifies glyph
coverage against the source fonts and preserves timestamps for reproducible bytes.
Characters absent from the original fonts continue to use the browser fallback.

Both fonts use `font-display: optional` and same-origin CORS preloads. They do not
swap in after the initial brief font-loading period. The local fallbacks in
`site.css` also match advance widths and vertical metrics at weight 400:

| Font | Units/em | Ascent | Descent | Line gap | A–Z/a–z/0–9 advance sum (em) |
| --- | --- | --- | --- | --- | --- |
| Geist | 1000 | 1005 | 295 | 0 | 36.785 |
| Geist Mono | 1000 | 1005 | 295 | 0 | 37.2 |

Reference advance sums are 35.90234375 em for Arial-compatible Liberation Sans
and 37.2060546875 em for Courier New-compatible Liberation Mono. These come from
the fonts' `hmtx` tables. `size-adjust` is target/reference width × 100%; each
vertical override is target metric / units-per-em / size-adjust × 100%.
Average width matching does not make every glyph identical; optional loading
prevents the late swap that would otherwise reflow text.

Vercel caches `/assets/fonts/*` for one year with `immutable`. If font bytes change
after deployment, version the filenames and update CSS/preloads together so
existing clients do not reuse the old immutable URL.
