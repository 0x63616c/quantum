# voxelite-mesh

Deterministic greedy surface geometry for immutable `voxelite-core` snapshots,
shared by GLB export and rendering without a renderer or GPU dependency.

Public API: `Quad`, `Mesh`, `greedy_mesh`, `greedy_mesh_with_limit`, `MAX_QUADS`,
`MeshError` and `MeshResult`. The normal dependencies are only `voxelite-core`
and `thiserror`. Error display strings retain the existing render wording for
compatibility; `voxelite-render` aliases `MeshError` as `RenderError`.

`Mesh::greedy_rectangles` is the shared, allocation-free face-mask merger. It
visits rows then columns, growing equal face keys across columns before rows.
Snapshot meshing supplies palette slots; the renderer supplies slot/AO keys and
keeps nonuniform AO faces as unit rectangles. The renderer still owns padded
composite sampling, AO, chunk invalidation, scheduling and vertex emission.
Snapshot quads retain their axis/slice/sign/row/column order and admission limit;
GPU chunks retain their existing order and AO-aware diagonals. Test-only
`reference.rs` implementations independently check snapshot geometry and renderer
vertex bits against the pre-refactor algorithms.

## CPU micro-benchmarks

Run the production greedy mesher with Divan (no GPU required):

```sh
cargo bench -p voxelite-mesh --locked
# Run just the terrain scale sweep:
cargo bench -p voxelite-mesh --locked --bench greedy -- terrain
```

The single-chunk (16³) fixtures cover solid, alternating occupied/empty
checkerboard, deterministic LCG sparse occupancy (about 1/16), and a one-voxel
hollow shell. Terrain arguments `2`, `4`, and `8` specify chunks per horizontal
side (32², 64², and 128² footprints), with stepped heights of 16–32 voxels and
caves crossing chunk seams. All fixtures use one palette slot and production
document/mesh limits.

Fixture construction and an initial successful mesh/occupied-count check happen
outside timing. Each sample calls `greedy_mesh` on the same immutable snapshot,
including its validation, compositing, allocations, and output destruction.
Throughput counts occupied input voxels, not empty volume or output quads.
Twenty samples per case keep the default sweep short; Divan CLI options can
override sampling (see `-- --help`). Compare revisions on the same otherwise
idle machine with identical toolchains and arguments. These CPU numbers are
separate from the renderer's GPU performance baselines in `bench/`.
