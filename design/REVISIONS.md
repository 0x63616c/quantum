# Voxelite design revisions

## Concepts — round 1 critique

Reviewed all six original PNGs at `design/concepts/png/` against the full art checklist and the owner's agents-first brief. Originals are preserved in `design/concepts/history/v1/`. Scores measure readiness for this product, not generic visual attractiveness: positioning/session workflow 30%, hierarchy/native character 25%, typography/accessibility 20%, theme discipline 15%, artwork 10%. None is ready to select unchanged.

| Concept | Dark | Light | Verdict |
| --- | --- | --- | --- |
| A — Aseprite-pro | 5.0/10 | 5.4/10 | Best editor skeleton; wrong information hierarchy. |
| B — Figma / Linear calm | 4.4/10 | 4.8/10 | Attractive canvas, but an anonymous web design tool. |
| C — Blender workspace | 5.3/10 | 5.6/10 | Most credible workstation; activity remains a logging afterthought. |

**A, dark:** The 94 px activity strip at the bottom of the inspector loses to 36 oversized swatches. “Agent connected” is a status light, not an agent workflow: no running call, transaction identity, history action, camera following, or evidence of what the agent sees. Three horizontal chrome bands consume 128 px while the right panel still has dead space. The #090909 inspector/titlebar and #0c0c0c fields violate the explicit pure-black direction. 8–11 px shortcuts, metadata and session text are too small; unlocked icons nearly disappear. Saturated traffic lights distract from the artwork. The repeated undo hint and bottom navigation lesson are needless copy. Keep the precise rail and docked composition; rebuild its hierarchy around a large session dock.

**A, light:** Strong white canvas and coherent cottage lighting, but tiny #999 metadata and low-opacity locks fail useful contrast. The selected pale gold layer is too dependent on a faint fill. The large palette consumes attention without increasing editing power. Labels, secondary actions and focus/selection need deliberate weight, not a dark-mode inversion. Agent session shortcomings are identical to dark. Raise type to 12 px minimum, make selection explicit, and give history/render evidence its own readable area.

**B, dark:** The detached inspector, 14 px corners, floating toolbar and oversized 72 px title area read as a browser mockup, not GPUI. A second disconnected 94 px card at the bottom-right cannot carry the product's primary workflow. “Drafts /” suggests an unexplained web storage model. The model is good, but the large empty top region and soft panel treatment make the UI feel uncommitted. Dark surfaces are near-black rather than #000. The sole ⌘K is inert footer text. Recast this as a focused live-build workspace, with a substantial session sidebar and an economical human-tool deck.

**B, light:** The white viewport is calm; rounded boxes, shadows and breadcrumb chrome make it even more obviously web-like. The violet accent is restrained but selection boundaries are weak. Inspector rows and the session's 9–10 px metadata cannot survive normal-size viewing. The floating toolbar and view label do not share a useful baseline. Retain the open-canvas concept, remove the inspector-card stack, and use native split panes plus a compact, genuinely operable command affordance.

**C, dark:** A 166 px tool shelf with enormous unused lower space, a 302 px properties stack, three top bands and a shallow console surround a smaller-than-necessary model. Activity duplicates connection/session information and shows only two tiny log lines; no useful transaction inspection or render evidence. The palette and transform controls demand more attention than the agent. Panel surfaces again violate pure black; 9–11 px console and status labels are unacceptably small. Keep the explicit workbench arrangement, but turn the bottom dock into a real transaction table with a separate multi-angle inspection area.

**C, light:** The native panel boundaries are the strongest of the six, but the same density imbalance persists. White space in the shelf and console is wasted while the right side is cramped. Very pale separators and muted log timestamps undermine hierarchy. The bottom copy repeats “Edit applied”, “Undo available”, and session state instead of offering an action. Use larger, aligned transaction rows, direct per-step actions, compact tool groups and a strong session header.

Shared: Geist/Geist Mono are merely named, not supplied; rendering depends on machine fallback. All six lack a prominent live tool call, clear added/removed diff counts, per-step undo, multi-angle render thumbnails and a follow-agent-camera control. These are release-blocking positioning failures. A human must be able to inspect the agent's work without surrendering the excellent voxel editor. Preserve the original cube-based cottage, top-left lighting, soft AO and crisp vector controls. Do not solve positioning with a marketing paragraph inside the app.

**Every Never-again item checked:**

| Checklist item | A dark/light | B dark/light | C dark/light | Round 1 action |
| --- | --- | --- | --- | --- |
| Redundant / over-explaining copy | Fail: navigation hints, repeated Undo | Fail: navigation hints, unexplained Drafts | Fail: navigation hints, duplicate log/status prose | Remove lessons and redundant state; use actionable session rows. |
| Flat generic dashboard | At risk: unprioritized right stack | Fail: detached rounded inspector/card | Pass on structure, weak session | Three distinct native layouts: right session dock, left live-build sidebar, bottom transaction workbench. |
| Pixel-stepped / jagged UI or renders | Pass | Pass | Pass | Preserve vector icons and DPR 2 antialiasing; inspect silhouettes and thumbnails after rendering. |
| Destructive action styled primary | Pass: trash subdued | Pass | Pass | Keep delete/undo secondary; pause is a neutral session action. |
| Busy/cluttered or too sparse clip-art | Fail: chrome bands + dead inspector space | Fail: oversized empty header, detached cards | Fail: redundant chrome, empty shelf/console | Reallocate space; keep the art central and use purposeful session content. |
| MagicaVoxel .vox export | Pass: only GLB/PNG | Pass | Pass | Preserve `.voxl` project, GLB/PNG exports; no `.vox` UI. |

Implementation scope: these remain offline HTML/CSS design prototypes for a future all-Rust GPUI app. Session events are a clearly documented fixture, not evidence of a connected MCP host. The plan must also correct its artist-first opening and outdated gray/Inter theme table, and cover agent API/spec, integrations, skills, docs, release polish and the post-v1 showcase milestone.

## Concepts — round 1 changes

Implemented the critique in the shared HTML generator, CSS and procedural renderer, then rebuilt and rendered all six concepts. This completes round 1 only; rounds 2 and 3 remain separate reviews.

**Layout changes:**

- **A — Session dock:** retained the useful document-tab/rail structure, moved the 36-color palette into a compact horizontal strip, and devoted the top-right dock to a 533 px session with layers beneath. Session hierarchy now outranks swatches. Primary canvas still fills most of the window; purposeful document/tool rows remain distinct.
- **B — Live build:** removed the floating toolbar, tall breadcrumb header and detached rounded inspector/activity cards. A 352 px left sidebar holds the current instruction, session, render evidence and next-instruction affordance; an aligned top toolstrip and bottom palette/layer deck serve manual work. The 499 px session is part of the primary workspace. This is structurally different from A, not a palette swap.
- **C — Transaction workbench:** tightened the labeled shelf to 152 px and the scene inspector to 272 px. Replaced the tiny console with a 292 px full-width session dock: transaction rows on the left and three large render previews on the right. Removed redundant console lines and unused transform controls. Kept the explicit 3D workstation structure.

**Shared corrections:** pure #000 dark and #FFF light surfaces; 1 px borders; locally bundled/embedded Geist and Geist Mono with OFL notices and native system fallbacks; 13–14 px labels and 12 px minimum metadata; readable lock/status controls; explicit selection edges; muted native-style traffic lights. Removed all “Drag Orbit / Drag Pan / Scroll Zoom” copy and duplicate undo instructions. Added the visible Commands ⌘K control with a real searchable, keyboard-operable palette and focus restoration. No network dependencies.

Each session shows named tool calls (`fill_box`, `set_voxels`, `render_view`), step IDs, added/removed counts, secondary per-step Undo/Redo actions, running/paused state, revision, validation status, camera follow and Front/Right/Back render previews. Thumbnails are generated from actual scene geometry, update on fixture undo, and switch the main camera when selected. Manual camera input disengages follow. Global undo/redo traverses the two fixture edits in order. The six highlighted canopy cells and original lit cottage remain intact in default captures.

The first regenerated set exposed two additional issues during image inspection: thumbnail shadows were scaled for the main viewport and washed the light thumbnails gray; C's eight-column palette left a half-empty row. Scaled the thumbnail shadows with their viewport, enlarged the preview models, expanded A's thumbnail area, and changed C to a complete six-column grid. Re-rendered and opened every final PNG after these fixes.

**Final visual review:**

| PNG | Score after round 1 | Verification |
| --- | --- | --- |
| A dark | 8.2/10 | Session is immediately visible, black surfaces/hairlines are clean, palette no longer dominates, art remains central. |
| A light | 8.3/10 | Legible secondary labels, explicit gold selection edge, clean white thumbnails and calm panel boundaries. |
| B dark | 8.2/10 | Left build/session and broad viewport establish agent-first identity; compact native tool/deck alignment replaces the web-card composition. |
| B light | 8.3/10 | White panes stay distinct through rules and spacing; instruction and render evidence are readable without shadows/cards. |
| C dark | 8.4/10 | Substantial transaction workbench, aligned diffs/actions and the largest multi-angle previews; human tools remain accessible. |
| C light | 8.4/10 | Readable table/status hierarchy, complete palette grid, proper thumbnail lighting and restrained teal state treatment. |

**Never-again recheck:** redundant tutorial/status prose removed; generic dashboard/card structure removed; all silhouettes and SVG controls antialiased at DPR 2; delete and undo remain secondary; unused shelf/console space is reclaimed for useful session content while preserving the artwork; project/exports remain `.voxl`, GLB and PNG with no MagicaVoxel `.vox` option. Both themes were opened and checked for each concept.

**Plan alignment:** updated the agents-first product contract, Vercel/Stripe pure-black/white theme and local Geist requirements; specified shared app/site/docs tokens, onboarding/empty/error states, command palette and accessibility. Added versioned spec-first MCP with per-tool examples, atomic batches, primitives, multi-angle rendering, validation and inspectable history. Canonical `.voxl` is now coordinate JSON for direct agent inspection and Git diffs, superseding the unimplemented ZIP/binary proposal. The plan covers a professional versioned docs site, ready-made repository skills, Cursor plugin/one-click integration, Codex CLI and Claude Code/Desktop install pages, benchmark/demo recordings, signed/notarized releases, auto-update, changelog, Homebrew cask, native/wasm CI and a separate M8 agent-created showcase/gallery milestone after working v1. These are planned deliverables, not claims that the product/integrations already exist.

**Verification evidence:**

- `npm run build:concepts` and `npm run render` pass; all six PNGs are 3200 × 2000, with 2,079 voxels in each default scene.
- Render checks cover offline loading/no HTTP requests, browser errors, dimensions/panel fit, pure black/white backgrounds, three distinct geometry-rendered angles, tools, mirrors, layer visibility, theme switching, step Undo/Redo (−6 and −24 cells, restored exactly), ordered global Undo/Redo, follow toggle, thumbnail camera, pause and keyboard command search.
- Inspected every final dark/light PNG. A separate browser audit found minimum visible HTML text size 12 px and both embedded font faces loaded in every theme. Lowest text-token contrast on the base surface: A dark 8.13:1 / light 5.58:1; B dark 8.13:1 / light 5.91:1; C dark 8.13:1 / light 5.94:1. This is a token audit, not a claim of full application accessibility certification.
- `history/v1/` matches the six pre-change Git PNGs byte-for-byte. `history/v2/` matches the six current final PNGs byte-for-byte. PNGs contain no text or EXIF metadata. `git diff --check` passes.

Scope boundary: HTML prototypes remain fixed-size, offline studies of the future Rust GPUI app. Session/connection/validation/timing data is a documented fixture; this round did not implement a real MCP host, product onboarding, shipping plugins or the marketing/docs sites. Native behavior, responsive layouts, large-model performance and production integration workflows remain their explicit implementation milestones. Rounds 2 and 3 should assess the next concrete artifacts without treating this review as release approval.
