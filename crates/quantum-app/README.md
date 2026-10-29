# Quantum native editor

A Rust/GPUI desktop shell for the shared Quantum engine. With no file argument,
it opens the welcome window; a `.voxl` path opens directly in the editor. An
invalid or unreadable path opens Welcome with an inline error naming the file.
Failed opens inside the app show an error and preserve the current document. The
Concept A shell uses bundled Geist fonts, pure black and white themes, a unified
toolbar, a persistent left tool rail, compact palette previews, layers and real
command history. The sidebar adapts at the 1024×640 minimum window size; large
palettes and layer lists scroll within bounded panels.

The sidebar switches between Session, Comments and History. Save is available
from the File menu and Cmd/Ctrl+S; the bottom-left cog opens Settings (also Cmd/Ctrl+,). Sharing
is deferred to the hosted edition. Static medium and semibold Geist faces preserve the intended weight hierarchy on Linux.

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo run -p quantum-app --bin quantum -- --theme dark
cargo run -p quantum-app --bin quantum -- examples/autumn-workshop.voxl --theme light
```

The `native` feature is enabled by default. The library and its tests have no
GPUI dependency when built with `--no-default-features`:

```sh
cargo test -p quantum-app --no-default-features
```

## Module layout

- `src/editor.rs` translates gestures into engine transactions;
  `src/editor/camera.rs` owns orbit geometry and voxel picking. The existing
  `editor::{OrbitCamera, Ray, Hit, raycast}` entry points remain re-exports.
- `src/actions.rs` derives shell metadata from the enabled command registry;
  `src/actions/bindings.rs` installs native menus and platform key bindings.
  Command regression tests live in `src/actions/command_tests.rs`.
- `src/ui/settings.rs` composes Settings pages. Its `settings/controls.rs`,
  `settings/history.rs` and `settings/window.rs` modules own control construction,
  navigation history and window lifecycle respectively. History tests live beside
  its private storage; page and search tests remain in `settings/tests.rs`.

Editor operations return `EditorResult<T>` with typed `EditorError` variants from
`src/editor/error.rs` for editor preconditions and underlying session, plugin,
I/O, format and encoding failures. Callers can match variants without parsing
user-facing messages; `Display` preserves the existing shell messages. Core
errors from tool and palette operations remain `EditorError::Core`; open failures
retain the requested path and a typed cause in `EditorError::Open`.

## UI capture states

The development-only `--show <state>` flag opens a surface at launch on the bundled
Autumn Workshop example, without scripted clicks. It is inert unless supplied,
implies `--example`, and rejects unknown/repeated states, project paths and
`--viewport-png`. Use a fresh `QUANTUM_DATA_DIR` for isolated preferences, palettes
and recents; `cargo xtask screenshots` does this automatically.

```sh
cargo run --locked -p quantum-app --bin quantum -- --show dialogs-about --theme light
cargo xtask screenshots --screen panels-history-dark --screen panels-history-light
```

Each full state below is a prefix plus one suffix. Its screenshot names are
`<state>-dark` and `<state>-light`.

| Prefix | Suffixes |
| --- | --- |
| `chrome-menu-` | `file`, `edit`, `view`, `tools`, `palette`, `layers`, `agent`, `extensions`, `window`, `help` |
| `panels-` | `palette`, `history`, `history-preview`, `comments-empty`, `comments`, `comment-thread`, `comment-draft`, `agent`, `agent-inspection` |
| `dialogs-` | `about`, `acknowledgements`, `keyboard-guide`, `commands`, `tools`, `extensions`, `unsaved-changes`, `save-waiting`, `palettes`, `palette-delete` |
| `popovers-` | `connect-agent`, `palette-chooser`, `palette-menu` |
| `overlays-` | `debug` |
| `settings-` | `general`, `appearance`, `viewport`, `shortcuts`, `agent`, `updates`, `advanced` |

The existing `welcome`, `editor-dark` and `editor-light` captures retain their args
and sizes. The editor captures already cover the title bar, toolbar, tool rail,
status bar, Session panel, layers and palette. All new editor surfaces use
1440×900; Settings uses its own **Quantum Settings** window at 880×640. About is
an editor overlay, not a separate window.

History fixtures add an empty layer and a named version through the editor;
preview uses an immutable snapshot. Comments use the normal plugin transactions.
The empty comments state selects the example’s empty Resolved filter. The agent
feed reuses a committed example entry and adds a fixed rejected request. Its
inspection state also renders three thumbnails; separate captures keep the expanded
event controls visible. The palette deletion fixture copies a palette in memory. No project is saved by these launch fixtures. The
save-waiting fixture holds the app-owned waiting/cancel surface with a static
spinner, without opening a desktop portal. Capture states omit live frame cadence in the status bar and debug overlay.

Native file pickers, macOS menus, Sparkle dialogs, the macOS eyedropper and hover
tooltip timing/placement still need platform/manual checks. The Linux captures
show the app menu surfaces and CPU debug overlay. Tooltips currently require
GPUI pointer dwell; there is no launch tooltip state. Newly introduced states
show “not on this side” when the before binary predates `--show`.

## Controls

Keyboard shortcuts are generated from `src/actions.rs`, which also supplies the
menus, command search and Settings → Shortcuts. Commands without a binding appear
in the settings sheet with “—”. Shortcut fields show the fixed registry bindings;
Reset is disabled while they are already at their built-in defaults. Regenerate
this table after editing the registry:

```sh
UPDATE_README=1 cargo test -p quantum-app --bin quantum readme_shortcuts_match_registry
```

`cargo test --workspace` checks that the table is current. “—” in the Windows /
Linux column denotes a macOS-only command.

<!-- BEGIN GENERATED COMMAND SHORTCUTS -->
| Action | Apple | Windows / Linux |
| --- | --- | --- |
| New project | ⌘N | Ctrl+N |
| Open project… | ⌘O | Ctrl+O |
| Open Sample… | ⇧⌘O | Ctrl+Shift+O |
| Save | ⌘S | Ctrl+S |
| Save as… | ⇧⌘S | Ctrl+Shift+S |
| Quit Quantum | ⌘Q | Ctrl+Q |
| Undo | ⌘Z | Ctrl+Z |
| Redo | ⇧⌘Z | Ctrl+Shift+Z |
| Search commands… | ⌘K | Ctrl+K |
| Settings… | ⌘, | Ctrl+, |
| Frame model | F | F |
| Toggle debug overlay | ⇧⌘D | Ctrl+Shift+D |
| Toggle Full Screen | ⌃⌘F | F11 |
| Place | B | B |
| Erase | E | E |
| Paint | P | P |
| Fill | G | G |
| Box | U | U |
| Color picker | I | I |
| Mirror X | ⇧X | Shift+X |
| Mirror Y | ⇧Y | Shift+Y |
| Mirror Z | ⇧Z | Shift+Z |
| Add comment pin | C | C |
| Welcome | ⇧⌘1 | Ctrl+Shift+1 |
| Minimize | ⌘M | Ctrl+M |
| Hide Quantum | ⌘H | — |
| Hide Others | ⌥⌘H | — |
<!-- END GENERATED COMMAND SHORTCUTS -->

Viewport and navigation controls:

| Action | Input |
| --- | --- |
| Apply tool | Left click, or Enter at the keyboard cursor |
| Sample original voxel color | Alt-click |
| Orbit | Right-drag |
| Pan | Middle-drag or Shift + right-drag |
| Zoom | Wheel |
| Keyboard cursor | Arrows for X/Z, Page Up/Down for Y |
| Previous / next palette color | `[` / `]` |
| Command search navigation | Type, arrows, Enter, Escape |
| Close settings | Escape or Done |
| Focus brush colors | Palette → Show Palette; arrows choose colors, Escape returns to the viewport |
| Cancel box or dismiss menus | Escape |
| Choose a preset palette | Palette menu, then Up/Down and Enter |
| Unsaved-changes prompt | Tab or Left/Right, Enter to confirm, Escape to cancel |

Apple shortcuts apply on macOS.

Box uses two clicked corners and commits one solid integer box. The first click
is only a preview. Place targets the adjacent face or the bottom work plane.
Paint, erase and fill require the visible hit to belong to the selected layer;
select its layer first. Hidden and locked layers reject voxel edits. Mirror
X/Y/Z reflects around the document's center planes in one undo transaction.
Enable mirrors in the left rail, Tools, command search or Shift-X/Y/Z. Active axes stay highlighted in the rail. The viewport's projected X/Y/Z controls select Right/Top/
Front; Iso and the separate Frame button return to the fitted isometric view.

Preset views, orientation clicks and agent views ease between camera poses over
250 ms. Orbit, pan and wheel gestures interrupt transitions and respond directly;
resize refits and opening a document snap immediately. Settings → General →
Animate camera disables transitions, and macOS Reduce Motion always disables them.

Selecting a preset does not change existing geometry or dirty the project.
Selecting a swatch changes the brush. Its color is reused or appended to the
project palette atomically with the next edit; a full palette fails without
partial edits. Presets use the exact shared-core PICO-8, ENDESGA 32, Sweetie 16,
grayscale, neutral, warm and vivid colors.

Layers can be added, selected, hidden and locked. Their metadata changes are
undoable. History and the connected MCP feed show actual edits and counts.

## Files and rendering

The split welcome window offers **New Project**, **Open…**, **Open Sample…**
(Cmd/Ctrl+Shift+O), and up to 12 recent projects. **Show at launch** is saved in
`settings.json`; turning it off starts a blank editor when no path is supplied.
Desktop settings writes run on the background executor, coalesce rapid changes,
and drain on graceful exit. Older settings files default to showing Welcome.
Tab (Shift+Tab in reverse) cycles through New Project, Open, Open Sample,
Show at launch, and the project list. Up/Down traverse recents followed by all
four samples in display order, clamping at either end; Enter opens the selection.
Left/Right also step through that order. Open Sample focuses the first sample.
Escape or Cmd/Ctrl+W closes Welcome, returning to an existing editor. Opening or creating a project closes Welcome and shows the
editor. If an editor is already open, its Save/Discard/Cancel flow protects edits.
**Window → Welcome** (Cmd/Ctrl+Shift+1) reopens the launcher; the native
**File → Open Recent** submenu opens a recent project directly. **Clear Recent
Projects** removes the history without deleting project files. Closing the last
window quits, following the existing desktop lifecycle.

Recent projects persist in versioned `recents.json` beside `settings.json`, with
canonical-path deduplication and missing files filtered out. The `thumbnails/`
cache stores 640 × 480 CPU-rendered PNGs of saved documents, keyed by canonical path
and file modification time/length plus renderer version. Saves update the preview; missing/stale previews are
rendered lazily in the background while Welcome shows a placeholder. Preview
failures do not prevent opening or saving. Cache hits are decoded within a fixed
preview budget; damaged entries regenerate without touching the project.
`QUANTUM_DATA_DIR` isolates both stores.
The same sample cards appear in a two-column grid for an empty history and a
four-column strip pinned below recents and a divider. The compact cards show
only titles; the empty-history grid also shows voxel and layer counts.
Long histories scroll above the strip; short histories leave spare space between
the list and divider. Names and metadata stay on one line and ellipsize within each card;
the full sample name remains available in its tooltip.

Autumn Workshop, Lighthouse, Cherry Blossom and Snow Cabin are embedded `.voxl`
projects. Samples open as untitled copies (no destination path) through the same
unsaved-change flow as file opens. Never-saved sessions also require confirmation
before New, Open, close or quit, even before the first edit. Save opens the path
picker; cancellation or failure keeps the copy open and unsaved. Only a successful
project save adopts a destination; exporting does not satisfy Save/Discard/Cancel.
The three new projects total about 235 KB;
Autumn Workshop was already embedded, so packaging needs no extra resources.
Sample counts come from the documents. Both samples and recents use the same
opt-in renderer treatment: isometric framing, directional face light, per-voxel
occlusion, soft contact shadow, transparency and 2× supersampling. A process-wide
render gate bounds concurrency; callers wait asynchronously before dispatching
blocking work. Repeat saves coalesce in a queue capped at 12 paths. Welcome owns
its decoded image cache and evicts obsolete revisions (at most 12 recent previews,
4 sample previews and the brand mark). One image at a time progressively replaces
a quiet placeholder. The regular viewport and export defaults are unchanged.

`--example` opens the bundled Autumn Workshop with no write destination, including
in the existing editor capture scripts. `--viewport-png` without a path still
renders that bundled example without opening a window.

File menus use GPUI's native path prompts (the XDG desktop portal on Linux).
The CLI path is also supported. Saved files preserve optional forward metadata,
portable view data, comments, versions and IDs. A same-directory temporary file
is synced and atomically renamed before clearing dirty state. New, open and
quit protect unsaved changes; the built-in example has no write destination
until Save As. Failed saves retain the document and dirty state. Open and Save
perform file work on a background executor while editor input is blocked.
“Save & continue” completes the pending New/Open/Quit only after a successful
save; cancellation or failure keeps the project open. While waiting for a Save As
file dialog, Cancel or Escape returns to the editor without saving or continuing
the pending command. Late responses from an abandoned picker are ignored.

Ordinary Save checks the destination against the bytes last opened/saved and
rejects external edits or deletion. Save As can replace an explicitly selected
regular file. Saving preserves existing permissions and syncs the parent
directory on Unix. Opening a symlink resolves its target; Save As rejects symlink
destinations. Conflict detection is optimistic: it does not lock out an unrelated
process racing the final rename.

Rendering uses `quantum-render` on a background executor, with one task in
flight and coalesced requests. Navigation uses smaller previews, then a 2×
supersampled image after a short idle interval. The display camera travels with
the image. Pointer edits wait for the current frame, preventing edits through a
stale camera, including after a render error. Render jobs from an older edit,
replaced document (even the same project ID) or viewport size are discarded.
Fitted views adapt to resizing until the user navigates. Top is an exact Y-axis
view; every named view fits its own projected bounds. Unchanged clicks and color
sampling do not trigger a new CPU render.
Development profiles optimize the CPU renderer, shared mesh generation and core
to keep iteration usable.

Dirty projects show “Edited” next to the in-app title and the native macOS close
button indicator. Clean projects have no status subtitle.

Appearance lives in Settings, with Light, Dark and System choices. System follows
live OS appearance changes. Preferences (appearance, grid, antialiasing,
navigation hints and render statistics) persist in `settings.json` alongside the
existing palette library. `QUANTUM_DATA_DIR`
can isolate desktop preferences. `--theme dark|light` overrides startup appearance
for reproducible captures.

Settings opens in a separate desktop window; repeated requests focus that window.
The titlebar capsule
toggles the inset sidebar and navigates section history. Search lists matching
sections and individual settings; selecting a setting opens its parent section.
Appearance offers System, Light and Dark window previews. Settings controls support Tab/Shift-Tab and Enter/Space, and changes apply to the editor
immediately. A failed save retains the previous preferences and reports the error.
On macOS, the native Quantum menu includes Settings, About, Services, application
visibility and Quit. The native menus share the editor action registry; standard
clipboard commands act on the focused text field. There is no voxel clipboard
or persistent voxel-selection tool yet. Packaged macOS builds with Sparkle support
Check for Updates from the Quantum menu and Settings → Updates. Builds without
the bundled updater disable the menu action and explain availability in Settings.
The transparent native titlebar places real traffic lights in the top toolbar.
Linux development builds retain their in-window menus.

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
cargo run -p quantum-app --bin quantum -- --viewport-png screenshots/viewport.png
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
mockup, and writes `screenshots/app-dark.png` and `screenshots/app-light.png`
to the ignored capture directory.
It also captures both themes at 1024×640. The round 2 capture workflow verifies
palette authoring, a human reply, a named version and
preview isolation by querying the live app through MCP. `bash scripts/review-polish-2.sh`
exercises real native HEX/GPL file dialogs, Save and Save As. Earlier review
captures are in the [private archive](https://github.com/0x63616c/quantum-archive); the reports remain
historical evidence for the preceding shell revision.
Use `QUANTUM_CAPTURE_UI_ONLY=1 QUANTUM_CAPTURE_DIR=/tmp/quantum-shots bash scripts/capture-native.sh`
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
of Zed's GPUI family. Linux development builds use X11, Wayland and font-kit.
Geist and Geist Mono
are copied into the crate with their OFL notices. The default example is embedded
at compile time.

See [product direction](../../docs/adr/0003-product-direction.md) for product scope
and deferred work. The app registers statically linked built-in plugins through
`app_host()`; external plugin loading and scripting remain deferred. Portable
data is retained on save.

## Polish round 2: local authoring and live agents

The left rail exposes Place/Erase/Paint/Fill/Box/Picker with Lucide icons and
highlights the active tool. Mirror X/Y/Z controls live below the tool group.
Shift-X/Y/Z toggle mirrors; C starts a comment pin. Tooltips include shortcuts.
Tools → Choose tool retains searchable selection without a toolbar dropdown.
The bottom-left cog opens Settings; Save is in the File menu and Cmd/Ctrl+S.

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
the live document, revision, dirty state and undo stack unchanged. Play/Pause
(Space while the History panel is active) replays the retained states at about
four steps per second, starting at the current entry or the first entry when at
the end. Playback stops at the last entry; scrubbing, edit/undo attempts, Return
to live, Restore, and incoming agent edits pause it. Slow renders slow playback
so each state can be displayed. Editing is
blocked in preview. Return to live (also available in the viewport banner) or Escape discards the preview. Restore creates one new
undoable edit and rejects an intervening live change; reselect a state to capture
new restore preconditions. Named versions survive normal Save/Open.

Each native window hosts MCP on a private Unix socket, shown under **Agent →
Agent connection**. Copy the MCP configuration there, or choose an explicit path:

```sh
cargo build -p quantum-app -p quantum-mcp
# In one terminal (the socket's parent directory must already exist):
target/debug/quantum examples/autumn-workshop.voxl --session-socket /tmp/quantum-session.sock
# Configure your MCP host to run this stdio bridge:
target/debug/quantum-mcp --connect /tmp/quantum-session.sock
```

Start with `quantum_capabilities_get` to discover the current model, epoch and
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

`QUANTUM_DATA_DIR` overrides local palette/settings/socket storage for isolated runs.
On macOS the default is `~/Library/Application Support/quantum`; other platforms
use `$XDG_DATA_HOME/quantum` or `$HOME/.local/share/quantum`. Each JSON file migrates
once from the legacy data directory when its new destination is absent. Migration
never replaces an existing destination and records a `.migrated` marker to prevent
legacy data from returning after recovery. Legacy files are left in place so an older build still finds them.
Preferences, recent projects and the palette library use bounded `JsonStore` storage.
Malformed JSON moves to a unique `*.corrupt` sibling before reporting an error
(recent projects fall back to an empty list). All replacements stage beside the
original, sync the file, rename, then sync the directory on Unix.
`bash scripts/capture-native.sh` now also captures main/tools/palettes/
comments/history/agent in both themes under `screenshots/polish-2/`. Its fixture
client drives the real stdio bridge and checks edit receipts. It also checks
selection reversal, repeated swatch moves and small-window feature states.

UI action glyphs are bundled Lucide SVGs with upstream ISC/Feather MIT notices in
`assets/icons/`. Settings uses an always-visible bottom-left cog, with the optional
keyboard guide to its right. The viewport mouse/key legend has
a full hover tooltip. Text fields support selection contraction/reversal,
clipboard, IME range conversion, pointer selection and horizontal caret scrolling.

### Native integration tests

`cargo test -p quantum-app --locked --features test-support` also exercises real
GPUI key dispatch and settings/history shutdown using its deterministic test
executor. CI enables this feature for workspace tests and Clippy. It is separate
from `native` so production builds and `--no-default-features` model tests do not
pull in GPUI's testing dependencies.

### Crash reports and feedback

Rust panics create redacted local reports in the app data directory's `crashes/`
folder. Next launch offers the complete preview from Welcome or the editor.
Help → Send feedback… composes bug or feature reports with version/OS metadata.
Nothing is sent automatically: Continue to GitHub opens a prefilled public issue
form and requires a GitHub account and submission there. No production relay is
configured; email and scene attachments are unavailable. Copy/Save handle reports
that exceed the browser URL limit. Browser opening never deletes the local crash;
Keep for later retains it and Delete report explicitly removes it.
See the [privacy note](../../site/content/getting-started.html) for collected fields,
redaction limits, storage/retention and unsupported crash types.

Capture states `dialogs-feedback`, `dialogs-feedback-relay`, `dialogs-crash-relay` and `dialogs-crash-report` open the independent
report window (880×760); both support `--theme dark|light`. The crash fixture writes
only to the selected local app data directory. Use an isolated `QUANTUM_DATA_DIR`
for manual capture runs. These fixtures are for UI review and never send data.
