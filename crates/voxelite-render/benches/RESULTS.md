# Chunk meshing benchmark diagnostics

Issue #95 replaces per-sync scoped threads with a reusable Rayon pool. The
implementation is covered by serial-reference comparisons, but **the improvement
and no-regression acceptance criterion is not established by these timings**.
Repeat on an otherwise idle machine before marking that item complete.

## Method

Baseline: `1a33c31`, with the same `chunks` benchmark added but without changing
the production mesher. Candidate: this PR. Rust 1.99.1, optimized bench profile,
Linux x86-64, eight available CPUs, both implementations capped at eight workers.
Each fixture has 100 Divan samples per run; three runs per revision. Fixture
construction and pool warmup are excluded; full synchronization and destruction
are included. AO, geometry, chunk size and render quality are unchanged. No GPU
is used by this benchmark.

```sh
CARGO_BUILD_JOBS=4 cargo bench -p voxelite-render --locked --bench chunks
```

Other workspace builds/tests ran concurrently. Observed system load averages
rose from roughly 6 to 21 on eight CPUs; results below are diagnostic, not an
authoritative speedup measurement. The large between-run spread also affects
the unchanged serial path. macOS/Metal performance and pinned visual checks
require their normal CI runners.

## Timings

Milliseconds; each cell is median of the three run medians (range of medians).

| Fixture | Chunks | Scoped threads | Rayon |
| --- | ---: | ---: | ---: |
| Solid | 1 | 0.782 (0.779–0.787) | 0.799 (0.789–0.827) |
| Solid | 3 | 2.019 (2.016–2.026) | 2.185 (2.041–2.321) |
| Solid | 4 | 1.496 (0.876–3.022) | 2.693 (1.470–2.902) |
| Solid | 8 | 2.748 (1.465–3.173) | 2.881 (2.830–5.558) |
| Solid | 32 | 8.742 (3.870–8.842) | 11.350 (9.345–20.010) |
| Solid | 128 | 23.700 (20.540–27.740) | 32.950 (22.960–40.020) |
| Sparse | 1 | 0.461 (0.459–0.481) | 0.462 (0.454–0.490) |
| Sparse | 3 | 1.482 (1.433–1.561) | 1.401 (1.389–1.498) |
| Sparse | 4 | 4.000 (1.734–4.007) | 2.143 (0.975–4.155) |
| Sparse | 8 | 4.976 (2.655–7.765) | 3.596 (2.508–5.451) |
| Sparse | 32 | 12.520 (7.169–15.490) | 8.218 (8.166–16.010) |
| Sparse | 128 | 26.730 (23.870–31.850) | 32.770 (25.670–33.240) |

The overlapping ranges do not rule out regressions, and several candidate
medians are worse. This evidence must not be described as meeting the performance
criterion. The unchanged small-batch path and deterministic geometry are tested
separately; neither substitutes for an idle-machine performance comparison.
