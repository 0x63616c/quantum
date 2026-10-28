---
name: quantum-primitives
description: Build Quantum subjects from integer primitive masses, mirrored boxes and bounded explicit voxel batches for curved shapes.
---

Requires a connected `voxelite.v1` MCP server. Inspect capabilities, current
model bounds, layers and palette. Use the returned epoch/revision and a fresh
operation UUID for every mutation. Preserve the user's chosen scale and subject.

Choose an origin, ground plane (usually Y=0), front (+Z) and a size budget. Break
the silhouette into base, body, roof/head, appendages and negative spaces. Use a
few large boxes for blockout, then subtract openings and add deliberate detail.

The current MCP catalog supports set, box, erase_region and fill batch members.
There are no ellipsoid, cylinder, line or transform tools in this build. For a
curved primitive, compute the occupied integer cells locally, sample cell centers
against its analytic shape, deduplicate coordinates and submit explicit `set`
rows. For rounded forms, verify that the resulting voxel silhouette is intentional.
Never invent an unsupported tool name.

Regions use inclusive min and exclusive max: [0,0,0] to [4,2,4] is 32 cells.
`box` supports solid/shell and discovered mode/slot conditions. Use slot zero
only in explicit set operations to erase; use `erase_region` for empty boxes.
Operations execute in order in one atomic batch. Explicit voxel inputs total at
most 65,536 across the whole request; check discovered batch/work limits too.
Split larger work into meaningful saved steps, not arbitrary one-cell calls.

Mirrors are explicit per operation: `axes` plus `plane2` (twice the plane in cell
index coordinates). Reflection maps coordinate p to plane2 − p. For indices 0…31,
plane2=31 mirrors the center of that range. Account for mirrored cells in bounds
and work budgets; do not assume the UI's selected mirror applies to MCP calls.

Use separate named layers where they improve later editing, but avoid hiding
parts to conceal broken geometry. Save a blockout version and inspect iso/front/
right/back using `quantum_render`. Add details only after proportions work from
all angles. Re-read on a conflict; a failed batch applies nothing.

Example request: “Block out a 32-cell robot from a torso box, a head box and
mirrored limbs. Review its stance from the front and back before adding a face.”
