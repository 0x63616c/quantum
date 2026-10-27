# Rendering harness

Two independent tools exercise the **real `voxelite_render::gpu::Viewport`**:
`cargo xtask bench` measures performance; `cargo xtask visual` compares pixels.
A failure in one does not require running the other. Neither invokes the CPU
rasterizer. They share scene generators and camera paths.

## Quality and scope

Both use the production pipelines, greedy meshes, baked AO, exact occupancy
shadows, **4x MSAA**, and `surface_size`'s **2x supersampling**. There are no
render-scale, AA, AO or shadow-quality switches. The existing production fallback
is reported as `shadow_path: "fallback"` if occupancy allocation/upload fails;
it is never forced by the harness. The `terrain10m` name alone does not determine
the shadow path. Exact and fallback baselines cannot be compared.

Captures contain tightly packed RGBA8 **sRGB-encoded** bytes from the final
`Rgba8Unorm` resolve texture. The shader already encodes sRGB; the PNG writer does
not apply another transfer function. This is the texture the production Mac
worker passes to NV12 conversion. Headless measurements stop at GPU completion
of the viewport; they exclude NV12 encoding, IOSurface acquisition, GPUI
presentation and display latency. Do not compare their timings directly to the
old Worker/NV12 profiling JSON.

The fixed style, `app-dark-grid-v1`, is the app's dark appearance: black opaque
background, `RenderOptions::default()` outlines off, default ground grid enabled.
It mirrors `ui/viewport/gpu_viewport.rs`, `Theme::Dark::line()` (`#232323`), and
the built-in gizmo-grid overlay: bounds minimum Y minus 0.05, X/Z extent plus
16 cells, spacing 4, opacity 0.24. The system appearance preference is deliberately
resolved to dark for repeatable output. Update this documented contract if the
app defaults change; do not weaken the renderer to improve a benchmark.

`Headless` and `Worker` call the same snapshot-sync helper. Camera-only frames
skip occupancy/mesh synchronization. Production workers collect no stats and
request no timestamp features. Only captures add `COPY_SRC` to the output
texture. Profiling requests `TIMESTAMP_QUERY` when the adapter supports it;
unsupported timestamps are absent metrics, never fabricated zeros. Query slots
cover fallback shadow redraw, MSAA and resolve. GPU total is a first-to-last
span, not the sum of possibly overlapping pass durations.

## Running locally

From the repository root:

```sh
cargo xtask help
cargo xtask visual --determinism
cargo xtask bench --scene autumn --runs 3 --out target/bench/after.json --markdown target/bench/after.md
cargo xtask bench --scene autumn --runs 3 --compare
cargo xtask bench --input target/bench/after.json --compare path/to/before.json --markdown target/bench/comparison.md
```

`--scene` can repeat. Defaults are `autumn`, `terrain1m`, `sponge1m`.
`xtask` always runs the harness in release with `--locked`, forwards flags without
shell evaluation, and propagates its exit status. Exit 1 means a comparison
failed; exit 2 means invalid arguments, incompatible data or an operational error.

Bench defaults to a **2064x1616 physical display viewport**, yielding 4128x3232
surface pixels. This matches the original profiling window's 1032x808 logical
viewport at Retina density. `--width` and `--height` select a viewport size,
preserving 2x supersampling and all rendering settings. Compare only identical
dimensions and camera sampling. Visual always uses display 640x400, surface
1280x800.

On an unpinned machine, ordinary visual comparison refuses to proceed:

```sh
# First invocation creates an explicitly local set; the second compares it.
cargo xtask visual --scene autumn --local-baseline
cargo xtask visual --scene autumn --local-baseline --determinism
# Explicitly replace an existing local set, after reviewing why it changed.
cargo xtask visual --scene autumn --local-baseline --rebaseline
```

Local references live in `target/golden/<machine_key>/`. Missing files in an
existing local set fail, including scenes not captured when the set was created;
use explicit rebaseline to add them. `--golden-set apple-m2-metal` opts into
cross-machine diagnosis against that set. It does not make an M5 Pro or Vulkan
result equivalent to pinned M2 evidence.

Linux uses the additive `headless-vulkan` feature, enabled only by the bench
crate on non-macOS. With lavapipe installed:

```sh
export CARGO_TARGET_DIR=/home/box/.cache/cargo-shared
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json
export XDG_RUNTIME_DIR=/tmp
cargo xtask visual --scene autumn --local-baseline
cargo xtask bench --scene autumn --runs 1 --frames 10
cargo test -p voxelite-bench --release --locked --test headless -- --ignored --nocapture
```

The shared target export is required in this Linux development workspace. The
repository's `target/` contains reports, not an additional Cargo build directory.
Software Vulkan smoke tests prove execution and comparison plumbing; their
pixels and performance are not Metal baselines.

## Scenes, paths and edit phases

`autumn` embeds `bench/scenes/autumn-workshop.voxl`, a pinned copy of the demo diorama so product edits to `examples/` cannot silently change what is measured (changing it changes the scene hash and invalidates goldens and baselines). `terrain1m` and `sponge1m` use
fixed seeds, integer hash noise and fixed-order f64 FBM. Noise is quantized to
24 bits before integer height searches or rank thresholds decide occupancy;
sponge rank ties use cell index. Scene hashes detect any cross-platform or
algorithm drift rather than comparing different inputs. No parallel floating
point reductions or fast-math assumptions are used.

Hashes are versioned FNV-1a-64 over streamed serde JSON of the full snapshot,
including bounds, palette, layer order and visibility. Voxel serialization sorts
coordinates. This is an input-change fingerprint, not a security signature.
Autumn's hash is covered by a fixed-value unit test. Changed hashes invalidate
both performance and screenshot comparison with instructions to restore the
scene or explicitly regenerate references.

`terrain10m` is opt-in locally; the Rendering checks workflow selects all four scenes unless its `scenes` input says otherwise:

```sh
cargo xtask bench --scene terrain10m
cargo xtask visual --scene terrain10m
```

`xtask` enables `voxelite-bench/large-scenes`, which enables the bench-only
`voxelite-core/unbounded-voxels` feature. That feature raises admission to
16,777,216 occupied cells, retaining coordinate/chunk/work constraints. It is off
by default; `cargo test --workspace --locked` retains the production 1,048,576
limit. Never enable it in application builds or use an all-features application
build as production. The scene is render-only test data, not a new supported
application document size.

Cameras use `Camera::fit` with the Iso preset. `close` is 4x zoom toward its
target. Orbits rotate 3 degrees per frame. Bench warms each path with 8 frames,
then restarts at angle zero for 90 measured frames (`--warmup`, `--frames`).
Phases are `cold`, `fit` (static), `orbit_fit`, `orbit_close`, `edit_1voxel`,
`edit_cube16`, `edit_cube40`, `edit_slab64`, and `edit_recolor`.

Device initialization is measured separately. Each cold run drops scene caches,
GPU scene buffers and targets, retaining device/pipelines. Scene generation,
hashing and camera fitting are outside frame timing. The cold snapshot clone is
also outside timing; the cold metric is occupancy + mesh + uploads + first frame.

Region edits **erase then restore original cells**, so near-limit 1M scenes do
not exceed production admission and do not lose content between phases. Regions
are clipped to document extents and selected around a deterministic occupied
cell. Recolor changes a used palette swatch and restores its actual original
color. Every command goes through `Engine::execute`; no-ops and failed restores
are fatal. There are 24 alternating apply/revert iterations, first 4 discarded.
`terrain10m` runs only 1-voxel and recolor edits, capped at 10 iterations including
4 warmups. Engine execution, snapshot clone, rendering and edit-to-pixels are
reported separately. Diagnostic timestamp mapping and RSS collection are outside
the reported frame/edit-to-pixels wall interval.

Fixed visual stills:

| Scene | Stills |
|---|---|
| autumn | fit, close, orbit90, orbit180 |
| terrain1m | fit, close |
| sponge1m | fit, close |
| terrain10m (opt-in) | fit |

`--determinism` renders each still twice with freshly initialized Headless
instances and requires byte equality, independent of comparison tolerances.
Nondeterminism is a hard failure, writes diagnostic images and prevents that
still from being rebaselined.

## Reading performance output

JSON has `schema_version: 1`, `meta`, and a sorted `results` map keyed by
`scene/phase`. Struct fields have stable serialization order. Meta includes git
commit (`GITHUB_SHA` or Git), dirty flag, UTC date, OS, hardware model/chip,
adapter/backend/driver, machine key, display/surface dimensions, fixed quality and
style, path version, frames/warmup/runs, device initialization, load averages at
invocation and run boundaries, scene hashes/counts and actual shadow paths.

Each phase has `samples_per_run`, `fps_p50`, and a sorted `metrics` map. Each
metric contains `p50`, `p95`, `p99`, `max`, `mean`. Each of those is a spread:
`{median, min, max, runs: [...]}` across repetitions in **one process**. Runs are
not pooled before percentiles. FPS is 1000 divided by each run's wall p50, then
aggregated. Percentiles interpolate at `(count - 1) * p`; one sample returns
itself. Empty or nonfinite samples are rejected.

Metrics include wall and submit-to-completion time; occupancy/mesh sync, uploads
and encoding; optional GPU main/resolve/shadow/total; main draw calls, extra
shadow draws, vertices, triangles, changed chunks, total upload bytes; resident
vertex capacity and occupancy; estimated target bytes; optional Metal device
allocation; phase-end process RSS MB. Upload bytes include occupancy, vertex and
uniform writes. The target estimate uses 80 bytes per surface pixel, plus a
fallback shadow texture when present. It excludes readback, NV12, driver
alignment and CPU memory. Metal's device counter and RSS have broader scopes;
do not add them together. Depth24Plus storage is estimated as 32 bits per sample.

Compare refuses mismatched machine keys, scene hashes, shadow paths, dimensions,
quality/style/path versions, frame counts or warmups, and missing phases or
previously available metrics. A subset of baseline scenes may be selected.
Default gates are:

| Metric | Regression condition |
|---|---|
| Wall p50 and edit-to-pixels p50 | increase >10% **and** >0.3 ms |
| GPU main/resolve/total p50 | increase >10% |
| Fit draw calls, triangles, upload bytes | any increase |
| Vertex, occupancy, target and device allocated bytes | increase >10% |
| Process RSS MB | increase >50% **and** >128 MB |

The wall absolute threshold avoids failing on trivial scheduling noise. RSS is
the current process resident memory sampled at each phase boundary, not a peak;
allocator history and OS memory compression can cause large swings between
phases and runs. Its separate gate fails only when new RSS exceeds both
`old + rss_mb` and `old * (1 + rss_pct / 100)`. GPU timing and memory byte gates
remain relative; deterministic fit geometry/uploads have no noise allowance.
Flags `--wall-pct`, `--wall-ms`, `--gpu-pct`, `--memory-pct`, `--rss-pct` (default
50) and `--rss-mb` (default 128) override thresholds explicitly. Improvements
are reported, never failed.

The M2 is a shared CI runner: load averages can be 10–40 or higher. Record and
inspect load and the complete per-run spread. Repeated medians do not make a
busy runner a laboratory. Manual performance comparisons are diagnostic in CI;
visual checks remain mandatory. Device initialization, cold resource retirement,
CPU caches, allocator history and driver caches can influence successive runs.

## Visual tolerances and artifacts

For each pixel, use its largest absolute RGBA channel delta. A pixel differs
when that delta is **greater than 2/255**. Default failure is differing pixels
**greater than 0.01% OR any pixel delta greater than 24**. The peak gate catches
a substantial change confined to one small visible voxel. These allowances cover
small driver arithmetic differences; they do not authorize reduced quality.
Phase-2 renderer changes should normally be byte-identical on a fixed device.

Reports include exact-match, mean absolute RGBA delta, peak delta, differing
pixel count/percentage, and PSNR over all four channels. Infinite PSNR is JSON
`null` with `exact_match: true`. Every mismatch writes actual, heatmap, and
`golden | actual | heatmap` PNGs in `target/visual/` (or `--out DIR`), plus
`report.json` and `report.md`. The heatmap preserves zero-delta pixels; changed
pixels use dimmed grayscale from the actual image with red amplified 8x.
Missing or wrong-size goldens can only produce an actual image and an error.

## Mini workflow and intentional rebaselines

The **Rendering checks** workflow uses the existing
`[self-hosted, macOS, ARM64, xcode]` runner labels, owner-only guard, no fork PRs,
pinned action SHAs, read-only repository permissions and per-ref concurrency.
Rendering-related PRs run `cargo xtask visual --determinism`. Manual dispatch
accepts a command, repetitions, and space-separated scenes:

- `visual`: screenshot comparison plus fresh-device determinism.
- `bench`: `--compare`, with JSON and markdown artifacts; diagnostic failure.
- `rebaseline-visual`: explicit `--rebaseline --determinism`; uploads goldens
  and reports without pushing.
- `bench-baseline`: explicit `--update-baseline`; uploads baseline JSON without
  pushing.

Visual goldens are owned by `apple-m2-metal`, as read from
`bench/pinned-machine.toml`. `visual --rebaseline [--scene S]` refuses on other
machines unless `--force` is explicit, and writes only that machine's own set.
It prints changed PNG/metadata paths and per-still max deltas. A selective
rebaseline updates every manifest still for the selected scene. Review **all
changed side-by-side images** before accepting new goldens; an intentional
appearance change needs an explanation, not just a passing rebaseline command.

Download workflow artifacts and add reviewed files under
`bench/golden/apple-m2-metal/` (both PNGs and their JSON sidecars) or
`bench/baseline/apple-m2-metal.json` in a follow-up commit. No generated reference
files are included in the initial harness change. Missing goldens deliberately
fail until that follow-up lands. `--update-baseline` never runs implicitly and
cannot be combined with `--compare` or `--input`.

## Phase-2 PR evidence

Record before and after with the same scene hashes, viewport dimensions, path
sampling, machine and driver. Use multiple runs and attach JSON. Include the
comparison markdown rows (wall/GPU/geometry/upload/memory changes), load and
spread in the PR or performance Page. Run visual comparison and determinism at
the **default tight tolerance** on the pinned mini. Attach side-by-side artifacts
for every investigated difference. A speedup with different pixels, lowered
quality or a changed shadow path is not an accepted renderer optimization.

## Verification notes

The default workspace checks require no GPU. Pure tests cover percentile edges,
serialization, compatibility and regression gates, diff thresholds, PSNR,
heatmaps, scene hash stability, reversible edits, target estimates, timestamp
span arithmetic and row unpadding. Adapter tests are explicitly `#[ignore]`.

The ignored `single_voxel_recolor_exceeds_default_visual_tolerance` test selects
a visible autumn voxel by an offset-center-ray/box intersection, changes exactly one
voxel's palette reference through the engine, renders both versions on the GPU,
and requires the default visual gate to fail. It writes its metrics and four
PNGs to `target/visual/recolor-proof/`. It does not change the fixture or goldens.

Linux/lavapipe validation on this implementation: all four autumn stills were
byte-identical on the second local-baseline invocation and with fresh-device
`--determinism`. The 10-frame autumn benchmark completed all nine phases and
passed comparison against itself. The one-voxel recolor at `[5, 29, 7]` changed
105 pixels (0.010254%), peak channel delta 183, mean absolute delta 0.007535,
PSNR 48.672 dB: **FAIL at default tolerance**, as required. An initial exact
center-ray selection grazed an occluded voxel; the proof now uses a fixed small
ray offset and rejects zero-length intersections. Using the GPU-rendered recolor
as a temporary local golden also made the actual visual CLI return exit 1 and
write mismatch artifacts; the original local golden was restored afterward.
The fixture stayed unchanged. Two-run aggregation was exercised in one process,
and the opt-in 10M visual scene rendered successfully with `shadow_path: exact`
on lavapipe. That does not imply a fallback or exact outcome on another adapter.

Metal device allocation, autorelease behavior, driver timestamps and pinned
pixel determinism still require validation on the mini. Linux checks cannot
compile the existing macOS interop module without an Apple SDK. All Metal/unsafe
interop stays in that existing module; the harness itself forbids unsafe code.
