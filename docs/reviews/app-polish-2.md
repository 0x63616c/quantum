# Native app polish round 2

Implemented on `build-v1`, 2026-10-25, from the `e141449` round 1 baseline.
This is an implementation review and executable local verification, not
independent design approval or macOS certification. `design/` was read only.
Plugin hosting is excluded from v1 as requested.

## Delivered

| Item | Implementation | Evidence |
| --- | --- | --- |
| Tool rail | Six tools, permanent X/Y/Z mirror toggles, shortcut tooltips, selected states; redundant toolbar picker removed | `afa35d8`; tools dark/light captures |
| Palette authoring | Persistent custom library, create/rename/duplicate/delete, slot hex/RGB picker, add/remove/reorder, exact presets, native HEX/GPL dialogs | `c62004e`; library tests; native input and dialog scripts |
| Share | Local-only popover, stable copyable link, access choices, participant editing, memory/file backend trait | `de28dfc`; backend contracts; native participant persistence assertion |
| Comments | Projected voxel/region anchors, occlusion/detached treatment, selection outline, threads, replies, resolve/reopen, filters | `00a0d8b`; core-backed tests; native reply verified through MCP |
| History | Read-only slider/keyboard previews, human/agent markers, persistent named versions, explicit guarded restore | `d51ef1f`; preview/restore tests; native named save and unchanged preview revision verified through MCP |
| Live agent | Private Unix socket, stdio bridge, shared engine, actual call/diff/error feed, pause and compensating per-step revert | `8ac3ffb`; real child-process integration test; real native MCP capture workflow |

The common native text field supports selection, clipboard, Unicode/IME input
and Tab focus navigation. Palette selection changes available brush colors;
existing geometry retains its document slots. Comments and versions use portable
core structures and survive `.voxl` saves.

## Native capture review

All images are actual GPUI windows rendered with Xvfb and Mesa lavapipe, using
bundled Geist fonts. No HTML images, composites or post-production edits are used.
The capture workflow opens the bundled art, drives real UI controls, then connects
an actual `voxelite-mcp --connect` child process to that window.

The first review found the Session empty state crowded the layer section. The
compact layout now uses a smaller empty state and keeps its palette accessible.
Comments and History use the available sidebar height. The palette manager has
bounded scrolling so custom controls fit within the minimum window. Tool rail
width participates in camera fitting and screen-ray geometry. Final review
checked dark/light contrast, pure black/white surfaces, selected states, swatch
legibility, model-space pins, thread text, and history preview visibility.

The native input checks duplicate PICO-8, rename it to a fixture label, edit its
first color, and assert the actual library JSON. They also add a local participant
and assert access settings. A human reply and named version are read back through
MCP, with role attribution. Scrubbing remains at live revision 8; it cannot invoke
editing shortcuts or silently commit the viewed state. Small waits after focusing
native fields let GPUI register their platform input handlers before synthetic
keystrokes are delivered.

| Artifact | Dark | Light |
| --- | --- | --- |
| Main, 1440×900 | [dark](../../screenshots/app-dark.png) | [light](../../screenshots/app-light.png) |
| Main, 1024×640 | [dark](../../screenshots/app-dark-small.png) | [light](../../screenshots/app-light-small.png) |
| Tool rail | [dark](../../screenshots/polish-2/tools-dark.png) | [light](../../screenshots/polish-2/tools-light.png) |
| Palette authoring | [dark](../../screenshots/polish-2/palettes-dark.png) | [light](../../screenshots/polish-2/palettes-light.png) |
| Local Share | [dark](../../screenshots/polish-2/share-dark.png) | [light](../../screenshots/polish-2/share-light.png) |
| Comments | [dark](../../screenshots/polish-2/comments-dark.png) | [light](../../screenshots/polish-2/comments-light.png) |
| History preview | [dark](../../screenshots/polish-2/history-dark.png) | [light](../../screenshots/polish-2/history-light.png) |
| Real agent activity | [dark](../../screenshots/polish-2/agent-dark.png) | [light](../../screenshots/polish-2/agent-light.png) |

The `mcp-*.json` capture receipts record six actual MCP mutations, including six
recolored voxels and three added voxels. `ui-*.json` records the live revision
after the human reply and version save, checked while the viewport is previewing
an older state. The immutable art fixture itself is unchanged.

## Verification

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace` — 107 tests, including the real stdio/socket process test.
- `cargo build --workspace` — native GPUI builds on Linux; no platform stub.
- `cargo test -p voxelite-app --no-default-features` — display-independent logic.
- `bash scripts/capture-native.sh` — both themes and sizes, fourteen feature PNGs,
  actual live agent edits, native palette/share input, and MCP verification of UI edits.
- `bash scripts/review-polish-2.sh` — real native HEX/GPL exports and GPL import,
  asserting all 36 original colors survive.

The socket test verifies private permissions, existing-path refusal, shared
geometry/history, agent origin, actual event differences, exact retry no-ops,
human revision conflicts, pause, stale epochs after replacement, EOF and socket
cleanup. Library tests cover validation, duplicate isolation, ordering,
interchange, persistence failure and deletion. Comment/history tests cover
orphaned anchors, replies, undo, save/reopen, immutable previews, redo previews,
intervening agent edits, and new reversible restores.

## Explicit boundaries

Sharing is a labeled local stub, not a hosted service or enforced remote access.
The live MCP bridge is supported on Unix (Linux/macOS); native macOS execution
and VoiceOver/IME platform certification were not available in this Linux box.
The app owns open/save operations while attached, preserving its existing file
conflict checks; the standalone MCP server retains its document lifecycle tools.
Session changes are memory-resident until the app saves. Recovery journaling,
hosted collaboration and runtime plugins are outside this round. Expensive MCP
renders run on the owner thread and may briefly stall native interaction; a
worker-rendering refactor remains separate from the shared mutation authority.
