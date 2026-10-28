# Salsa pipeline spike — 2026-10-27

**Recommendation: don't adopt Salsa for this pipeline; don't merge this prototype.**
The current path is already better for interactive small edits. Salsa offers a
real bulk-edit improvement here, but not a general speedup or simpler ownership:
single edits are 3.36× slower, memory roughly doubles, and full rebuilds/palette
changes do not improve consistently. Investigate the existing exact-cell diff
cost separately before adding a query database. No Salsa migration is proposed.

## Scope and files

- New unpublished `crates/incremental-spike/`: binary, Salsa queries, scene/Engine
  fixtures, private mesher adapter, differential tests, build script, runner and
  README. Production code removed: **zero**. New Rust: **849 lines**, including
  324 query/adapter-coordination lines and 116 test lines; runner: 55 lines.
- Root `Cargo.toml`: one shared Salsa dependency, exactly `=0.28.5`, defaults off,
  enabling only `macros` and `inventory`. `Cargo.lock`: 98 added lines, nine new
  registry packages plus the spike; **no existing package version changed**.
- No existing crate source, app/UI/renderer/site/script/workflow/xtask file was
  edited. No public API changed. No commits, pushes or git-config changes.
- Base checkout: `7d58723c3fca1b502b72c4d9838255533fbc24ff`.

## Current invalidation, verified in code

`voxelite-core/src/chunk.rs` stores 16³ chunks behind copy-on-write `Arc`s.
`changed_chunks` scans chunk identities, skipping shared Arcs and comparing
non-shared contents; `changed_cells` enumerates exact differences in those chunks.
`Engine` stages/validates commands atomically and advances revision/history only
on actual changes. The spike leaves all of that authoritative.

`voxelite-render/src/chunk_mesh.rs` retains a snapshot and ordered nonempty meshes.
Palette, bounds, layer order or visibility changes force a rebuild. Other edits
use visible-layer cell differences, skip occluded changes, and invalidate only
chunks intersecting each changed cell's ±2-cell culling/AO halo. It reconstructs a
20³ local block and bakes linear color/AO into meshes. At least four dirty chunks
use up to eight scoped native workers. GPU buffer replacement/deletion follows a
deterministic dirty order and preserves the transient quad-budget failure rule.

Occupancy has its own exact cell/brick invalidation; palette changes do not alter
occupancy. `RasterCache` shares occupancy via `Arc<Mutex<_>>`; CPU geometry/AO
still run per render. App frame identity skips scene synchronization on camera
motion. These are separate caches, not one existing query graph.

`voxelite-render-plan` is a small logical kernel-selection list, independent of
document geometry. `Host::render_plan` invokes enabled callbacks, scopes settings,
records timings, quarantines panics and discards a faulted owner's staged passes.
Memoizing that entire operation would change observable behavior.

`voxelite-mesh::greedy_mesh` validates and meshes a complete visible composite.
GLB export uses it; CPU PNG rendering also uses it. Its cross-chunk greedy quads
and output ordering differ from GPU chunk/AO meshes. Thumbnail caching is already
persistent, keyed by canonical path and version/mtime/length; unchanged files
skip rendering, changed files render under a bounded global mutex and recheck the
stamp before publishing. Salsa does not replace that disk validity check.

## Prototype and fairness

The build script includes the production chunk cache/mesher source verbatim in
`OUT_DIR`, removes only its module-local test declarations, and appends a private
mesher entry point. Nothing is exported from or written into the renderer.

Salsa inputs hold document membership/settings, palette and per-coordinate layer
chunks, including stable identities for empty positions. Tracked queries compute
linear colors, padded composites, production chunk meshes, an ordered geometry
list and the independent render plan. Padded-block equality backdates unchanged
halos. `Arc` results and `no_eq` avoid comparing entire vertex buffers. High/
medium/low durability separates palette/identity, geometry and view settings.
Both paths use at most eight scoped threads; Salsa queries run on database clones.

Input synchronization still needs `changed_chunks` and layer/bounds bookkeeping.
Core exposes `&Chunk`, not an owned shared chunk, so this spike clones changed
chunk contents. A future owned-chunk API could avoid roughly 21 MiB of initial
copying in this scene; it would not remove the ~41 MiB padded-block cache. This
prototype would replace about 100 lines of cache synchronization with 324 lines of
query/adapter coordination; it does not demonstrate code simplification.

The plan probe is a pure `RenderPlan::standard()` equivalent, **not host callback
benchmarking**. Its repeated call averages about 30 ns current / 38 ns Salsa.

## Hardware and measurement

Linux x86_64 KVM VM, Intel Xeon family 6/model 207, 8 exposed vCPUs, 16,002 MiB
RAM, no swap. Rust **1.99.1 (58cf8f9c3 2026-10-13)**, release thin LTO, one
codegen unit. Shared VM; one-minute load was **3.36–5.58** during the main suite.
These are diagnostic numbers, not a pinned-machine performance gate.

Deterministic terraced/sloped terrain: **9,945,088 voxels, 2,688 occupied chunks**,
256³ bounds, one visible layer, three colors, 396,160 exposed faces and about
one million vertices. This is a new fixture, not the official `terrain10m` scene.
The existing benchmark-only voxel admission feature is explicitly enabled.

Each cell below is the median of **three fresh-process medians**, each based on
nine timed iterations after two warmups. Modes run sequentially with order
alternated. Parentheses show the three process medians' min–max, not confidence
intervals. Raw files retain all 27 samples. Timed work starts from already
committed snapshots and includes input sync, discovery, meshing, aggregation and
plan retrieval. Engine transactions, fixture loading, digests and old-cache drop
are excluded. Cold rebuild includes fresh cache/database construction. No GPU,
occupancy, encoding or disk I/O is measured.

| Workload | Current ms (range) | Salsa ms (range) | Salsa/current |
| --- | ---: | ---: | ---: |
| Single surface recolor | 0.696 (0.684–0.699) | 2.337 (2.311–2.350) | 3.36× |
| Boundary voxel removal | 0.921 (0.829–1.021) | 3.808 (3.670–3.848) | 4.13× |
| Bulk brush | 138.025 (132.103–138.252) | 59.966 (34.253–69.126) | 0.43× |
| Full rebuild | 140.941 (135.951–184.690) | 156.352 (149.894–162.731) | 1.11× |
| Palette RGB change | 133.752 (118.577–137.997) | 132.888 (131.286–133.406) | 0.99× |
| Identical snapshot | 0.108 (0.100–0.114) | 0.103 (0.103–0.106) | 0.96× |
| Lighting toggle | 0.101 (0.100–0.104) | 0.106 (0.104–0.110) | 1.05× |

The bulk brush changes **16,384 cells in 480 chunks**, requiring 512 meshes after
halo expansion in both paths. Salsa recomputes 896 padded composites. Single and
boundary edits execute 1/4 mesh queries and 6/21 padded queries respectively.
Palette changes execute 2,688 mesh queries and zero padded queries. No-op and
lighting changes execute zero mesh queries. The initial untuned Salsa version
spent ~1.27 ms validating geometry for lighting changes; durability tuning removes
that avoidable cost and is included in all final numbers above.

CPU-affinity sensitivity (`taskset -c 0`, verified one available worker; same
three-process/nine-sample protocol): single edit **0.705 → 2.195 ms**, bulk
**293.833 → 205.927 ms**, full rebuild **933.557 → 956.744 ms**, palette
**935.851 → 918.059 ms**. Bulk process medians were 290.642–294.482 ms current
and 204.078–208.863 ms Salsa. The direction persists without thread-scheduling
advantages, while the bulk speedup is smaller than in the eight-worker suite.

An isolated nine-sample diagnostic measures bulk `changed_cells` at **94.76 ms**
versus **1.79 ms** for `changed_chunks`. This supports attributing much of the bulk
win to avoiding exact-cell enumeration, not to memoization alone. A simpler
manual alternative was not implemented or benchmarked; its potential is an
inference, not a measured speedup. Today's scheme is near-minimal in remeshing
scope for small edits, but its bulk diff is demonstrably expensive.

Fresh-process warm RSS for the single-edit workload is **66.8 MiB current /
131.1 MiB Salsa**, above input-only RSS of ~23.7 MiB: cache/process growth is
**43.1 / 107.4 MiB**, an extra **64.2 MiB**. Median post-workload RSS is
125.5 / 219.2 MiB for bulk, 65.4 / 142.8 MiB for full rebuild, and 125.1 / 203.1
MiB for palette changes. Bulk peak RSS is 135.9 / 226.4 MiB. RSS includes allocator
retention and worker stacks; it is not a precise live-allocation count.

## Dependency and compile cost

Crates.io `cargo search`/`cargo info` verified **Salsa 0.28.5** as latest stable;
its metadata still labels the project experimental. License: **Apache-2.0 OR
MIT**; upstream MSRV 1.88 fits this workspace. Only macros/inventory are enabled,
avoiding default Rayon/accumulator/unstable features. See the
[versioned Salsa API](https://docs.rs/salsa/0.28.5/salsa/).

Normal/build dependency closure grows from **29 to 56 packages** including the
spike and its workspace dependencies: +27 packages in this binary's graph, nine
new registry entries in the workspace lockfile. Existing workspace packages
account for the difference.

Fresh separate target directories, already-downloaded sources, same release
settings: control without Salsa **12.00 s**, Salsa **14.91 s** (**+2.91 s / 24%**).
One clean build per mode, so indicative only. Binary grows **1,160,168 → 1,628,704
bytes** (+40%); target artifacts **75.35 → 125.84 MB**. This measures the whole
spike with/without its query implementation, not an application release build.

## Threading, compatibility and risks

Salsa's storage is Send but not Sync; cloneable database handles support parallel
queries, verified by the prototype's background-thread test. Input mutation
cancels outstanding computations and waits for other handles to drop. A GUI-owned
database with long-lived worker clones could therefore block the GPUI main
thread. Captured exports must not assume database clones are independent MVCC
snapshots. See [Storage](https://docs.rs/salsa/0.28.5/salsa/struct.Storage.html) and
the cancellation contract in
[Database](https://docs.rs/salsa/0.28.5/salsa/trait.Database.html).

Any future adoption would need a worker-owned database, explicit cancellation/
join policy and owned immutable output handoff. Engine history/atomicity, host
callbacks/quarantine, GPU upload bookkeeping and stale-frame identity checks
would remain. Empty input identities live until database disposal; document
closing/eviction requires an explicit lifecycle. Those are additional obligations,
not code this spike removes.

Prototype limits: the Salsa aggregate enforces the final quad budget but does not
replicate transient per-update failure ordering, incremental GPU upload/delete
lists, or asynchronous cancellation recovery. Full export/thumbnail reuse would
require preserving their different geometry/order and settings/file identities.
No export/thumbnail speedup is claimed. No GPU quality setting changed.

## Verification and artifacts

All completed successfully:

- `cargo fmt --all -- --check` and `git diff --check`.
- `cargo clippy -p incremental-spike --all-targets --locked -- -D warnings`,
  also with `--features large-scenes` and with `--no-default-features`.
- `cargo test -p incremental-spike --locked`: **4 passed**, also **4 passed**
  with `--features large-scenes`. Tests compare every vertex float bit and face
  count, exercise edits/additions/deletions, negative coordinates, layer order/
  visibility, palette/bounds, memo reuse, halo equality and background clones.
- All **42 paired geometry digest/count comparisons** across the final eight-
  worker and single-core benchmark suites passed (21 pairs per suite).
- `cargo test -p voxelite-render --locked --test boundary
  normal_workspace_dependencies_preserve_renderer_boundary`: **1 passed**.
- `cargo xtask tidy`: passed. The nine untracked prototype files were also
  inspected for size/junk; all are below 11 KiB. Tidy examines tracked files.
- `cargo deny check`: advisories, bans, licenses and sources passed; existing
  multiple-version warnings remain. `cargo machete`: no unused dependencies.
- `python3 -B scripts/check-plugin-deps.py`: passed;
  `python3 -B -m unittest discover -s scripts/tests -p test_plugin_deps.py`:
  **18 passed**.
- Release builds with and without Salsa, using `--locked`: passed.

No existing public API changed, so there are no changed-API dependents to test.
Full GPUI/macOS workspace tests, AppKit/Metal execution, pinned GPU visual checks,
browser/wasm behavior and actual export/thumbnail end-to-end performance were not
verified. Linux mesh-bit equality is not a claim of macOS presentation coverage.

Reproduce using `crates/incremental-spike/README.md`. Artifacts outside the repo:
`salsa.bench.json` and `.log` (final tuned suite), `salsa.untuned.bench.json`,
`salsa.single-core.bench.json` and `.log`, `salsa.diff.json`, `salsa.compile.json`,
compile logs, dependency trees and `salsa.deny.log`. Prototype and all changes
remain uncommitted.
