---
name: quantum-palettes
description: Choose, create or apply Quantum palettes with readable material roles and stable slot mappings, without unintended recoloring.
---

Requires Quantum MCP `quantum.v1`. Discover capabilities and read
`quantum_palette_list` before editing. Use captured epoch/revision and fresh
operation UUIDs for mutations; check results before continuing.

Start with material roles: darkest structure, base material, lit face, secondary
material and one focal accent. Four to eight well-separated values usually read
better than many near-duplicates. Inspect the intended lighting before adding
shading colors: CPU renders already provide face lighting and AO.

Preset IDs: `pico8`, `endesga32`, `sweetie16`, `grayscale`, `neutral`, `warm`, `vivid`.
Use `quantum_palette_apply_preset` to get the exact built-in colors; do not
reconstruct a named palette from memory. Presets/imports append into free slots
and return `slot_mapping`; source positions are not stable document slot IDs.
Existing model appearance is preserved by these append operations.

Use `quantum_palette_set_slot` with a unique name, an explicit stable slot and
opaque RGBA8 (alpha 255). Replacing an occupied slot recolors every reference:
check counts and the user's intended scope first. Slot zero is air, never a
color. Up to 256 slots exist. If capacity rejects, keep the existing model and
choose a smaller palette; don't silently remap or delete colors.

`quantum_palette_import` / `quantum_palette_export` support bounded HEX/GPL
text. Read the discovered schema for exact text fields. App palette-library
selection changes brush choices, not existing geometry. Library management and
PNG palette extraction must not be assumed to be MCP tools.

Render iso and front with the new material mapping. Check the focal object
against the ground, shadow-value separation and small details. Return the
role-to-slot mapping and note any intentional recoloring. On stale state, read
the palette again and rebase rather than retrying with invented IDs.

Example request: “Give this workshop a restrained autumn palette: cool slate,
warm timber, amber leaves and one cream highlight. Preserve the layer geometry.”
