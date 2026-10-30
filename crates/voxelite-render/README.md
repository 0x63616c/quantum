# voxelite-render

Reusable rendering over immutable voxel snapshots, consumed by `voxelite-app`
and `voxelite-mcp`. CPU rendering needs no window, display or GPU.

Public API:

- Cameras: `Camera`, `PresetView`, `Projection`, `visible_bounds`.
- CPU images: `View`, `RenderOptions`, `RenderedImage`, `RasterCache`, `render`.
- Shading: `AmbientOcclusionStrength` validates `0..=2`; default `1` preserves
  the original pixels. `RenderOptions::ambient_occlusion` still gates AO.
- Errors: `RenderError` (an alias of `voxelite_mesh::MeshError`), `RenderResult`.
- Optional `gpu` feature: `gpu::Style`, plus `gpu::Worker` and `gpu::Frame` on macOS.

Occupancy, shadow traversal, chunk meshes, shaders and GPU resources are private.
Greedy geometry belongs to `voxelite-mesh` and is not re-exported here.

`gpu::Style::ambient_occlusion_strength` applies the same strength range to
cached vertex AO in both exact-shadow and shadow-map paths, without remeshing.
The CPU rasterizer continues sampling local voxel contacts per pixel. See the
[AO comparisons](../../docs/ambient-occlusion.md) for export examples and settings.

Normal dependencies on workspace crates are limited to `voxelite-core` and
`voxelite-mesh`. Encoders and plugin hosts must not depend on the renderer;
the app and MCP supply render callbacks to exporters. Export tests and examples
may use the renderer as a dev-dependency.
