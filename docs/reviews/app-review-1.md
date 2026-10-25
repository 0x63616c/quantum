# App review round 1

Review baseline: `69a24d0`, branch `build-v1`, 2026-10-25. Review performed by the
implementing agent, acting as principal engineer and product-design reviewer;
this is not independent signoff. Read `PLAN.md`, `design/ART-CHECKLIST.md` and
Concept A's HTML and dark/light references. No changes under `design/`.

**Disposition: all eight medium/high findings below fixed.** The existing native
stage is materially safer and closer to Concept A. This is not full v1 release
approval; the integration and hardware gates below remain explicit.

## Ranked findings and fixes

| ID | Severity | Finding and reproduction | Fix and evidence |
| --- | --- | --- | --- |
| A01 | High | Open the same file in two editors, save externally, then Save in the first editor: external work was silently overwritten. Atomic save through a symlink replaced the link itself. | Bounded disk fingerprints reject external edits/deletion; opening resolves the target; Save As rejects symlinks. Creation uses no-clobber persistence, existing permissions survive, and Unix directory entries are synced. Tests exercise two writers, deletion, recovery to a new path, symlinks and failed saves. |
| A02 | High | Select Fill and press Enter at an empty keyboard cursor: the whole connected empty volume filled. Keyboard Picker rejected a locked/hidden active layer although sampling is read-only. | Empty Fill returns before palette import; Picker samples the visible composite before checking edit permissions. Tests assert unchanged document/revision/dirty state and sampling from another visible layer. |
| A03 | High | Native file dialogs did not isolate keyboard input; parsing/serialization blocked the UI thread. Saving from the unsaved-changes prompt abandoned the requested New/Open/Quit. | Background open/save, a busy input barrier, close-request protection, and save receipts. Save & continue resumes only on success. Cancel/failure retains the document. The prompt has keyboard navigation and defaults to Save. Native smoke tests inject editing keys into the app during the portal dialog and assert an unchanged title; save-and-new checks the saved voxel count. |
| A04 | High | A failed render could leave an old image visible while picking used the new camera. Reopening the same document ID at revision zero could install a previous editor's result. Resizing also accepted stale dimensions. | Every render carries an editor generation, revision, camera and target size. Installation rejects old generations/revisions/sizes. Picking requires a current settled image and uses its camera; overlay projection uses actual image dimensions. Native tests replace documents while render work is active. Existing projection/raycast tests plus the new camera-fit test verify geometry. |
| A05 | Medium | The permanent left rail violated F01 and reduced viewport space. A large palette preceded history, reversing the reference's activity-first hierarchy. | Removed the rail; moved active-tool selection/search and mirrors into the unified toolbar. Local session/history precedes layers and the compact palette. Kept truthful local activity rather than inventing an agent transcript. Both theme captures were compared again with Concept A. |
| A06 | Medium | At 1024×640 the palette popup exceeded available height, viewport hints collided with camera controls, and long names displaced toolbar actions. Disabled controls were nearly invisible. Resizing clipped a previously framed model. | Bounded scrolling, responsive history/layer/palette sizing, title ellipsis, separate hint/control rows, readable disabled tokens/reasons and platform-specific shortcuts. Fitted cameras adapt to aspect changes until manually navigated. Boundary captures exercise 64 layers, 256 colors and a 128-byte project name. |
| A07 | Medium | Palette selection required the mouse, camera presets had no selected state, and Top was tilted. Tool choice was absent from command search during the first fix pass. | Arrow/Enter/Escape palette navigation with scroll-to-selection; visible camera states, an Iso return action, exact top-down basis and per-view framing. Tool search is reachable from the toolbar, Tools menu and command palette. Unit tests check fitted bounds and exact top-ray direction; native keyboard workflows pass. |
| A08 | Medium | Sampling a color, clicking empty space or rejecting an edit always launched another CPU render, temporarily blocking pointer editing despite unchanged geometry. | Report handling only requests a render when the current frame identity is invalid. No-op tools remain clean and retain palette/history, covered by editor tests. |

## Product-design assessment

The initial app screenshots had the right black/white foundation and real art,
but the left rail, large palette and faint disabled states made the workspace
feel like a different editor. The revised shell follows Concept A's unified
toolbar and activity/layers/palette hierarchy. The viewport remains the largest
surface; labels use Geist, coordinates/shortcuts use Geist Mono, and the single
blue accent denotes active tools, mirrors, camera and focus. Destructive Discard
is text/outline, with Save selected by default.

The sample remains the original 27,897-voxel Autumn Workshop. It intentionally
differs from the concept's Woodland cottage. No concept pixels were pasted into
the application. Dark/light base surface samples are exact `#000000`/`#ffffff`.
At minimum size, layers and palettes have their own scroll regions; all nine
menus, command search, save, camera controls and active color remain reachable.
The full 36-color example palette is visible at 1440×900.

## Verification

Environment: Linux, Rust 1.99.1 stable, pinned GPUI 1.25.3, Xvfb X11, Mesa
lavapipe software Vulkan and the GTK desktop portal. No hardware GPU/display is
required for these checks. Native GPUI builds without a Linux exclusion.

| Check | Result |
| --- | --- |
| Baseline fmt, clippy, workspace tests | Pass; 90 tests |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| `cargo test --workspace` | Pass; **96 tests**, including **22 editor tests** |
| `cargo test -p voxelite-app --no-default-features` | Pass; **22 tests**, no GPUI |
| `cargo build --workspace` | Pass, including the native GPUI app |
| `bash scripts/capture-native.sh` | Pass; dark/light at 1440×900 and 1024×640 plus display-free viewport |
| `bash scripts/review-native.sh` | Pass; workflows, boundary layouts and dark/light at 1.5×/2× |

Native regression script assertions cover command typing isolation, searchable
tool selection, keyboard palette selection, dirty-title transitions, successful
Save & continue followed by New, exactly one persisted voxel edit, keyboard input
isolation during the real native file chooser, cancellation, reopening through
the chooser, Cancel without losing edits, and Discard followed by New. It uses
only copied/temp fixtures. GTK's Location field under bare Xvfb required clicking
the actual Open button; the script records this rather than pretending the
chooser's Enter activation was reliable.

## Evidence

- [Dark, 1440×900](../../screenshots/app-dark.png)
- [Light, 1440×900](../../screenshots/app-light.png)
- [Dark, 1024×640](../../screenshots/app-dark-small.png)
- [Light, 1024×640](../../screenshots/app-light-small.png)
- [Display-free viewport](../../screenshots/viewport.png)
- [Tool picker](../../screenshots/review-1/tools.png),
  [command search](../../screenshots/review-1/command-search.png),
  [palette chooser](../../screenshots/review-1/palettes.png),
  [unsaved changes](../../screenshots/review-1/unsaved.png)
- [Dark boundary layout](../../screenshots/review-1/boundary-dark.png),
  [light boundary layout](../../screenshots/review-1/boundary-light.png)
- [Dark 1.5×](../../screenshots/review-1/app-dark-1.5x.png),
  [light 1.5×](../../screenshots/review-1/app-light-1.5x.png),
  [dark 2×](../../screenshots/review-1/app-dark-2x.png),
  [light 2×](../../screenshots/review-1/app-light-2x.png)
- [Capture script](../../scripts/capture-native.sh),
  [native workflow script](../../scripts/review-native.sh)

## Limits of this signoff

No medium/high finding in the reviewed native-stage implementation is deferred.
The broader v1 release gates already documented by the native stage remain:
live session attachment, plugin hosts, web parity, recovery journals, palette
library authoring, comments/version browsing, export UI, persistent/system theme
preferences and assistive-technology certification. A local history header is
not a working agent session. macOS, Wayland and VoiceOver were not tested here.

The shared CPU renderer limits output to 1024 pixels per axis. Native chrome was
captured at 1×/1.5×/2×; the viewport texture remains bounded and is softer when
scaled up. These captures establish layout and rendering correctness, not the
production GPU viewport's latency or resolution gates. Save conflict protection
is an optimistic comparison, not a filesystem lock against a process writing
between the final check and rename. Neither limitation is concealed as a
production release guarantee.
