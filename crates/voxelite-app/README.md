# Voxelite native editor

A Rust/GPUI desktop shell for the shared Voxelite engine. It opens the bundled
Autumn Workshop diorama, or a `.voxl` path supplied on the command line. The
Concept A shell uses bundled Geist fonts, pure black and white themes, a unified
toolbar, a compact tool rail, palette previews, layers and real command history.

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
| Undo / redo | Ctrl/Cmd Z / Shift Ctrl/Cmd Z |
| New / open / save / save as | Ctrl/Cmd N / O / S / Shift S |
| Search commands | Ctrl/Cmd K; type, arrows, Enter, Escape |
| Cancel box or dismiss menus | Escape |

Box uses two clicked corners and commits one solid integer box. The first click
is only a preview. Place targets the adjacent face or the bottom work plane.
Paint, erase and fill require the visible hit to belong to the selected layer;
select its layer first. Hidden and locked layers reject voxel edits. Mirror
X/Y/Z reflects around the document's center planes in one undo transaction.

Selecting a preset does not change existing geometry or dirty the project.
Selecting a swatch changes the brush. Its color is reused or appended to the
project palette atomically with the next edit; a full palette fails without
partial edits. Presets use the exact shared-core PICO-8, ENDESGA 32, Sweetie 16,
grayscale, neutral, warm and vivid colors.

Layers can be added, selected, hidden and locked. Their metadata changes are
undoable. History shows actual committed edits and counts; no simulated agent
activity is displayed. Agent and Extensions menus explain the current stage's
integration boundary.

## Files and rendering

File menus use GPUI's native path prompts (the XDG desktop portal on Linux).
The CLI path is also supported. Saved files preserve optional forward metadata,
portable view data, comments, versions and IDs. A same-directory temporary file
is synced and atomically renamed before clearing dirty state. New, open and
quit protect unsaved changes; the built-in example has no write destination
until Save As. Failed saves retain the document and dirty state.

Rendering uses `voxelite-render` on a background executor, with one task in
flight and coalesced requests. Navigation uses smaller previews, then a 2×
supersampled image after a short idle interval. The display camera travels with
the image. Pointer edits wait for the current frame, preventing edits through a
stale camera. Render jobs from an older edit or replaced document are discarded.
Development profiles optimize the CPU renderer and core to keep iteration usable.

The app takes the system appearance at startup; the toolbar toggles it for the
current session. `--theme dark|light` makes captures reproducible.
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

This stage implements the requested native shell, not every later acceptance
row in `PLAN.md`. Runtime plugins, live MCP session attachment, collaborative
comments, palette authoring/import UI, persistent appearance preferences,
recovery journals, GLB export UI and screen-reader certification remain future
integration work. The underlying portable data is retained on save.
