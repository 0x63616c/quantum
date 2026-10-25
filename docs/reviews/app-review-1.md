# App review round 1

Review baseline: `69a24d0`, branch `build-v1`, 2026-10-25. Review performed by the
implementing agent, acting as principal engineer and product-design reviewer;
this is not independent signoff. Read `PLAN.md`, `design/ART-CHECKLIST.md` and
Concept A's HTML and dark/light references. No changes under `design/`.

Scope is the existing native GPUI stage. The full v1 product is **not release
ready**: the documented session/plugin/web, recovery, palette authoring, comments,
version browsing, export UI and assistive-technology gates still require their
own implementation stages. Those missing foundations are not represented by
fake connected controls or by claims of full spec conformance in this review.

## Ranked findings

| ID | Severity | Finding and impact | Resolution |
| --- | --- | --- | --- |
| A01 | High | Ordinary Save blindly replaced a file modified by another editor/agent. Saving a symlink replaced the link itself. | Added bounded disk fingerprints, external-change/deletion rejection, resolved open paths, explicit symlink rejection for Save As, no-clobber creation, permission preservation and directory sync. Regression tests cover two writers, deletion, recovery to a new path and symlinks. |
| A02 | High | Keyboard Fill at an empty cell flooded the entire connected empty volume; keyboard Picker incorrectly rejected locked/hidden active layers. | Empty Fill is a no-op before palette import. Picker samples visible composite before edit eligibility checks. Both have regression tests. |
| A03 | High | Native file dialogs did not isolate keyboard input; file parsing/serialization ran on the UI thread. Save from an unsaved-changes prompt abandoned the requested New/Open/Quit. | Fix in progress. |
| A04 | High | A failed render left the old image visible while picking used the new camera; same-ID reopen and viewport resize could accept stale results. | Fix in progress. |
| A05 | Medium | Permanent left tool rail violates F01 and steals viewport width. Palette dominates the sidebar, with history below the fold. | Fix in progress. |
| A06 | Medium | At 1024×640, palette popup and viewport navigation hints overflow; long project names displace essential toolbar controls. Disabled controls are almost invisible. | Fix in progress. |
| A07 | Medium | Palette chooser is mouse-only; inactive/selected camera states are indistinguishable; Top is tilted rather than orthographic top. | Fix in progress. |

## Baseline evidence

- `cargo fmt --all -- --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: pass.
- `cargo test --workspace`: 90 passed.
- Inspected all three `screenshots/*.png`, both Concept A references, and the
  underlying native layout/input/render/persistence code.
- Existing art is original Autumn Workshop, intentionally distinct from the
  concept's Woodland cottage. Pure black/white surfaces and Geist are already
  present; the review targets composition, hierarchy, controls and behavior.

## Final verification

Pending completion of UI fixes and actual native recaptures.
