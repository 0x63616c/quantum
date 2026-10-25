# Voxelite UI concepts

Round 3 of 3: three distinct native workspace studies for the all-Rust GPUI app, each in dark and light. The viewport remains central while humans can inspect agents building in a prominent session panel.

| Concept | Layout | Preview |
| --- | --- | --- |
| A — Session dock | Document tabs, compact rail, horizontal palette, right session and layers | [HTML](concept-a/index.html) · [Dark](png/concept-a-dark.png) · [Light](png/concept-a-light.png) |
| B — Live build | Left agent workspace and local instruction draft, broad canvas, bottom human editing deck | [HTML](concept-b/index.html) · [Dark](png/concept-b-dark.png) · [Light](png/concept-b-light.png) |
| C — Transaction workbench | Labeled tool shelf, scene inspector, full-width transaction table and render review dock | [HTML](concept-c/index.html) · [Dark](png/concept-c-dark.png) · [Light](png/concept-c-light.png) |

The [scored critique and change record](../REVISIONS.md) cover all six previews and every item in the [art checklist](../ART-CHECKLIST.md). [v1](history/v1/) preserves the originals; [v2](history/v2/) preserves round 1; [v3](history/v3/) preserves round 2; [v4](history/v4/) contains this final round's verified renders.

Each HTML file opens directly and contains its CSS, vector icons, licensed Geist/Geist Mono fonts, controls and procedural renderer. No server or network is required. Embedded fonts are sourced from [local assets](../assets/fonts/README.md), with native system fallbacks. Pure black/white surfaces, 1 px borders, 12 px minimum metadata and one accent per concept are deliberate. These are HTML design prototypes; the production app remains Rust GPUI.

The same original 2,079-voxel cottage appears throughout. Main view and Front / Right / Back thumbnails project actual cube geometry with consistent top-left lighting, ambient occlusion and antialiased edges. Images are not pasted illustrations. The six outlined canopy cells identify the latest edit in the main view; captured renders omit editor overlays.

Session events, timings, validation status and connection state are **a deterministic local fixture**, not a real MCP connection or proof the cottage was made by an AI host. The live `render_views` row demonstrates an active three-view request; “Captured views” identifies completed evidence by revision. Captures preserve their geometry and visible-layer set until Refresh; edits or a visibility mismatch mark them Outdated. Camera motion leaves them unchanged. Refresh explicitly captures all three angles locally. Pause/resume changes the row and session badge together. Per-step Undo/Redo removes/restores the corresponding fixture voxels and updates the main view/count/document revision and net session total. Both change titles and History open the actual fixture coordinate diff, with applied/undone state, immutable original revision, current document revision and Undo/Redo. These local diff objects are not protocol request/response examples. Cell ranges are inclusive coordinates; API region maxima remain exclusive. Production conflict-aware history and every real tool are specified in [PLAN.md](../../PLAN.md).

Interactive preview controls: tool keys B/E/P/G/U/I; palette selection with arrow/Home/End navigation; layer selection/visibility/lock state; mirror toggles; visibly pressed grid; Home to frame; pointer orbit, Shift-drag pan and wheel zoom; real zoom selector; light/dark toggle; PNG download. Focus the viewport for arrow-key orbit, Shift+arrow pan and +/− zoom; the axis gizmo follows camera orientation. Render thumbnails set the main camera angle and mark its preset; they remain captured evidence, not live miniature viewports. Follow agent returns to the fixture's camera and disengages on manual camera input. ⌘K (or Ctrl+K) opens a searchable command palette supporting arrows, Enter, Escape, continued typing and focus return; no-match searches show an empty state. Shift+F toggles camera follow. C's Layer details computes cell ranges and voxel counts from the active layer. B's instruction draft copies text for an external host; it does not send a prompt to an agent. Empty drafts disable Copy; clipboard failure selects the draft for manual copying. Editing, save, imports and GLB exports report preview-only actions; the footer identifies the concept preview.

```sh
npm ci
npm run build:concepts
npm run render
```

Sources: `scripts/build-concepts.mjs` owns markup/icons; `scripts/lib/concepts.css` owns shared tokens/layouts; `scripts/lib/concept-controls.js` owns local interactions; `scripts/lib/voxel-scene.js` owns the scene and multi-angle renderer. The builder embeds all dependencies into each HTML.

The Playwright render script captures 1600 × 1000 CSS pixels at DPR 2, producing six 3200 × 2000 PNGs. It checks offline loading, browser errors, panel fit, pure black/white surfaces, fonts, text/control contrast, distinct render angles and preview controls. Workflow checks include undo/redo, original/current revision separation, captured-image hashes across edits/visibility/camera motion, explicit refresh, hidden undone evidence, projected orientation, keyboard camera and zoom, clipboard success/fallback with a local mock, and keyboard command search. Manual inspection of every final PNG is recorded in REVISIONS.md. These checks do not establish GPUI runtime, responsive-layout or production MCP readiness.
