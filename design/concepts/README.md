# Voxelite UI concepts

**Declutter round 1 of 3: concept A only.** A keeps the document-editor composition with a modern macOS unified toolbar, translucent sidebar and shadcn-style menus, popovers and dialogs. The viewport leads; Session and Comments share a sidebar. B and C retain their prior HTML and PNGs byte-for-byte.

| Concept | Current state | Preview |
| --- | --- | --- |
| A — Woodland studio | Declutter round 1; menus, palettes, comments, sharing and history | [HTML](concept-a/index.html) · [Dark](png/concept-a-dark.png) · [Light](png/concept-a-light.png) |
| B — Live build | Unchanged prior study | [HTML](concept-b/index.html) · [Dark](png/concept-b-dark.png) · [Light](png/concept-b-light.png) |
| C — Transaction workbench | Unchanged prior study | [HTML](concept-c/index.html) · [Dark](png/concept-c-dark.png) · [Light](png/concept-c-light.png) |

| A detail state | Dark | Light |
| --- | --- | --- |
| File menu | [PNG](png/concept-a-dark-menu.png) | [PNG](png/concept-a-light-menu.png) |
| Palette manager | [PNG](png/concept-a-dark-palettes.png) | [PNG](png/concept-a-light-palettes.png) |
| Comments + anchored thread | [PNG](png/concept-a-dark-comments.png) | [PNG](png/concept-a-light-comments.png) |
| Share popover | [PNG](png/concept-a-dark-share.png) | [PNG](png/concept-a-light-share.png) |
| History, scrubbed to step 4 | [PNG](png/concept-a-dark-history.png) | [PNG](png/concept-a-light-history.png) |

The [scored review](../REVISIONS.md#declutter-round-1) records every A PNG against the [art checklist](../ART-CHECKLIST.md). [v5](history/v5/) preserves all three incoming HTML files, six PNGs and the incoming README before this revision. v1–v4 remain untouched. Earlier recommendations and scores are historical; the current direction selects A and supersedes dated editor chrome.

Each HTML opens directly with embedded CSS, SVG icons, licensed [local Geist fonts](../assets/fonts/README.md), controls and procedural renderer. No server/network assets are required. Base surfaces are pure #000/#fff, metadata is at least 12px, component radii are 6–8px, and shadows are reserved for floating layers. The implementation is an offline HTML design prototype; the production app remains Rust GPUI.

A's interactions:

- File / Edit / View / Tools / Layers / Palette / Agent / Extensions / Help menus, right-aligned shortcuts, separators, nested menus and viewport/layer/palette right-click menus. Menu arrows, Enter, Escape, outside dismissal and focus restoration work. Secondary exports, grid, zoom, mirror options, layer operations, capture refresh and theme live in menus. Tool keys B/E/P/G/U/I and ⌘K/Ctrl+K command search remain available. Custom → Scatter moss demonstrates an agent-made plugin tool entry; runtime execution/registration remains a product requirement.
- The session fixture exposes a live operation, follow/pause and three captured angles. Expand either recent step for its actual coordinate diff and per-step Undo/Redo. Global undo/redo traverses the two canopy edits. Captures preserve their geometry/revision until refreshed; orbit and layer visibility do not silently overwrite them. Pointer orbit, Shift-drag pan, wheel zoom, focused keyboard arrows/Shift-arrows/+/- and Home work.
- The active palette has a compact two-row scrollable swatch area and a switcher. The manager creates, renames, duplicates and deletes custom palettes; adds/removes/reorders swatches; validates hex edits; and persists custom palettes locally when storage is available. Presets are immutable; duplicate to edit. Switching the brush palette deliberately preserves the cottage's existing colors. Woodland is a 16-color custom working palette drawn from the original art.
- `.hex` import/export uses one six-digit RGB hex value per line, with a final newline. `.gpl` uses `GIMP Palette`, `Name:`, `Columns:`, a comment separator and decimal RGB rows with color labels. Downloads are generated in-browser; imports parse the selected local file and reject invalid/empty/oversized input without replacing a palette. Preset values are verified against the published [PICO-8](https://lospec.com/palette-list/pico-8), [Endesga 32](https://lospec.com/palette-list/endesga-32) and [Sweetie 16](https://lospec.com/palette-list/sweetie-16) lists; Grayscale is exactly 00, 11, …, FF on each channel. No unverified extra presets are included.
- Three subtle comment pins are present in the calm capture. Anchors project from voxel coordinates on every camera redraw and hide when their voxel/layer is absent. Comments opens the sidebar tab; a pin opens its thread, with replies, resolve/reopen and open/resolved/all filters. The canopy thread includes a clearly attributed `MCP · comments.reply` fixture reply. View also toggles pin visibility. Context menus can anchor a new local comment to a picked voxel.
- Share opens view/comment/remix permission segments and a copyable reserved `.example` preview link, with clipboard fallback. Two synthetic human initials and one agent avatar illustrate presence. No link is published and no real service or collaboration session is connected.
- The clock or View → History opens a scrubber with human circles and agent diamonds. Each step rebuilds the viewport from immutable fixture cells: empty, ground, walls, roof, garden, tree structure, canopy and two leaf-detail edits. Step 4 has 1,816 voxels; the complete scene has 2,079. Closing/returning to latest restores the live snapshot. Named versions save the current geometry, restore it explicitly, compare actual added/removed cells with solid/dashed overlays, and branch into a local preview lineage. Versions/comments are in-memory fixtures; the branch indicator does not invoke Git.

The cottage is actual projected cube geometry with the original top-left lighting and soft AO. Session events, actor origins, MCP notes and timings are deterministic demonstration data, not evidence of a connected agent or agent-made artwork. Project new/open/save, GLB export, layer creation/rename/duplicate/delete and extension management report preview-only feedback. Painting/placing tools are selectable but do not implement the production editing engine. Read the [product requirements](../REVISIONS.md#product-requirements-2026-10-25) and [roadmap](../../PLAN.md) for the open-core plugin system, real comments/sharing/history/MCP, future Git integration and precision modelling.

```sh
npm ci
npm run build:concepts
npm run render
# During A-only iteration:
npm run render -- --only-a
```

For this revision, the builder writes **A only**. `scripts/lib/concept-a.mjs`, `concept-a.css` and `concept-a-controls.js` own its components and interactions. `scripts/lib/voxel-scene.js` owns the original procedural cottage with A-only history/comment hooks. The previous B/C markup, styles and controls remain in the shared sources and their generated standalone files are preserved.

`npm run render` produces twelve A images at 1600×1000 CSS pixels, DPR 2 (3200×2000 PNGs). It also runs the existing B/C interaction checks without writing their PNGs. A checks cover offline loading/errors, fonts/minimum text, pure base colors, default/sidebar and floating-layer fit, menus/submenus/context menus, palette selection/editing/validation/import/export bytes, immutable captures, undo, camera-projected pins, comment reply/resolve/reopen, share permissions/copy fallback, actual geometry scrubbing, named save/restore/compare/branch and command keyboard behavior. Manual image review is recorded separately. This is not production GPUI/MCP, native accessibility or responsive-layout certification.
