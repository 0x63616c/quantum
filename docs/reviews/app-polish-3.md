# App polish round 3

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

Pending implementation, native recapture, visual inspection and workspace checks.
