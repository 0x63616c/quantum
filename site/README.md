# Quantum website

A static marketing site and Rust-generated documentation. No cookies,
remote fonts, or runtime framework. Every page loads Vercel Web Analytics and
Speed Insights from same-origin `/_vercel/` paths: cookie-free, anonymous page-view
and performance counts, nothing in the editor or your files. Dark and light themes share the app's Geist
fonts and neutral tokens.

From the repository root:

```sh
scripts/build-site.sh
python3 -m http.server 8090 --directory site --bind 127.0.0.1
```

Open `http://127.0.0.1:8090`. Generated site output is ignored by Git.
Nothing in these scripts deploys or pushes. `site/vercel.json` is published at
the root of `site-dist` and adds immutable font cache headers and permanent
redirects from retired editor URLs to the download page.

Edit page markup in `crates/quantum-site/templates/`, shared
styles in `assets/site.css`, documentation bodies in `content/`, and portable
skills in the repository's `skills/` directory. Edit product metadata in [`brand.toml`](../brand.toml); the version comes from
workspace `Cargo.toml`. The small `quantum-brand` loader is shared with xtask.
Templates and authored docs use `@@product@@`, `@@tagline@@`, `@@repo_url@@`,
`@@site_url@@`, `@@download_url@@`, `@@app_bundle@@`, `@@release_asset@@` and
`@@version@@` tokens; unknown tokens fail the build. Installation JSON and
deeplinks use typed values before encoding.

The changelog page reads the release task’s `CHANGELOG.md`. Blog posts live in
`content/blog/` as Markdown, starting with one `# Title` line. Lowercase dated
filenames become stable URL slugs; the index lists them newest first. Both
sections include Markdown copies and appear in the sitemap.
The Quantize header mark and SVG/PNG favicons come from `cargo xtask icons`;
edit the [SVG masters](../packaging/macos/ARTWORK.md) and regenerate rather than
changing those derivatives.

The generator uses `https://quantum.worldwidewebb.co` for canonical URLs, social
previews, structured data and crawl files. `sitemap.xml` lists the homepage,
download page, docs, changelog and blog; the 404 page is marked `noindex` and has no
canonical URL. Run the build after content/skill changes. The
`quantum-site` Rust crate wraps documentation and generates all 38 MCP references
from the committed [tool catalog](../spec/mcp/v1/tools.json), without linking the
MCP server. The MCP crate checks that catalog against `QuantumServer::tools()`.
After changing tool metadata or schemas, refresh it with:

```sh
UPDATE_TOOLS_JSON=1 cargo test -p quantum-mcp --locked --test tools_catalog
```

Commit the refreshed catalog; do not edit generated `docs/` or schema copies.

Commit the sources, not the built site. CI builds the site on every PR and push
to `main`; trusted `main` builds publish to `site-dist`. Ignored output includes
`index.html`, `404.html`, `download/`, `changelog/`, `blog/`, `docs/`, `docs.md`, `llms*.txt`,
`sitemap.xml`, `robots.txt`, `assets/mcp-tools.json`, `assets/integrations/`,
`assets/skills/` and `assets/quantum-skills.zip`. Keep `content/`, this
README, `vercel.json`, `appcast.xml` and the remaining assets
(styles, scripts, fonts, images, badges, licenses and sample project) tracked:
the site build does not recreate them. The appcast is also a generator input.

The MCP and skills pages share `templates/install.html`. All MCP deeplinks,
commands and downloadable examples come from one server configuration in
`src/install.rs`: the Mac bundle's `Contents/MacOS/quantum-mcp` executable.
Cursor and VS Code use `${workspaceFolder}`; Claude/Codex CLI commands use
`"$PWD"` and must run from the project directory. The same build refreshes
the tracked root `integrations/` (including its README from
`templates/integrations.md`), their downloadable copies and the skills ZIP.
Edit the Rust definition/templates, not the generated examples. Per-skill install
commands use the metadata from
`skills/*/SKILL.md`. Tabs enhance static panels; every host's instructions and
deeplinks remain available without JavaScript. Copy buttons use `assets/site.js`.

Site fonts are variable WOFF2 subsets, preloaded by the shared metadata generator
in every site template, including the 404. `font-display: optional` prevents late
font swaps, with metric-matched local fallbacks for cold/slow loads.
See [font build notes](assets/fonts/README.md)
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

`site/appcast.xml` starts as a valid empty feed with no installable items. The
deployment fetches the highest stable signed release feed before building. The
Rust generator copies it unchanged to another output directory and preserves it
in place. Do not edit a generated feed: its signature covers the exact bytes.
See [automatic updates](../docs/releasing-macos.md#automatic-updates-sparkle).

Generator tests build into temporary directories and do not require a site build:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo test --locked -p quantum-site
```

After `scripts/build-site.sh`, verify the complete local site:

```sh
python3 scripts/check-site.py
node scripts/capture-site.mjs screenshots/site/final
node scripts/check-site-install.mjs
```

The site checks verify both Markdown aliases, discovery links, local links and
fragments, exact code/schema parity, and inclusion in both LLM indexes. The Rust
tests validate fresh generator output, Markdown aliases, LLM indexes, navigation
and the versioned MCP catalog. The MCP crate tests catalog freshness and runs the
published tutorial against the server. Browser checks cover complete Markdown
copying, download/clipboard failures, and viewing raw Markdown without JavaScript.

Capture output is ignored by Git. Historical captures and verification records
are in the [private archive](https://github.com/0x63616c/quantum-archive).

The browser script requires the local server and the repository's Playwright
installation (`npm ci`, `npx playwright install chromium` when absent). It captures
1440×900 and 390×900 homepages (full page and first viewport) in both themes,
plus docs/news views, both mobile comparison states, and both expanded phone menus
(32 captures total). It checks every page type's widths
from 320 to 1440px (including header breakpoint edges and 701/768px), the bounds
of each visible header control, the 12px docs code floor, scrolling MCP tables,
skip-link hover/press and text-control press contrast, keyboard comparison controls,
focus, reduced motion, theme persistence, clipboard copying, no-JavaScript navigation/comparison and absence
of third-party homepage requests. It also verifies that section indexes match
article headings, phone menus close when focus leaves without moving it and
still restore focus on Escape, and the first build example fits the 390px viewport
without horizontal scrolling. Hidden comparison images are explicitly loaded
for capture so lazy loading cannot leave incomplete screenshots.

`python3 scripts/capture-site-proof.py` records a new real MCP edit and produces
the matched-camera detail comparison, with SHA-256 provenance. Build
`quantum-mcp` first. It reads the sample and edits an in-memory document; the
source model remains intact. The original native-session hero and newly recorded
headless comparison are different sessions and have distinct revision labels.

`python3 scripts/scan-pii.py` scans source/assets and reachable Git history,
printing the report with redacted findings to stdout; it takes a few minutes
and does not rewrite history. Host application loading and macOS
packaging remain untested here; guides disclose those boundaries.

## Deployment

`.github/workflows/deploy-site.yml` builds the complete static site on
GitHub-hosted Ubuntu runners for pushes to `main`, pull requests, manual runs,
and successful owner-triggered **Release macOS** workflow completions. It fetches
the highest numeric stable release containing an appcast, validates its asset
URLs and sizes, and runs the local site checks. Confirmed absence of a release
feed uses the empty checked-in RSS; API/download/validation failures stop the
deployment. Pull requests only build and verify. Production publishes from
`main` only when triggered by `0x63616c` in `0x63616c/quantum`. A manual run
on `main` retries a failed post-release deployment without replacing any assets.

The workflow publishes the built files at the root of `site-dist`, excluding this
README and `content/` sources. It preserves deployment history and skips unchanged
output. The workflow then deploys the same built artifact to Vercel with the
configured `VERCEL_TOKEN` secret. The Vercel project has no Git integration;
pushing `site-dist` alone does not deploy it. Without the token, the workflow
publishes the branch and reports that Vercel deployment was skipped.

After building, run `QUANTUM_BRAND_SITE=site cargo xtask check brand` to scan
published output with the same stale-name checker as tracked source. Exact
`generated_paths` in the existing compatibility groups allow frozen wire names
and historical release/rename copy; those paths must match when output is checked.

Deploy site checks generated naming before fetching the published signed appcast.
The feed then passes the dedicated appcast validator; historical product names,
signatures and immutable release URLs are preserved byte for byte.
