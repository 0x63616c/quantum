# Voxelite native editor

A Rust/GPUI desktop shell for the shared Voxelite engine. It opens the bundled
Autumn Workshop diorama, or a `.voxl` path supplied on the command line. The
Concept A shell uses bundled Geist fonts, pure black and white themes, a unified
toolbar, a persistent left tool rail, compact palette previews, layers and real
command history. The sidebar adapts at the 1024×640 minimum window size; large
palettes and layer lists scroll within bounded panels.

The [polish round 2 review](../../docs/reviews/app-polish-2.md) records the
current visual assessment, verification and both-theme captures. The sidebar
switches between Session, Comments and History; the toolbar keeps Save within
reach. Sharing is deferred to the hosted edition. Static medium and semibold
Geist faces preserve the intended weight hierarchy on Linux.

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo run -p voxelite-app --bin voxelite -- --theme dark
cargo run -p voxelite-app --bin voxelite -- examples/autumn-workshop.voxl --theme light
```

The `native` feature is enabled by default. The library and its tests have no
GPUI dependency when built with `--no-default-features`:

```sh
cargo test -p voxelite-app --no-default-features
```

## Controls

| Action | Input |
| --- | --- |
| Place / erase / paint / fill / box / picker | B / E / P / G / U / I |
| Apply tool | Left click, or Enter at the keyboard cursor |
| Sample original voxel color | Alt-click |
| Orbit | Right-drag |
| Pan | Middle-drag or Shift + right-drag |
| Zoom / frame | Wheel / F |
| Keyboard cursor | Arrows for X/Z, Page Up/Down for Y |
| Previous / next palette color | `[` / `]` |
| Undo / redo | Ctrl+Z / Ctrl+Shift+Z; Apple: ⌘Z / ⇧⌘Z |
| New / open / save / save as | Ctrl+N / Ctrl+O / Ctrl+S / Ctrl+Shift+S; Apple: ⌘N / ⌘O / ⌘S / ⇧⌘S |
| Search commands | Ctrl+K (Apple: ⌘K); type, arrows, Enter, Escape |
| Settings | Ctrl+, (Apple: ⌘,); Escape or Done closes the settings view |
| Focus brush colors | Palette → Show Palette; arrows choose colors, Escape returns to the viewport |
| Cancel box or dismiss menus | Escape |
| Choose a preset palette | Palette menu, then Up/Down and Enter |
| Unsaved-changes prompt | Tab or Left/Right, Enter to confirm, Escape to cancel |

Apple shortcuts apply on native macOS and in browsers on Mac, iPhone and iPad.

Box uses two clicked corners and commits one solid integer box. The first click
is only a preview. Place targets the adjacent face or the bottom work plane.
Paint, erase and fill require the visible hit to belong to the selected layer;
select its layer first. Hidden and locked layers reject voxel edits. Mirror
X/Y/Z reflects around the document's center planes in one undo transaction.
Enable mirrors in the left rail, Tools, command search or Shift-X/Y/Z. Active axes stay highlighted in the rail. The viewport's projected X/Y/Z controls select Right/Top/
Front; Iso and the separate Frame button return to the fitted isometric view.

Selecting a preset does not change existing geometry or dirty the project.
Selecting a swatch changes the brush. Its color is reused or appended to the
project palette atomically with the next edit; a full palette fails without
partial edits. Presets use the exact shared-core PICO-8, ENDESGA 32, Sweetie 16,
grayscale, neutral, warm and vivid colors.

Layers can be added, selected, hidden and locked. Their metadata changes are
undoable. History and the connected MCP feed show actual edits and counts.

## Files and rendering

File menus use GPUI's native path prompts (the XDG desktop portal on Linux).
The CLI path is also supported. Saved files preserve optional forward metadata,
portable view data, comments, versions and IDs. A same-directory temporary file
is synced and atomically renamed before clearing dirty state. New, open and
quit protect unsaved changes; the built-in example has no write destination
until Save As. Failed saves retain the document and dirty state. Open and Save
perform file work on a background executor while editor input is blocked.
“Save & continue” completes the pending New/Open/Quit only after a successful
save; cancellation or failure keeps the project open.

Ordinary Save checks the destination against the bytes last opened/saved and
rejects external edits or deletion. Save As can replace an explicitly selected
regular file. Saving preserves existing permissions and syncs the parent
directory on Unix. Opening a symlink resolves its target; Save As rejects symlink
destinations. Conflict detection is optimistic: it does not lock out an unrelated
process racing the final rename.

Rendering uses `voxelite-render` on a background executor, with one task in
flight and coalesced requests. Navigation uses smaller previews, then a 2×
supersampled image after a short idle interval. The display camera travels with
the image. Pointer edits wait for the current frame, preventing edits through a
stale camera, including after a render error. Render jobs from an older edit,
replaced document (even the same project ID) or viewport size are discarded.
Fitted views adapt to resizing until the user navigates. Top is an exact Y-axis
view; every named view fits its own projected bounds. Unchanged clicks and color
sampling do not trigger a new CPU render.
Development profiles optimize the CPU renderer and core to keep iteration usable.

Appearance lives in Settings, with Light, Dark and System choices. System follows
live OS appearance changes. Preferences (appearance, grid, antialiasing,
navigation hints and render statistics) persist in `settings.json` alongside the
existing palette library, or in origin-local storage on the web. `VOXELITE_DATA_DIR`
can isolate desktop preferences. `--theme dark|light` overrides startup appearance
for reproducible captures.

Settings opens in a separate desktop window; repeated requests focus that window.
The web edition uses the same view in an in-page modal. Sections are searchable,
controls support Tab/Shift-Tab and Enter/Space, and changes apply to the editor
immediately. A failed save retains the previous preferences and reports the error.
On macOS, the native Voxelite menu includes Settings, About, Services, application
visibility and Quit. The native menus share the editor action registry; standard
clipboard commands act on the focused text field. There is no voxel clipboard
or persistent voxel-selection tool yet. Packaged macOS builds with Sparkle support
Check for Updates from the Voxelite menu and Settings → Updates. Builds without
the bundled updater disable the menu action and explain availability in Settings.
The transparent native titlebar places real traffic lights in the top toolbar.
Linux and web retain their in-window menus.

The bottom bar shows hovered voxel coordinates, active layer/tool, zoom relative
to the last fitted view, model bounds and viewport performance. Selection size
and count describe the inclusive box-tool preview; after commit they return to
zero. Performance shows the active voxel renderer (`CPU`), rolling average FPS
and average time between newly painted viewport images over about one second.
Only successful paints of new images count; render completions that never reach
the viewport and repaints of unchanged content are excluded. These timings include
rendering, scheduling and presentation cadence, not just CPU render cost or monitor
refresh rate. The bar holds its last average while idle (or shows `idle` before
two frames are timed). Starting work after an idle pause longer than one second
opens a fresh sample window; slow renders still count their full frame interval.
`--exit-after SECONDS` (alias `--screenshot-delay`) exits after the specified
interval for unattended captures. This explicit automation option does not
prompt about unsaved edits.

A PNG can be rendered without creating GPUI, a window, or a GPU:

```sh
cargo run -p voxelite-app --bin voxelite -- --viewport-png screenshots/viewport.png
```

## Linux dependencies and captures

The native Linux build is supported by the pinned GPUI family; no GUI stub or
platform exclusion is needed. Tested with Rust 1.99.1 on Debian, X11/Xvfb and
Mesa lavapipe. macOS is wired through GPUI's native platform but has not been
built or exercised in this Linux environment.

```sh
sudo apt-get install libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev \
  libvulkan-dev libx11-xcb-dev libxcb1-dev libfontconfig-dev libasound2-dev \
  libssl-dev libclang-dev xvfb mesa-vulkan-drivers imagemagick xdotool \
  dbus-x11 xdg-desktop-portal xdg-desktop-portal-gtk
bash scripts/capture-native.sh
```

The capture script uses real GPUI windows under Xvfb, not HTML or a composited
mockup, and writes `screenshots/app-dark.png` and `screenshots/app-light.png`.
It also captures both themes at 1024×640. The round 2 capture workflow verifies
palette authoring, a human reply, a named version and
preview isolation by querying the live app through MCP. `bash scripts/review-polish-2.sh`
exercises real native HEX/GPL file dialogs, Save and Save As. Earlier review
captures and reports remain historical evidence for the preceding shell revision.
Use `VOXELITE_CAPTURE_UI_ONLY=1 VOXELITE_CAPTURE_DIR=/tmp/voxelite-shots bash scripts/capture-native.sh`
for the focused UI capture suite: both main-window sizes, settings sections,
appearance persistence, clipboard search, window reuse/reopen and box telemetry.
The macOS menu, traffic-light positioning and background blur still require a
native Mac smoke test; Linux captures do not verify AppKit behavior.

`bash scripts/capture-native-scales.sh` adds native 150% and 200% density captures
in both themes while preserving a logical 1440×900 workspace.
A working session D-Bus and desktop portal backend are needed for file dialogs;
a minimal server without them can still open paths through the CLI and render
PNGs. Dialog errors appear in the status bar and preserve the current project.

## Dependency and stage boundaries

`gpui-unofficial` and `gpui-platform-gpui-unofficial` are pinned to **1.25.3**, with
the full resolved dependency graph in `Cargo.lock`. These are the Apache-2.0
[GPUI Archipelago registry distribution](https://github.com/gpui-archipelago/gpui-unofficial)
of Zed's GPUI family. Native platform features are X11, Wayland and font-kit;
web code and plugin crates are not built by this stage. Geist and Geist Mono
are copied into the crate with their OFL notices; `design/` is never a build-time
mutable output. The default example is embedded at compile time.

This stage implements the native editor polish scope, not every release row in
`PLAN.md`. Plugin hosting remains required by PLAN.md and is not implemented by this native polish stage. Recovery journals, GLB export UI, hosted collaboration and screen-reader
certification remain outside this round. Portable data is retained on save.

## Polish round 2: local authoring and live agents

The left rail exposes Place/Erase/Paint/Fill/Box/Picker with Lucide icons and
highlights the active tool. Mirror X/Y/Z controls live below the tool group.
Shift-X/Y/Z toggle mirrors; C starts a comment pin. Tooltips include shortcuts.
Tools → Choose tool retains searchable selection without a toolbar dropdown.
The toolbar provides Save and Settings actions. Plugin hosting remains an open v1 gate.

**Palette → Manage palettes** opens a persistent custom library. Create, rename,
duplicate, delete, edit a six-digit hex color, choose a hue or drag RGB channels,
add/remove/reorder slots, and import/export `.hex` / `.gpl` through native dialogs.
Presets and project colors are read-only in the manager; Duplicate creates an
editable brush palette. These operations never silently recolor document slots.
Libraries are validated and atomically replaced in the app's local data directory.

Hosted sharing is deferred to v2. The local Share preview, participant/access
settings and Save a copy action have been removed. Use **File → Save as…** to
save a `.voxl` under a new name; that destination becomes the working file.

**Comments** lists Open/Resolved/All threads. Click a projected numbered pin to
open its docked sidebar thread, or Add/C then click a voxel. Switch Anchor to Region and click
two corners for a region discussion. Enter can anchor at the keyboard cursor.
Reply, resolve and reopen use core commands and shared undo. Occluded/hidden/orphan
pins are muted and retain coordinates; the selected anchor has a model-space
outline. The snapshot's own comments appear while previewing history.

**History** has a draggable timeline (also Left/Right when the panel is active),
human circles and agent diamonds, and persistent named versions. Scrubbing keeps
the live document, revision, dirty state and undo stack unchanged. Editing is
blocked in preview. Return to live (also available in the viewport banner) or Escape discards the preview. Restore creates one new
undoable edit and rejects an intervening live change; reselect a state to capture
new restore preconditions. Named versions survive normal Save/Open.

Each native window hosts MCP on a private Unix socket, shown under **Agent →
Agent connection**. Copy the MCP configuration there, or choose an explicit path:

```sh
cargo build -p voxelite-app -p voxelite-mcp
# In one terminal (the socket's parent directory must already exist):
target/debug/voxelite --session-socket /tmp/voxelite-session.sock
# Configure your MCP host to run this stdio bridge:
target/debug/voxelite-mcp --connect /tmp/voxelite-session.sock
```

Start with `voxelite_capabilities_get` to discover the current model, epoch and
revision. Agent edits run through the **same engine** as human tools. The Session
feed contains completed MCP calls, actual voxel differences, errors and revisions;
expand a committed edit to revert it. Pause rejects new agent mutations but permits
reads. Local file dialogs defer agent requests until they close. New/Open changes
the session epoch and clears receipts so stale calls cannot edit a replaced project.

The bridge supports both MCP protocol profiles already supported by the standalone
server. It reserves stdout for JSON-RPC. Socket mode is 0600; up to 8 connections
and 16 queued requests are admitted. The UI processes up to 4 calls per poll.
Read-only MCP rendering pins immutable content on the owner, then executes on
one background worker per attachment. A second render receives `render_busy`;
mutations and pause remain available. Results retain their captured revision
and epoch. Completed preset inspections appear as three immutable thumbnails.
The opt-in Follow agent camera follows requested preset views and turns off on
manual orbit, pan, zoom, framing or capture selection. Explicit custom MCP camera
renders are returned to the caller but do not move the orthographic editor camera.
Document mutations remain atomic on the owner thread. Changes are memory-resident
until the app saves. Attached document create/open/save are excluded from the
catalog because file lifecycle and external-change detection belong to the app.
The standalone server retains those tools. This is a local session, not hosted
multi-user collaboration or a durable recovery journal.

`VOXELITE_DATA_DIR` overrides local palette/settings/socket storage for isolated runs.
The default is `$XDG_DATA_HOME/voxelite` or `$HOME/.local/share/voxelite`.
`bash scripts/capture-native.sh` now also captures main/tools/palettes/
comments/history/agent in both themes under `screenshots/polish-2/`. Its fixture
client drives the real stdio bridge and checks edit receipts. It also checks selection reversal, repeated swatch moves and small-window feature states. See
[the round 2 review](../../docs/reviews/app-polish-2.md) for validation and limits.

UI action glyphs are bundled Lucide SVGs with upstream ISC/Feather MIT notices in
`assets/icons/`. Save uses an icon and label. The viewport mouse/key legend has
a full hover tooltip. Text fields support selection contraction/reversal,
clipboard, IME range conversion, pointer selection and horizontal caret scrolling.
