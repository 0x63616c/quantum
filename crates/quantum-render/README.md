# quantum-render

Reusable rendering over immutable voxel snapshots, consumed by `quantum-app`
and `quantum-mcp`. CPU rendering needs no window, display or GPU.

Public API:

- Cameras: `Camera`, `PresetView`, `Projection`, `visible_bounds`.
- CPU images: `View`, `RenderOptions`, `RenderedImage`, `RasterCache`, `render`.
- Shading: `AmbientOcclusionStrength` validates `0..=2`; default `1` preserves
  the original pixels. `RenderOptions::ambient_occlusion` still gates AO.
- Errors: `RenderError` (an alias of `quantum_mesh::MeshError`), `RenderResult`.
- Optional `gpu` feature: `gpu::Style`, plus `gpu::Worker` and `gpu::Frame` on macOS.

Occupancy, shadow traversal, chunk meshes, shaders and GPU resources are private.
Greedy geometry belongs to `quantum-mesh` and is not re-exported here.

`src/gpu.rs` coordinates viewport resources and render passes. Its private
`src/gpu/targets.rs` module constructs render targets, attachments and pipelines;
the split preserves the same formats, sample counts and attachment lifetimes.

`gpu::Style::ambient_occlusion_strength` applies the same strength range to
cached vertex AO in both exact-shadow and shadow-map paths, without remeshing.
The CPU rasterizer continues sampling local voxel contacts per pixel. See the
[AO comparisons](../../docs/ambient-occlusion.md) for export examples and settings.

Normal dependencies on workspace crates are limited to `quantum-core` and
`quantum-mesh`. Encoders and plugin hosts must not depend on the renderer;
the app and MCP supply render callbacks to exporters. Export tests and examples
may use the renderer as a dev-dependency.

## Chunk meshing workers

Dirty GPU chunks use an indexed Rayon `par_iter` in a reusable private pool,
with at most eight workers. Batches below four chunks stay serial. A machine
with only one available CPU, or a failed pool allocation, also uses the serial
path. Results retain dirty-set order; cache updates and quad-budget checks run
serially, preserving failure and recovery behavior. Geometry and AO arithmetic
are unchanged. Composite preparation visits occupied sparse entries, clipped to
the same two-cell halo, while dense chunks retain indexed reads. Face planes
without exposed cells skip rectangle traversal.

Benchmark the production chunk synchronization path (CPU only, including AO):

```sh
CARGO_BUILD_JOBS=4 cargo bench -p quantum-render --locked --bench chunks
```

The Divan fixtures cover solid and sparse volumes with 1, 3, 4, 8, 32 and 128
chunks. Fixture construction and pool warmup are outside timing; full cache
synchronization and output destruction are inside. Compare the same benchmark
on both revisions, with the same CPU availability and an otherwise idle machine.
These CPU timings do not measure Metal, presentation or GPU rendering quality.
