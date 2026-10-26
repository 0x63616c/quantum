# History playback self-review

Reviewed against `c93da80` on `feat/demo-history`.

- **Playback must never replay commands into the live engine.** It advances the
  existing immutable previews, leaving content, revision, dirty state, undo and
  redo unchanged. End-of-history playback restarts at the first retained state;
  an undone live head starts at that head and includes retained redo entries.
- **A timer alone is insufficient.** A late tick advances only one state and
  sets a fresh 250 ms deadline. The UI waits for the current render before
  advancing, so slow CPU rendering cannot discard every intermediate frame.
  Render errors pause playback instead of leaving it waiting indefinitely.
- **Cancellation must precede early returns.** Scrubbing pauses even on the
  already selected marker. Rejected edits and undo attempts also pause. Undo and
  redo remain invocable during playback, including through native menus, but
  retain the existing read-only preview guard. Return to live or Restore is
  still required before changing the document.
- **Old timers must not affect a new playback session.** The UI owns a cancellable
  task, drops it before restarting, and cancels it when replacing the editor.
  Model-level cancellation also covers named versions, Restore, Return to live,
  direct editor mutations and incoming agent changes. Revision checks prevent
  stepping across an unreported live change.
- **Expired history is not the initial state.** Playback and scrubbing share the
  retained-entry iterator. An evicted initial state is never requested; fewer
  than two available states cannot start playback.
- **Keyboard handling must respect focus.** Bare Space toggles only with the
  History panel active, after text-field, modal and focused-control guards.
  Repeated key-down events do not repeatedly toggle. The control reuses the
  existing focus ring, tooltip, keycap and Lucide icons. Return to live uses the
  exit icon to distinguish it from playback.

Seven deterministic state-machine tests cover deadlines, delayed ticks,
pause/resume, start/restart/end, redo preservation, rejected edits, scrubbing,
versions/restoration, incoming edits and history eviction. Existing history
tests continue to cover saving and restoration conflicts.

Linux native checks at 1024×640 cover button playback, Space pause/resume,
Ctrl+Z interruption, mouse scrubbing, automatic stopping, restarting, replacing
the project and the tooltip. Timed screenshot comparisons confirm that the
timeline remains stationary after scrub, undo and automatic stop. Both themes
were visually checked.

Mac hand-check: reviewed the cfg-gated updater and initialization code, native
menu construction and enablement, shortcut installation, and editor/settings
window menu refresh. Playback introduces no Mac-only imports or bindings; its
code is compiled by the Linux checks. Native Undo/Redo route through the shared
guard. No new Clippy issues found by inspection. AppKit execution was not tested
on this Linux host.

Passed:

```sh
cargo fmt --check
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
```
