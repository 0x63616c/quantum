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
| 1 · app-dark | 6.0 | Bottom palette value touches the status divider; lengthy gesture caption; unlicensed bespoke glyphs; text-only Save; unnecessary permanent rail against the current Concept A contract. |
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

Pending fixes, native captures, visual inspection, and workspace verification.
