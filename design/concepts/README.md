# Voxelite UI concepts

Round 2 of 3: three distinct native workspace studies for the all-Rust GPUI app, each in dark and light. The viewport remains central while humans can inspect agents building in a prominent session panel.

| Concept | Layout | Preview |
| --- | --- | --- |
| A — Session dock | Document tabs, compact rail, horizontal palette, right session and layers | [HTML](concept-a/index.html) · [Dark](png/concept-a-dark.png) · [Light](png/concept-a-light.png) |
| B — Live build | Left agent workspace and instruction composer, broad canvas, bottom human editing deck | [HTML](concept-b/index.html) · [Dark](png/concept-b-dark.png) · [Light](png/concept-b-light.png) |
| C — Transaction workbench | Labeled tool shelf, scene inspector, full-width transaction table and render review dock | [HTML](concept-c/index.html) · [Dark](png/concept-c-dark.png) · [Light](png/concept-c-light.png) |

The [scored critique and change record](../REVISIONS.md) cover all six previews and every item in the [art checklist](../ART-CHECKLIST.md). [v1](history/v1/) preserves the originals; [v2](history/v2/) preserves round 1; [v3](history/v3/) contains this round's verified renders.

Each HTML file opens directly and contains its CSS, vector icons, licensed Geist/Geist Mono fonts, controls and procedural renderer. No server or network is required. Embedded fonts are sourced from [local assets](../assets/fonts/README.md), with native system fallbacks. Pure black/white surfaces, 1 px borders, 12 px minimum metadata and one accent per concept are deliberate. These are HTML design prototypes; the production app remains Rust GPUI.

The same original 2,079-voxel cottage appears throughout. Main view and Front / Right / Back thumbnails project actual cube geometry with consistent top-left lighting, ambient occlusion and antialiased edges. Images are not pasted illustrations. The six outlined canopy cells identify the latest edit.

Session events, timings, validation status and connection state are **a deterministic local fixture**, not a real MCP connection or proof the cottage was made by an AI host. The live `render_views` row demonstrates an active three-view request; “Latest renders” identifies the last captures by revision. Pause/resume changes the row and session badge together. Per-step Undo/Redo removes/restores the corresponding fixture voxels and updates the views/count/revision and net session total. Inspect/History opens the actual fixture coordinate diff, with applied/undone state and Undo/Redo. These local diff objects are not protocol request/response examples. Production conflict-aware history and every real tool are specified in [PLAN.md](../../PLAN.md).

Interactive preview controls: tool keys B/E/P/G/U/I; palette selection with arrow/Home/End navigation; layer selection/visibility/lock state; mirror toggles; grid; Home to frame; pointer orbit, Shift-drag pan and wheel zoom; light/dark toggle; PNG download. Render thumbnails change the main camera and mark the selected view. Follow agent returns to the fixture's camera and disengages on manual camera input. ⌘K (or Ctrl+K) opens a searchable command palette supporting arrows, Enter, Escape, continued typing and focus return; no-match searches show an empty state. Shift+F toggles camera follow. C's selection inspector computes bounds and voxel counts from the selected layer. Editing, save, imports, GLB exports and the instruction composer clearly report preview-only actions.

```sh
npm ci
npm run build:concepts
npm run render
```

Sources: `scripts/build-concepts.mjs` owns markup/icons; `scripts/lib/concepts.css` owns shared tokens/layouts; `scripts/lib/concept-controls.js` owns local interactions; `scripts/lib/voxel-scene.js` owns the scene and multi-angle renderer. The builder embeds all dependencies into each HTML.

The Playwright render script captures 1600 × 1000 CSS pixels at DPR 2, producing six 3200 × 2000 PNGs. It checks offline loading, browser errors, panel fit, pure black/white surfaces, fonts, distinct render angles and preview controls, including real fixture undo/redo and keyboard command search. Manual inspection of every final PNG is recorded in REVISIONS.md. These checks do not establish GPUI runtime, responsive-layout or production MCP readiness.
