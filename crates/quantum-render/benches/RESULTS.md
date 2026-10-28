# Chunk meshing benchmark results

## Revisions and method

Issue #95 replaces per-sync scoped threads with a reusable private Rayon pool.
Measured on 2026-10-28: main `7d4e750` versus candidate `67c88d5` (the main
merge; subsequent changes only document these measurements). Main received the
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
There is no material regression beyond the measured spread. In particular, sparse
128 is +2.2%, with a paired interval of −1.4 to +10.5%; do not claim it speeds up.
These measurements support retaining the four-chunk threshold; serial operation
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
and the longer series above do not support changing the algorithm for those cases.

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
