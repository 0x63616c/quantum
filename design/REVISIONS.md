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

## Concepts — round 3 critique

Reviewed each of the six supplied round-2 PNGs, the complete art checklist (including theme, positioning and every Never-again item), and the underlying interactions. The same weighted rubric applies: agent workflow 30%, hierarchy/native character 25%, typography/accessibility 20%, theme 15%, artwork 10%. Earlier 8.5–8.6 scores overstate the reliability of the evidence and camera controls. The layouts are now credible; final-round work must make their information trustworthy and their hierarchy sharper. Preserve v1/v2/v3; archive this round as v4.

| Concept | Dark | Light | Verdict |
| --- | --- | --- | --- |
| A — Session dock | 8.0/10 | 8.1/10 | Strong canvas and human editor; the dock makes the current agent action too hard to find. |
| B — Live build | 8.2/10 | 8.2/10 | Best agents-first foundation; evidence and input semantics still need discipline. |
| C — Transaction workbench | 8.1/10 | 8.2/10 | Best detailed review surface; inspection labels and table semantics are not yet precise. |

**A, dark — 8.0/10.** The cottage, rail and black surfaces are convincing. The right dock is less resolved: the running call sits below two completed changes and an expanded coordinate block, so a viewer must read history before learning what the agent is doing now. The free-standing “Change 18 · bounds / Inspect” block visually detaches evidence from its operation. “History” is buried beside validation. Move current activity above a clearly labeled recent-change list, make both change titles inspectable, and attach the compact cell range to step 18. The three high-contrast thumbnail boxes compete with Follow/Pause; reserve the accent edge for the selected camera and use quiet hairlines on the other previews.

**A, light — 8.1/10.** White chrome, cream masonry and terracotta are excellent; do not warm the background or add shadows to panels. Gold wireframes in the miniature canopy contaminate what is labeled as a render, obscuring its material color. The grid button gives no visible on/off state, and “100%” has a dropdown arrow but resets the camera. Preserve legible gold selected edges, deliver clean captured renders, give the grid a visible pressed marker, and replace the misleading zoom button with a real zoom selector. The filename tab/toolbar now earns its height; another top-bar compression would solve the wrong problem.

**B, dark — 8.2/10.** The left-to-right build-to-canvas hierarchy is the strongest positioning. But the current render request begins about halfway down the sidebar; the newest work should be visible immediately under session controls. The double-line change inspector and right-aligned Undo leave awkward gaps in short rows. Align inspectable titles, diffs and secondary actions, and retain the layer target in metadata. Violet outlines on the tiny preview tree are especially distracting against black. The bottom composer implies that a prompt can be sent through an external MCP connection, although this fixture has no host prompt channel. Make it an explicit local instruction draft with a useful Copy action; do not suggest a shipped in-app agent runner.

**B, light — 8.2/10.** The pale violet selected tool and restrained panes work. Heavy gray borders around all three render tiles create more visual weight than the active scene layer; quiet unselected tile edges while preserving clear text labels, focus rings and the selected underline. “Latest renders r018” implies stable captured evidence, yet toggling visibility or undo silently redraws them. Pin the captured scene and revision; mark it outdated after edits and provide an explicit refresh. Keep the larger thumbnail size and usable palette deck. The instruction draft must retain keyboard access, show an empty disabled Copy state and confirm successful copying without claiming an agent ran it.

**C, dark — 8.1/10.** The workbench is a useful differentiator, not a dashboard. However, the table's “History” heading contains both Undo and Running: the column is actually action/state. “Selection” reports all 263 cells in the active layer, not a voxel selection; call it Layer details. The 32³ document size is duplicated above and below the viewport. Remove the upper duplicate. Both completed change titles should lead to their diff; emphasize the inspected/latest row modestly. The fixed axis gizmo currently lies after orbit, and keyboard users cannot operate the viewport. Project the gizmo from the camera and provide focused viewport navigation.

**C, light — 8.2/10.** The tool shelf, generous render previews and aligned transaction rows are the best-resolved human workspace. Remaining imprecision is conspicuous at this quality: “bounds” displays inclusive cell extrema while the planned API defines exclusive region maxima; the detached detail strip does not identify these as cell coordinates. Use “Cells” with an accessible inclusive-range description; retain API region semantics in the spec. Teal wireframe overlays similarly obscure render evidence. Keep the main edit overlay, remove it from captures, and retain one restrained active-row edge. Do not enlarge the inspector or add more fields to fill space.

**Shared final-round corrections:** captured renders must have immutable content/revision until explicitly refreshed, with clear outdated state after a document or visibility change. A local orbit must not alter the captures. History must distinguish the original change revision from the current document revision; repeated Undo/Redo must not rewrite the identity of step 18. The coordinate summary/callout must hide when that edit is undone. Remove inert camera chevrons; expose real zoom, dynamic orientation and keyboard orbit/pan/zoom with visible focus. Keep errors, clipboard feedback and fixtures honest; no production connection or saved-file claim should be inferred from a demonstration action. These changes remain offline prototype behavior, not implementation of MCP, GPUI or a host prompt transport.

**Every Never-again item checked:**

| Checklist item | A dark/light | B dark/light | C dark/light | Round 3 disposition |
| --- | --- | --- | --- | --- |
| Redundant / over-explaining copy | Fail: detached repeated change identity | At risk: detached details; composer implies an unsupported prompt channel | Fail: duplicate dimensions, misleading Selection/History/bounds labels | Attach evidence to its change; use exact brief labels; explicit local draft. |
| Flat generic dashboard look | Pass: document/rail/dock | Pass: agent workspace and editing deck | Pass: native shelf/outliner/workbench | Preserve all three distinct split-pane structures. |
| Pixel-stepped / jagged edges | Pass: silhouettes and vector UI | Pass | Pass | Preserve DPR 2 antialiasing; inspect clean previews and projected gizmo after rendering. |
| Destructive actions styled primary | Pass: neutral Undo/Delete | Pass | Pass | Keep neutral actions, including history; no accent-filled destructive controls. |
| Busy/cluttered or too sparse clip-art | At risk: coordinate interruption and strong render boxes | At risk: misplaced active call and boxed thumbnails | At risk: repeated metadata, overlay noise in evidence | Current-call-first sidebars, quieter preview boundaries, concise contextual details; retain the original composed cottage. |
| MagicaVoxel .vox export | Pass: .voxl, GLB, PNG | Pass | Pass | No .vox export control or promise. |

**Theme/positioning check:** all six already use #000/#FFF, local Geist, hairlines and a restrained accent; retain those decisions. Every layout retains a prominent MCP API v1 session, per-step diffs/undo, multi-angle feedback, Follow/Pause, commands and complete visible human tools. PLAN.md already covers the agents-first marketing hook, versioned tool spec/errors/examples, diffable .voxl, CLI/headless, shared tokens, host integrations, repo skills, full versioned docs, onboarding, large-model budgets, demos/benchmarks and signed/notarized release/auto-update/Homebrew/native+wasm CI. Preserve those commitments and the separate post-v1 M8 gallery/art milestone. This round is final concept refinement, not permission to advertise concept art as AI-made or to mark those milestones shipped.

## Concepts — round 3 changes

Completed the third and final concept revision. Rebuilt the three standalone HTML prototypes, regenerated all six PNGs, opened and visually inspected each final dark/light image, and copied those exact files to `design/concepts/history/v4/`. Earlier critique/change records and all 18 images in v1/v2/v3 are unchanged. The incoming PNGs match v3 byte-for-byte.

**A — Session dock:** the current `render_views` call now appears directly below Follow/Pause. Recent changes have a clear heading and adjacent History action; both change titles open their own diff. The step-18 cell range sits directly below its operation without a repeated title or detached Inspect block. Captured-view tiles use quiet hairlines, readable 13 px labels and a clear gold active-camera edge. The large cottage, tab/rail arrangement and horizontal palette retain their proportions.

**B — Live build:** current activity is immediately below session controls, ahead of history. Completed rows expose their inspect action, diff, tool and Canopy target together. Clean captures and quieter unselected borders keep the violet accent purposeful. The composer is now an explicit local instruction draft: Copy is disabled when empty, copies text for an external agent host, and falls back to selecting the text if clipboard access fails. It no longer implies an implemented prompt transport through MCP.

**C — Transaction workbench:** renamed History to Action / state and Selection to Layer details; removed the duplicate upper document dimensions. Both change titles are inspectable, the latest edit has a restrained teal edge, and every transaction cell remains aligned with its heading. The compact detail strip identifies inclusive cell coordinates. The chronological workbench and larger three-angle previews remain distinct from A/B's current-call-first sidebars.

**Shared evidence and camera corrections:**

- “Captured views” preserves scene geometry, visible-layer set and source revision until explicit Refresh. Undo/Redo or a visibility mismatch marks the captures Outdated; orbiting cannot alter them. Fresh captures omit agent wireframes/tints while the main viewport retains the six-cell edit overlay. The refresh action is a synchronous local fixture capture, not a claimed remote MCP result.
- History separates immutable original change revision from current document revision. Undo hides the corresponding cell-range summary and main edit callout; Redo restores them. The net session count continues to reflect applied edits. Cell extrema are explicitly labeled as inclusive coordinates, without changing the API's exclusive region-bound convention.
- Replaced the false zoom dropdown with a real selector that also reflects wheel/keyboard zoom values. Added a visible pressed-grid marker, camera-derived axis projection, current camera label, and focused arrow-key orbit / Shift+arrow pan / +/− zoom. Manual navigation disengages Follow and clears preset framing; Home restores the front camera and 100% zoom. Inert viewport-heading chevrons are gone.
- The footer now identifies a Concept preview instead of claiming a file was saved. Save/import/GLB actions retain explicit preview feedback. Pure #000/#FFF panels, local fonts, original lighting/AO, neutral Undo/Delete, and GLB/PNG-only exports remain intact.

**Final visual review:** these scores assess concept quality, not production readiness.

| PNG | Final score | Observed result |
| --- | --- | --- |
| A dark | 8.8/10 | Current action is easy to find; condensed change evidence, clean captures and quiet borders support the hero canvas. |
| A light | 8.8/10 | Gold selection remains legible without tinting captured materials; white panels, grid state and zoom affordance are clear. |
| B dark | 8.9/10 | Strongest agents-first reading order; complete tool/target metadata, cleaner evidence and an honest draft workflow. |
| B light | 8.9/10 | Violet is restrained; render labels and focus/selection cues are clear; palette, layers and instruction draft fit cleanly. |
| C dark | 8.8/10 | Precise transaction headings, visible latest edit, truthful layer details and clean large captures; native workbench character preserved. |
| C light | 8.8/10 | Teal evidence selection and aligned rows read cleanly; no duplicated dimensions or misleading selection/bounds fields. |

Recommend **B** as the starting point for the next GPUI implementation review because the agent workflow leads the composition. A remains the strongest document-editor alternative; C provides the clearest detailed transaction review. This recommendation does not silently select or implement a production layout.

**Never-again recheck:** no redundant instruction prose or repeated change heading; no generic dashboard/card layout; crisp vector UI and antialiased cube silhouettes/captures; neutral destructive/revert actions; hierarchy and evidence density improved without adding filler art; no MagicaVoxel `.vox` export. All six themes were opened and inspected, including a full-resolution status-bar crop to confirm small text survived rendering.

**Verification:** `npm run build:concepts`, `npm run render` and `git diff --check` pass. All six outputs are 3200 × 2000 at DPR 2 with the unchanged 2,079-voxel default scene, loaded local fonts, no HTTP requests or browser errors, and no panel overflow. Visible metadata is at least 12 px. Minimum text-token contrast including selected tint: A 7.45:1 dark / 5.25:1 light; B 7.47:1 / 5.54:1; C 7.42:1 / 5.57:1. Essential control-token contrast is 4.69:1 dark / 3.41:1 light. Quiet decorative tile borders are not treated as essential input boundaries; labels, selected edges and focus rings identify their controls.

The render workflow additionally verifies immutable capture hashes across visibility/undo/redo/camera input, changed capture hashes on refresh, original/current revision separation, hidden undone evidence, projected orientation, keyboard camera and real zoom, grid state, and draft-copy success/fallback using a local clipboard mock. Existing checks for per-step/global history, layer selection/visibility/locks, mirrors, palette navigation, Follow/Pause, command search and focus return remain green. The initial C alignment check needed its selector updated from the former text span to the new Inspect button; the full suite passed after that correction. v4 matches the final PNGs byte-for-byte; v1/v2/v3 match their pre-round Git blobs. All final PNGs have no text/EXIF metadata. No personal information was added.

**Plan and completion:** updated PLAN.md's theming/interaction section and the concept README with the final capture, history, camera and draft contracts. The plan retains all requested product, marketing, docs, skills/integration, performance and release-polish milestones, including the separate post-v1 interactive GLB showcase and original agent-made pieces. All three concept revision rounds are complete. Real GPUI/MCP implementation, host installers/skills, responsive/native accessibility checks, large-model measurements, production docs/site and signed releases remain the explicit implementation milestones; these static-size HTML studies do not establish their completion.

## Concepts — final review

Visual review of all six PNGs against ART-CHECKLIST.md: black/white surfaces, restrained accents, crisp UI, lit cottage artwork and visible agent controls meet the direction. Scores assess the images; keyboard operation cannot be verified from PNGs.

| Concept | Score | Remaining weaknesses |
| --- | --- | --- |
| A dark | 8.8/10 | Icon-only tool rail slows discovery; single-row palette is crowded; floor grid is difficult to see against black. |
| A light | 8.8/10 | Icon-only tool rail slows discovery; single-row palette is crowded; pale floor grid gives weak depth cues. |
| B dark | 8.9/10 | Large unused gap above the draft; compact captures limit inspection detail; icon-only toolstrip slows discovery. |
| B light | 8.9/10 | Large unused gap above the draft; compact captures limit inspection detail; icon-only toolstrip slows discovery. |
| C dark | 8.8/10 | Bottom dock reduces viewport height; running call follows completed changes; left tool shelf has substantial unused space. |
| C light | 8.8/10 | Bottom dock reduces viewport height; running call follows completed changes; left tool shelf has substantial unused space. |

**Recommend B:** its agent-first reading order makes live work and evidence prominent while preserving a generous viewport and accessible human editing controls.

## Product requirements (2026-10-25)

**Voxelite is “an open Linux-like thing”: a tiny open core where nearly everything, built-in tools included, is a plugin anyone—human or agent—can swap, fork or extend, like Linux or VS Code.** Local plugins just run. Permissions stay minimal: only a lightweight one-time prompt for network access or files outside the project.

- **Extensibility is core architecture.** Explore a sandboxed WASM runtime with a small stable host interface. Plugins can add tools, brushes, palettes, importers/exporters, panels, toolbar buttons, menu items, commands and MCP tools. External API network access is allowed. Users and agents can create tools; an agent can author, register and invoke a new tool via MCP at runtime. Built-in tools use the same extension interfaces. The runtime and ABI need a feasibility proof, not an extra permission workflow in the concept.
- **Comments for humans and agents.** Pins anchor to voxels or regions in model space and project through the active camera. Threads support replies, resolve/reopen and open/resolved filtering. MCP tools must list, read, reply to and resolve comments; agent authorship stays visible without adding badges everywhere. Keep a selected discussion in one reading surface; a docked thread must not cover the model or draw connectors across its silhouette. Define anchor behavior when geometry is removed or a version changes.
- **Sharing.** Share links offer Can view / Can comment / Can remix. Permissions must be enforced by the future service/host; a remix includes viewing/commenting and creates an independent project lineage without editing the original. Restricted links use individual permissions only; open links use the higher of link and individual access. Real-time collaboration and presence are required roadmap work, likely v2. The concept's synthetic avatars and reserved `.example` link are fixtures, not connected collaborators or a published project.
- **History and versions.** A scrubbable edit timeline reconstructs the model at a selected step, distinguishes human and agent edits with shape as well as color, and returns to the live state without changing it. Named versions support save, restore, compare and branch. Comparisons show actual added/removed counts and a spatial overlay or side-by-side view. MCP needs version list/read/save/restore/compare/branch tools and history inspection. Restore is an explicit edit; previewing a historical step is not. Disable editing commands during preview. Save preview saves the geometry currently being viewed, and actor names accompany human/agent markers.
- **Palettes.** Users define their own palettes or choose accurate preset lists. Support create/rename/edit/reorder/duplicate/delete, hex editing and `.hex`/`.gpl` interchange. Keep the active palette compact; move management to a trailing sheet/menu. Presets have one clear Duplicate to edit action; place Applies to new voxels beside Use palette.
- **Git integration, post-v1 unless trivial.** `.voxl` files live in repositories; the history panel can show commits and visual diffs. Preserve canonical diffable files and named-version lineage. The concept shows only a subtle branch indicator.
- **Precision modelling, post-v1 unless trivial.** CAD-style dimensions, measurements, snapping and numeric input; scope and exact interaction contract remain to be defined with the maintainer. No speculative precision controls in the default concept.

These requirements supersede the earlier runtime-plugin exclusion and the previous recommendation to start with B. A's layout is the selected design direction. They are product/roadmap requirements; the HTML study does not implement the production GPUI app, service, MCP transport or WASM runtime.

## Declutter round 1

Completed **round 1 of 3**, on concept A only. The incoming three HTML files, six PNGs and README were verified and preserved in `design/concepts/history/v5/` before changes. B/C's HTML and PNG hashes remain identical to the incoming files; v1–v4 are unchanged. Work stayed on `master`; no changes were made to the separate plan branch or checkout.

The earlier 8.8 scores for A were too generous against the owner's current brief. The three chrome bands, always-visible tool rail, 36 swatches, repeated document copy and expanded agent coordinates still read as a dated editor. The palette selector offered one choice. Those are structural problems, not problems solved by changing an accent or rounding the same panels.

**What changed:** A now uses a 32px native-style menu bar and one 64px unified toolbar above a 1,272 × 874px viewport, with a 328px sidebar. The old tool rail, document tab band, mirror strip, export buttons and horizontal palette deck are gone. File, Edit, View, Tools, Layers, Palette, Agent, Extensions and Help have real dropdowns, shortcuts, separators and submenus, with context menus for the viewport, layers and palette. Secondary actions live there or in overflow. The toolbar carries one tool picker, command search, subtle synthetic presence, Comments, History and Share. Repeated document labels were removed.

The sidebar uses Session/Comments segmented tabs, generous spacing and few rules. The session retains the current operation, follow/pause, two recent steps and actual three-angle captured geometry. Step diffs, coordinates and per-step undo are disclosed only on expansion. Palette and layers remain compact below. Pure black/white base surfaces, embedded Geist/Geist Mono, one blue accent, 1px hairlines, 6–8px controls and translucent blurred surfaces replace the former hard panel stack. Shadows are restricted to floating menus/popovers/dialogs and pins. This is CSS approximation of native vibrancy, not a claim of macOS material APIs.

The palette manager supports custom create/rename/duplicate/delete, swatch add/remove/reorder and validated hex edits. Presets include exact PICO-8, Endesga 32 and Sweetie 16 values, plus a mathematically defined 16-step grayscale; source links are in the concept README. Presets can be duplicated. Active palettes stay in a two-row scrollable area. Custom data persists in local storage; deleting the final custom palette falls back to a preset. Actual browser-generated `.hex` and `.gpl` downloads round-trip through the import parser. Invalid input leaves existing palettes intact. Palette selection does not unexpectedly recolor the cottage.

Three subtle pins anchor to actual voxel coordinates and follow camera projection. The Comments tab lists threads; a pin opens replies, resolve/reopen and a reply field. The fixture includes a visible `MCP · comments.reply` agent response. Right-click picking can create a new anchored thread. Sharing has view/comment/remix segments, copy/fallback and two synthetic human avatars plus an agent; the reserved `.example` link and preview note make its unconnected state explicit.

History reconstructs the model from immutable fixture geometry at each of eight edits, with human circles and agent diamonds. The mid-way capture is step 4: **1,816 voxels**, before the tree; the default is **2,079**. Returning/closing restores the live geometry and revision rather than committing a scrub. Named versions save current geometry, restore explicitly, compare added/removed cells with solid/dashed spatial overlays, and create a separate in-memory branch snapshot with parent/version lineage. The subtle branch name does not imply Git integration. Extensions and the tool picker's Custom → Scatter moss entry illustrate the requested plugin direction without a permanent plugin panel.

**Corrections made during inspection:** the first default sidebar clipped palette metadata; spacing was reduced without reducing type. The first Comments state also pushed the palette down; the comment list now has its own bounded scroll region. Removed the palette tagline and redundant comment instructions. Moved the history popover to avoid an exposed strip of sidebar alongside it. Added scroll bounds for larger palettes/threads, fixed arrow navigation for palettes shorter than eight swatches, and verified the final-custom-palette deletion case. The full render suite passes after these corrections.

**Per-PNG review:** opened and inspected all twelve final A PNGs individually, including both themes for every state. Scores assess design quality against this expanded brief, not production readiness. They deliberately leave room for the next two rounds.

| PNG | Score / 10 | Harsh critique |
| --- | --- | --- |
| [A dark](concepts/png/concept-a-dark.png) | 8.3 | Viewport dominates and the session is calm. Pure black makes sidebar vibrancy almost imperceptible; captured views are still too small for close review. |
| [A light](concepts/png/concept-a-light.png) | 8.4 | White space and neutral selections work well. The floor grid is faint and the small clock/comment icons rely on tooltips for first-use discovery. |
| [A dark — menu](concepts/png/concept-a-dark-menu.png) | 8.5 | Proper shortcut alignment and separators replace fake menu buttons. The dark material is restrained, but hover/focus and submenu motion still need native-platform review. |
| [A light — menu](concepts/png/concept-a-light-menu.png) | 8.6 | Clean floating boundary and readable shortcuts; no panel clutter. The shadow is slightly stronger than necessary on white. |
| [A dark — palettes](concepts/png/concept-a-dark-palettes.png) | 8.3 | Custom/preset navigation and editing are clear. The small reorder/delete/add row could be more discoverable; a long preset library will need search. |
| [A light — palettes](concepts/png/concept-a-light-palettes.png) | 8.4 | Strong color/selection legibility with a neutral destructive action. Name appears in both heading and edit field; inline rename could remove that duplication in round 2. |
| [A dark — comments](concepts/png/concept-a-dark-comments.png) | 7.9 | Anchoring and human/agent authorship are clear, and the palette now fits. The open thread covers part of the very canopy being discussed; the selected message repeats in the sidebar and popover. |
| [A light — comments](concepts/png/concept-a-light-comments.png) | 8.0 | Thread, reply, resolve and MCP note read cleanly. The large white popover interrupts the silhouette; test alternate placement or a docked thread for longer conversations. |
| [A dark — share](concepts/png/concept-a-dark-share.png) | 8.3 | Permission segments and copy action are calm. The truncated preview link hides destination details; the meaning of remix will need concise product help. |
| [A light — share](concepts/png/concept-a-light-share.png) | 8.4 | Clear selected permission, good spacing and readable presence. It obscures session status while open; production collaboration still needs offline/error/revocation states. |
| [A dark — history](concepts/png/concept-a-dark-history.png) | 8.0 | Scrubbing visibly removes the tree and preserves the cottage. The history popover is tall, hides the session, and offers three repeated actions per named version. |
| [A light — history](concepts/png/concept-a-light-history.png) | 8.1 | Timeline origin shapes and named-version hierarchy are understandable. The native range track is visually heavier than surrounding hairlines; a large history needs search/grouping. |

The four preserved B/C reference PNGs were also opened and inspected. Their prior visual limitations and scores remain historical; this round did not revise or re-score them.

**Checklist recheck:**

| Item | Round 1 result |
| --- | --- |
| No clutter; secondary actions in menus | Default has no open menus/dialogs/details. One tool picker, a compact palette and progressive disclosure replace dense always-visible controls. History/comments deliberately spend more space only when opened. |
| No fixed single palette | Custom palettes, four exact preset lists, editable swatches and valid `.hex`/`.gpl` interchange are exercised in-browser. |
| No dated editor chrome | Unified toolbar, segmented tabs, restrained modern popovers and blur; no bevels, heavy stacked panels or tiny tool grid. Native material fidelity remains a GPUI implementation task. |
| No redundant / over-explaining copy | Removed duplicate document titles, viewport study label, palette tagline and comment instructions. Some heading/form and selected-thread duplication remains noted above. |
| No flat generic dashboard | The original detailed cottage and editor workflow lead; no metric cards or generic dashboard content. |
| No jagged UI / renders | SVG UI and DPR-2 antialiased procedural geometry remain crisp. Voxel edges are intentionally cubic. |
| No primary destructive actions | Delete, undo, restore and resolve remain neutral; no destructive accent-filled action. |
| No busy clutter or sparse clip-art | Viewport area increases, original cottage lighting/AO stays intact, and real evidence remains accessible. Long-session/list density needs further review. |
| No MagicaVoxel export | Only `.voxl`, GLB, PNG and palette interchange appear; no `.vox` export. |
| Theme / type / agent strengths | Pure #000/#fff bases, local Geist, 12px minimum visible metadata, one accent, hairlines, live fixture, per-step undo, captured angles, follow-agent and working ⌘K. |

**Verification:** `npm run build:concepts`, `npm run render` and `git diff --check` pass. A produces twelve 3200 × 2000 PNGs with no text/EXIF chunks. Browser checks cover no network requests/errors, font loading, minimum text size, at least 4.5:1 text-token contrast on base/floating surfaces, sidebar/floating-layer containment and three distinct rendered angles. Interaction checks exercise menu keyboard/submenus/context menus, palette editing and last-custom deletion/reload, file import validation and exact downloaded export bytes, short-palette keyboard wrapping, real geometry undo/scrub/restore/compare/branch, capture immutability, projected pins/hidden layers, picked-voxel comment creation, replies/resolve/reopen/filters, share permissions and clipboard fallback, and command search. B/C's existing checks pass without writing their PNGs. v5 matches the incoming Git blobs byte-for-byte; B/C file hashes remain unchanged. No personal information was added to fixtures, docs or assets.

**Scope and next review:** round 1 is complete; rounds 2 and 3 remain pending. Comments/versions/branch lineage are local in-memory fixtures; palettes persist locally and files download for real. Sharing/presence, MCP transport, WASM plugins, Git and production editing are not connected implementations. The README identifies preview-only project/layer/extension operations. Future reviews should test native materials, pin occlusion/placement, long lists and threads, accessible control states, smaller windows and a genuine GPUI/plugin host. PLAN.md now carries the requested open-core/plugin, collaboration, history/version, Git and precision requirements without treating this HTML prototype as shipped product functionality.


## Declutter round 2

Completed **round 2 of 3**, concept A only. The independent round-1 review scored the set **6.3/10**; that is a more useful baseline than the earlier self-review’s generous eights. The main weaknesses were invisible material depth, collapsed agent evidence and ambiguous interaction states. Before editing, all three HTML concepts, all sixteen incoming PNGs and the README were copied to `design/concepts/history/v6/`. B/C remain byte-for-byte unchanged; the separate plan branch and checkout were not used.

**Changes against all twelve image critiques:**

- **Main, dark/light (6 and 12):** the opaque session fill is gone. Toolbar and sidebar use 58%/62% material over a restrained blue, sage and warm underlay, with actual backdrop blur. Pure black/white base surfaces remain. The 360px sidebar shows both +/- diffs, one expanded change with Undo step, optional coordinate disclosure, explicit Follow On/Off and labeled Pause/Resume. It scrolls at shorter heights and resizes from 320–440px by pointer or keyboard. The original cottage, lighting and captured angles are preserved.
- **Comments, dark/light (1 and 7):** the thread sits in the left margin, preserving the canopy silhouette. A projected wireframe identifies the actual voxel; a quiet connector links it to the thread. Removed “Open thread,” added timestamps to both actors, collapsed MCP provenance into Agent details, enlarged the composer and simplified the selected-thread hierarchy to one edge. Reply/resolve/reopen, filters, camera projection and right-click anchoring still work.
- **History, dark/light (2 and 8):** history now occupies the sidebar instead of a large floating drawer. The timeline explicitly labels 0 as Initial and 1–8 as edits; circle/diamond actor markers match in ticks, list and legend. All nine states are reachable in a scrollable list and via previous/next buttons. Scrubbing rebuilds geometry and shows a persistent History preview banner with Return to latest. Named versions use one overflow menu for restore/compare/branch. The subtle `main` indicator remains; no speculative Git controls were added.
- **Menus, dark/light (3 and 9):** floating borders are clearer and light shadows tighter. The menu PNG now demonstrates File → Export with keyboard focus. Tools, Palette, View and Agent expose radio/checkbox states; Redo is disabled when unavailable and keyboard navigation skips it. Context menus remain operational. The new tools PNG shows Custom → Scatter moss selected, its agent badge and editable density/seed settings; Extensions exposes the same tool/configuration. ⌘K/Ctrl+K still selects commands without losing typing focus.
- **Palettes, dark/light (4 and 10):** reduced the manager to 670px and its library to 180px, removed the repeated title/name field and heavy full-window blur. Import/export/delete and duplicate are in palette overflow; reorder/remove are beside the selected hex field in swatch overflow. Custom edits explicitly save on Enter or blur; active palettes say In use with a disabled action. Presets show read-only state and Duplicate to edit. The palette PNG now demonstrates immutable PICO-8. Switching palettes preserves existing geometry/colors; `.hex`/`.gpl` exports remain real downloadable text and imports round-trip.
- **Sharing, dark/light (5 and 11):** added Restricted access, explanations for each permission (remix creates an independent copy), synthetic named people/agent, per-member controls and current activity. The main toolbar has subtle presence dots. Copy link is the primary action, with visible Copied and permission-change feedback. Removed the fixed prototype-status sentence from the popover; the tiny footer identifies the study.

The existing requirements section and PLAN roadmap already contain the open Linux-like plugin philosophy, minimal local-plugin permissions, MCP comments/versions/runtime tools, sharing/presence, Git integration and future precision modelling. Those requirements remain intact. This round clarifies the interaction contract; it does not implement the production MCP service, collaboration backend, Git or WASM plugin runtime.

**Visual review:** opened and inspected every one of the fourteen final A PNGs at full-image scale, including both tools captures. Scores below assess design readiness, not feature completion. The cottage still carries more of the polish than the interface; none earns a nine.

| PNG | Score / 10 | Critical assessment |
| --- | --- | --- |
| [A dark — main](concepts/png/concept-a-dark.png) | 7.7 | Agent state and Undo are finally legible in the default image. Dark material has visible temperature and depth, but native vibrancy under motion still needs a GPUI proof; captures are small. |
| [A light — main](concepts/png/concept-a-light.png) | 7.8 | White artwork ground and tinted sidebar separate well. The broader sidebar trades 32px of canvas for useful change inspection; eye controls remain hover discoveries. |
| [A dark — comments](concepts/png/concept-a-dark-comments.png) | 7.6 | Canopy is unobstructed and the anchor is explicit. The long connector crosses the roof; more complex scenes need collision-aware placement and occlusion handling. |
| [A light — comments](concepts/png/concept-a-light-comments.png) | 7.7 | The thread has a usable composer and calmer grouping. A large white thread still competes with the scene; multiple simultaneous threads need a separate design pass. |
| [A dark — history](concepts/png/concept-a-dark-history.png) | 7.7 | Clear preview mode, reliable exit, consistent actors and quiet version actions. Only a few events fit at once; the scroll hint helps but large timelines need grouping/search. |
| [A light — history](concepts/png/concept-a-light-history.png) | 7.9 | Docking removes the broad shadow and keeps the scene dominant. Thin timeline and initial-state labels are clearer; restore still needs production conflict/error treatment. |
| [A dark — menu](concepts/png/concept-a-dark-menu.png) | 8.0 | Submenu separation and keyboard focus are convincing. File actions beyond PNG export remain fixture placeholders; a screenshot cannot establish their behavior. |
| [A light — menu](concepts/png/concept-a-light-menu.png) | 8.1 | Strongest floating component: precise spacing, restrained shadow and aligned shortcuts. Native menu timing and screen-reader behavior still need platform validation. |
| [A dark — palettes](concepts/png/concept-a-dark-palettes.png) | 7.6 | Immutable preset and duplication are explicit; secondary actions recede. Central modal still interrupts composition and disabled swatch actions are visually weak. |
| [A light — palettes](concepts/png/concept-a-light-palettes.png) | 7.8 | Preset colors and selection outline read clearly, with a much lighter backdrop. Empty library space and modal placement remain opportunities for round 3. |
| [A dark — share](concepts/png/concept-a-dark-share.png) | 7.6 | Scope, remix meaning, actor activity and copy success are understandable. The popover remains tall enough to cover the session; presence is synthetic, not network evidence. |
| [A light — share](concepts/png/concept-a-light-share.png) | 7.7 | Better action emphasis and specific member access. Link/member controls need real revocation, reconnect and invite states before this can be called a finished sharing system. |
| [A dark — tools](concepts/png/concept-a-dark-tools.png) | 7.9 | Agent-made tool, selected state and configuration are visible. Settings lengthen the picker over the roof; configuration persistence is local to this page and tool execution is not implemented. |
| [A light — tools](concepts/png/concept-a-light-tools.png) | 8.0 | Clear Custom grouping, checkmark and focus outline. A large plugin collection will need search and richer tool descriptions, not a longer dropdown. |

**Checklist recheck:** viewport remains the hero; no tool rail, bevels, dense icon grid or permanent cards were added. Metadata stays at least 12px with embedded Geist/Geist Mono; one accent and 1px boundaries remain. Base colors are #000/#fff; material tints are translucent layers over them. Floating surfaces alone receive shadows. Secondary operations moved into menus, delete stays neutral, palettes are user-defined or accurate presets, and no `.vox` export was introduced. Short-window behavior and keyboard states are tested separately from PNG inspection.

**Validation:** `npm run build:concepts` and `npm run render` generate fourteen 3200×2000 A PNGs and exercise the existing B/C checks without rewriting B/C images. Checks cover offline loading/browser errors, font loading/minimum text, base colors and contrast tokens, material alpha/blur/underlay, floating fit, 1100×760 layout/scrolling/resizing, selected/checked/disabled menu states, keyboard traversal/focus/submenus, context menus, custom tool settings, ⌘K, palette immutability/custom editing/import/export bytes and preservation of scene colors, agent Undo/Redo/follow/pause/captures, comment anchors/provenance/replies/resolve, restricted sharing/permissions/copy success/fallback, and geometry-backed history/save/restore/compare/branch. The historical capture contains 1,816 voxels at step 4, versus 2,079 latest; the initial state has zero. Manual image review is separate evidence. Native accessibility and production integration remain unproven.

Integrity checks also confirmed that all six B/C files retain their incoming SHA-256 hashes, all twenty v6 archive files match the incoming revision, and A PNGs contain no text or EXIF metadata. The final render is byte-identical to the fourteen manually reviewed images. Additional 1100×760 main/history captures were inspected locally; lower sidebar content scrolls while the viewport and history exit stay reachable. `git diff --check` passes.

Round 3 should concentrate on native material behavior, long-list/long-thread layouts, collision-aware thread placement, plugin discovery and production error states. Round 2 does not claim those remaining limitations are solved.

## Concept A round 3

Completed **round 3 of 3, concept A only**. The incoming A HTML and all fourteen A PNGs were copied to [history/v7](concepts/history/v7/) and committed first as **Concept A round 3: snapshot**. Four subsequent **Concept A round 3: WIP …** commits separate comments/session, palettes/sharing, history/tools and interaction verification. Work stayed on master; B/C and the excluded branch/workspace were not changed.

The independent round-2 score of **6.9/10** is the baseline. Its criticism was accurate: the artwork was more polished than the controls. This revision concentrates on coherent states and useful inspection surfaces.

**Changes:**

- **Comments:** the selected thread has one reading surface, docked in the sidebar. The selected message is omitted from the remaining thread summaries. There is no floating viewport card, connector or voxel wireframe. This avoids silhouette collisions structurally, including during orbit and resize. The pin has a local selection halo. Resolve lives in the thread header, message identities have distinct spacing, the MCP reply note is visible without another disclosure, and the reply button belongs to the composer.
- **Palettes:** a 400px trailing sheet replaces the centered modal. Opening it reserves canvas space, keeping the camera gizmo clear. Custom and preset lists share a compact library area. Presets show one read-only label and one Duplicate to edit button; their swatch modification controls are disabled and hidden, while hex is displayed as text. Selection is a single thin outline. Use palette and Applies to new voxels share a row immediately after the color inspection area. Custom rename/add/remove/reorder/hex edits, persistence and actual .hex/.gpl downloads remain intact.
- **Sharing:** Link access and People & agents are distinct groups. View, comment and remix are cumulative; remix includes an independent copy, without changing the original. The higher of link and individual permissions applies; Restricted uses individual grants only. A compact URL/copy row gives one Copied confirmation. Presence subtitles and duplicate success text are removed. This is still a permission UI fixture, not service enforcement.
- **History:** a single track contains the scrub thumb and human/agent ticks. All nine states fit in compact rows at the default size; a bounded scroll area handles shorter windows. Rows name Artist 01, Designer 02 or Canopy agent, with a consistent selected state. Previous/Next are explicit controls. During historical preview the toolbar says Read only, editing menus/tool shortcuts/undo are unavailable, and command search cannot silently leave preview. Save preview explicitly records the displayed geometry. Restore and branch remain explicit operations; comparison counts come from actual cells.
- **Tools and menus:** both the tool picker and Extensions have search, filtered keyboard navigation and empty results. Built-in icons replace the empty gutter; only selected items have checks. Scatter moss has one Agent badge. Configuration moves into a separate contextual sidebar section. Export retains its filled highlight while the submenu is open, the child menu is narrower, and shortcut text is more legible.
- **Main hierarchy/materials:** task and agent identity form one compact group, with Checking silhouettes given the strongest activity emphasis. Completed steps retain +/- counts, tool calls, coordinate disclosure and per-step undo. Follow remains an explicit On/Off switch. Captured canvases grow from 64px to 148px tall, with one large view and Front/Right/Back segments; their geometry stays frozen until refreshed. Layer eye/lock buttons are always visible. Selected layers, history rows, library rows and segments share restrained fills/borders. Neutral translucent chrome and lighter form surfaces replace the blue sidebar/black input mismatch; A's dark ground grid is more legible. The only permanent prototype label is the small Concept footer.

The open Linux-like core/plugin philosophy, minimal permission model, runtime MCP tool authoring, human/agent comments and versions, sharing/presence, future Git integration and precision modelling remain in the product requirements and PLAN roadmap. This round adds the preview/access/comment-placement contracts there. The requested dated-editor Never-again entry was already present and remains unchanged.

**Per-image review:** every final PNG was opened and inspected individually. These are critical self-review scores for the visual concept, not independent approval or production readiness. No image earns a nine.

| PNG | Score / 10 | Remaining issues |
| --- | --- | --- |
| [Dark — main](concepts/png/concept-a-dark.png) | 8.0 | Current activity and selected layer now read immediately; captures support inspection. The neutral sidebar is coherent but its vibrancy is still subtle in a static image. Expanded evidence leaves little vertical reserve at this window height. |
| [Light — main](concepts/png/concept-a-light.png) | 8.2 | Artwork, state hierarchy and layer controls are clear. The single captured angle is substantially larger, but comparing angles still requires switching. Overflow icons depend on tooltips for first use. |
| [Dark — menu](concepts/png/concept-a-dark-menu.png) | 8.3 | Export → PNG has a continuous filled path, readable shortcuts and no misleading focus perimeter. Hover timing and diagonal pointer travel are browser approximations of a native menu. |
| [Light — menu](concepts/png/concept-a-light-menu.png) | 8.4 | The cleanest floating treatment: fitted submenu, precise separators and restrained shadow. The parent menu could size more tightly around its short labels; native platform accessibility is not established. |
| [Dark — palettes](concepts/png/concept-a-dark-palettes.png) | 7.8 | The sheet preserves the art and makes immutable presets unambiguous. A short preset still leaves substantial unused sheet space; the library needs search for a large personal collection. |
| [Light — palettes](concepts/png/concept-a-light-palettes.png) | 8.0 | Read-only inspection, duplication and palette application have a clear order. A black selected swatch still attracts more attention than pale colors, despite the thinner single outline. Long custom names need further layout work. |
| [Dark — comments](concepts/png/concept-a-dark-comments.png) | 8.1 | No duplicated conversation or artwork-crossing geometry. Author rhythm and composer grouping are stronger. The pin-to-thread relationship now relies on matching numbers; long conversations need stronger unread/navigation behavior. |
| [Light — comments](concepts/png/concept-a-light-comments.png) | 8.3 | Docking preserves the silhouette and makes the discussion easy to scan. Compact remaining threads omit their subject text while one is open. The attachment behavior after deleting an anchor still needs a production contract. |
| [Dark — share](concepts/png/concept-a-dark-share.png) | 7.9 | Access groups, cumulative remix meaning and one copy result are clear. The popover still covers live activity; a future invite/revoke/error flow must fit without inflating it again. |
| [Light — share](concepts/png/concept-a-light-share.png) | 8.1 | Stronger permission selection and a much shorter action path. The URL is truncated at this width, and understanding effective access still requires reading the single rule above people. |
| [Dark — history](concepts/png/concept-a-dark-history.png) | 8.0 | All fixture rows are visible, collaborator provenance is useful, and preview cannot be mistaken for editable state. A single track will need grouping or zoom for hundreds of edits; nine steps cannot validate that scale. |
| [Light — history](concepts/png/concept-a-light-history.png) | 8.2 | Selection, scrub position and Save preview agree. The preview banner occupies noticeable canvas space. Restore conflicts and branch navigation remain outside this fixture. |
| [Dark — tools](concepts/png/concept-a-dark-tools.png) | 8.0 | Search, real icons and one custom-tool badge make selection compact. The open menu still masks part of the roof. Configuring Scatter moss adds sidebar scrolling at the default height. |
| [Light — tools](concepts/png/concept-a-light-tools.png) | 8.2 | The picker has a coherent menu structure and consistent selection check. Plugin discovery beyond one installed custom tool needs richer descriptions and grouping; the screenshot does not demonstrate executing a plugin. |

**Checklist:** pure #000/#fff bases, embedded Geist/Geist Mono, minimum 12px visible metadata, one accent, 1px boundaries and 6–8px controls remain. Translucent toolbar/sidebar materials use backdrop blur over a restrained ambient underlay; floating UI layers alone receive elevation shadows. The cottage geometry, original lighting and AO are preserved. Secondary actions remain in menus; no bevels, dense icon grid, primary destructive action, redundant prototype sentences or MagicaVoxel export were added. Exact preset palettes and downloadable interchange remain functional.

**Verification:** **npm run build:concepts**, the full **npm run render** and **git diff --check** pass. The renderer writes fourteen A PNGs at 3200×2000 and runs existing B/C checks without writing their images. Tests cover menus/submenus/context menus and parent-active state; keyboard search in tools/extensions; palette edits, immutable presets, imports and exact .hex/.gpl export bytes; geometry-preserving palette switches; camera-projected comments, a single docked thread, reply/resolve/reopen and separation from the artwork at three angles; share scope, copy success/fallback and one feedback message; captured-angle switching and immutable captures; visible layer toggles; session undo/redo/follow/pause; actual history reconstruction, preview editing locks through keys/menus/commands, saving the preview's geometry, restore/compare/branch; fonts, text contrast, material alpha/blur, floating bounds and 1100×760 scrolling/resizing. History step 4 is 1,816 voxels; latest is 2,079; initial is empty.

B/C hashes match the incoming revision, v7 matches the incoming A Git blobs, and prior archives are unchanged. All A PNGs have no text/EXIF metadata. Fixtures contain only generic collaborator identities. Real MCP transport, editing tools, plugin execution, collaboration service, permission enforcement and Git remain product implementation work; this completes the three requested design rounds.
