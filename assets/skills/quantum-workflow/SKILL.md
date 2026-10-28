---
name: quantum-workflow
description: Create or refine voxel art through Quantum MCP, with atomic edits, visual review, named versions and portable exports.
---

Use a connected Quantum server exposing `quantum.v1`. Discover tools rather
than assuming the full planned API exists. This workflow needs no GPU/display.

1. Call `quantum_capabilities_get`; capture epoch and open model IDs. In an
   attached native session, use the existing model. Standalone may create/open
   a project; attached mode deliberately omits create/open/save tools.
2. Read document info, layers, palette and history before changing existing art.
   Agree on the subject, scale, bounds and output. For a new prop, 32³ or 64³ is a
   useful starting scale. Save a named version before a substantial revision.
3. Compose major masses before details. Use named material roles and reuse stable
   palette slots. Prefer boxes and atomic `quantum_voxels_batch` operations over
   one call per cell. Bounds are min-inclusive/max-exclusive; Y is up.
4. Every content mutation and document save uses the captured epoch/revision and a fresh UUID operation ID.
   Accept the new revision only after checking `isError`, `ok` and returned data.
   A batch must be fully valid; hidden/locked layers reject voxel edits.
5. Call `quantum_render` for iso/front/right/back at the same captured revision.
   Inspect the returned PNGs, not just counts. Repair silhouette, grounding,
   unreadable materials and accidental occlusion before fine decoration.
6. Save an accepted named version. In standalone mode save `.voxl` with
   `quantum_document_save`, then export GLB/PNG to new relative paths inside the
   server root. Export tools use `revision`, without `expected_epoch`,
   `expected_revision` or `operation_id`; consult their discovered schemas.
   Attached mode needs a native app save. An export is not a save.

On a revision conflict, reread and rebase; do not overwrite intervening work.
A lost response can be retried only with the identical tool, arguments and
operation ID in the same process. Cancellation may occur after commit; inspect
history before retrying. Reduce batch/render size after a budget error.

Deliver actual saved paths, captured revision, views reviewed and any remaining
visual problems. Do not claim persistence when the result is memory-only.

Example request: “Build a warm brass lantern in 32³ cells. Save a blockout version,
inspect four angles, repair the handle silhouette, then save .voxl and GLB.”
