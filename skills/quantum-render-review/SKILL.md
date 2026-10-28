---
name: quantum-render-review
description: Inspect real Quantum MCP renders from multiple angles, identify spatial flaws, and verify repairs at a consistent document revision.
---

Requires Quantum `quantum.v1` with `quantum_render`. Discover the catalog and
read document info first. The current server has no tool named screenshot;
`quantum_render` is the supported model-image feedback tool. It works headlessly.
A separate host screenshot tool may inspect native UI when available, but is not
required and must not substitute a fabricated image for an MCP render.

Request iso/front/right/back, width=512, height=512, `image_mode="both"`, and the
captured `revision`. The default contact sheet is 1024²; keep all images from one
revision. On concurrent edits, reread and recapture rather than mixing old and
new views. Preserve the returned image metadata with the evidence.

Inspect the actual PNG image blocks. If the host cannot display them, export PNG
and open it with the host's image viewer; otherwise report the inspection limit.
Never infer visual quality from a successful render status or voxel count.

Evaluate at thumbnail scale, then inspect details:
- Silhouette: readable subject, connected parts, deliberate negative space.
- Structure: ground contact, consistent thickness, no accidental floating cells.
- Occlusion: rear/side details, holes and hidden intersections.
- Materials: focal contrast, distinguishable roles, consistent palette.
- Framing: no cropped extremes, useful scale, transparent or solid background
  chosen for the intended export.

Front is viewed from +Z, right from +X. Add top/left or an explicit camera when
these four views cannot expose a suspected flaw. Do not claim hidden geometry
has been checked when it has not. At most six views, 1024² per view and 4,194,304
combined view/sheet pixels are allowed; lower size after budget errors.

Prioritize two or three concrete repairs with coordinates/layers. Apply an
atomic patch with captured guards, rerender the same camera set and compare.
Stop when the requested result is met; avoid endless speculative polishing.
Save the accepted named version, then persist through standalone document_save
or the native app. Report reviewed views, revision, saved outputs and remaining
issues. If edits continue afterward, mark earlier captures stale.

Example request: “Review this lantern from four angles. Fix a handle that merges
with the body, and show the same views after the repair.”
