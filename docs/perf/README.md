Camera and placement benchmark
==============================

Build the **real shared GPUI editor** with opt-in instrumentation, then run:

```sh
VOXELITE_PERF=1 bash scripts/build-web.sh
node scripts/bench-camera.mjs --out docs/perf/camera-before.json --runs 3
```

The script starts its own loopback server for `web/dist`, uses installed Playwright,
1440×900, DPR 1, Chromium's SwiftShader WebGL2 backend, and saves JSON, Markdown,
initial screenshots, and raw traces. `--backend webgpu` selects SwiftShader Vulkan
and WebGPU. These are **software GPU results**, not measurements of a hardware GPU.
Use the same backend, machine and browser for comparisons. `--duration` defaults
to 6000 ms **per gesture**, `--runs` to three. No deploy or site files are touched.

Typical is the bundled Autumn Workshop (27,897 layer voxels). Large repeats it
four times at offsets ±40 in X/Z (111,588 layer voxels), with valid 160×64×160
bounds. The generator is in the script and scene content hashes are recorded.

Orbit, middle-button pan, wheel zoom and hover-only preview use real CDP events
scheduled at 60 Hz independently of the page thread. Each phase reports actual
wall duration and queued-event drain beyond the fixed sending interval. Metrics
count **unique viewport images**, not UI-only repaint or `requestAnimationFrame`
callbacks. RAF frequency is separate. Span times are inclusive, so do not sum
nested spans. Instrumentation is compiled out without Cargo's `perf` feature.
JS↔wasm boundary overhead is not isolated: it is explicitly null, not zero.

Presentation is a proxy: the first RAF following a viewport paint submission.
It does not measure physical display photons or GPU timestamps. Input latency
includes event-to-next-image delay; it does not prove an intermediate coalesced
camera state was displayed. Dispatch delay uses browser event timestamps.

Ten isolated clicks wait for a completed render before the next click. Fifty
repeated placements are offered at 30 Hz. Counts of attempted, accepted and
visible edits accompany latency; a renderer which rejects clicks cannot claim
success from the latency of its few accepted edits. The original editor does
not place continuously on held-left drag; this benchmark preserves that behavior
and exercises repeated clicks along a grid. Raw traces allow alternate analysis.

Per-scene `median` takes the coordinate-wise median across three runs; all runs
are retained. Empty distributions are null. CPU times and frame pacing depend on
host scheduling; compare distributions and accepted edits as well as averages.

The new viewport is deliberately **opt-in** while the compositor transfer is
unfinished. It always uses device-resolution 4× MSAA, baked vertex AO and a 2048²
shadow map. Its outline shader is available to renderer callers; outlines remain
off in the editor, matching the CPU renderer's existing default. No moving/idle
quality transition exists in this GPU path. The legacy CPU viewport remains the
ordinary default to avoid shipping the measured software-GPU regression.

```sh
node scripts/bench-camera.mjs --query viewport=gpu --out docs/perf/camera-after.json --runs 3
node scripts/bench-camera.mjs --out docs/perf/camera-default-after.json --runs 3
node scripts/check-gpu-viewport.mjs
VOXELITE_GPU_VIEWPORT=1 cargo run -p voxelite-app --bin voxelite
```

The browser checker tests WebGL2 and WebGPU, requires actual GPU initialization,
compares viewport pixels and saved voxel data across place/undo/redo, checks that
camera changes perform no mesh rebuild or vertex upload, and exercises resize.
The benchmark rejects silent CPU fallback when `--query viewport=gpu` is used.
Standalone-click settling includes the successful edit's revision, preventing a
queued render from being mistaken for an already completed frame.
