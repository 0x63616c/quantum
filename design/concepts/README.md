# Voxelite UI concepts

Three standalone 1600 × 1000 editor concepts. Open any `index.html` directly; each file contains its own CSS, SVG icons, and procedural canvas renderer. No network assets or application server are required.

| Concept | Layout | Preview |
| --- | --- | --- |
| A — Aseprite-pro | Compact tool rail, docked palette and layers, agent session, status bar | [HTML](concept-a/index.html) · [Dark](png/concept-a-dark.png) · [Light](png/concept-a-light.png) |
| B — Figma / Linear calm | Floating toolbar, rounded inspector, separate agent session | [HTML](concept-b/index.html) · [Dark](png/concept-b-dark.png) · [Light](png/concept-b-light.png) |
| C — Blender-inspired workspace | Tool shelf, outliner, properties tabs, agent activity console | [HTML](concept-c/index.html) · [Dark](png/concept-c-dark.png) · [Light](png/concept-c-light.png) |

The concepts follow the [art checklist](../ART-CHECKLIST.md), including its black/white surface direction and visible agent sessions. Typography uses Geist / Inter when available and local system fallbacks. Nothing is downloaded at runtime.

Set `data-theme="dark"` or `data-theme="light"` on `<html>`, or use the sun button. Dark is the default. Theme colors are CSS variables.

The same 2,079-voxel woodland cottage appears in every concept. The renderer projects real cube faces with perspective, face lighting, ambient occlusion, a soft floor shadow, and an antialiased grid. Six canopy voxels mark a recent agent edit.

The preview supports drag to orbit, Shift-drag or middle-drag to pan, scroll to zoom, and Home to frame. Tool shortcuts, palette selection, mirror toggles, layer visibility and lock state, theme switching, and viewport PNG download are interactive. Editing, file saving, palette import, GLB export, and the MCP session are visual concepts; controls identify preview-only actions. No agent connection is made.

From the repository root:

```sh
npm ci
npm run build:concepts
npm run render
```

`scripts/build-concepts.mjs` owns the layouts, inline icons, styles, and preview controls. `scripts/lib/voxel-scene.js` owns the shared procedural scene. The build embeds both into each standalone HTML file.

`scripts/render.mjs` uses Playwright Chromium to capture all three concepts in both themes at a 1600 × 1000 viewport and device scale factor 2. The six PNGs are 3200 × 2000 pixels. Rendering also checks browser errors, offline loading, panel fit, model generation, and the main preview controls.
