# macOS surface viewport

## What changed

The macOS editor defaults to a wgpu Metal viewport presented with GPUI's
`Window::paint_surface`. `VOXELITE_CPU_VIEWPORT=1` forces the software viewport.
The software rasterizer remains the export/thumbnail renderer and the automatic
fallback. Its existing interactive resolution/refinement behavior is retained;
the active GPU path always uses the same full-quality render settings.

The separate browser canvas, wasm viewport dependencies, browser harness and
GPUI web/wgpu patches have been removed. `crates/voxelite-app/src/web.rs` matches
`public/main`; existing web application code is otherwise outside this change.
No wasm build was attempted for this Mac-only scope.

A dedicated owner thread performs adapter initialization, chunk meshing/AO,
resource allocation, encoding, completion polling and GPU resource teardown.
Camera-only requests reuse chunk geometry. The editor allows one outstanding
request, coalesces subsequent input, and installs only frames matching the
current document generation, revision, preview and dimensions. Picking also
requires the frame identity most recently painted by GPUI. Preview boxes,
selection feedback, comments and the orientation gizmo remain GPUI overlays.

The presentation sequence is:

1. Multiply the laid-out viewport bounds by the window scale factor to obtain
   physical display pixels. Render the scene at twice those dimensions, with
   4x MSAA, ambient occlusion, hard shadows, outlines and an opaque background/grid.
2. Resolve linear shading to nonlinear RGB in an RGBA8Unorm target. Convert it
   with the exact inverse of GPUI 1.25.3's full-range BT.601 matrix, including
   its four-decimal coefficients and 0.5 chroma center.
3. Write full-size luma and half-width/half-height interleaved CbCr directly into
   the IOSurface planes of a `420YpCbCr8BiPlanarFullRange` CVPixelBuffer. Chroma
   averages each 2x2 RGB block, retaining one chroma sample per display pixel.
4. Wait for wgpu's submitted-work completion callback on the worker before
   transferring the completed buffer to GPUI. No production pixel readback or
   CPU upload occurs in this handoff.

The CoreVideo pool implements a bounded three-buffer ring: its minimum buffer
count and allocation threshold are both three. CoreVideo decides reuse from
ownership; the implementation never samples retain counts or blindly rotates
slots. Resizing replaces the pool while existing frames keep old allocations
alive until their consumers release them.

## Review findings and fixes

- **Actual supersampling:** the earlier implementation only doubled the NV12
  planes and repeated luma texels. All scene/MSAA/resolve targets now render at
  2x device dimensions. Checked multiplication handles zero, overflow, device
  texture limits and the allocation budget without silently lowering GPU quality.
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
- Workflow configuration targets `[self-hosted, homelab-voxelite]` on pushes to
  `main` and `perf/**`, plus manual dispatch. It runs `scripts/package-macos.sh`
  and uploads `dist/Voxelite.dmg` as the `Voxelite.dmg` artifact. No release or
  push was performed from this worktree; the workflow itself was not executed.

## Untested and remaining risks

No physical Mac or GPU was available. CoreVideo allocation/import, actual Metal
shader compilation, GPUI compositing, ring reuse under GPU load, visual fidelity,
frame pacing, Retina monitor migration, device removal and DMG packaging remain
unverified at runtime. No speedup or responsiveness measurement is claimed.

The 2x scene plus 4x MSAA costs roughly 85 bytes per supersampled pixel before
mesh/shadow allocations. A 24-million-supersampled-pixel admission limit bounds
those targets to roughly 2 GiB; large Retina/external-display viewports can fall
back to CPU. Old-size leased surfaces and driver allocation overhead can briefly
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
