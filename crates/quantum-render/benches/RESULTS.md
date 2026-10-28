# Chunk meshing benchmark results

**Review status:** the performance acceptance item in #95 is not yet met.
Fresh current-main comparisons below reproduce small-batch gains but do not
establish the required absence of regression for larger batches. Earlier
acceptance conclusions were too strong for the measured uncertainty.

## Revisions and method

Issue #95 replaces per-sync scoped threads with a reusable private Rayon pool.
The original 2026-10-28 measurements below identify main `7d4e750` and
candidate `67c88d5` by their historical, pre-#90-rewrite IDs. Those IDs are
measurement labels only; no old commit was merged or pushed. The takeover
uses the fresh rewritten history and confirms newly built binaries below. Main received the
identical `chunks.rs` fixture, its Divan dev-dependency/bench declaration, and
explicit test-module paths required when including the private production source.
Its production meshing algorithm was unchanged.

Linux x86-64, Intel Xeon VM, eight available CPUs, Rust 1.99.1, optimized bench
profile; both implementations capped at eight workers. Every Cargo command used
`CARGO_BUILD_JOBS=4` and `CARGO_TARGET_DIR=/home/box/.cache/cargo-shared`.
Build both revisions before timing:

```sh
cargo bench -p quantum-render --locked --bench chunks --no-run
cargo bench -p quantum-mesh --locked --bench greedy --no-run
```

Copy each emitted executable before building the other revision: the shared target
can reuse executable paths. Run those immutable copies with `--bench --color never`.
For chunks, filter one fixture at a time, e.g. `'::solid::4$'`. Alternate main then
candidate on odd rounds, candidate then main on even rounds, for every fixture.
Twenty rounds per revision, 100 Divan samples per fixture per round. Fixture
construction and pool warmup are excluded; full cold-cache synchronization,
geometry/AO, cache admission and output destruction are included. No GPU is used.

The machine is shared. Initial five-run whole-suite and five-run paired diagnostics
were unstable (one-minute load reached 15.6; even unchanged serial cases moved by
22%). They prompted the longer, alternating comparison below rather than acceptance
of their apparent regressions. No builds/tests from this task overlap any timing
series. During the final twenty rounds no Rust build processes were observed after
the first few rounds; another application's screenshot capture remained active.
One-minute load declined from about 13 to 5. A further four-round confirmation
started below 5 (recorded range 4.24–7.37). This is measured shared-machine evidence,
not an idle-machine or Metal performance claim.

## Twenty-round comparison

Milliseconds: median of run medians, with minimum–maximum run median in parentheses.
Negative change is faster. All fixtures have twenty runs on each revision.

| Fixture | Chunks | Main ms | Rayon ms | Change |
| --- | ---: | ---: | ---: | ---: |
| Solid | 1 | 0.969 (0.882–1.209) | 0.956 (0.899–1.275) | -1.4% |
| Solid | 3 | 2.495 (2.353–2.795) | 2.516 (2.354–3.332) | +0.9% |
| Solid | 4 | 1.218 (0.986–3.974) | 1.100 (0.917–7.937) | -9.7% |
| Solid | 8 | 1.776 (1.154–4.315) | 1.696 (1.062–3.371) | -4.5% |
| Solid | 32 | 4.017 (3.350–10.610) | 3.926 (3.156–11.950) | -2.3% |
| Solid | 128 | 11.800 (9.550–24.380) | 10.725 (9.336–40.270) | -9.1% |
| Sparse | 1 | 0.544 (0.510–1.028) | 0.547 (0.517–0.826) | +0.5% |
| Sparse | 3 | 1.734 (1.657–2.654) | 1.776 (1.680–2.603) | +2.5% |
| Sparse | 4 | 0.934 (0.717–3.115) | 0.865 (0.662–2.439) | -7.3% |
| Sparse | 8 | 1.355 (0.926–4.670) | 1.286 (0.790–7.675) | -5.0% |
| Sparse | 32 | 4.247 (3.613–9.420) | 4.102 (3.427–10.200) | -3.4% |
| Sparse | 128 | 15.875 (12.710–31.610) | 16.220 (13.480–45.890) | +2.2% |

Four/eight-chunk solid and sparse fixtures improve by 4.5–9.7%. A paired bootstrap
of the twenty per-round percentage changes (10,000 resamples, fixed seed 95) gives
95% intervals entirely below zero for these four fixtures: solid 4 −12.6 to −5.5%,
solid 8 −7.3 to −1.4%, sparse 4 −16.6 to −6.3%, sparse 8 −14.8 to −4.2%.

The largest slower full-series median is 2.5% (sparse 3, an unchanged serial path).
This does not establish absence of a material regression. In particular, sparse
128 is +2.2%, with a paired interval of −1.4 to +10.5%; do not claim it speeds up.
These measurements demonstrate small-batch gains; serial operation
below four and the bounded, reused pool remain unchanged from the candidate.
A separate twenty-pair solid-one-chunk check was 0.975 → 0.983 ms (+0.8%, ranges
0.932–1.302 and 0.959–1.330), unlike the initial noisy 22% apparent regression.

<details>
<summary>Twenty-round per-run medians (ms), in run order</summary>

| Fixture | Revision | Runs 1–20 |
| --- | --- | --- |
| solid 1 | main | 0.9756, 1.049, 0.9731, 0.9291, 0.967, 1.147, 1.209, 0.9025, 0.9544, 0.9394, 1.188, 0.9483, 1.174, 0.9443, 0.9711, 1.126, 0.882, 1.153, 0.89, 0.9269 |
| solid 1 | candidate | 0.989, 0.984, 0.9675, 0.9545, 1.275, 0.9622, 0.919, 0.9741, 0.9481, 0.952, 0.9516, 0.9463, 1.184, 0.928, 1.245, 0.9037, 1.145, 0.8995, 0.9408, 0.9574 |
| solid 3 | main | 2.557, 2.555, 2.489, 2.412, 2.586, 2.382, 2.556, 2.771, 2.353, 2.469, 2.454, 2.557, 2.463, 2.795, 2.607, 2.775, 2.42, 2.47, 2.501, 2.435 |
| solid 3 | candidate | 2.527, 2.563, 2.537, 2.534, 3.332, 2.485, 2.649, 2.375, 2.354, 2.794, 2.583, 2.467, 2.417, 3.027, 3.086, 2.417, 2.432, 2.423, 2.506, 2.375 |
| solid 4 | main | 1.127, 1.218, 3.974, 1.966, 1.309, 1.069, 0.9862, 3.167, 1.14, 1.14, 1.285, 1.227, 1.187, 1.199, 1.308, 1.179, 1.218, 1.311, 1.42, 1.036 |
| solid 4 | candidate | 0.9883, 1.282, 7.937, 1.656, 1.158, 1.061, 0.9167, 1.692, 1.079, 1.07, 1.25, 0.9437, 1.121, 0.9972, 1.189, 1.027, 1.166, 1.006, 1.302, 0.9518 |
| solid 8 | main | 1.846, 1.99, 1.873, 4.315, 1.846, 1.706, 1.714, 3.798, 1.815, 1.748, 1.315, 1.787, 1.328, 1.231, 1.827, 1.154, 1.373, 1.765, 1.84, 1.209 |
| solid 8 | candidate | 1.733, 1.817, 1.889, 3.371, 1.779, 1.668, 1.692, 3.243, 2.765, 1.628, 1.336, 1.704, 1.303, 1.213, 1.699, 1.062, 1.665, 1.6, 1.702, 1.58 |
| solid 32 | main | 4.089, 6.462, 4.904, 5.748, 4.262, 3.945, 4.684, 10.61, 6.665, 4.207, 4.136, 3.783, 3.583, 3.35, 3.767, 3.485, 3.394, 3.615, 3.641, 3.615 |
| solid 32 | candidate | 5.16, 4.731, 5.265, 6.182, 4.753, 3.826, 5.091, 11.95, 7.302, 3.974, 4.03, 3.574, 3.527, 3.425, 3.879, 3.156, 3.624, 3.633, 3.551, 3.449 |
| solid 128 | main | 14.24, 24.38, 23.42, 23.37, 11.51, 11.93, 16.41, 10.76, 20.17, 12.25, 12.42, 9.839, 9.79, 12.74, 10.78, 10.07, 10.2, 11.67, 10.11, 9.55 |
| solid 128 | candidate | 13.56, 23.51, 40.27, 15.36, 10.51, 10.61, 14.86, 13.14, 37.98, 10.66, 10.48, 9.562, 10.09, 10.79, 9.336, 10.29, 12.52, 11.55, 10.22, 9.798 |
| sparse 1 | main | 0.5555, 0.5509, 0.5919, 0.599, 0.51, 0.5615, 0.5333, 0.5344, 0.5356, 0.5474, 0.5414, 0.5203, 0.5566, 0.5476, 0.5152, 0.5407, 1.028, 0.5518, 0.5409, 0.5355 |
| sparse 1 | candidate | 0.5718, 0.58, 0.6848, 0.6397, 0.5222, 0.8255, 0.5472, 0.546, 0.539, 0.5529, 0.526, 0.551, 0.5449, 0.5632, 0.5167, 0.5419, 0.5194, 0.5465, 0.7348, 0.5306 |
| sparse 3 | main | 1.856, 1.818, 1.716, 1.885, 1.689, 1.73, 1.76, 1.708, 1.731, 1.756, 1.737, 1.706, 1.765, 2.654, 1.657, 1.707, 1.707, 1.79, 1.745, 1.728 |
| sparse 3 | candidate | 1.957, 1.824, 1.769, 1.784, 1.728, 1.964, 1.727, 1.743, 1.792, 1.928, 1.754, 1.711, 1.809, 2.603, 1.695, 1.731, 1.735, 1.862, 2.151, 1.68 |
| sparse 4 | main | 0.8933, 1.376, 1.08, 3.115, 0.8929, 0.7364, 0.717, 0.9083, 2.65, 1.06, 0.8068, 0.9054, 0.96, 1.027, 0.8564, 0.86, 0.9591, 0.8662, 0.968, 1.005 |
| sparse 4 | candidate | 0.6953, 1.294, 0.9301, 2.393, 0.6618, 0.674, 0.8961, 0.9104, 2.439, 0.8267, 0.6809, 0.823, 0.9671, 0.9784, 0.9098, 0.8027, 0.8347, 0.7009, 0.7972, 0.8961 |
| sparse 8 | main | 1.437, 2.37, 4.621, 1.839, 1.355, 1.339, 1.33, 1.447, 4.67, 1.392, 0.9264, 1.03, 1.39, 1.185, 1.071, 1.022, 1.348, 1.041, 2.085, 1.354 |
| sparse 8 | candidate | 1.368, 2.164, 7.675, 2.711, 1.35, 1.295, 1.27, 1.233, 2.967, 1.717, 0.7897, 0.9058, 1.301, 0.9552, 0.9477, 0.8406, 1.296, 0.791, 1.248, 1.278 |
| sparse 32 | main | 4.882, 8.089, 6.637, 5.673, 4.247, 4.841, 3.735, 4.217, 9.42, 5.536, 3.65, 3.613, 4.247, 4.349, 3.623, 4.097, 3.862, 3.651, 3.781, 4.319 |
| sparse 32 | candidate | 4.573, 10.2, 4.712, 5.984, 3.75, 4.766, 3.933, 4.242, 4.018, 4.991, 3.56, 3.622, 4.28, 4.663, 3.496, 3.427, 3.653, 3.615, 3.961, 4.186 |
| sparse 128 | main | 18.41, 21.15, 15.89, 14.74, 17.01, 15.86, 31.61, 21.42, 17.17, 13.32, 13.96, 16.35, 17.78, 14.67, 14.15, 14.81, 14.09, 13.85, 16.31, 12.71 |
| sparse 128 | candidate | 20.46, 20.36, 16.87, 16.03, 20.44, 17.3, 45.89, 16.55, 19.69, 14.68, 14.48, 15.67, 15.48, 15.96, 13.48, 16.41, 13.57, 14.51, 16.44, 15.17 |

</details>

## Lower-load four-round confirmation

Same binaries/settings/order; four rounds per revision, 100 samples per fixture.
This independent series confirms improvements in every solid parallel fixture and
sparse 4/8/32. Sparse 128 and single-chunk timings still vary; overlapping ranges
leave their regression status unresolved.

| Fixture | Chunks | Main ms | Rayon ms | Change |
| --- | ---: | ---: | ---: | ---: |
| Solid | 1 | 0.958 (0.933–1.279) | 1.007 (0.897–1.242) | +5.0% |
| Solid | 3 | 2.482 (2.445–2.539) | 2.484 (2.472–2.571) | +0.1% |
| Solid | 4 | 1.365 (1.045–1.736) | 1.179 (1.093–1.206) | -13.7% |
| Solid | 8 | 1.781 (1.316–1.940) | 1.653 (1.244–1.737) | -7.2% |
| Solid | 32 | 3.903 (3.671–4.175) | 3.337 (3.151–4.068) | -14.5% |
| Solid | 128 | 10.705 (9.404–11.990) | 10.011 (9.666–10.500) | -6.5% |
| Sparse | 1 | 0.609 (0.521–0.686) | 0.563 (0.543–0.817) | -7.5% |
| Sparse | 3 | 1.799 (1.713–2.628) | 1.788 (1.755–2.618) | -0.6% |
| Sparse | 4 | 0.941 (0.889–1.108) | 0.869 (0.747–0.877) | -7.6% |
| Sparse | 8 | 1.353 (1.033–1.404) | 1.300 (0.937–1.906) | -3.9% |
| Sparse | 32 | 4.104 (3.593–4.962) | 3.756 (3.581–5.071) | -8.5% |
| Sparse | 128 | 14.980 (13.460–22.090) | 15.845 (12.970–17.120) | +5.8% |

## Existing greedy-mesh benchmarks

Also ran the existing `quantum-mesh --bench greedy` suite twice on each revision,
back to back, 20 samples per fixture. Its source/dependencies are unchanged and
both builds emitted the identical executable (SHA-256
`8a07a43b99c3f04a115c5bf4ae9662678217fce9298f0cbf8e37290be74add6e`).
This control helps quantify shared-machine noise, not the Rayon speedup.
One-minute load was approximately 4.3–4.6. The identical binary varies by up
to 2.9% between these control medians, matching the scale of the small slower
chunk-sync medians above. Milliseconds, median (range) of two run
medians; terrain arguments are chunks per side.

| Fixture | Chunks | Main ms | Rayon ms | Change |
| --- | ---: | ---: | ---: | ---: |
| Chunk | Checkerboard | 0.915 (0.886–0.944) | 0.894 (0.885–0.903) | -2.3% |
| Chunk | HollowShell | 0.510 (0.503–0.517) | 0.498 (0.480–0.517) | -2.3% |
| Chunk | Solid | 1.504 (1.495–1.513) | 1.470 (1.452–1.488) | -2.3% |
| Chunk | Sparse | 0.168 (0.165–0.170) | 0.171 (0.168–0.173) | +1.8% |
| Terrain | 2 | 8.881 (8.826–8.937) | 9.119 (9.053–9.184) | +2.7% |
| Terrain | 4 | 45.105 (43.840–46.370) | 45.635 (43.170–48.100) | +1.2% |
| Terrain | 8 | 243.750 (237.200–250.300) | 236.800 (234.200–239.400) | -2.9% |

## Correctness and platform evidence

`rayon_chunk_batches_preserve_serial_order_and_vertex_bits` repeatedly compares
0/1/3/4/7/8/27/64 dirty chunks with serial output, including empty chunks, negative
seams and AO. Positions, normals and color/AO floats match bit for bit. Existing
quad-budget order/failure/recovery tests pass. `cargo xtask validate --strict`
passed all 15 gates, including all 904 workspace tests and doctests;
`cargo xtask lint` passed all three gates. CI macOS, rendering determinism, Linux,
headless UI snapshots and generated before/after screenshots passed on `67c88d5`.
Welcome and both editor themes are byte-identical in before/after PNG captures.
Native AppKit/Metal verification comes from CI, not these Linux CPU benchmarks.

## Post-rewrite, current-main confirmation

After merging rewritten `origin/main` `1d72fdc` into candidate `a1449c0`,
rebuilt both revisions with the same fixture and settings above. Neither
production meshing implementation changed in this merge. Main was prepared in
a detached worktree from the fresh history; no pre-rewrite commit is reachable
from the candidate. Immutable executable SHA-256 values:

- Main: `f8ab17fef8087b8c2414786d5fe3815d036418f698578e16ab2fd282e42ee8b8`
- Candidate: `10698045af519e3f6cf82cef57a1d692a6ac2bb8b67c23eb33fb37c22f87c65a`

An initial two-pair diagnostic started at load 1.3–1.8, but other builds started
during timing and large-case medians became unstable. The following complete
ten-pair series alternates order, with 100 samples per fixture per run. No
builds/tests from this task overlap timing. Other build activity is recorded;
one-minute load spans 7.50–18.45. These are confirmation diagnostics under
contention, not a replacement for the longer acceptance series above.
Milliseconds: median (minimum–maximum) of ten run medians.

| Fixture | Chunks | Main ms | Rayon ms | Change |
| --- | ---: | ---: | ---: | ---: |
| Solid | 1 | 0.897 (0.863–1.451; n=10) | 0.894 (0.865–1.409; n=10) | -0.3% |
| Solid | 128 | 16.985 (11.560–31.120; n=10) | 16.440 (11.620–44.010; n=10) | -3.2% |
| Solid | 3 | 2.356 (2.249–7.231; n=10) | 2.356 (2.245–6.285; n=10) | -0.0% |
| Solid | 32 | 6.730 (3.556–13.150; n=10) | 5.934 (3.431–19.990; n=10) | -11.8% |
| Solid | 4 | 1.845 (0.877–4.161; n=10) | 1.450 (0.840–6.252; n=10) | -21.5% |
| Solid | 8 | 2.521 (1.625–5.672; n=10) | 2.526 (1.572–9.059; n=10) | +0.2% |
| Sparse | 1 | 0.512 (0.486–0.603; n=10) | 0.518 (0.499–0.542; n=10) | +1.0% |
| Sparse | 128 | 20.120 (14.130–44.060; n=10) | 19.055 (14.600–53.230; n=10) | -5.3% |
| Sparse | 3 | 1.648 (1.595–1.727; n=10) | 1.681 (1.596–2.501; n=10) | +2.0% |
| Sparse | 32 | 6.534 (4.414–12.760; n=10) | 4.999 (3.412–19.750; n=10) | -23.5% |
| Sparse | 4 | 0.908 (0.647–4.040; n=10) | 1.180 (0.606–2.541; n=10) | +30.0% |
| Sparse | 8 | 1.571 (1.314–5.111; n=10) | 1.458 (1.255–6.873; n=10) | -7.2% |

Sparse 4's ratio of aggregate medians is +30.0%, but its median paired
per-round change is +1.7%, with a paired bootstrap 95% interval of −19.5 to
+13.4% (10,000 resamples, seed 95). Timing spikes move between revisions; this
series does not resolve a regression or a speedup for that fixture. The earlier
twenty-pair interval and lower-load confirmation both favor Rayon. The unchanged
serial cases remain within 2%; no threshold change is justified by this noisy
confirmation. All slower medians are retained above.

Fresh Linux validation after the merge: `cargo xtask validate --strict` passes
16 gates (zero failures/skips), including 911 workspace tests and doctests;
`cargo xtask lint` passes all three gates. The new spelling gate required
excluding compressed `.git` metadata; authored source remains checked. A stale
shared-target xtask library was rebuilt from this checkout before verification.

## Senior review: fresh current-main comparison

Main `523138e` versus reviewed candidate `4afe98a161e458f0518b350d0232d7cfff485ef9`, built from the fresh clone
with the same fixture and settings above. The candidate adds explicit one-, two-
and eight-worker determinism checks, serial fallback coverage, and budget/recovery
coverage across the parallel threshold. Production scheduling remains unchanged.
The benchmark-only dead-code allowance is now a documented expectation.

Twenty paired rounds per revision, 100 samples per fixture; reverse revision
order on alternate rounds. No task-owned builds or tests overlap measurements.
Linux x86-64, eight-core Intel Xeon VM, Rust 1.99.1; one-minute load ranged
5.36–10.16. This is a shared machine, not an idle reference.

Immutable executable SHA-256 values:

- Baseline: `f8ab17fef8087b8c2414786d5fe3815d036418f698578e16ab2fd282e42ee8b8`
- Candidate: `6e194d79af3355903351cc1ef39e83333513304718c1c15d82897175dc23dc4a`

Milliseconds are median (minimum–maximum) of twenty run medians. The last column
is the median paired percentage change and its bootstrap 95% interval (10,000
resamples of paired changes, seed 95). Negative change is faster.

| Fixture | Chunks | Main ms | Rayon ms | Aggregate change | Paired change (95% interval) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Solid | 1 | 0.859 (0.795–0.886) | 0.863 (0.792–0.937) | +0.5% | +1.5% (+0.5 to +2.0) |
| Solid | 3 | 2.210 (2.076–2.334) | 2.237 (2.135–2.543) | +1.2% | +0.9% (-0.3 to +2.3) |
| Solid | 4 | 1.070 (0.884–1.643) | 0.891 (0.831–1.506) | -16.7% | -10.7% (-18.7 to -7.1) |
| Solid | 8 | 1.570 (0.952–1.828) | 1.503 (0.896–2.006) | -4.2% | -3.8% (-5.3 to +3.7) |
| Solid | 32 | 3.175 (2.874–4.136) | 3.463 (2.745–4.184) | +9.1% | +3.8% (-2.5 to +6.3) |
| Solid | 128 | 9.623 (8.462–19.370) | 10.034 (8.771–14.300) | +4.3% | +0.6% (-9.3 to +5.2) |
| Sparse | 1 | 0.492 (0.449–0.509) | 0.493 (0.451–0.536) | +0.3% | +0.0% (-1.3 to +0.9) |
| Sparse | 3 | 1.597 (1.496–1.721) | 1.623 (1.473–1.759) | +1.7% | +1.9% (+0.4 to +2.1) |
| Sparse | 4 | 0.805 (0.639–1.118) | 0.629 (0.556–1.174) | -21.9% | -17.5% (-27.0 to -8.9) |
| Sparse | 8 | 1.227 (0.726–1.949) | 1.158 (0.600–1.237) | -5.6% | -6.3% (-12.1 to -5.1) |
| Sparse | 32 | 3.442 (3.093–4.041) | 3.306 (3.126–4.058) | -3.9% | -1.9% (-5.1 to +3.0) |
| Sparse | 128 | 12.215 (11.310–15.540) | 13.455 (11.640–19.570) | +10.2% | +2.6% (-1.3 to +9.6) |

Four-chunk solid/sparse batches improve 16.7%/21.9%. However, solid 32 is +9.1%
and sparse 128 is +10.2% by aggregate medians. Their paired intervals include
zero, but also allow material slowdowns. Overlap with zero is not evidence of
non-regression. The unchanged serial paths move by up to 1.7%, which does not
justify dismissing the larger-case differences as noise.

A separate twenty-round, three-way diagnostic compared main, the reviewed
candidate, and a temporary `with_max_len(1)` variant. Each round rotated and
reversed revision order; all twelve fixtures used 100 samples. This tested
whether finer work splitting would improve balancing. It did not resolve the
sparse-128 slowdown, so the variant was discarded and is not in the PR.
One-minute load ranged
4.73–9.13. Median of run medians (ms):

| Fixture | Chunks | Main | Reviewed Rayon | Finer splitting | Variant vs main | Paired variant change (95% interval) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Solid | 1 | 0.868 | 0.870 | 0.873 | +0.5% | +1.6% (-0.3 to +5.3) |
| Solid | 3 | 2.263 | 2.248 | 2.260 | -0.1% | +1.0% (-0.1 to +1.6) |
| Solid | 4 | 1.129 | 1.045 | 1.080 | -4.3% | -10.0% (-15.4 to +0.1) |
| Solid | 8 | 1.362 | 1.484 | 1.481 | +8.7% | -4.5% (-6.6 to +13.1) |
| Solid | 32 | 3.292 | 3.223 | 3.192 | -3.0% | +0.2% (-5.2 to +3.7) |
| Solid | 128 | 8.856 | 9.296 | 9.363 | +5.7% | +1.6% (-1.8 to +6.8) |
| Sparse | 1 | 0.497 | 0.506 | 0.499 | +0.5% | +0.3% (-0.9 to +1.6) |
| Sparse | 3 | 1.623 | 1.631 | 1.647 | +1.5% | +1.3% (-0.2 to +3.0) |
| Sparse | 4 | 0.804 | 0.648 | 0.666 | -17.1% | -15.9% (-20.9 to -11.5) |
| Sparse | 8 | 1.030 | 0.919 | 0.857 | -16.8% | -10.1% (-19.4 to -5.1) |
| Sparse | 32 | 3.279 | 3.264 | 3.219 | -1.8% | -2.3% (-3.8 to -0.3) |
| Sparse | 128 | 12.285 | 12.660 | 12.840 | +4.5% | +4.8% (+0.3 to +6.0) |

The second series still has a slower reviewed sparse-128 median (12.285 →
12.660 ms, +3.1%); the finer variant is slower again (12.840 ms, +4.5%).
Neither series satisfies the issue's no-regression requirement. Obtain a
controlled comparison with sufficiently narrow uncertainty before checking that
acceptance item; do not adjust thresholds or weaken checks to accept these results.

<details>
<summary>Fresh paired comparison: per-run medians in milliseconds</summary>

| Fixture | Revision | Runs 1–20 |
| --- | --- | --- |
| solid 1 | baseline | 0.8861, 0.8516, 0.8738, 0.8595, 0.8704, 0.8619, 0.8361, 0.8541, 0.8469, 0.8388, 0.8658, 0.8592, 0.8532, 0.8095, 0.8593, 0.8632, 0.8674, 0.8705, 0.7946, 0.8134 |
| solid 1 | candidate | 0.9372, 0.8632, 0.8931, 0.875, 0.8716, 0.872, 0.8498, 0.8597, 0.8519, 0.8563, 0.8672, 0.8635, 0.8714, 0.8627, 0.875, 0.85, 0.8847, 0.8633, 0.7919, 0.8572 |
| solid 3 | baseline | 2.237, 2.258, 2.334, 2.208, 2.178, 2.133, 2.197, 2.198, 2.19, 2.21, 2.192, 2.318, 2.184, 2.236, 2.211, 2.244, 2.264, 2.214, 2.076, 2.229 |
| solid 3 | candidate | 2.234, 2.229, 2.257, 2.225, 2.237, 2.202, 2.543, 2.246, 2.244, 2.176, 2.242, 2.204, 2.236, 2.225, 2.252, 2.245, 2.273, 2.202, 2.135, 2.254 |
| solid 4 | baseline | 1.173, 1.144, 0.9627, 0.8843, 0.9127, 0.8887, 0.9155, 1.013, 1.479, 0.9185, 1.643, 1.383, 1.299, 1.045, 1.061, 1.078, 1.109, 1.147, 1.034, 1.103 |
| solid 4 | candidate | 0.9396, 0.8691, 0.9316, 0.8674, 0.8312, 0.8372, 0.834, 0.9696, 0.8476, 1.037, 1.506, 1.315, 1.116, 0.863, 0.8473, 0.912, 0.8582, 1.006, 0.9397, 0.8678 |
| solid 8 | baseline | 0.9672, 1.068, 1.6, 1.58, 0.9515, 1.002, 1.572, 1.587, 1.01, 1.41, 1.628, 1.828, 1.577, 1.568, 1.615, 1.644, 1.615, 1.033, 1.247, 1.115 |
| solid 8 | candidate | 0.9438, 0.8962, 1.514, 1.521, 1.377, 1.398, 1.513, 1.526, 1.494, 1.493, 1.548, 1.546, 1.516, 0.921, 1.532, 1.478, 1.68, 0.9312, 1.289, 2.006 |
| solid 32 | baseline | 3.203, 3.453, 3.775, 3.113, 2.882, 2.934, 3.461, 2.916, 2.874, 4.136, 3.708, 3.694, 3.31, 3.459, 3.263, 3.068, 2.997, 3.146, 3.013, 3.088 |
| solid 32 | candidate | 3.392, 3.46, 3.51, 4.184, 3.067, 3.04, 3.601, 3.88, 3.53, 3.367, 4.052, 3.631, 3.489, 3.221, 3.466, 3.909, 3.002, 2.961, 2.745, 2.985 |
| solid 128 | baseline | 8.736, 9.64, 9.407, 11.66, 8.462, 12.34, 10.91, 8.919, 10.99, 11.25, 19.37, 8.671, 10.88, 10.08, 9.499, 8.725, 8.535, 9.606, 8.653, 10.89 |
| solid 128 | candidate | 10.57, 10.9, 8.784, 9.421, 10.2, 10.22, 9.404, 8.828, 11.24, 10.22, 14.3, 10.57, 8.875, 10.5, 8.771, 9.079, 8.884, 10.21, 9.651, 9.868 |
| sparse 1 | baseline | 0.489, 0.4604, 0.4956, 0.4684, 0.4962, 0.4957, 0.4768, 0.4892, 0.5042, 0.4826, 0.5022, 0.4947, 0.4515, 0.4489, 0.4765, 0.4977, 0.4807, 0.5041, 0.5088, 0.4954 |
| sparse 1 | candidate | 0.493, 0.4511, 0.5002, 0.4711, 0.4936, 0.4999, 0.4696, 0.4894, 0.4983, 0.5009, 0.5357, 0.4866, 0.4564, 0.4962, 0.4568, 0.4885, 0.4639, 0.5035, 0.5088, 0.5036 |
| sparse 3 | baseline | 1.6, 1.496, 1.61, 1.559, 1.58, 1.655, 1.6, 1.591, 1.603, 1.579, 1.721, 1.63, 1.586, 1.638, 1.528, 1.593, 1.583, 1.628, 1.613, 1.574 |
| sparse 3 | candidate | 1.612, 1.473, 1.641, 1.605, 1.625, 1.625, 1.629, 1.623, 1.635, 1.635, 1.759, 1.634, 1.594, 1.618, 1.558, 1.605, 1.624, 1.665, 1.564, 1.574 |
| sparse 4 | baseline | 0.8319, 0.9934, 0.639, 0.7627, 0.6719, 1.02, 1.118, 1.027, 0.6721, 0.6554, 0.8024, 0.9969, 0.8879, 0.7779, 0.6988, 0.8913, 0.8083, 0.7379, 0.6916, 0.8164 |
| sparse 4 | candidate | 0.7954, 0.9151, 0.5761, 0.9943, 0.5556, 0.5949, 0.808, 0.6434, 0.5674, 0.6099, 1.174, 0.7165, 0.5718, 0.6138, 0.5755, 0.6565, 0.6484, 0.8682, 0.6106, 0.5801 |
| sparse 8 | baseline | 1.24, 1.016, 0.7761, 1.329, 1.232, 1.241, 1.227, 1.139, 1.255, 1.226, 1.949, 0.911, 0.726, 0.7439, 0.7262, 1.248, 0.9857, 1.25, 1.271, 0.7702 |
| sparse 8 | candidate | 1.168, 0.7555, 1.112, 1.168, 1.174, 1.128, 1.148, 1, 1.185, 1.237, 1.23, 0.6748, 0.8486, 0.5996, 0.639, 1.209, 0.8861, 1.176, 1.192, 1.173 |
| sparse 32 | baseline | 3.232, 3.217, 3.093, 3.874, 3.442, 3.225, 3.701, 3.449, 3.883, 3.37, 3.464, 3.82, 3.291, 3.324, 3.185, 3.441, 3.494, 4.041, 3.425, 3.653 |
| sparse 32 | candidate | 3.126, 3.211, 3.139, 3.689, 3.702, 3.419, 3.382, 3.191, 4.058, 3.184, 3.2, 3.231, 3.196, 3.214, 3.17, 3.878, 3.851, 3.724, 3.589, 3.616 |
| sparse 128 | baseline | 11.42, 11.68, 11.67, 14.92, 11.39, 11.31, 12.02, 15.54, 13.23, 14.4, 14.3, 12.67, 11.88, 13.75, 12.25, 14.32, 14.57, 11.6, 12.18, 11.67 |
| sparse 128 | candidate | 11.76, 11.83, 13.95, 19.57, 11.64, 11.97, 15.47, 15.4, 12.56, 12.31, 13.95, 13.93, 12.34, 13.52, 13.39, 13.82, 13.28, 14.46, 13.56, 11.91 |

</details>
