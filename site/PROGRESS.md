# Site, docs and skills stage

Started 2026-10-25 from `9eb0d95`. Local work only; never deploy or push.
`design/` is read-only and excluded from commits.

## Deliverables

- [x] Marketing site, explicit dark/light themes, real app screenshots.
- [x] Rust-generated docs and live MCP tool catalog; format/build guides.
- [x] Four portable agent skills and host installation notes.
- [x] Playwright desktop 1440×900 and mobile 390px full-page captures, both themes.
- [x] Three independent screenshot critique/fix/recapture rounds.
- [x] Repository and reachable Git history personal-info scan.

## Review scores

Scores refer to the images reviewed before each round's fixes. Final captures
are separate evidence, not retroactively higher review scores.

| Round | Desktop /10 | Mobile /10 | Evidence | Fixes / recapture |
| --- | --- | --- | --- | --- |
| 1 | 6.5 | 5.5 | `docs/reviews/site-review-1.md` | Device disclosure, enlarged crop, native evidence, palette, links; `screenshots/site/round-2/` |
| 2 | 7.2 | 6.5 | `docs/reviews/site-review-2.md` | Native hero, clean canvas crop, proof order, device-specific labels; `screenshots/site/round-3/` |

| 3 | 7.7 | 7.1 | `docs/reviews/site-review-3.md` | Actual before/after MCP recording, immediate mobile session proof, shorter narrative, setup CTA, caption wrapping; `screenshots/site/final/` |

## Verified boundaries

The existing web bundle uses the shared Rust GPUI editor. Browser MCP pairing
is not implemented. Native attached MCP and standalone headless MCP are
available. macOS release packages and signed downloads are not available here.
No unverified download, hosting, benchmark or collaboration claims belong on
the site. Existing Rust toolchain: 1.99.1; default native Linux build works.

## Resumed verification

- Linux `cargo build --locked --workspace` and headless workspace tests pass.
- Actual tutorial JSON executes through create → batch → four-view render → save → GLB export; saved project reopens canonically.
- Valid one-voxel download added and format-reader tested.
- `scripts/check-site.py` checks local links/fragments, all 38 schemas, skills download parity and app bundle parity.
- Skills ZIP includes integration examples and preserves portable directory structure.

## Completion verification — 2026-10-25

All six site/docs/skills stage deliverables are complete. This does not claim
completion of the broader product/release plan. No deployment, push or remote
creation was performed; no file under `design/` was modified.

- `cargo build --locked --workspace`: passes, including native GPUI on Linux.
- `cargo test --locked --workspace`: **112 passed**; headless workspace tests also pass.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: passes.
- `cargo fmt --all -- --check`: passes.
- Full `scripts/build-site.sh`: passes; Rust wasm build and bundled `/app/` verified.
- Bundled `/app/?backend=webgl`: real Autumn Workshop GPUI canvas starts under headless software WebGL2, without page errors.
- `python3 scripts/check-site.py`: 7 pages, 221 local links/assets, all 38 live tool schemas, self-contained skill ZIP and editor bundle parity pass.
- Four skills pass the skill frontmatter validator. Host installation locations were checked against the linked official documentation; actual host launches remain untested.
- Playwright Chromium 156.0.8078.4: final 1440×900 and 390×900 full-page captures in both themes, plus four docs viewport captures; see `screenshots/site/final/verification.json`.
- All five docs routes also fit 320px; keyboard skip link, reduced motion, copy-to-clipboard, theme switching/persistence, no-JavaScript navigation, and no third-party homepage requests pass.
- The recorded before/after demo uses the actual MCP server, adds exactly 3 visible voxels, and records identical camera/lighting plus PNG/source hashes in `screenshots/site/proof/provenance.json`. Original native screenshot r8 and the new demonstration r1 are separate, labeled sessions.
- Personal-info scan: `docs/reviews/pii-scan.md`. Scanned working tree/generated bundles and 122 commits / 2,863 unique historical blobs at the scan snapshot. Identity headers, historical build paths, font attribution and one spec display-name example remain documented; no token-pattern hits. No trivially safe authored path leak required editing. Final captures were visually inspected as well.

Reproduce local build, captures and checks using `site/README.md`. Scores above
are the independent pre-fix reviews; final captures are not assigned invented
post-fix scores.
