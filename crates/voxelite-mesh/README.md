# voxelite-mesh

Deterministic greedy surface geometry for immutable `voxelite-core` snapshots,
shared by GLB export and rendering without a renderer or GPU dependency.

Public API: `Quad`, `Mesh`, `greedy_mesh`, `greedy_mesh_with_limit`, `MAX_QUADS`,
`MeshError` and `MeshResult`. The normal dependencies are only `voxelite-core`
and `thiserror`. Error display strings retain the existing render wording for
compatibility; `voxelite-render` aliases `MeshError` as `RenderError`.
