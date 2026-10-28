# Headless UI golden snapshots

`cargo xtask ui-snapshots` paints real Quantum views through GPUI 1.25.3's
`HeadlessAppContext`, `Window::render_to_image`, and `WgpuHeadlessRenderer` on
Linux Mesa lavapipe. It removes `DISPLAY` and `WAYLAND_DISPLAY` from each capture
process. There is no window server, Xvfb, screen scraping, or layout-only substitute.

## Run and review

Install the Linux native build dependencies from the
[native guide](verification/native-app.md) plus `mesa-vulkan-drivers`. The harness checks for
`/usr/share/vulkan/icd.d/lvp_icd.json`. The Ubuntu workflow lists the exact packages.

```sh
cargo xtask ui-snapshots
cargo xtask ui-snapshots --screen 'settings-*'
cargo xtask ui-snapshots --rebaseline
cargo xtask ui-snapshots --rebaseline --screen 'panels-palette-*'
```

Selectors use the PR screenshots convention: exact names or one trailing `*`,
repeatable, deduplicated in registry order. Unknown or unmatched selectors fail.
No selector means all 26 key screens. Only explicit `--rebaseline` writes
`crates/voxelite-app/tests/ui-goldens/*.png`, after all selected captures have
succeeded and their set and dimensions have been validated. Review every changed
PNG in both themes before committing it. Ordinary runs never write references.

The comparison requires exact dimensions. A pixel counts as changed if **any**
RGBA channel differs by more than **5/255**. At most **1/2000 (0.05%)** of the
pixels may change; the integer comparison does not round up a small-image budget.
This tolerates small rasterization noise, not layout shifts or changed surfaces.
Missing references, unknown files, malformed PNGs, missing renders, and renderer
errors fail. Failed comparisons write `actual/`, `expected/`, and magenta-on-black
`diff/` PNGs beneath `target/ui-snapshots/`. A missing reference has only an actual
image; a failed renderer cannot supply pixels. CI uploads available evidence.

## Coverage and determinism

The harness shares the [screen registry](../xtask/src/screenshots.rs), bundled
Autumn Workshop and [recent-project fixtures](../xtask/src/screenshot_fixtures.rs).
It covers empty Welcome and Welcome with five recents, the editor, focused palette (including all four layers), empty resolved comments, About, and all seven
Settings pages. Every view has light and dark references. `welcome` is the
existing registry name for dark empty Welcome; `welcome-light` is its counterpart.

Logical sizes are 760×620 for Welcome, 1440×900 for editor/panels/About, and
880×640 for Settings. GPUI TestPlatform uses a fixed **2×** scale; PNGs contain
the resulting physical pixels without resizing. Bundled Geist/Geist Mono fonts
and grayscale text rendering are used. Each capture has a fresh app data folder,
UTC timezone, fixed fixture clock, stable displayed home labels, deterministic
GPUI dispatcher seed, reduced motion, and a fixed completed elapsed time for
image/text reveals (reduced motion alone still fades). Real CPU rendering and asset loads
must settle before capture; recents never come from the user's settings. The
`ui-snapshots` build displays version `0.0.0` with no build id, so release
version bumps do not invalidate goldens.

The editor uses its existing production Linux CPU viewport renderer and paints
that image through GPUI with the normal editor chrome and overlays. This covers
UI composition and the CPU image path. It does **not** verify the macOS Metal
renderer, GPU surface synchronization, AppKit chrome, native file dialogs, or
Core Text. The independent rendering harness continues to own Metal correctness
and performance. Native macOS UI goldens (closed draft #194) are parked as an
optional later addition, not required for this gate.

The `ui-snapshots` app feature enables GPUI test support only when explicitly
requested. Production builds do not enable it. No plugin or shared UI crate gains
an app/domain dependency, no unsafe code is added, and the vendored Apple patch
is unchanged. A feasibility spike on 2026-10-28 rendered the real Welcome view,
including fonts, borders, icons and sample images, with both display variables
absent. The pinned versions needed no dependency upgrades.

## CI and design review

The [UI snapshots workflow](../.github/workflows/ui-snapshots.yml) filters UI,
renderer, fixture, harness, dependency and golden changes. Its stable final job
succeeds on non-UI PRs without building the renderer; detection/render failures
fail the final job. It runs unprivileged on Ubuntu, including fork PRs, and never
rebaselines. Wall time is measured from the `snapshots` job's start/end timestamps
and recorded in the implementation PR, including installation/build overhead.

Follow the [screenshot skill](../.agents/skills/ui-pr-screenshots/SKILL.md) for
Before | After PR evidence. Review both themes, token spacing/radii, focus, hover,
pressed, empty/loading states, and long-label truncation. Default-state goldens
complement those checks; they do not prove every interaction state.
