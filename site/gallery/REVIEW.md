# Gallery reviews

Each round starts from a fresh Playwright capture, with the prior screenshots
kept intact. The assessments below are separate visual and interaction passes
by the implementing agent, not claims of an external reviewer. Viewports are
1440×900 and 390×900, dark and light, grid and open lightbox, plus full walls.

## Round 1 — 7.4 / 10

Evidence: `screenshots/gallery/round-1/` (12 page captures, actual editor remix,
and machine-readable interaction results).

The models are coherent and the neutral type/color system fits the site. The
execution is still too much of a product catalog: identical rectangles, evenly
weighted art, little exploratory rhythm. Mobile wastes precious vertical space
on an orphan Nature filter. Tiny category labels become visual grit. The hidden
line break joins two words on mobile. A technically successful render does not
excuse the windmill's disconnected rotor or the kiosk's visually heavy roof.

Repairs before the next round:
- Give the wall varied tile proportions and masonry flow; quiet the borders.
- Reduce introductory vertical space; use a single horizontally scrollable
  filter row at narrow widths and increase mobile caption legibility.
- Restore subtitle spacing and support light appearance without JavaScript.
- Add a physical windmill axle and replace the kiosk's flat roof with a stepped
  ridge. Regenerate the affected sources, all cameras, and both themes via MCP.
- Announce model titles when navigating inside the lightbox.

Keyboard arrows, filtered wraparound, Escape, focus trap/return, theme persistence,
file downloads, no-JavaScript cards and actual wasm Remix all passed in this round.

## Round 2 — 8.3 / 10

Evidence: `screenshots/gallery/round-2/`. The masonry flow adds rhythm and the
lightbox has a convincing hierarchy in both themes. The mobile headline now
breaks at the wrong word; it needs an intentional two-line lockup. Curation is
still weak: two characters occupy the opening row, while the most detailed
scene is buried at the end. The filter targets should reach 44px.

The four-view review also warranted a face-connected geometry check. That caught
small gaps not obvious in the page thumbnails: Orbit's left shoulder, one fox
eye, the observatory dome bearing, one balcony post, and two pond reeds. This
is exactly why attractive screenshots alone are insufficient provenance.

Repairs before the next round:
- Deliberately break the mobile headline between its two sentences.
- Lead the masonry columns with Autumn Workshop, Weekender, Slow Sunday and
  Orbit; preserve a varied sequence beneath them.
- Raise mobile filter targets to 44px.
- Repair the five disconnected details through the same MCP recipe, re-render
  four views and both web themes, and make geometry connectivity a build check.
- Expand the no-JavaScript check to assert the actual page background, not just
  the image URL. Both themes pass.

The existing whole-site Playwright suite passes all 20 captures/check groups,
including generated navigation at 320–1440px. Voxelite-site's two integration
tests pass. The actual editor screenshot displays Orbit and its 11,334-cell
model, confirming that Remix loaded the requested source rather than the sample.

## Round 3 — 9.1 / 10

Evidence: `screenshots/gallery/round-3/`. Reviewed all eight required viewport/
theme/state combinations and the full walls. The first desktop row now has four
distinct silhouettes and balanced color weight. The phone headline is deliberate,
all six categories fit at 390px, and the wall keeps its rhythm without a giant
hero. Black is actually black. Light renders sit cleanly on white. The modal
prioritizes the sculpture, then title, palette and actions, with both actions
visible in the required phone viewport.

The newly rendered four-view sheets confirm the model repairs. Every original
sculpture passes the independent face-connectivity check. The workshop retains
its original art. The second camera is restrained; reduced motion removes it.
Keyboard and no-JavaScript checks still pass. No remaining visual blocker.

Final polish: keep the close control sticky in short lightboxes; make static
card labels describe their download action and enhance them with dialog semantics
only when JavaScript runs. Add short-viewport and missing/invalid/oversized Remix
checks. The remaining taste tradeoff is deliberately quiet card framing rather
than decorative backgrounds; the art is the color system.

## Final verification

Final evidence: `screenshots/gallery/final/`. All 19 models have editable sources,
manifest counts/dimensions/palettes and verified SHA-256 hashes. Both themes have
640px main/alternate tiles and 1024px large images in WebP and AVIF (228 images,
about 4.1 MB combined). No external image/font/CDN service is requested.

Verification commands:

```sh
CARGO_TARGET_DIR=/workspace/voxelite-gallery/target-gallery cargo test --locked -p voxelite-site
CARGO_TARGET_DIR=/workspace/voxelite-gallery/target-gallery cargo fmt --all -- --check
python3 scripts/check-site.py
node scripts/capture-gallery.mjs screenshots/gallery/final
VOXELITE_SITE_URL=http://127.0.0.1:8191 node scripts/capture-site.mjs screenshots/gallery/site-regression
```

The browser bundle is generated locally and ignored by Git, as in the existing
site workflow. Source changes to its loader are committed and reproducible with
`CARGO_TARGET_DIR=target-gallery scripts/build-web.sh`. No push, merge or deploy.
