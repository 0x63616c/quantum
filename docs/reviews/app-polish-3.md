# App polish round 3

> Layout update (2026-10-26): the toolbar Save button described in this historical
> review has been removed. Save remains in File and Cmd/Ctrl+S. Settings is now
> an always-visible bottom-left cog beside the optional keyboard guide.

Independent review of the five supplied images, 2026-10-25. Baseline:
`63a7cdd` on `build-v1`. Reviewed against `PLAN.md`, `design/ART-CHECKLIST.md`
and Concept A. This review judges the existing implementation, not prior
review scores. Target: 9/10 for a polished native creative tool. Screenshots
establish visual quality, not macOS HIG or accessibility certification.

## Before scores

| Supplied image | Score /10 | Assessment |
| --- | ---: | --- |
| 1 · app-dark, 1440×900 | 8.0 | Excellent original art and restrained black chrome; low framing, redundant empty activity, and a silently truncated palette reduce finish. |
| 2 · app-light, 1440×900 | 8.2 | Clear white surfaces, readable type and crisp icons. The same layout issues remain, and inconsistent sidebar insets are more apparent. |
| 3 · app-dark-small, 1024×640 | 7.4 | Core actions fit, but the artwork crowds the camera controls and only 16 of 36 colors are visible without a scroll cue. Gesture glyphs are difficult to interpret. |
| 4 · concept-a-dark | 8.6 | Strong hierarchy, comfortable framing and meaningful agent activity. Reference-only: the supplied image uses charcoal panel surfaces rather than the required pure black, and does not prove small-window behavior. |
| 5 · concept-a-light | 8.8 | The strongest balance of artwork, sidebar rhythm and contextual activity. Reference-only: dense secondary controls and unproven compact behavior keep it below a release-quality 9. |

## Ranked findings — fix only these five

1. **Model framing is too low.** The desktop image leaves much more air above
   the tree than beneath the terrain; at minimum size the terrain approaches
   the camera toolbar. Center the projected occupied scene and retain framing
   margin across aspect ratios, without changing the art or renderer crate.
2. **The palette looks truncated.** “36 colors” accompanies 32 desktop or 16
   compact visible swatches, with no indication of additional rows. Use exact
   whole-row sizing and an explicit overflow affordance; keep the active color
   footer visible and all swatches reachable.
3. **The idle session overstates empty activity.** Connect agent, Recent changes,
   disabled undo/redo, No edits yet and Your changes will appear here repeat an
   empty state. Replace this with a compact, purposeful connection invitation;
   preserve genuine human history and live agent activity.
4. **The gesture hint is a cryptic string of symbols.** Preserve the required
   Lucide gesture glyphs, but separate the groups, provide individual tooltips,
   and add short action labels when space permits. Align it with the camera bar.
5. **Sidebar spacing lacks a shared rhythm.** The 12/16/20/24px insets and mixed
   section spacing produce small but visible alignment jumps. Align section
   headings/content and selected-row insets using the existing gutter token.

The persistent left Lucide tool rail, local Share link/Copy and View/Comment/Remix
controls, Save icon + label, gesture glyphs, Geist fonts, pure `#000` dark and
white light themes are requirements, not candidates for removal. No changes
under `design/`; implementation changes are confined to `crates/voxelite-app`.

## After review

Implementation commits: `be5d5ef` (framing) and `a2c8898` (the remaining four
findings). Actual GPUI windows were captured on Linux/Xvfb with Mesa software
Vulkan at 1440×900 and 1024×640, using the build/launch/resize/import portion of
`scripts/capture-native.sh`. No mockups, image generation or screenshot retouching.
Both themes and both window sizes were inspected, including the supplementary
interaction captures below. The reference images remain unchanged.

| Supplied image / updated capture | Before | After /10 | Assessment |
| --- | ---: | ---: | --- |
| 1 · [app-dark](../../screenshots/app-dark.png) | 8.0 | 8.8 | Balanced silhouette framing, purposeful connection invitation, all 36 colors visible, aligned sidebar content and readable gesture groups. |
| 2 · [app-light](../../screenshots/app-light.png) | 8.2 | 8.9 | Matching improvements; especially clear section alignment and selected swatches on white. |
| 3 · [app-dark-small](../../screenshots/app-dark-small.png) | 7.4 | 8.5 | Artwork clears the camera bar; complete swatch rows, visible color value and explicit scrolling controls fit the minimum window. |
| 4 · concept-a-dark | 8.6 | 8.6 | Reference only; no implementation score uplift. |
| 5 · concept-a-light | 8.8 | 8.8 | Reference only; unchanged. |

The additional [small light capture](../../screenshots/app-light-small.png)
scores **8.6/10**. These are improvements toward the 9/10 target, not a claim
that every editor state has reached it.

### Resolution and verification

1. **Framing:** project visible voxel extents onto the camera basis, center
   the occupied silhouette, and retain the conservative fit with 4% extra
   breathing room. Empty documents retain the document-bounds fallback.
   A regression test checks the actual asymmetric sample in four views and
   four aspect ratios, verifies centering/margins, and checks isometric
   clearance for the minimum-window camera controls. Existing projection,
   picking and exact top-view tests also pass.
2. **Palette:** desktop allows five complete rows; compact mode allows two.
   Overflow adds explicit previous/next scroll controls beside the persistent
   color value. Native clicks were checked against the sample's actual RGB
   values: the final two rows become visible and previous returns to row one.
   See [dark](../../screenshots/polish-3/palette-small-dark.png) and
   [light](../../screenshots/polish-3/palette-small-light.png).
3. **Idle session:** one invitation replaces the repeated empty-history copy
   and disabled history controls. An already connected agent gets a waiting
   state. Real human edits retain their history, including compact mode:
   [dark](../../screenshots/polish-3/human-history-small-dark.png),
   [light](../../screenshots/polish-3/human-history-small-light.png).
4. **Gesture hint:** retained Lucide mouse/key/action glyphs, added quiet
   separators and individual tooltips, and aligned the hint with the view
   controls. Orbit/Pan/Zoom labels appear when there is room.
5. **Sidebar rhythm:** connection and actual agent content use the shared
   24px gutter. Layer selection surfaces extend 8px outside that alignment;
   the border is accounted for in the icon inset. Layers absorb spare vertical
   room while palette rows remain whole. Actual MCP edits and three captured
   views were exercised and inspected in
   [dark](../../screenshots/polish-3/agent-dark.png) and
   [light](../../screenshots/polish-3/agent-light.png).

Hard-requirement smoke checks retained the tool rail, Save icon + label, Geist,
gesture glyphs and both explicit themes. Pixel checks verified pure black and
white base surfaces at both sizes. Native Share copied its persisted URI through
the clipboard into a text field; the test compared that text with the stored
link, removed the temporary entry, and verified persisted Remix access.
[Dark Share](../../screenshots/polish-3/share-dark.png) and
[light Share](../../screenshots/polish-3/share-light.png) retain link/Copy feedback,
View/Comment/Remix choices, the local-preview notice and secondary Save a copy.

Checks completed successfully:

- `cargo fmt --all -- --check`
- `cargo build --workspace`, including the native Linux GPUI binary
- `cargo test --workspace` — **110 tests passed**, no failures
- `cargo clippy --workspace --all-targets -- -D warnings`
- Native captures and the palette/Share/MCP interaction checks described above
- `git diff --check`; no edits under `design/`; `/target/` remains ignored

### Remaining

- Compact glyph hints still need hover for exact mouse-button semantics. The
  larger layout now names each action, but minimum-width discoverability is
  still below the 9/10 target.
- At 1024×640, real human history pushes the palette below the initial fold;
  the sidebar must scroll. Active agent sessions also show few events at once,
  and the three capture thumbnails remain small. A resizable sidebar and
  further progressive disclosure would need a separate layout pass.
- Palette scroll buttons remain enabled at either end; they safely clamp but
  do not yet communicate that boundary with a disabled state.
- This pass did not repeat high-density, macOS, screen-reader or full keyboard
  accessibility acceptance. Those require separate evidence before describing
  the app as production-certified. No new platform claims are made here.
