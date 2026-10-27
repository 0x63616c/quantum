# voxelite-render

Reusable rendering over immutable voxel snapshots, consumed by `voxelite-app`
and `voxelite-mcp`. CPU rendering needs no window, display or GPU.

Public API:

- Cameras: `Camera`, `PresetView`, `Projection`, `visible_bounds`.
- CPU images: `View`, `RenderOptions`, `RenderedImage`, `RasterCache`, `render`.
- Errors: `RenderError` (an alias of `voxelite_mesh::MeshError`), `RenderResult`.
- Optional `gpu` feature: `gpu::Style`, plus `gpu::Worker` and `gpu::Frame` on macOS.

Occupancy, shadow traversal, chunk meshes, shaders and GPU resources are private.
Greedy geometry belongs to `voxelite-mesh` and is not re-exported here.

Normal dependencies on workspace crates are limited to `voxelite-core` and
`voxelite-mesh`. Encoders and plugin hosts must not depend on the renderer;
the app and MCP supply render callbacks to exporters. Export tests and examples
may use the renderer as a dev-dependency.
