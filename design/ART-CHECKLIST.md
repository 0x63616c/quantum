# Voxelite — Art Checklist

## Target
Pro creative tool. Concept A now leads: modern macOS Sonoma/Sequoia toolbar and sidebar, shadcn-style menus/popovers/dialogs, Vercel/Stripe restraint, Figma collaboration and Blender viewport conventions. Aseprite informs keyboard workflow, not dense or dated editor chrome. B/C remain historical studies.

## Rules
- Viewport is the hero; chrome recedes. Neutral greys, one accent colour.
- System light + dark mode, both first-class (not inverted afterthoughts).
- Crisp vector UI, 1px hairlines, consistent 4/8px spacing, locally bundled Geist / Geist Mono with system fallbacks, tabular numerals; 12 px minimum metadata and 13–14 px labels.
- Sample voxel model must read as real art, lit with a single key light from top-left plus soft AO.

## Never-again list
- Redundant or over-explaining UI copy ("such an AI thing") — 2026-10-24
- Flat generic dashboard look — 2026-10-24
- Pixel-stepped / jagged edges on UI or renders — 2026-10-24
- Destructive actions styled as primary — 2026-10-24
- Busy/cluttered or too sparse clip-art — 2026-10-24
- MagicaVoxel .vox export (dropped from v1) — 2026-10-24
- No clutter; secondary actions go in menus ("ui is too cluttered… not using menus") — 2026-10-25
- No fixed single palette; users must define palettes or pick from preset lists — 2026-10-25
- Dated editor chrome; use modern macOS/shadcn components — 2026-10-25

## Theme direction (2026-10-24)
Vercel / Stripe aesthetic. Dark mode = pure black #000 backgrounds, crisp 1px hairline borders (#1f1f1f–#2e2e2e), Geist / Geist Mono typography, restrained single accent, very clean. Light mode = pure white with matching hairlines. Applies to all concepts, the GPUI app theme and the marketing site. App is all-Rust GPUI (Zed-level native polish), not Tauri.

## Positioning (2026-10-24)
Voxelite is THE optimal voxel editor for AI agents, while fully excellent for humans. Concepts must show a prominent agent session panel: live tool calls, added/removed diff counts, per-step undo, multi-angle render thumbnails, and a follow-agent camera toggle. Humans watch agents build and retain a complete keyboard-first editor. ⌘K must be visible and operable. Use native split panes, muted macOS traffic lights, readable secondary text and genuinely distinct concept layouts. Marketing leads with the agent workflow; docs and integrations make it reproducible.

## Palette system (2026-10-25)

The compact active palette has a switcher for custom palettes and accurate named presets. A separate manager supports create, rename, duplicate, delete, add/remove/reorder swatches and hex editing. Presets are immutable originals; duplicate to customize. Support `.hex` and `.gpl` import/export with valid downloadable text. Palette choice does not silently recolor existing model geometry. Keep import/export and management in the Palette menu or overflow.

## Declutter direction (2026-10-25)

Use a unified toolbar and translucent, blurred sidebar over pure #000/#fff base surfaces, 1px hairlines, 6–8px component radii, generous spacing and muted readable text. Soft shadows belong only to floating layers. No bevels or dense icon grids. One restrained accent; the artwork supplies color. Native-style menus include File, Edit, View, Tools, Layers, Palette, Agent, Extensions and Help, plus contextual menus. Keep live session, per-step undo, captured angles, follow-agent and ⌘K; expand a step for its diff/coordinates.

Three subtle model-space comment pins can remain in the calm viewport. Comments share the activity sidebar with Session; threads, sharing permissions and history open on demand. History includes human/agent markers and named versions; only a subtle branch indicator is appropriate for future Git integration. Extensions belong in menus and the tool picker, including an agent-made custom tool.
