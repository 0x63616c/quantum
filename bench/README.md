# Rendering reference data

The independent tools are `cargo xtask bench` (performance) and
`cargo xtask visual` (GPU screenshots). See
[the rendering harness guide](../docs/perf/rendering-harness.md).

`pinned-machine.toml` identifies the Apple M2 Metal golden owner. Authoritative
images belong in `golden/apple-m2-metal/<scene>_<still>.png`, with the matching
JSON metadata beside each image. Performance baselines belong in
`baseline/<machine_key>.json`.

Bench comparison gates process RSS separately with `--rss-pct 50 --rss-mb 128`:
RSS fails only when its increase exceeds both 50% and 128 MB. RSS is current
resident memory sampled at phase boundaries, not a peak, and varies with
allocator history and OS memory compression. Vertex, occupancy, target and device
allocated byte gates retain `--memory-pct 10` with no absolute slack.

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

## Recorded M2 baseline status

`bench/baseline/apple-m2-metal.json` was recorded by the `bench-baseline` workflow
dispatch (run 40613358223, 3 runs, commit 7d58723) with no other CI job on the
mini. The 1-minute load average was 7.0 at start, then 3.6, 2.9 and 7.7 after each run
(see `meta.load_average` and `meta.run_load`). Run-to-run p50 spread is under 2% for
steady phases. Cold and recolor phases vary more, up to about 15% on `terrain10m`,
so judge them from the spread, not one run. All four scenes are included, `terrain10m`
too. Re-record it (`gh workflow run render-checks.yml -f command=bench-baseline`)
after intentional performance changes or a macOS/driver update. Visual goldens are
unaffected by load.
