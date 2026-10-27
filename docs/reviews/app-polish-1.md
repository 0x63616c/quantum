# App polish round 1

> Layout update (2026-10-26): the toolbar Save button described in this historical
> review has been removed. Save remains in File and Cmd/Ctrl+S. Settings is now
> an always-visible bottom-left cog beside the optional keyboard guide.

Baseline: `739f767`, `build-v1`, 2026-10-25. Reviewed the actual 1440×900
native screenshots against `PLAN.md` §§8–9, `design/ART-CHECKLIST.md`, Concept A's
HTML, both concept themes and the palette reference. This is a deliberately
strict review by the implementing agent, not independent design approval.
Nothing under `design/` is edited.

## Before: dark 5.8/10, light 5.5/10

The artwork carries these screens. The shell is functional, but looks like an
engineering preview: almost uniform typography, arbitrary whitespace, too many
small outlines and no compelling activity hierarchy. A score of 9 would require
consistent states and convincing native finish; 10 is reserved for exceptional,
independently validated execution. These scores assess visual craft, not feature
completion or Apple accessibility certification.

| Area | Dark | Light | Concrete defects |
| --- | ---: | ---: | --- |
| Spacing and alignment | 6 | 6 | Sidebar gutters alternate between 16 and 20 px; headings, rows and swatches do not share an optical edge. History leaves a dead middle above an oversized disabled Undo. Navigation help floats 66 px above the bottom while camera controls sit on a different baseline. |
| Type and hierarchy | 5 | 5 | Project, tool, section titles and body labels all read at roughly 13 px and similar weight. The project is visually weaker than its toolbar deserves. `NATIVE / v0.1`, `r000`, `Human` and `CPU · 2× AA` give implementation metadata undue prominence. |
| Contrast and surfaces | 7 | 6 | Black/white bases are correct and secondary text is readable. Dark chrome lacks hierarchy beyond dividers; light outlines accumulate into a wireframe. The palette and layer panels draw adjacent horizontal separators. |
| Iconography | 5 | 5 | Four identical cube icons cannot communicate four camera directions. The active Frame button resembles a toggled mode, although it executes an action. Every unlocked layer has a tiny padlock competing with its name; icon-only controls need explicit tooltips. No orientation reference accompanies the model. |
| Borders and radii | 5 | 4 | Every swatch has an outer border, inset gap and inner rounded rectangle. This is particularly harsh on white. Radii of 3/5/6/7 px appear without a clear control hierarchy. The camera group adds another boxed perimeter. |
| States | 5 | 5 | Selected camera and layer rely predominantly on fill/color; selected swatches have a thin, weak blue outline. Disabled Undo has an active-looking pointer cursor. Hovering a selected action replaces its selection fill. |
| Palette previews | 5 | 4 | 36 double-framed squares dominate the lower sidebar. Color samples are needlessly smaller than their targets. The bracket hint is cryptic. The chooser compresses each palette to a 9 px strip and truncates palettes at 32 colors. |
| Empty history | 4 | 4 | “Local session / Human / History / Project ready / Your edits will appear here” repeats status across five labels without a useful next action. A large disabled Undo receives more emphasis than the empty state. |
| Viewport / artwork | 8 | 8 | The original autumn diorama has strong material separation and a readable silhouette. Large roof/foliage texture is slightly soft; the floor grid blankets the whole lower viewport. The giant upper void and low model placement weaken balance. |
| Theme-specific finish | 6 | 5 | Dark is appropriately black, but sparse hierarchy makes it feel unfinished. White exposes the repetitive swatch frames and weak type scale more severely. Neither yet meets the reference's calm density. |

## Fix scope

Rework the existing native shell: purposeful type hierarchy, shared gutters and
control metrics, calmer toolbar, honest local activity empty state, integrated
history actions, coherent selected/hover/disabled states, readable layer controls,
unframed color tiles with a strong selected marker, fuller preset previews and
clear camera orientation. Preserve real art, real document/history state and
pure black/white base surfaces. Re-capture and inspect both themes and the minimum
window size, then exercise changed interactions and run workspace checks.

The reference's live agent transcript, comments, captured-view inspector,
palette authoring and native material integration are broader implementation
gates already identified in the app README. Decorative fake activity, fake
presence or nonfunctional tabs would make this review worse, not close those
gates. No claim of full v1 acceptance follows from visual improvements.

## After: dark 8.1/10, light 8.2/10

The final native captures are clearly better. The typography now has actual
weight contrast, the palette reads as color rather than a matrix of outlined
buttons, and the sidebar has a deliberate reading order. Light mode benefits
most from removing the double swatch frames. The artwork remains the dominant
surface. These are strong improvements, not a 9/10 release signoff.

| Area | Dark | Light | Implemented result |
| --- | ---: | ---: | --- |
| Spacing and alignment | 8 | 8 | Shared 24 px panel/header gutters; 32 px minimum control targets; 36 px layer rows, 32 px at minimum height. Viewport help and camera controls share a baseline. Four layers and the active color fit at 1024×640. |
| Type and hierarchy | 8.5 | 8.5 | 15 px semibold project title, 14 px semibold section headings, 13 px labels, 12 px metadata. Diagnosed the Linux font matcher resolving variable-font weight requests to regular; bundled genuine 500/600 instances of the existing licensed Geist font. |
| Contrast and surfaces | 8 | 8.5 | Exact black/white bases retained, one blue accent, quieter grid, redundant divider removed. Primary Save has explicit foreground/background tokens in both themes. |
| Iconography | 8 | 8 | Camera-relative X/Y/Z orientation controls invoke the real Right/Top/Front views. Text camera tabs replace repeated cube icons. Frame is a separate action. Layer and color tooltips identify their actions/values. |
| Borders and radii | 8.5 | 8.5 | 6 px controls/swatches, 8 px floating panels and compact icon containers. Single subtle swatch edges replace the nested frames; shadows are confined to floating UI. |
| States | 8 | 8 | Camera selection has an underline, layer selection a solid edge and heavier text, swatch selection a thick edge plus luminance-contrasted check. Selected buttons retain their state on hover. Disabled history actions no longer use a pointer cursor. |
| Palette previews | 8.5 | 8.5 | Full 32 px color tiles, 6 px gaps, readable selected hex and “Brush color” caption. Preset chooser shows discrete color previews, exact counts, preset/project identity and an explicit truncation label above 32 colors. No geometry is recolored by choosing a palette. |
| Empty history | 8 | 8 | “Recent changes” has compact Undo/Redo actions. One honest “No edits yet” message and an operable Keyboard guide replace repeated status copy and the large disabled button. Actual edits populate the same area. |
| Viewport / artwork | 8 | 8 | Original diorama preserved. Reduced grid contrast and simplified camera controls give the art more breathing room. Camera fitting and the CPU renderer remain unchanged. |
| Theme-specific finish | 8 | 8.5 | Dark remains genuinely black with readable secondary labels. White now feels intentionally composed, with restrained separators and clean palette samples rather than nested white frames. |

### Capture / critique iterations

1. Reworked the hierarchy, spacing, activity, swatches and camera controls;
   compiled, then captured real GPUI windows. The runtime caught GPUI rejecting
   a second hover-style assignment; consolidated the primary button hover in
   the shared button builder before proceeding.
2. Inspected the first successful dark/light captures. They exposed SVGs that
   did not inherit text color and a compact sidebar that clipped the last layer
   and color caption. Restored explicit icon tokens and rebalanced compact
   section heights. Static fonts visibly restored medium/semibold hierarchy.
3. Captured both themes again at both window sizes. Confirmed the fixes, replaced
   the guide's misleading down-chevron with a forward arrow, and refreshed the
   final screenshots. Inspected preset previews, guide/modal, axis views, long
   names, 64 layers/256 colors and fractional/high-density chrome as well.

### Verification

Linux, Rust 1.99.1, pinned GPUI 1.25.3, Xvfb X11 and Mesa lavapipe. These are real
native application captures, not HTML mockups or edited/composited screenshots.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| `cargo test --workspace` | Pass, 96 tests |
| `cargo test -p voxelite-app --no-default-features` | Pass, 22 tests |
| `cargo build --workspace` | Pass, including the Linux GPUI binary |
| `bash scripts/capture-native.sh` | Pass; both themes at 1440×900 and 1024×640, plus headless viewport PNG |
| `VOXELITE_REVIEW_ARTIFACTS="$PWD/screenshots/polish-1" bash scripts/review-native.sh` | Pass; native input/dialog assertions, boundary layouts and both themes at 1.5×/2× |

The native script confirms axis clicks and mirror changes leave the document
clean, command typing does not edit the scene, keyboard palette selection works,
Save & continue persists exactly one voxel and opens New, and dialog input,
Cancel and Discard retain their established behavior. Both guide actions opened
the real help dialog. No new tests merely duplicate style constants.

### Evidence

| Theme | Before | Final | Minimum window |
| --- | --- | --- | --- |
| Dark | [5.8/10](../../screenshots/polish-1/before-dark.png) | [8.1/10](../../screenshots/app-dark.png) | [1024×640](../../screenshots/app-dark-small.png) |
| Light | [5.5/10](../../screenshots/polish-1/before-light.png) | [8.2/10](../../screenshots/app-light.png) | [1024×640](../../screenshots/app-light-small.png) |

- Preset chooser: [dark](../../screenshots/polish-1/palette-dark.png),
  [light](../../screenshots/polish-1/palette-light.png).
- Keyboard guide: [dark](../../screenshots/polish-1/guide-dark.png),
  [light](../../screenshots/polish-1/guide-light.png).
- Camera axes: [Right](../../screenshots/polish-1/axis-right.png),
  [Top](../../screenshots/polish-1/axis-top.png); these include hover tooltips.
- [Active mirror controls](../../screenshots/polish-1/mirror-active.png),
  [command search](../../screenshots/polish-1/command-search.png),
  [unsaved changes](../../screenshots/polish-1/unsaved.png).
- Boundary fixture: [dark](../../screenshots/polish-1/boundary-dark.png),
  [light](../../screenshots/polish-1/boundary-light.png).
- 1.5×: [dark](../../screenshots/polish-1/app-dark-1.5x.png),
  [light](../../screenshots/polish-1/app-light-1.5x.png).
- 2×: [dark](../../screenshots/polish-1/app-dark-2x.png),
  [light](../../screenshots/polish-1/app-light-2x.png).

### Remaining limits / next review

The renderer still caps its texture at 1024 px per axis, so the art is softer
than the vector chrome at high density. Framing still leaves more upper space
than the concept, and navigation overlays can cross the fitted artwork in
orthographic views. Those remain concrete reasons for the viewport's 8/10.
Palette scrolling has no persistent scrollbar, and the preset chooser remains
a chooser rather than the concept's full authoring manager. Native vibrancy,
resizable panes, persistent appearance, full keyboard focus traversal and
macOS/VoiceOver validation remain open stage gates. This round does not claim
to implement those, real agent sessions or comments. The live agent experience
is still the largest gap against the agents-first checklist, despite the more
polished, truthful local-session presentation.
