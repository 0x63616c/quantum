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

## Concepts — round 2 critique

Reviewed all six attached round-1 renders, the source controls and the complete art checklist. `history/v1/` already preserves the originals; the six incoming PNGs match `history/v2/` byte-for-byte. Neither archive will be overwritten. Scores use the same weighted rubric as round 1. The prior 8.2–8.4 scores were generous: these are credible layouts, but several controls communicate states the prototype does not actually maintain.

| Concept | Dark | Light | Verdict |
| --- | --- | --- | --- |
| A — Session dock | 7.5/10 | 7.4/10 | Good document editor; activity is readable but insufficiently inspectable. |
| B — Live build | 7.6/10 | 7.5/10 | Strongest agents-first composition; repeated chrome and tiny evidence dilute it. |
| C — Transaction workbench | 7.7/10 | 7.6/10 | Strong human workspace; table alignment and selection semantics need correction. |

**A, dark — 7.5/10.** The 128 px title/tab/context stack is too tall for the compact-editor proposition, and the document strip's “Local session ⌘1” advertises an inert navigation shortcut. The task subtitle repeats the instruction while the actual six-cell change has no inspectable coordinates or result. “Live”, “Running”, two revision spellings and a repeated layer count add noise without establishing which state is current. The 36 swatches are only about 26 px wide; selection needs a robust outline and useful keyboard navigation. Latest-change outlines are useful, but the tag has no visible attachment to the cells. Preserve the excellent large cottage and right dock; shorten chrome, make the change inspectable, and distinguish session state from captured-render revision.

**A, light — 7.4/10.** White surfaces and cottage lighting are strong. Gold is used on too many equivalent states: running card, selected layer, mirror and callout. Faint alpha borders on the selected mirror and preview buttons cannot carry an essential control boundary; the current camera has no selected-thumbnail state at all. The dark and light PNGs both show unlocked/visible icons with no reliable state distinction after clicking. Reinforce selection with a solid edge and text/shape cues, retain neutral borders elsewhere, and keep the active camera synchronized with follow/manual navigation.

**B, dark — 7.6/10.** This is the best agents-first starting point, but “Build / History”, “Current instruction”, the repeated filename, “Agent session”, “Local agent / MCP”, and “Local MCP session” spend roughly a third of the sidebar establishing the same context. History looks like a tab but is inert. The tall unused area above the composer and the redundant 36 px “Model” bar below the canvas coexist with undersized 68 px render canvases. The selected palette color has no visible hex value in this layout. Reclaim the redundant rows for larger render evidence and a concrete change summary; expose real history inspection and the selected color.

**B, light — 7.5/10.** The violet direction works, but the washed-out selected tool/mirror relies on background tint rather than a precise selection marker. Three equally outlined thumbnails hide the selected camera. The blank-result command palette has no empty state, and navigating its results moves focus away from search, making continued typing awkward. The inspector deck consumes too much vertical space for two compact controls. Tighten the deck, retain a legible instruction hierarchy, and make keyboard search and palette navigation work without a mouse.

**C, dark — 7.7/10.** The bottom dock is prominent, but “Tool / change” does not describe its columns: changes, diffs, tool names and target/timings sit under a single vague heading. The running row reverses that order, so the eye has to learn a second grid. The right inspector's static “Origin 6, 3, −3” implies a layer transform that the fixture does not have; selecting another layer leaves “Canopy” there. A second “Woodland cottage” in the workspace bar and a duplicate layer footer add bookkeeping. Use one aligned transaction grid, an actual change inspector with computed coordinates, and a slightly larger artwork within the reclaimed viewport.

**C, light — 7.6/10.** The native split panes are convincing, but the pale teal outline is insufficient to identify selected camera/tool controls. The selection inspector's static origin is especially conspicuous against the otherwise clean white UI. Grid lines continue almost undiminished into the foreground and compete with the model's base; soften their peripheral emphasis in both themes without losing orientation. Increase the model's scale modestly, keep the top-left key/AO, and use a single accent edge on the live row instead of another large outlined card.

**Shared state and accessibility corrections:** use `render_views` for a three-view request, matching PLAN.md; expose MCP API v1 without adding marketing prose. Inspectable history must identify step, target, added/removed coordinates and revision. Pause must update the session badge as well as the row, and the session summary must describe the active net edit rather than grow each time Undo/Redo is exercised. Front/Right/Back selection, frame/follow state, lock glyphs and current-tool glyphs must track interactions. Render annotation should be attached to the actual highlighted cells and remain contained while orbiting/zooming. Keep existing preview-only messaging for actions that are not implemented. Essential controls need ≥3:1 boundary/selection contrast; metadata stays ≥12 px and regular text ≥4.5:1. Do not equate passing these checks with GPUI or full assistive-technology certification.

**Every Never-again item checked:**

| Checklist item | A dark/light | B dark/light | C dark/light | Required correction |
| --- | --- | --- | --- | --- |
| Redundant / over-explaining copy | Fail: session subtitle/local label, repeat revision/count | Fail: instruction/file/local labels, Model strip | Fail: duplicate document title/count, vague table heading | Remove repetitions; use exact labels and inspectable state. |
| Flat generic dashboard | Pass: rail + document dock | Pass: left build workflow | Pass: workbench + outliner | Keep the three native arrangements; no marketing or metric cards in the editor. |
| Pixel-stepped / jagged UI or renders | Pass | Pass | Pass | Preserve vector UI and antialiasing; inspect outlines and new thumbnails at full resolution. |
| Destructive actions styled primary | Pass | Pass | Pass | Keep Undo/Delete neutral, including history inspection. |
| Busy/cluttered or too sparse clip-art | At risk: tall chrome, repeated status | Fail: tiny render evidence versus redundant bands | At risk: unaligned table, over-present grid | Reallocate space to artwork and meaningful evidence; retain the original composed scene. |
| MagicaVoxel .vox export | Pass: .voxl / GLB / PNG | Pass | Pass | Preserve supported formats; no .vox export control or promise. |

Round 2 scope is the concept HTML/CSS, faithful local preview interactions, render verification and plan alignment. The all-Rust GPUI implementation, real MCP transport, host installers, skills packages, polished docs/marketing, release signing and post-v1 agent-made showcase remain their explicitly planned milestones. Concept artwork must not be presented as an agent-made production showcase.

## Concepts — round 2 changes

Rebuilt all three standalone HTML prototypes from shared sources, rendered all six themes, opened every final PNG, and copied those exact files to `design/concepts/history/v3/`. Round-1 critique/changes and the v1/v2 image archives are preserved. This is round 2 of 3.

**A — Session dock:** reduced title/tab/context chrome from 128 to 116 px and the palette dock from 100 to 88 px, returning 24 px to the viewport. Removed the inert local-session shortcut, redundant task subtitle and duplicate layer footer count. Added computed latest-change bounds and a working Inspect action within the session; tightened layer rows so the full dock still fits. The large cottage remains the primary visual element.

**B — Live build:** removed the repeated filename/local-session footer and the 36 px Model band; shortened instruction/header spacing and the material deck. The render canvases grow from 68 to 88 px high, with readable selected-view markers. The selected color's hex value is now visible beside Palette. History opens the real fixture inspector. The composer has a clear pane boundary; the remaining breathing room accommodates longer session logs in the eventual app, rather than adding filler content.

**C — Transaction workbench:** aligned all completed/running rows to one six-column grid with exact headings for step, change, added/removed counts, tool, target/time and history. Added a compact coordinate-bound summary and Inspect action below the rows. Removed the repeated workspace document label. Replaced the invented origin with computed layer voxel counts/bounds; selecting Roof updates the inspector and hides the Canopy-only change link. Narrowed the shelf by 8 px and increased the model's projection scale approximately 8% without enlarging chrome or obscuring the silhouette.

**Shared corrections:**

- Sessions expose `MCP · API v1`; the three-angle operation uses the planned `render_views` name. “Latest renders” uses a single `r018` revision spelling, separating captured evidence from active-call state. Inspect/History opens a keyboard-accessible dialog with actual fixture coordinates, tool, target, revision and applied/undone state. Neutral Undo/Redo changes the geometry and updates the dialog. These are fixture diffs, not fabricated wire-protocol examples.
- Solid selected-tool edges, mirror outlines, swatch rings and active-camera underlines replace reliance on pale tints. The live row uses one accent edge with a subdued fill. Follow/Pause and render-preview boundaries use the contrast-checked control token. Main panel surfaces remain exactly #000/#FFF with hairline separators and locally embedded Geist fonts.
- Follow, preset camera selection and manual orbit/frame actions now agree. Pause updates the session badge and active row together; the paused badge also changes shape. C's session total reflects the net active edit instead of accumulating each Undo/Redo cycle. Lock and visibility glyphs and A's current-tool glyph update with their controls.
- The canopy annotation is attached to projected edited-cell geometry, flips when needed and hides when it cannot fit. The floor grid fades radially at its edges; top-left lighting, AO, antialiasing, three distinct preview angles and all 2,079 scene voxels are preserved.
- Palette swatches have roving keyboard focus with arrows/Home/End. Command search keeps focus while navigating results, allows continued typing, shows a no-match state, and restores focus on close. Inspected the new history dialog and command empty state visually as well as exercising them in the browser.

**Final visual review:**

| PNG | Round 2 score | Verification |
| --- | --- | --- |
| A dark | 8.5/10 | More canvas height, attached edit marker, inspectable coordinates and legible right dock; pure black remains dominant. |
| A light | 8.5/10 | Solid gold selection cues, clear active camera, readable controls; white panels and original material lighting preserved. |
| B dark | 8.6/10 | Agent workflow remains first in the hierarchy; larger evidence views, leaner instruction area and deck, visible selected color. |
| B light | 8.6/10 | Violet marks selection precisely; instruction, change evidence and composer are cleanly separated without extra cards. |
| C dark | 8.6/10 | One aligned table, computed selection inspector, larger cottage and quieter grid; full render review remains visible. |
| C light | 8.6/10 | Teal active edges remain clear on white; table labels match their cells and the fabricated transform is gone. |

**Never-again final audit:** redundant session/document/count labels removed; all three native split-pane arrangements remain distinct with no generic dashboard cards; SVG controls, cottage silhouettes and render thumbnails remain antialiased; Delete/Undo remain secondary; reclaimed chrome supports useful evidence and artwork rather than filler; only `.voxl` projects and GLB/PNG exports are shown, with no `.vox` export. All six themes pass this review, not just the dark variants.

**Verification evidence:** `npm run build:concepts`, `npm run render` and `git diff --check` pass. The renderer checks offline loading, fonts, errors, 1600 × 1000 layout at DPR 2, pane fit, pure black/white surfaces, 12 px minimum visible text, three distinct thumbnail angles, table-column alignment and default annotation containment. New workflow checks cover inspectable coordinate data, dialog Undo/Redo and focus return, stable net session totals, dynamic selection bounds/counts, lock glyphs, roving swatches, camera/follow/orbit/frame state, pause badge state and command no-match/continued typing. Existing scene/tool/mirror/layer/global history checks remain green. An initial new assertion incorrectly assumed coordinate insertion order; it was corrected to compare the six coordinate positions as a set, and the complete suite then passed.

| Theme | Minimum text-token contrast, including selected tint | Minimum essential control/selection contrast |
| --- | --- | --- |
| A dark / light | 7.45:1 / 5.25:1 | 4.69:1 / 3.41:1 |
| B dark / light | 7.47:1 / 5.54:1 | 4.69:1 / 3.41:1 |
| C dark / light | 7.42:1 / 5.57:1 | 4.69:1 / 3.41:1 |

These are prototype token/state checks, not full assistive-technology certification. All final PNGs are 3200 × 2000 with no text/EXIF metadata. v3 matches current PNGs byte-for-byte; v1/v2 match their pre-round Git blobs. No personal information was added.

**Plan alignment and remaining scope:** PLAN.md's theming section retains the Vercel/Stripe #000/#FFF, hairline and Geist direction and now specifies the exact selection, follow, pause, history, revision and keyboard-state contract. Its agents-first marketing hook, versioned spec-first tool API, deterministic `.voxl`, CLI/headless workflow, professional versioned docs, per-host installers, repository skills, shared tokens, benchmarks/demos, onboarding/accessibility/performance, signed/notarized builds, auto-update, changelog, Homebrew and native/wasm CI remain explicit milestones. The interactive GLB showcase and original agent-made pieces remain M8 after working v1. This round does not claim those production features shipped. Round 3 remains pending.
