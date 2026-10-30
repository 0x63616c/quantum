# Voxelite website

A local-first static marketing site and Rust-generated documentation. No tracking,
remote fonts, or runtime framework. Dark and light themes share the app's Geist
fonts and neutral tokens. The bundled editor is the actual Rust GPUI wasm app.

From the repository root:

```sh
scripts/build-site.sh
python3 -m http.server 8090 --directory site --bind 127.0.0.1
```

Open `http://127.0.0.1:8090`. The browser editor is at `/app/`. Site-only iterations
can reuse the verified `web/dist/` with `scripts/build-site.sh --skip-web`.
`site/app/` and `web/dist/` are generated and ignored by Git. Nothing in these
scripts deploys, pushes, or configures hosting.

Edit landing-page markup in `index.html`, shared styles in `assets/site.css`,
documentation bodies in `content/`, and portable skills in the repository's
`skills/` directory. Run the build after content/skill changes. The
`voxelite-site` Rust crate wraps documentation and generates all 38 MCP references
from `VoxeliteServer::tools()`; do not edit generated `docs/` or schema copies.

## SEO and the public base URL

The single deployment origin is `BASE_URL` in
`crates/voxelite-site/src/seo.rs`, initially `https://voxelite.vercel.app`.
To change domains, edit that constant to an absolute HTTPS origin without a
trailing slash, path, query, or fragment, then regenerate from the repository root:

```sh
cargo run --offline --locked -p voxelite-site -- site
python3 scripts/check-site-seo.py
```

This refreshes canonical URLs, Open Graph and Twitter cards, homepage
SoftwareApplication JSON-LD, `robots.txt`, and `sitemap.xml` together. Edit the
homepage description in `seo.rs` and docs descriptions beside the page definitions
in `crates/voxelite-site/src/main.rs`. The homepage's `Generated SEO` block is
generated; its remaining markup stays authored in `site/index.html`. The editor
shell is generated into `site/app/index.html` from `web/index.html`, preserving
its styles, scripts, and body. JS and Wasm remain copied from `web/dist/`.

The sitemap lists `/`, `/app/`, and all five documentation routes. The 404 page
has complete metadata and `noindex, follow`, and is excluded from the sitemap.
`content/*.html` are article source fragments, not standalone pages; robots
excludes that directory. The social image reuses `assets/session-dark.png`.
Metadata contains no personal attribution or social account handles.

The SEO checker uses only Python's standard library and makes no network calls.
It parses every HTML file, distinguishes the source fragments, and validates all
complete pages including the bundled editor and 404. Regeneration alone needs
cached Rust dependencies but does not rebuild the editor; for an offline full
site refresh using an existing bundle, run
`CARGO_NET_OFFLINE=true scripts/build-site.sh --skip-web`.

Verification:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo test --locked -p voxelite-site
python3 scripts/check-site.py
python3 scripts/check-site-seo.py
node scripts/capture-site.mjs screenshots/site/final
```

The browser script requires the local server and the repository's Playwright
installation (`npm ci`, `npx playwright install chromium` when absent). It captures
1440×900 and 390×900 homepages (full page and first viewport) in both themes,
plus docs views, both mobile comparison states, and both expanded phone menus
(20 captures total). It checks homepage/docs widths
from 320 to 1440px, keyboard comparison controls, focus, reduced motion, theme
persistence, clipboard copying, no-JavaScript navigation/comparison and absence
of third-party homepage requests. It also verifies that section indexes match
article headings, phone menus dismiss and restore focus correctly, and the first
build example fits the 390px viewport without horizontal scrolling. Hidden comparison images are explicitly loaded
for capture so lazy loading cannot leave incomplete screenshots.

`python3 scripts/capture-site-proof.py` records a new real MCP edit and produces
the matched-camera detail comparison, with SHA-256 provenance. Build
`voxelite-mcp` first. It reads the sample and edits an in-memory document; the
source model remains intact. The original native-session hero and newly recorded
headless comparison are different sessions and have distinct revision labels.

Review findings and scores are in `PROGRESS.md` and `docs/reviews/site-review-*.md`
at repository root. `python3 scripts/scan-pii.py` scans source/assets and reachable
Git history, writing redacted locations to `docs/reviews/pii-scan.md`; it takes a
few minutes and does not rewrite history. Host application loading and macOS
packaging remain untested here; guides disclose those boundaries.
