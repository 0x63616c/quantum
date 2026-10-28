# macOS surface viewport

## What changed

The macOS editor defaults to a wgpu Metal viewport presented with GPUI's
`Window::paint_surface`. `VOXELITE_CPU_VIEWPORT=1` forces the software viewport.
The software rasterizer remains the export/thumbnail renderer and the automatic
fallback. Its existing interactive resolution/refinement behavior is retained;
GPU supersampling adapts to surface limits while MSAA, AO and shadows are retained.

A dedicated owner thread performs adapter initialization, chunk meshing/AO,
resource allocation, encoding, completion polling and GPU resource teardown.
Camera-only requests reuse chunk geometry. The editor allows one outstanding
request, coalesces subsequent input, and installs only frames matching the
current document generation, revision, preview and dimensions. Picking also
requires the frame identity most recently painted by GPUI. Preview boxes,
selection feedback, comments and the orientation gizmo remain GPUI overlays.

The presentation sequence is:

1. Multiply the laid-out viewport bounds by the window scale factor to obtain
   physical display pixels. Prefer twice those dimensions, then 1.5x or 1x if
   needed to fit the surface budget and device texture limit. Round up to even
   dimensions for NV12. Retain 4x MSAA, ambient occlusion, exact voxel shadows
   and an opaque background/grid at every size.
   Outlines follow the CPU viewport default (disabled).
2. Resolve linear shading to nonlinear RGB in an RGBA8Unorm target. Convert it
   with the exact inverse of GPUI 1.25.3's full-range BT.601 matrix, including
   its four-decimal coefficients and 0.5 chroma center.
3. Write full-size luma and half-width/half-height interleaved CbCr directly into
   the IOSurface planes of a `420YpCbCr8BiPlanarFullRange` CVPixelBuffer. Chroma
   averages each 2x2 RGB block, retaining one chroma sample per display pixel at
   2x supersampling, with fewer samples at the reduced factors.
4. Wait for wgpu's submitted-work completion callback on the worker before
   transferring the completed buffer to GPUI. No production pixel readback or
   CPU upload occurs in this handoff.

The CoreVideo pool implements a bounded three-buffer ring: its minimum buffer
count and allocation threshold are both three. CoreVideo decides reuse from
ownership; the implementation never samples retain counts or blindly rotates
slots. Resizing replaces the pool while existing frames keep old allocations
alive until their consumers release them.

Directional shadows now trace the visible composite with the shared `occupancy.rs`
f32 DDA and its WGSL mirror. Each 8³ brick occupies 16 contiguous u32 words; a
coarse bitset skips empty bricks. The maximum 256³ storage buffer is 2,101,296
bytes including its 48-byte header. Rays start at the surface plus 0.001 voxel
along its axis normal, in grid-local coordinates to preserve the epsilon near
±32768. Tied crossings visit X, Y, then Z. Shadow visibility is binary; the
existing illumination, AO, and MSAA remain unchanged.

Both GPU and software viewports retain occupancy across camera frames. Immutable
chunk differences and visible-layer membership identify changed cells; only dirty
brick word ranges and coarse mask words are uploaded on scene changes. A single
changed brick needs 64 bytes plus its 4-byte mask word. Hidden layers never cast;
layer order and palette colors cannot change boolean occupancy. Stateless export
renders build occupancy once per call.

The former 4096² depth map, fit, comparison sampler, and PCF shader are isolated
in `ShadowFallback` / `gpu/shadow_map.wgsl`. They are allocated only if the
occupancy buffer exceeds device limits or a scoped allocation/upload fails. The
fallback persists for that viewport's lifetime; normal frames perform no shadow
depth pass or fit. Device loss still selects the existing software fallback.

## Ambient occlusion

Settings → Viewport / Rendering exposes an AO toggle (on by default) and a
strength slider from 0 to 2, with 1 preserving the original look. Both are saved
in app-local `settings.json`. Changes apply to every open viewport, including
the CPU fallback; frames rendered at an older strength are rejected.

The GPU keeps the original per-vertex AO in chunk meshes and applies strength
through a per-frame uniform in both shadow shaders. Changing strength does not
rebuild geometry or occupancy. At strength 1 the interpolated AO value passes
through unchanged. The CPU keeps its per-pixel voxel-neighbor samples and scales
their darkening. Strength 0 disables only AO, leaving directional shadows intact.
Exports and thumbnails use renderer defaults independently of viewport settings;
MCP `render` and `export_png` accept an explicit `ambient_occlusion_strength`.
See [CPU before/after comparisons](../ambient-occlusion.md).

## FPS integration and face banding regression

The rebase retains main's native UI, gizmo, preferences, status bar and single-line
tooltips. Theme/grid preference changes request new GPU content. The FPS meter
records visible content at the `paint_surface`/`paint_image` handoff, with a new
serial per installed GPU frame. Repainting that surface does not count again;
neither pooled pixel-buffer addresses nor scene identities identify new content.
GPU work uses the same idle-window reset as CPU work. Fallback immediately resets
the meter to CPU, and deferred GPU paint callbacks cannot switch it back. GPU
navigation still bypasses the CPU half-resolution render and 180 ms refinement.

The earlier shadow-map banding investigation (now fallback-only) reproduced self-shadowing on an isolated solid with
AO and outlines disabled. The fitted map's 3×3 PCF compared all taps against the
center receiver depth. Caster slope bias and a normal offset did not cover the
full diagonal filter footprint, especially near grazing incidence and face edges.
The fallback shader derives the receiver plane's depth gradient from the orthographic
light projection, corrects each tap's depth, and biases for the four internal
bilinear samples. That bias scales with the fitted texel size. The existing
caster slope bias and light-space normal offset remain in use. No face noise,
texture sampling or mesh change was required. GPU outlines were also enabled
unconditionally even though the CPU viewport disables them; both now use the
CPU default, removing the extra dark crease lines.

An isolated Linux Vulkan/llvmpipe harness exercised the production mesh, fitted
shadow math, WGSL and render passes, replacing only the Metal adapter/presenter
with offscreen readback. A solid box `[-8,0,-8]..[8,16,8]`, orthographic height 24,
pitch 25°, distance 50 and yaw 55°..85° in 5° steps produced respectively
4, 5, 1183, 307, 229, 163 and 634 changed pixels when shadows were enabled before
the fix (256×256 output). After the fix, all seven images exactly match their
unshadowed controls. The actual WGSL gradient and comparison-depth expressions
also have host regressions against independent ray/plane intersections and
bilinear sample depths across zoom, aspect, slope sign and subtexel phase.
A close-up of the bundled workshop still has real cast shadows: enabling them
changes 247632 of 921600 pixels, with a maximum RGB-channel delta of 29. These
checks exercise shading; they do not validate Metal or GPUI presentation.

## Review findings and fixes

- **Actual supersampling:** the earlier implementation only doubled the NV12
  planes and repeated luma texels. All scene/MSAA/resolve targets now render at
  the selected 2x, 1.5x or 1x device dimensions. Admission handles zero, overflow,
  device texture limits and the allocation budget before allocating targets.
  The display aspect ratio is preserved even when NV12 requires even rounding.
- **Color and layout:** the converter matches the pinned Metal shader rather
  than using BT.709 or video-range YCbCr. Tests evaluate the actual WGSL expression
  and Metal matrix, including 8-bit quantization. CoreVideo allocation checks
  IOSurface backing, full-range format, plane count/dimensions and minimum row
  strides. Y imports as R8Unorm; CbCr imports as RG8Unorm. The actual Metal plane
  dimensions, formats, sample/mip counts and usage flags are checked before HAL
  import. CoreVideo handles row padding; no tightly packed stride is assumed.
- **Device sharing:** the GPUI fork caches its chosen device and exposes its
  registry ID. wgpu selects that physical GPU, and the producer's texture cache
  is created from wgpu's actual MTLDevice. Failure to match causes CPU fallback.
- **Both fences matter:** producer retirement callbacks retain each plane's
  CVMetalTexture and CVPixelBuffer. GPUI retains its own wrappers and pixel
  buffers through consumer completion, including when scenes are replaced.
  Failed encodings release unsubmitted leases. Cache flushing also runs after
  consumer completion, so unused cache entries cannot indefinitely pin the ring.
- **Thread transfer:** only a private, non-cloneable completed-buffer token is
  marked Send. The public Frame stays thread-bound and is created on receipt.
  The worker never mutates a handed-off allocation; private retirement leases
  only retain/release it. Worker autorelease pools cover Metal work and teardown.
- **Fallback:** adapter/init/allocation errors, validation/device loss, producer
  completion/pool starvation and GPUI surface import failures select CPU rendering.
  Completion/reuse polling has a five-second deadline on the worker. Idle health
  monitoring catches errors without requiring another input event. Fallback
  clears the GPU frame and recomputes CPU dimensions, and an outstanding GPU
  response cannot re-enable a failed viewport. Pending CPU work is also scheduled
  when its computed dimensions have not changed.
- **Resize and HiDPI:** dimensions use actual layout bounds and the current
  backing scale, with at least one physical pixel per axis. Layout changes
  request another frame; obsolete dimensions are rejected on completion.
  Compact UI layout is based on logical width, independent of Retina resolution.

The only vendored upstream source change is GPUI Apple's `metal_renderer.rs`;
see [vendor notes](../../vendor/README.md). Apple requires retaining the
[CoreVideo texture wrappers through GPU execution](https://developer.apple.com/documentation/corevideo/cvmetaltexturecachecreatetexturefromimage(_:_:_:_:_:_:_:_:_:)),
which is why retaining only the underlying Metal textures is insufficient.

## Verification on the Linux host

After the rebase and FPS/banding fixes (2026-10-26):

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --locked -- -D warnings`: passed.
- `cargo test --workspace --locked`: passed, including GPU/CPU meter transitions,
  duplicate presentation handling, actual WGSL math and shadow filter guards.
- `cargo clippy -p voxelite-render --features gpu --target aarch64-apple-darwin --tests --locked -- -D warnings`:
  passed. The macOS UI paint/fallback code was also manually checked for cfg,
  ownership and Rust 2024 Clippy issues; no full macOS UI/runtime pass is claimed.

Earlier branch verification and platform constraints:

- `cargo fmt --all`: passed; `cargo fmt --all -- --check`: passed.
- `cargo test`: passed, including incremental chunk invalidation, negative chunk
  seams, AO neighbors, visibility/palette changes, shader validation, WGSL/Metal
  matrix round-trips and supersampled dimension/admission tests.
- `cargo build`: passed for the native Linux workspace.
- `cargo check -p voxelite-render --features gpu --target aarch64-apple-darwin --tests`:
  passed, including the Metal interop and opt-in hardware test code.
- `cargo check --target aarch64-apple-darwin`: **blocked on the clean GPUI build**:
  `include_bytes!` cannot find `shaders.metallib`. Upstream's build script is
  compiled for the host and only generates Metal shaders on an Apple host.
  An earlier apparent success used a zero-byte shader artifact inherited from
  the previous session. That artifact was removed and the check repeated; this
  is not a successful complete macOS build. Rust checking with that artifact
  had reached the app and GPUI patch, but did not validate Metal shaders/linking.
- Workflow configuration targeted `[self-hosted, homelab-voxelite]` at the time of this note (the workflows now use `[self-hosted, macOS, ARM64, xcode]`) on pushes to
  `main` and `perf/**`, plus manual dispatch. It runs `scripts/package-macos.sh`
  and uploads `dist/Voxelite.dmg` as the `Voxelite.dmg` artifact. No release or
  push was performed from this worktree; the workflow itself was not executed.

## Untested and remaining risks

No physical Mac or hardware GPU was available; the banding experiment used
software Vulkan (llvmpipe). CoreVideo allocation/import, actual Metal
shader compilation, GPUI compositing, ring reuse under GPU load, visual fidelity,
frame pacing, Retina monitor migration, device removal and DMG packaging remain
unverified at runtime. No speedup or responsiveness measurement is claimed.

The 2x scene plus 4x MSAA costs roughly 85 bytes per supersampled pixel before
mesh/shadow allocations. A 24-million-supersampled-pixel admission limit bounds
those targets to roughly 2 GiB. Large Retina/external-display viewports step down
to 1.5x or 1x supersampling. If even 1x cannot fit, CPU rendering is used at that
size; the next physical resize (including a display-scale change) retries GPU
rendering. Device and mesh failures still disable GPU rendering for the session.
Old-size leased surfaces and driver allocation overhead can briefly
increase memory during resizing. NV12 remains 8-bit and chroma-subsampled; the
matrix test bounds uniform-color quantization error, not every edge or display
color-profile artifact. Wide-gamut/HDR output is not implemented.

The worker prevents viewport driver work from blocking AppKit, but a driver call
that itself hangs cannot be interrupted by the polling deadline. GPUI's existing
window drawable acquisition/presentation behavior is unchanged. The device is
pinned for the process; device removal falls back rather than migrating GPUs.
The GPUI error bridge assumes Voxelite's single editor window. Future multiwindow
support needs per-window error routing and a new device/lifetime review.

On macOS, run the ignored producer regression explicitly:

```sh
cargo test -p voxelite-render --features gpu metal_planes_completion_leases_and_resize -- --ignored
```

It checks actual GPU-written Y/CbCr values, producer completion, odd display
sizes, three retained buffers preventing further acquisition, a consumer-style
lease preventing reuse, and old frames surviving resize. Its readback exists
only in test code; it does not exercise GPUI's consumer command buffer.

Before relying on the path in a release, build/package on the Mac runner and
exercise both themes, saturated colors and fine edges; sustained orbit/pan/zoom;
resize and Retina/non-Retina monitor moves; edits/undo/history previews during
queued renders; overlay/picking alignment; forced CPU mode; and injected adapter,
allocation, device-loss and consumer-import failures. Check ring reuse and memory
with old scenes retained and with the window hidden or closed during a render.
