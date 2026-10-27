# Profiling reference, main d81e670 (2026-10-26)

Measured with the one-off `gpu::Worker` profiling instrumentation before this harness existed. Render code between d81e670 and the harness base (aa62c36) is unchanged, so these numbers describe both.

**The M2 mini was under CI/build load (load average 12-80) during these runs, so M2 numbers are inflated and noisy. They are a provisional reference only. Re-record with `bench-baseline` on an idle mini before gating on them.** M5 Pro was in normal use.

Machine-readable data: [`profiling-2026-10-26-d81e670.json`](profiling-2026-10-26-d81e670.json). Harness-format baselines (`bench/baseline/<machine_key>.json`) come from the `bench-baseline` workflow run and have different phase names.

Viewport 1032x808 logical, scale 2 (4128x3232 px target), 4x MSAA, exact voxel shadows.

| Scene | Phase | M2 p50 ms | M2 p95 ms | M2 fps | M5 Pro p50 ms | M5 Pro fps | draws | tris |
|---|---|---|---|---|---|---|---|---|
| autumn | cold | 289.3 | 289.3 | 3.5 | 346.6 | 2.9 | 30 | 34402 |
| autumn | edit_1voxel | 21.6 | 22.9 | 46.3 | 8.0 | 124.8 | 30 | 34402 |
| autumn | edit_cube16 | 44.6 | 48.2 | 22.4 | 17.4 | 57.3 | 30 | 33012 |
| autumn | edit_cube40 | 124.2 | 157.9 | 8.1 | 49.7 | 20.1 | 49 | 30636 |
| autumn | edit_recolor | 83.7 | 100.9 | 11.9 | 37.4 | 26.7 | 30 | 34230 |
| autumn | orbit_close | 41.7 | 66.4 | 24.0 | 14.3 | 70.1 | 30 | 34402 |
| autumn | orbit_fit | 21.2 | 26.4 | 47.1 | 7.8 | 128.5 | 30 | 34402 |
| autumn | same_fit | 21.2 | 21.4 | 47.2 | 7.8 | 128.7 | 30 | 34402 |
| terrain1m | cold | 817.1 | 817.1 | 1.2 | 317.9 | 3.1 | 260 | 149192 |
| terrain1m | edit_1voxel | 44.2 | 50.3 | 22.6 | 16.6 | 60.1 | 260 | 149200 |
| terrain1m | edit_cube16 | 51.0 | 73.5 | 19.6 | 22.3 | 44.9 | 260 | 148138 |
| terrain1m | edit_cube40 | 212.5 | 253.1 | 4.7 | 54.5 | 18.4 | 272 | 147584 |
| terrain1m | edit_recolor | 801.9 | 1196.1 | 1.2 | 259.1 | 3.9 | 257 | 150768 |
| terrain1m | edit_slab64 | 147.2 | 213.9 | 6.8 | 46.5 | 21.5 | 259 | 148838 |
| terrain1m | orbit_close | 76.6 | 127.4 | 13.1 | 23.1 | 43.3 | 260 | 149192 |
| terrain1m | orbit_fit | 30.8 | 46.8 | 32.4 | 10.4 | 95.8 | 260 | 149192 |
| terrain1m | same_fit | 21.4 | 22.8 | 46.8 | 7.8 | 128.2 | 260 | 149192 |
| sponge1m | cold | 1135.0 | 1135.0 | 0.9 | 522.0 | 1.9 | 512 | 912356 |
| sponge1m | edit_recolor | 1275.5 | 1692.6 | 0.8 | 453.8 | 2.2 | 512 | 912356 |
| sponge1m | orbit_close | 24.3 | 35.2 | 41.2 | 8.0 | 124.5 | 512 | 912356 |
| sponge1m | orbit_fit | 18.3 | 21.4 | 54.7 | 7.5 | 132.7 | 512 | 912356 |
| sponge1m | same_fit | 19.0 | 21.2 | 52.5 | 5.3 | 189.0 | 512 | 912356 |
| terrain10m | cold | 4983.7 | 4983.7 | 0.2 | 3304.7 | 0.3 | 1568 | 875968 |
| terrain10m | edit_1voxel | 249.9 | 358.7 | 4.0 | 26.8 | 37.3 | 1568 | 875966 |
| terrain10m | edit_cube16 | 413.1 | 543.0 | 2.4 | 38.2 | 26.2 | 1568 | 874912 |
| terrain10m | edit_cube40 | 512.2 | 648.3 | 2.0 | 68.5 | 14.6 | 1580 | 869710 |
| terrain10m | edit_recolor | 5819.8 | 10251.5 | 0.2 | 3116.1 | 0.3 | 1562 | 870500 |
| terrain10m | edit_slab64 | 161.9 | 471.6 | 6.2 | 46.1 | 21.7 | 1574 | 876144 |
| terrain10m | orbit_close | 111.8 | 181.4 | 8.9 | 31.8 | 31.4 | 1568 | 875968 |
| terrain10m | orbit_fit | 60.7 | 72.9 | 16.5 | 20.1 | 49.7 | 1568 | 875968 |
| terrain10m | same_fit | 45.8 | 46.0 | 21.8 | 15.6 | 64.2 | 1568 | 875968 |
