# Incremental pipeline spike

Throwaway, unpublished comparison against Salsa **0.28.5**, the latest stable
crates.io release checked on 2026-10-27. No product runtime depends on this crate.
See [RECOMMENDATION.md](RECOMMENDATION.md) for measurements and the decision.

## Run

```sh
cargo test -p incremental-spike --locked
cargo clippy -p incremental-spike --all-targets --locked -- -D warnings
cargo build -p incremental-spike --release --features large-scenes --locked
python3 -B crates/incremental-spike/run.py \
  --binary "$CARGO_TARGET_DIR/release/incremental-spike" \
  --out results/salsa.bench.json
"$CARGO_TARGET_DIR/release/incremental-spike" diff bulk 9 256
```

The binary accepts `<current|salsa|diff> <workload> [runs=9] [size=256]`.
Sizes must be multiples of 16, between 32 and 256. Without `large-scenes`, use
size 64 or less to stay within production document admission. `large-scenes`
enables only core's existing benchmark feature. Never enable it in app builds.
`--no-default-features` builds the current-path control without Salsa, for a
compile/dependency comparison. Linux `/proc` supplies RSS and machine-load data.

## What is compared

`build.rs` reads the existing renderer's `chunk_mesh.rs`, omits its module-local
tests, and appends a private padded-cells adapter in `OUT_DIR`. The actual
`ChunkMeshes::sync_validated`, mesher, AO, color conversion, quad cap, ordering,
and native thread scheduling in the **current** path are unchanged. No hooks or
public exports were added to any existing crate. The original tests remain in
place. This deliberate source inclusion is suitable only for a disposable spike;
its marker check fails if the source's test-module layout changes.

Salsa has document membership/settings, palette, and per-coordinate chunk inputs.
Every chunk coordinate inside the bounds has a stable identity, even when empty.
Visible layers are retained in order; changed core chunks are cloned into inputs
after comparing immutable snapshots with `Volume::changed_chunks`. The public
core API provides `&Chunk`, not an owned `Arc<Chunk>`, so this copies chunk contents.

Tracked queries compute linear palette colors, a two-cell padded composite per
active chunk, the **same production mesher**, an ordered geometry list, and a
separate `voxelite_render_plan::RenderPlan`. Equal padded blocks backdate changes,
avoiding neighbor remeshing after interior edits. Geometry results use `Arc` and
`no_eq`; hashing/byte comparison of every mesh is not charged to query execution.
Palette/identity inputs have high durability, geometry medium, and view settings
low, so settings changes do not validate geometry. Both paths cap background work
at eight scoped threads; Salsa clones database handles and drops them before any
input setters. Query execution counters are reported in the benchmark output.

The plugin host is **not** inside a tracked query: its callbacks, timings,
enablement and panic quarantine are observable effects. The tiny plan probe
compares `RenderPlan::standard()` with a memoized equivalent, not `Host::render_plan`.
Geometry does not determine plugin kernel selection; they are separate outputs.

## Method

The deterministic terraced terrain has 9,945,088 occupied voxels in 2,688 chunks
at size 256, three palette slots, and one visible layer. Its sloped/terraced
surface has 396,160 exposed unit faces and approximately one million vertices.
It stresses dense interiors as well as chunk boundaries; it is not a solid cube
or the pinned renderer harness's `terrain10m` scene.

Fixture loading, validation and actual `Engine` transactions are outside timing.
Each process alternates two immutable snapshots with shared unchanged chunks,
discards two warm-up iterations, and reports nine timed samples. Timed work
includes input synchronization, dirty discovery, meshing, geometry aggregation,
and plan retrieval. A cold rebuild includes new-cache/database construction;
old-cache destruction is outside its timer. Digest computation is outside timing.
The runner launches three fresh sequential processes per workload and mode,
alternates mode order, retains every sample, and rejects mismatched geometry
digests/counts. Never run compilation or another benchmark alongside it.

Workloads: one surface recolor inside a chunk, boundary voxel removal, an 8x8
surface brush on each 16x16 terrain tile (16,384 writes), cold full rebuild,
palette RGB change, identical snapshots, and lighting toggles. `diff` separately
times the existing `changed_chunks` and `changed_cells` functions for attribution.
RSS includes allocator retention and thread stacks; it is not an exact live heap
measurement. Compare fresh processes, warm/input deltas, and peak RSS together.

## Limits and correctness

Tests compare every position/normal/color/AO float bit and exposed-face count with
fresh production meshes. They cover edits, deletion/addition of chunks, negative
coordinates, layer order/visibility, palette/bounds changes, no-op query reuse,
halo backdating, lighting independence, and background database clones.

The Salsa path enforces a final aggregate quad limit; it does **not** reproduce
the production cache's transient per-update budget-failure order or incremental
GPU upload/delete list. Failed-query/cancellation integration, input eviction,
document closing and asynchronous UI scheduling are not productionized. No
renderer, plugin, document, persistence or exported public API is changed.

The benchmark excludes occupancy/shadow synchronization, GPU upload/draw,
AppKit/Metal presentation, GLB/PNG encoding and thumbnail disk I/O. The CPU/GLB
mesher merges across chunks and differs from the GPU AO mesher, so their bytes
cannot be assumed reusable. The thumbnail disk cache already skips rendering
unchanged files. These results do not establish end-to-end export or thumbnail
speedups, and Linux cannot establish macOS latency or visual equivalence.
