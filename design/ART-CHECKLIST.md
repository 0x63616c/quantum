# Voxelite — Art Checklist

## Target
Pro creative tool. References: Aseprite (dense, tool-first, shortcut-driven), Figma (calm panels, precise type), Blender (3D viewport conventions, gizmos), Linear (restraint, crisp dark mode).

## Rules
- Viewport is the hero; chrome recedes. Neutral greys, one accent colour.
- System light + dark mode, both first-class (not inverted afterthoughts).
- Crisp vector UI, 1px hairlines, consistent 4/8px spacing, Inter/system type, tabular numerals.
- Sample voxel model must read as real art, lit with a single key light from top-left plus soft AO.

## Never-again list
- Redundant or over-explaining UI copy ("such an AI thing") — 2026-10-24
- Flat generic dashboard look — 2026-10-24
- Pixel-stepped / jagged edges on UI or renders — 2026-10-24
- Destructive actions styled as primary — 2026-10-24
- Busy/cluttered or too sparse clip-art — 2026-10-24
- MagicaVoxel .vox export (dropped from v1) — 2026-10-24

## Theme direction (2026-10-24)
Vercel / Stripe aesthetic. Dark mode = pure black #000 backgrounds, crisp 1px hairline borders (#1f1f1f–#2e2e2e), Geist / Geist Mono typography, restrained single accent, very clean. Light mode = pure white with matching hairlines. Applies to all concepts, the GPUI app theme and the marketing site. App is all-Rust GPUI (Zed-level native polish), not Tauri.

## Positioning (2026-10-24)
Voxelite is THE optimal voxel editor for AI agents, while fully excellent for humans. Concepts must show a visible agent activity/session panel (live edits humans can watch).
