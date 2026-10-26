# Exact voxel shadow review

Reviewed the initial commit `b1867a6` and amended it with the following fixes:

- Full occupancy builds were inserting every occupied brick into a redundant
  dirty set even though the first upload covers the whole buffer. Skip those
  insertions while a full upload is pending. The CPU viewport now acknowledges
  its dirty state too, since it has no GPU upload consumer.
- Random traversal comparisons originally used only cubic grids. Add rectangular
  dimensions, partial final bricks, and rays aimed at known voxel centers across
  dimensions up to 256, including world origins near both coordinate extremes.
- Add assertions for the actual WGSL header offsets and epsilon, and for both
  GPU buffer limits at their exact acceptance/rejection boundary. Preserve the
  actual allocation/upload error in fallback diagnostics.

## Correctness audit

The Rust reference, fine-only control, and GPU-equivalent f32 transliteration
agree on axis rays, zero/near-zero components, negative directions, boundary
starts, adjacent blockers, XYZ ties, and randomized rays/grids. The shader is
also parsed and validated with naga. Crossing times are calculated from the
original origin rather than accumulated; the brick jump corrects rounding with
those same comparisons. A tie processes one axis at a time (X, Y, Z), including
zero-length touched cells. Bounds are tested before all occupancy reads, and at
most 768 axis crossings are required. Outside starts intentionally return clear:
an outward-facing surface ray starting outside the document cannot re-enter it.

The normal epsilon is applied in grid-local f32 space. The GPU interpolates local
positions from the vertex shader and uses a flat axis normal. This avoids losing
0.001 at world coordinates near ±32768. AO and ambient/diffuse coefficients are
unchanged. The primary shader uses no shadow map, texture comparison, PCF, or
fragment derivatives.

A maximum grid uses 2,101,296 bytes: a 48-byte header, 2 MiB occupancy bitset, and
4 KiB coarse mask. Each brick count is at most 512 and fits u16. The GPU checks
both maximum buffer size and storage binding size. Dirty ranges include coarse
mask changes; a single dirty brick uploads 68 bytes. Full bounds changes upload
the header and complete grid even when the allocation size stays the same.

Snapshot tests cover overlap, hiding/showing, deletion, reorder, palette changes,
undo/redo, bounds changes, repeated snapshots, random edits, and simulated partial
GPU uploads. Occupancy is the boolean union of visible nonempty cells, so palette
or layer ordering cannot affect it. Both viewport caches persist across camera
frames; only stateless export calls rebuild. The app change is an Editor cache
field plus exactly one line in ui.rs.

The old map resources and fit are allocated only on occupancy failure, after
scoped validation/OOM/internal errors or a buffer limit rejection. Dirty state
is acknowledged only on successful upload. The map fallback persists until the
viewport is recreated. Device loss or failure of that fallback still selects the
existing software path. Metal runtime behavior, actual allocation-failure
injection, and GPU performance remain unverified on this Linux host.

## Validation and measurements

All completed successfully:

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo check -p voxelite-render --features gpu`
- `cargo test -p voxelite-render --features gpu`
- `cargo clippy -p voxelite-render --all-targets --features gpu -- -D warnings`
- GPU render check and clippy (including test targets) for `aarch64-apple-darwin`.

Release measurements on the shared Linux host, seven renders after warm-up,
1024×640 with one sample, AO and shadows enabled, excluding PNG encoding:

| Demo camera | origin/main CPU | Shared DDA CPU |
| --- | ---: | ---: |
| Wide | 236.079 ms | 135.011 ms |
| Close-up | 472.406 ms | 327.312 ms |

Full demo occupancy build: 321.095 µs (200 iterations). Single cell mutation plus
dirty range generation: 0.062 µs (20,000 iterations). Snapshot single-cell sync
plus dirty ranges: 3.348 µs (1,000 iterations). The demo buffer is 51,348 bytes.
The existing CPU API limits each image dimension to 1024, hence 1024×640 instead
of 1280×800. Reproduce with `cargo run --release -p voxelite-export --example
shadow_bench -- /tmp/shadows`.

The uncommitted `before-after.png` shows close-up and wide views. Left: the CPU
renderer at `origin/main` (`09b29d3`) in `/tmp/vx-before`. Right: this implementation.
Both use the same scene, camera, output settings and unchanged AO. The left side
already uses CPU ray-traced shadows; it does not emulate the former GPU map or
claim to demonstrate removal of GPU banding. No screenshot binaries, temporary
baseline example, or PNG are committed.
