# App polish round 2 — independent review

Baseline: `12cfee0`, branch `build-v1`, 2026-10-25. Reviewed the supplied
screenshots, Concept A, PLAN.md §§1/7–9/17 and design/ART-CHECKLIST.md, then the
native app and attached MCP implementation. This reviewer did not implement the
baseline. Fixes below are subsequently self-verified, not independent signoff.
`design/` is read-only. The required Lucide pass was completed first in `b4bb684`.

## Before scores

Scores judge a finished creative application, not progress against an early demo.
The art is the strongest part; operational hierarchy and interaction integrity
are substantially below the requested bar.

| Supplied image | Score /10 | Concrete defects |
| --- | ---: | --- |
| 1 · app-dark | 6.0 | Bottom palette value touches the status divider; lengthy gesture caption; bespoke non-Lucide glyphs; text-only Save; unnecessary permanent rail against the current Concept A contract. |
| 2 · app-light | 6.2 | Same layout defects; faint floor is acceptable, but the black-on-white interface exposes inconsistent control outlines and weak section hierarchy. |
| 3 · agent-dark | 4.5 | Calls presented as an undifferentiated list; no captured views/follow-camera; no disclosure affordance for per-step revert; says “Live” after the agent disconnected. |
| 4 · agent-light | 4.7 | Same missing agent inspection controls; large empty list space and metadata overwhelm a weak activity heading. |
| 5 · comments-dark | 4.0 | A second 300px thread panel covers the artwork and duplicates the selected card; raw coordinates/state names; 28px pins; no consistent thread number in the sidebar. |
| 6 · comments-light | 4.2 | Floating discussion casts a conspicuous shadow over the model; redundant text competes with the thread; same docking and identity defects. |
| 7 · history-dark | 4.5 | Restore is the primary action despite replacing content; a bare slider has no human/agent landmarks; raw “comments add” labels; no viewport return action. |
| 8 · history-light | 4.7 | Same defects; selected blue strip is louder than the read-only context; named versions have no stable order or selected treatment. |
| 9 · main-dark | 6.0 | Duplicate of image 1; same concrete defects. |
| 10 · main-light | 6.2 | Duplicate of image 2; same concrete defects. |
| 11 · concept-a-dark | 8.3 | Strong hierarchy, docked inspection and real tool context. Grey surfaces conflict with the newer pure-black rule; tiny metadata and large empty viewport at this scale. Reference only. |
| 12 · concept-a-dark-share | 8.0 | Clear permission hierarchy; truncated URL and permission-cycling affordances need care. Hosting is future scope, so this cannot be copied as a fake working feature. Reference only. |
| 13 · concept-a-dark-comments | 8.5 | Correct docked thread with numbered identity. Long replies need scrolling, draft preservation and explicit keyboard focus. Reference only. |
| 14 · concept-a-dark-history | 8.4 | Proper preview exit and actor landmarks. Timeline needs a keyboard model and small-window proof. Reference only. |

## Findings to fix

| ID | Severity | Finding and required correction |
| --- | --- | --- |
| P01 | Medium | Replace bespoke icons and prose gesture legend; audited action glyphs and full tooltips. **Fixed first, `b4bb684`.** |
| P02 | Medium | Restore Concept A unified tool picker; remove the permanent rail, correct camera width, and keep the palette footer inside the sidebar at 1024×640 and 1440×900. |
| P03 | High | Dock comments; the current overlay obscures art. Pins incorrectly enumerate the live snapshot during history preview, and Resolved filtering includes open pins. Use the displayed snapshot and one filter for list/pins; preserve thread numbering and show filtered empty states. |
| P04 | High | History points to the last retained step after Undo rather than the actual live head. Add correct live selection, actor landmarks, readable operation labels, stable version ordering, a persistent viewport exit, and secondary Restore styling. |
| P05 | Medium | Agent activity lacks captured views, opt-in camera following, discoverable detail/revert and honest connected/idle state. Capture immutable revisions, expose actual inspection views, and stop following on manual navigation. |
| P06 | High | Attached MCP undo reports the remaining head as a newly committed entry, offering a revert of the wrong edit. Associate events with actual transaction receipts, including no-op/retry handling. |
| P07 | High | CPU render MCP requests run synchronously on the UI owner and can freeze pause/navigation. Render immutable snapshots on a bounded worker; preserve revision/epoch identity and keep mutations serialized. |
| P08 | High | Text selection has no moving end: Shift-left then Shift-right expands instead of contracting; arrow keys skip past selections; long input loses its caret outside the field. Track anchor/head, normalize UTF-16/IME ranges, and scroll the caret into view. |
| P09 | Medium | Palette reordering leaves selection at the old index, so repeated Move operations affect the wrong swatch. Keep selection attached to the moved color and verify persistence. |
| P10 | Medium | Share presents a copyable nonfunctional URI and editable imaginary permissions. Replace the prototype surface with useful local file sharing and accurate scope; no pretend hosted controls. |
| P11 | Medium | Hidden text fields retain focus when switching panels; subsequent tool shortcuts silently go nowhere. Restore focus when changing panels/threads. Named-save controls must not offer mutation during preview. |

The existing documentation incorrectly says plugins were removed from v1;
PLAN.md explicitly includes them. Correct that statement without claiming this
polish pass implements the entire remaining release program. Native macOS,
VoiceOver, wasm parity, plugin hosting and release distribution are separate
unverified acceptance gates; this Linux review cannot certify them.

## After review

All eleven medium/high findings above have been corrected. The new captures are
the actual GPUI window running on Linux/Xvfb with Mesa software Vulkan, not HTML
mockups or composited artwork. The supplied references under `design/` are unchanged.

| Supplied image / replacement | Before | After /10 | Assessment of the result |
| --- | ---: | ---: | --- |
| 1 · [app-dark](../../screenshots/app-dark.png) | 6.0 | 8.2 | Unified tool context, consistent Lucide strokes, icon + label Save, compact gesture legend, contained palette footer. Strong art/chrome separation. |
| 2 · [app-light](../../screenshots/app-light.png) | 6.2 | 8.3 | Same repairs; crisp white surfaces and restrained selection treatment. The faint floor remains intentional. |
| 3 · [agent-dark](../../screenshots/polish-2/agent-dark.png) | 4.5 | 7.8 | Actual three-angle captures with revision, opt-in following, expandable activity and honest disconnected status. Dense sidebar still limits how much history is visible at once. |
| 4 · [agent-light](../../screenshots/polish-2/agent-light.png) | 4.7 | 7.9 | Matching inspection hierarchy and readable diff counts. Thumbnails are useful for silhouette checks, not detailed texture evaluation. |
| 5 · [comments-dark](../../screenshots/polish-2/comments-dark.png) | 4.0 | 8.4 | Thread is docked; model stays visible; numbered identity agrees with pins; reply controls stay within the sidebar. |
| 6 · [comments-light](../../screenshots/polish-2/comments-light.png) | 4.2 | 8.5 | Removes the floating duplicate and its large shadow; clear author/message grouping and secondary Resolve. |
| 7 · [history-dark](../../screenshots/polish-2/history-dark.png) | 4.5 | 8.0 | Read-only toolbar, persistent exit, actor landmarks, readable labels, secondary Restore and stable named versions. Long history still uses a compact scrolling list. |
| 8 · [history-light](../../screenshots/polish-2/history-light.png) | 4.7 | 8.1 | Same corrections; blue identifies preview/selection without turning Restore into a primary action. |
| 9 · [main-dark](../../screenshots/polish-2/main-dark.png) | 6.0 | 8.2 | Same replacement state as image 1. |
| 10 · [main-light](../../screenshots/polish-2/main-light.png) | 6.2 | 8.3 | Same replacement state as image 2. |
| 11 · concept-a-dark | 8.3 | 8.3 | Reference only; not edited or re-scored as implemented software. |
| 12 · concept-a-dark-share | 8.0 | 8.0 | Reference only. Native Share now honestly supports saving a project copy, rather than pretending to host links. |
| 13 · concept-a-dark-comments | 8.5 | 8.5 | Reference only; its docked hierarchy now informs the implemented thread. |
| 14 · concept-a-dark-history | 8.4 | 8.4 | Reference only; preview exit and actor landmarks now exist in the app. |

### Resolution and regression evidence

| Finding | Implemented correction and evidence |
| --- | --- |
| P01 | Bundled pinned Lucide SVGs, explicit per-theme icon colors, action glyphs in controls/menus, Save icon + label, mouse/Shift/gesture legend with the complete text in its tooltip. Attribution includes Lucide's actual ISC license and the applicable Feather MIT license; the app remains MIT. See [icon provenance](../../crates/voxelite-app/assets/icons/README.md). |
| P02 | Toolbar tool picker replaces the rail; viewport framing uses its full available width. Palette grids scroll within a bounded height and retain their value/footer. Main, tools, palette manager and minimum-window captures inspected in both themes. |
| P03 | Docked threads, 32px pins, one displayed snapshot/filter for comments and pins, stable in-session ordinals, filtered empty states, independent message scrolling and pinned reply controls. Native capture posts a Human reply and reads it back through MCP. The initial-history capture has only the original pin, not the later tree comment. |
| P04 | Timeline uses the engine's applied head after Undo, scrolls the selected row into view, distinguishes Human/Agent, maps protocol labels to readable text and sorts named versions consistently. Preview shows a locked tool context and Return to live; Escape exits; Restore is secondary. The history regression test covers Undo with a retained redo entry. Native capture creates a named version, scrubs history and verifies that preview did not change live revision 8. |
| P05 | Successful MCP render events carry the immutable inspected snapshot and requested presets. The sidebar displays real revision-specific Front/Right/Back images; following defaults off and manual navigation disables it. Activity disclosures expose diffs and per-entry revert. Disconnected status is accurate. |
| P06 | Activity entries now come from the successful operation receipt, never the remaining head after Undo. Undo/Redo have readable labels and no misleading revert target. The real stdio attachment integration test exercises both and checks that their event entry is absent. |
| P07 | `voxelite_render` runs on one background worker per attachment; extra renders receive `render_busy`. Mutations remain serialized on the owner. Worker output preserves the captured revision; stale-epoch UI events are discarded. Unit coverage mutates authority after preparing a render and verifies the original revision/content; stdio integration verifies actual image output and inspection metadata. |
| P08 | Anchor/head selection with reversal, correct arrow collapse, Unicode boundaries, normalized UTF-16 ranges, constrained IME selection, mouse selection and horizontal caret scrolling. Focus returns to the editor on panel/control actions. A Unicode selection regression test and real native Shift-left/Shift-right text entry both pass. |
| P09 | Move left/right updates selection to the moved swatch before refreshing fields. The capture moves the same edited color right twice, checks persisted RGBA at slot 2, then moves it back. |
| P10 | Share offers Save a copy and, for saved files, Copy file path. A copy does not adopt its destination or clear the working file's dirty state. Native portal automation saves a real `.voxl` copy, validates its name and 27,897 voxels, then verifies that ordinary Save still requests the untitled working project's own destination. No fake hosted URI or permission controls remain in the app surface. |
| P11 | Panel/thread/control transitions restore root focus. Tab visits only fields in the visible, editable panel. Read-only preview disables editing tool actions and removes comment/named-version write controls. Native typing includes letters that would otherwise trigger tool shortcuts; reply and version persistence are checked through the actual bridge. Keyboard captures exercise Tab then Right/Left in history preview. |

### Additional visual checks

Reviewed both themes at 1440×900 and the supported 1024×640 minimum. At the
minimum, the session sidebar scrolls to layers/palette; thread messages scroll
independently of the composer. Captures include agent/comments/history at both
sizes, the initial history state, tool/mirror selection, the palette manager and
local Share. Small-window layouts score **7.5–8.0**: usable, with less breathing
room and more scrolling than the full workspace.

Also captured both themes at **150% and 200% native UI density**, preserving a
logical 1440×900 window (`scripts/capture-native-scales.sh`). SVG icons, labels,
dividers and viewport framing remain coherent. These are native density checks,
not resized 1× screenshots. Palette manager scores **8.0**, local Share **8.0**,
and tool/mirror state **8.2** in this review.

### Verification

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace` — 109 tests passed, no failures.
- `cargo build --workspace` — includes the Linux GPUI application.
- `cargo test -p voxelite-app --no-default-features` — 32 headless app tests passed.
- `bash scripts/capture-native.sh` — both themes, real MCP mutation/render,
  native text selection, swatch reorder persistence, reply/version readback and
  history isolation checks.
- `bash scripts/review-polish-2.sh` — native HEX/GPL import/export and Share's
  Save a copy dialog, with output validation.
- `bash scripts/capture-native-scales.sh` — 150% and 200% density captures.
- `git diff --check`; no changes under `design/`.

### Remaining limits

This is a substantially more coherent native editor, not a 10/10 release
certification. The compact agent pane shows only a small amount of activity at
once, large palettes require scrolling, and the gesture glyphs rely on their
tooltip for first-use clarity. These are low-severity density/discoverability
tradeoffs; none obscures or substitutes the underlying operation.

Camera following currently supports the orthographic presets; explicit custom
MCP cameras still render correctly for the agent but do not drive the native
camera. The worker change isolates `voxelite_render`, not every expensive export
operation. Linux screenshots do not establish macOS native-menu, VoiceOver,
full keyboard accessibility or platform IME acceptance. Hosting, wasm parity,
plugin hosting and release distribution remain separate release work. Plugins
are still required by PLAN.md; they have not been silently dropped from v1.
