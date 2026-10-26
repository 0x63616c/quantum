# Voxelite website

A static marketing site and Rust-generated documentation. No cookies,
remote fonts, or runtime framework. Every page loads Vercel Web Analytics and
Speed Insights from same-origin `/_vercel/` paths: cookie-free, anonymous page-view
and performance counts, nothing in the editor or your files. Dark and light themes share the app's Geist
fonts and neutral tokens. The bundled editor is the actual Rust GPUI wasm app.

From the repository root:

```sh
scripts/build-site.sh
python3 -m http.server 8090 --directory site --bind 127.0.0.1
```

Open `http://127.0.0.1:8090`. The browser editor is at `/app/`. Site-only iterations
can reuse the verified `web/dist/` with `scripts/build-site.sh --skip-web`.
`site/app/` and `web/dist/` are generated and ignored by Git. If the bundle is
missing, run `scripts/build-web.sh` first. Nothing in these scripts deploys or
pushes. `site/vercel.json` is published at the root of `site-dist` and adds
immutable font cache headers.

Edit page markup in `crates/voxelite-site/templates/`, shared
styles in `assets/site.css`, documentation bodies in `content/`, and portable
skills in the repository's `skills/` directory. Edit metadata in
`crates/voxelite-site/src/` and the browser editor's HTML in `web/index.html`.
The generator uses `https://voxelite.worldwidewebb.co` for canonical URLs, social
previews, structured data and crawl files. `sitemap.xml` lists the homepage,
download page, docs and editor; the 404 page is marked `noindex` and has no
canonical URL. Run the build after content/skill changes. The
`voxelite-site` Rust crate wraps documentation and generates all 38 MCP references
from `VoxeliteServer::tools()`; do not edit generated `docs/` or schema copies.

The MCP and skills pages share `templates/install.html`. All MCP deeplinks,
commands and downloadable examples come from one server configuration in
`src/install.rs`: the Mac bundle's `Contents/MacOS/voxelite-mcp` executable.
Cursor and VS Code use `${workspaceFolder}`; Claude/Codex CLI commands use
`"$PWD"` and must run from the project directory. The same build refreshes
`integrations/` (including its README from `templates/integrations.md`), their
downloadable copies and the skills ZIP. Edit the Rust definition/templates,
not the generated examples. Per-skill install commands use the metadata from
`skills/*/SKILL.md`. Tabs enhance static panels; every host's instructions and
deeplinks remain available without JavaScript. Copy buttons use `assets/site.js`.

Site fonts are variable WOFF2 subsets, preloaded by the shared metadata generator
in every site template, including the 404. `font-display: optional` prevents late
font swaps, with metric-matched local fallbacks for cold/slow loads. The browser
editor's canvas still embeds the app's own TTF assets; its HTML loading screen uses
system fonts and needs no WOFF2 preload. See [font build notes](assets/fonts/README.md)
to regenerate subsets after adding new characters.

The same build converts those article bodies (including every MCP schema) to
Markdown. `/docs/` publishes `/docs.md` and `/docs/index.md`; each
`/docs/topic/` publishes identical `/docs/topic.md` and `/docs/topic/index.md`
files. Links use absolute URLs so they also work in copied Markdown and the
combined document. Links to specific sections retain the HTML URL and its
authored anchor; other documentation links use the Markdown URL.

`llms.txt` follows the [llms.txt format](https://llmstxt.org/) and links to every
Markdown doc with its description. `llms-full.txt` concatenates all docs in
navigation order with source links. These files are generated, never maintained
separately. Doc pages advertise Markdown and the LLM index with alternate and
describedby links. “Copy page as Markdown” copies the raw file, with “View .md”
available without JavaScript or clipboard access. The sitemap includes the LLM
indexes and one canonical Markdown URL per page; robots.txt allows crawling and
advertises the indexes in comments.

An optional `site/appcast.xml` is passed through unchanged by the Rust generator
when building to another output directory, and preserved by the normal in-place
site build. If it is absent, no feed is created. Supply a real release feed when
available; the build does not invent update metadata.

Verification:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo test --locked -p voxelite-site
python3 scripts/check-site.py
node scripts/capture-site.mjs screenshots/site/final
node scripts/check-site-install.mjs
```

The site checks verify both Markdown aliases, discovery links, local links and
fragments, exact code/schema parity, and inclusion in both LLM indexes. The Rust
tests also compare a fresh generator run against the published files to catch
stale prose or metadata. Browser checks cover complete Markdown copying,
download/clipboard failures, and viewing raw Markdown without JavaScript.

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

## Deployment

`.github/workflows/deploy-site.yml` builds the complete site and wasm editor on
GitHub-hosted Ubuntu runners for pushes to `main`, pull requests, and manual runs.
It includes the latest release's `appcast.xml` when available and runs the local
site checks. Pull requests only build and verify. Pushes and manual runs on `main`
publish only when triggered by `0x63616c` in `0x63616c/voxelite`.

The workflow publishes the built files at the root of `site-dist`, excluding this
README, `PROGRESS.md`, and `content/` sources. It preserves deployment history and
skips unchanged output. Set Vercel's production branch to `site-dist`, framework
preset to **Other**, Build Command override to empty, and output directory to `.`
([static build settings](https://vercel.com/docs/builds/configure-a-build#skip-build-step)).
Vercel deploys that branch; GitHub needs no Vercel secrets.
