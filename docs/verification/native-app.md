# Native GPUI stage verification

The latest [app polish round 1](../reviews/app-polish-1.md) records visual scores,
typography and layout refinements, and fresh native captures in both themes.

The subsequent [app review round 1](../reviews/app-review-1.md) records the latest
fixes, **96 passing workspace tests**, minimum-window and high-density captures,
and native dialog regression checks. The stage evidence below is historical.

Verified 2026-10-25 in the Linux build worktree. Scope: `voxelite-app`, binary
`voxelite`. No plugin or web application was built, and `design/` was unchanged.

## Build and automated checks

- Rust 1.99.1 stable, edition 2024.
- `gpui-unofficial = 1.25.3`, `gpui-platform-gpui-unofficial = 1.25.3`; complete
  registry resolutions and checksums are recorded in `Cargo.lock`.
- `cargo build -p voxelite-app --bin voxelite`: passes on Linux with native GPUI.
- `cargo fmt --all -- --check`: passes.
- `cargo clippy --workspace --all-targets -- -D warnings`: passes.
- `cargo test --workspace`: **90 passed, 0 failed**, including 16 editor tests.
- `cargo test -p voxelite-app --no-default-features`: **16 passed**, with GPUI
  excluded. Camera/raycast, tools, mirror, locks/visibility, palette capacity,
  atomic failure, undo/redo, file round trips and forward metadata are covered.

## Native smoke checks

Actual GPUI windows ran under Xvfb at 1440×900 using Mesa's llvmpipe/lavapipe
Vulkan adapter. The image is a CPU render of the embedded 27,897-voxel Autumn
Workshop, composed by GPUI; the chrome is native Rust, not an HTML capture.

Checked both explicit themes, palette previews, command-search text isolation,
search/arrow/Enter operation, in-window menus, tool selection, hover picking,
pointer placement, keyboard placement, undo, native GTK Open and Save As via
GPUI/XDG portal, subsequent saves to that path, and the CLI path argument.
Reopening the original diorama through the native Open dialog restored the
27,897-voxel document and reset transient history. A saved
keyboard/pointer edit produced 27,898 voxels; undo and save restored 27,897.
Saving uses the canonical format and retains project identity and metadata.

Repeated 1024×640 → 1440×900 resizes, orbit drags and hover changes were inspected.
The sidebar scrolls at the minimum window size. Pixel comparison confirmed
unchanged menu text across the orbit stress captures. Replaced viewport images
are explicitly removed from GPUI's texture atlas. Grayscale font antialiasing
keeps the bundled Geist fonts consistent on software rendering.

Navigation has coalesced lower-resolution previews and an antialiased settled
frame. A 1024×900 CPU-only PNG took approximately 2.3 seconds on this runner in
the development profile with optimized core/renderer; this is one observation,
not a performance guarantee. No GPU or display is initialized by `--viewport-png`.

## Artifacts and reproduction

- [Dark native window](../../screenshots/app-dark.png)
- [Light native window](../../screenshots/app-light.png)
- [Display-free viewport](../../screenshots/viewport.png)
- [Capture script](../../scripts/capture-native.sh)
- [Build dependencies, controls and stage boundaries](../../crates/voxelite-app/README.md)

The screenshots are 1440×900 and were visually inspected after regeneration.
The viewport PNG is 1024×900. Dark/light surface samples are exact `#000000` /
`#ffffff`. The script creates a private Xvfb display and session D-Bus, uses the
Mesa software driver, captures actual windows with ImageMagick, and tears down
its processes. Set `RUST_LOG` for GPUI/WGPU diagnostics if needed.

This is Linux native-build and headless-runtime evidence. macOS hardware,
Wayland runtime, assistive technology and the future session/plugin integrations
have not been certified by these checks.
