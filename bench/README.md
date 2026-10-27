# Rendering reference data

The independent tools are `cargo xtask bench` (performance) and
`cargo xtask visual` (GPU screenshots). See
[the rendering harness guide](../docs/perf/rendering-harness.md).

`pinned-machine.toml` identifies the Apple M2 Metal golden owner. Authoritative
images belong in `golden/apple-m2-metal/<scene>_<still>.png`, with the matching
JSON metadata beside each image. Performance baselines belong in
`baseline/<machine_key>.json`.

Reference files are intentionally absent from the initial harness change. Run
`rebaseline-visual` and `bench-baseline` through the Rendering checks workflow on
the mini, download the artifacts, review changed side-by-side images, and add the
reference files in a follow-up commit. CI never pushes them. Missing goldens fail;
there is no bootstrap exemption for PRs.

For another machine, `cargo xtask visual --local-baseline` initializes an absent
local set in `target/golden/<machine_key>/`; subsequent invocations compare it.
Updating an existing local set requires `--local-baseline --rebaseline`.
Never commit local sets or Linux software-renderer results as Metal goldens.

`scenes/autumn-workshop.voxl` is the single tracked scene file: a pinned copy of
`examples/autumn-workshop.voxl`. Procedural scenes (terrain, sponge) are generated
deterministically and verified by scene hash.
