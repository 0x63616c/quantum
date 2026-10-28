# Native UI screenshot goldens

`cargo xtask ui-snapshots` compares the native macOS UI against reviewed PNGs in
`bench/ui-golden/macos/`. The **UI screenshot goldens** workflow runs on
UI-affecting same-repository, owner-triggered PRs on the `golden` Mac, using the
same guard as Rendering checks. Skipped fork/non-owner runs are not verification.
The Linux `cargo xtask screenshots` PR description pipeline stays separate.
Both tools share the screen registry and the app's `--show` launch states.

Coverage includes the empty Welcome window, Autumn Workshop editor (including
layers), every Settings page, palette panel, About and empty resolved comments,
in both themes and at native 1x and 2x. Native captures include window chrome but
exclude the window shadow and pointer. The window must match the registry size
in physical pixels at its requested backing density. Resampling is not used.

## Prepare the golden Mac

Use a dedicated, unlocked, logged-in GUI session with fixed macOS version,
English locale, bundled fonts, no notifications, and no overlapping windows.
Install ImageMagick (`identify` and `compare`) and the pinned Rust/Xcode tools.
Grant Accessibility and Screen Recording to the process hosting the runner;
SSH alone does not provide capture permission. Configure displays at **both**
1x and 2x with at least 1440×900 logical points of usable space. The helper
selects the appropriate native display for each capture; missing display scales
or permissions fail rather than substituting resized images. Only this workflow
may use that GUI session while it runs. Review initial captures for focus,
window chrome, system appearance and desktop interference before accepting them.
Linux cannot validate AppKit, permissions, native densities or these references.

## Review and update references

Comparison never creates missing goldens. Missing images, wrong dimensions,
capture errors and ImageMagick errors fail. The tolerance permits at most 0.05%
of pixels to differ after 2% color fuzz, matching the existing PR comparison.
Actual PNGs, application logs, highlighted diff PNGs and a report stay under
`target/ui-snapshots/capture-*/` and are uploaded even on failure.

On the configured golden GUI session, explicitly run:

```sh
cargo xtask ui-snapshots --rebaseline
cargo xtask ui-snapshots
```

Alternatively dispatch **UI screenshot goldens** on the trusted branch with
`rebaseline: true`. Download its `ui-goldens-*` artifact; inspect every image,
then add the reviewed PNGs under `bench/ui-golden/macos/` in the same PR as the
intentional UI change. The workflow never commits or pushes references. A normal
PR run must then pass without rebaseline. Never use Linux or resampled images as
Mac references, and never update goldens merely to silence unexplained failures.

For initial rollout, Calum must provision the GUI session, review and commit the
first native references, and run a deliberate regression test PR: change a
visible shared token, confirm the normal job fails with highlighted diff images,
then revert the token and confirm a pass. Link both runs in the implementation
PR. Unit tests of the threshold do not replace this end-to-end evidence.
