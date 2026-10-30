# Site, docs and skills stage

Started 2026-10-25 from `9eb0d95`. Local work only; never deploy or push.
`design/` is read-only and excluded from commits.

## Deliverables

- [x] Marketing site, explicit dark/light themes, real app screenshots.
- [x] Rust-generated docs and live MCP tool catalog; format/build guides.
- [x] Four portable agent skills and host installation notes.
- [x] Playwright desktop 1440×900 and mobile 390px full-page captures, both themes.
- [x] Five screenshot critique/fix/recapture rounds, with independent pre/post scoring in rounds 4–5.
- [x] Repository and reachable Git history personal-info scan.

## Review scores

Rounds 1–3 retain their original pre-fix scores. Rounds 4–5 independently score
both the existing final captures and the newly recaptured result; earlier scores
are not changed retroactively. The final screenshot directory contains round 5;
round 4 is preserved at commit `6695364`.

| Round | Desktop /10 (pre → post) | Mobile /10 (pre → post) | Evidence | Fixes / recapture |
| --- | --- | --- | --- | --- |
| 1 | 6.5 | 5.5 | `docs/reviews/site-review-1.md` | Device disclosure, enlarged crop, native evidence, palette, links; `screenshots/site/round-2/` |
| 2 | 7.2 | 6.5 | `docs/reviews/site-review-2.md` | Native hero, clean canvas crop, proof order, device-specific labels; `screenshots/site/round-3/` |
| 3 | 7.7 | 7.1 | `docs/reviews/site-review-3.md` | Actual before/after MCP recording, immediate mobile session proof, shorter narrative, setup CTA, caption wrapping; `screenshots/site/final/` |
| 4 | 7.8 → **8.6** | 7.0 → **8.2** | `docs/reviews/site-review-4.md` | Complete workspace in first desktop viewport, mobile comparison controls, clean session crop, setup shortcuts and code bars; 16 inspected captures in `screenshots/site/final/` |
| 5 | 8.3 → **8.8** | 7.8 → **8.7** | `docs/reviews/site-review-5.md` | Shorter mobile narrative, artwork + recorded session in the first viewport, compact phone docs menus, generated section index, readable build command; 20 inspected captures in `screenshots/site/final/` |

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

## Round 4 follow-up — 2026-10-25

- Rebuilt web bundles with stable source-path remapping and no debug information;
  no home/workspace/root path bytes remain. Both copies match. The original
  email-shaped Wasm match is confirmed random binary data, not an email address.
  See the remediation commands and hash in `docs/reviews/pii-scan.md`.
- Replaced the spec's personal display-name example with “the project owner.”
- All eight original final images were visually reviewed before changes, then
  all new homepage/docs views and mobile comparison states were reviewed after
  recapture. The site remains below the 9/10 target; the review names the gaps.
- Linux workspace build, 112 tests, Clippy with warnings denied, formatting,
  static-site checks and software-WebGL2 editor startup pass.
- The capture suite now checks five widths, native radio keyboard/pointer use,
  no-JavaScript comparison/setup navigation, and captures the first viewport as
  well as full-page composition. `verification.json` records all 16 captures.

Reproduce local build, captures and checks using `site/README.md`. Round 4's
post-fix scores are judgments of the new images, not a claim that all broader
product, gallery or documentation release requirements are complete.


## Round 5 follow-up — 2026-10-25

- Independent baseline critique and ranked fixes are in
  `docs/reviews/site-review-5.md`. Homepage scores: desktop **8.3 → 8.8**,
  mobile **7.8 → 8.7**; still below 9, with remaining evidence/design gaps named.
- Mobile homepage is about 20% shorter. The artwork and complete recorded
  session fit the first 390×900 viewport. One closing invitation replaces two.
- Rust generates real article section indexes. Desktop docs have a narrower
  reading column; phone/tablet navigation uses accessible native disclosures.
  The first complete build command fits the phone viewport.
- All **20** final screenshots were recaptured and visually inspected, including
  both mobile docs menu states. The suite checks five widths, both themes,
  keyboard/focus behavior, clipboard, no-JavaScript navigation, generated anchor
  parity, runtime errors and the two first-viewport layout requirements.
- Linux workspace build, 112 tests, Clippy with warnings denied, formatting,
  static-site checks and bundled software-WebGL2 startup pass. Site tests and
  Clippy passed again after the last generator/content changes.
- `design/` stayed unchanged. All work remained local.
