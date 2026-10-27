# Stage 1 core

`voxelite-core`, `voxelite-format` and `voxelite-mesh` run without a GPU, window system, network service or GUI dependency. Bounded GLB/PNG encoding lives in `voxelite-plugin-exporters`, using portable geometry and export errors from `voxelite-plugin-api`. The API has no renderer or built-in exporter dependency; MCP encodes render-feedback PNGs directly with `png`.

`voxelite-core` owns finite half-open bounds, stable slots 1–256 (0 is air), bottom-to-top layers, sparse/dense 16³ chunks and copy-on-write snapshots. All commands prepare against private evolving state, validate the complete result and reserve history before installation. No-ops do not advance revision. Human and agent attribution share one timeline. Hosts must add authority epochs, operation deduplication and durable journal receipts around `Engine`; the pure engine does not perform I/O.

Commands cover explicit writes, erase, capped six-connected fill, mirrors, solid/hollow boxes, ellipsoids, cylinders, supercover lines, copy/move transforms, palette/layer operations, comments and named versions. Batch errors include the zero-based failing command index. Work estimator version 1 charges candidate visits and mirror candidates before deduplication. Region erase and transform scan stored source cells. Fill charges all distinct visited cells, including boundary cells of other colors.

History uses reversible immutable before/after states with shared chunk Arcs. Admission conservatively charges complete before/after content (including version snapshots), even where allocations are shared. This is intentionally stricter than physical allocation accounting; large documents can hit the history budget early. Entries are evicted whole and `initial_retained()` exposes the boundary. A single over-budget entry rejects. Checkpoints reference retained cursors. Versions are independent portable snapshots. Preview never mutates the cursor; revert checks affected fields and preserves unrelated cell changes. Structural layer, palette and comment changes use field-level conflict detection and may conservatively reject when later edits touched the same collection.

Palette sources, retrieved 2026-10-25:

- [PICO-8 manual](https://www.lexaloffle.com/dl/docs/pico-8_manual.html), standard 16 colors by Lexaloffle.
- [Endesga 32](https://lospec.com/palette-list/endesga-32), published by ENDESGA.
- [Sweetie 16](https://lospec.com/palette-list/sweetie-16), by GrafxKid.
- Grayscale is exactly RGB `17*n` for n = 0…15. Neutral, Warm and Vivid are original Voxelite palettes under the repository MIT license.

HEX and JASC-PAL preserve RGB and display order; GPL additionally preserves representable names. These formats do not encode document slot identity. Duplicate swatches are preserved. Selecting a custom/library preset is separate from editing a document; applying colors requires explicit slot commands.
