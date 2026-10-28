# Bevy ECS spike — do not adopt now

**Recommendation: keep the current model; leave this spike unmerged.** The existing
built-ins are mostly stateless callbacks, not competing owners of scene data. ECS
adds storage, scheduling and synchronization here without removing the Engine,
capability adapters, transaction admission or immutable publication boundary.
Revisit if Quantum gains many independently queried scene objects and profiling
shows substantial repeated cross-feature queries, or concrete synchronization
defects persist after centralizing admission/reconciliation. Require a measured
application-level benefit before migrating.

Reviewed checkout: `7d58723c3fca1b502b72c4d9838255533fbc24ff`, 2026-10-27. No existing
crate, root manifest, root lockfile, identifier or release asset was changed.
Nothing was committed, pushed or configured in Git.

**What exists today.** The premise of plugins keeping private scene state does not
describe these built-ins: MirrorPlugin, CommentsPlugin, GizmoGridPlugin,
ShadowsLightingPlugin, ToolsPlugin and PalettePlugin are unit structs. Feature
behavior is separated from durable document state and host-retained presentation
state. Ownership below was checked in implementation, not inferred from the roadmap.

| State | Actual owner and sharing path |
| --- | --- |
| Scene/document | `Document.content: Snapshot` owns layers, sparse voxel volumes, stable palette slots and comments; `Document` also owns versions/extensions. `Engine` holds an `Arc<Document>` plus bounded before/after history. Mutations prepare privately and commit atomically. [model.rs](../../crates/voxelite-core/src/model.rs:216), [engine.rs](../../crates/voxelite-core/src/engine.rs:130) |
| Selection/tool gesture | `Editor.layer`, `tool`, `color`, `box_start`; there is no generic scene-object selection collection here. `reconcile()` repairs a deleted active layer and clears the pending box after admitted edits, undo/redo and attached MCP updates. [editor.rs](../../crates/voxelite-app/src/editor.rs:399), [reconciliation](../../crates/voxelite-app/src/editor.rs:550) |
| Mirror | Three transient, validated settings in `HostedPlugin.settings`, retained across disable/enable. The stateless callback recursively rewrites eligible proposed commands using scoped `TransformCx`; core performs reflection during command execution. Settings are neither document history nor `.voxl` data. [host.rs](../../crates/voxelite-plugin-host/src/host.rs:312), [mirror callback](../../crates/plugins/voxelite-plugin-mirror/src/lib.rs:106) |
| Comments/pins | Durable comments and messages are in Snapshot and edited through core commands. `Editor.comments_state` holds selected thread, numbering, draft/filter/placement state. CommentsPanel returns proposals; app admits commands before adopting that state. Views and overlays read capabilities. [comments API](../../crates/voxelite-plugin-api/src/comments.rs:10), [panel](../../crates/plugins/voxelite-plugin-comments/src/panel.rs:33), [admission](../../crates/voxelite-app/src/comments.rs:30) |
| Gizmo/grid | Camera stays in Editor; pointer drag state stays in `EditorView.overlay_drag`. The camera-orientation widget is an overlay, not a voxel transform gizmo. Grid enablement is a host setting bridged from existing app preferences. [overlay API](../../crates/voxelite-plugin-api/src/overlay.rs:153), [shell](../../crates/voxelite-app/src/ui/overlays.rs:1), [grid bridge](../../crates/voxelite-app/src/editor.rs:717) |
| Lighting | Stateless plugin selects the existing exact-lighting kernel. Host builds an owned RenderPlan from enabled declarations; actual shading/GPU resources remain renderer-owned. There is no mutable lighting-component population to unify. [lighting plugin](../../crates/plugins/voxelite-plugin-shadows-lighting/src/lib.rs:10), [plan construction](../../crates/voxelite-plugin-host/src/pass.rs:33) |
| Palette/other plugins | App retains PaletteState and brush selection; palette callbacks propose state/library bytes/core edits. App persists library bytes or admits commands before adopting results. Tools propose commands; exporters provide callbacks; host retains enablement, settings, timings and quarantine. [palette state](../../crates/voxelite-plugin-api/src/palette.rs:45), [palette admission](../../crates/voxelite-app/src/palettes.rs:66), [host](../../crates/voxelite-plugin-host/src/host.rs:327) |

GPUI reads Editor fields and portable views, retaining native text/focus/scroll
state in the shell. Attached MCP queues work to the editor owner thread and borrows
the **same** DocumentSession and Host; standalone MCP owns its own session. MCP
comments use the same panel callback with live content and default presentation
state. Undo/redo swap Engine snapshots and advance monotonic revisions; playback
reads immutable previews. See [UI comments](../../crates/voxelite-app/src/ui/comments.rs:1),
[attachment](../../crates/voxelite-app/src/attachment.rs:44),
[MCP attachment](../../crates/voxelite-mcp/src/attached.rs:324),
[MCP comments](../../crates/voxelite-mcp/src/comments.rs:22),
[undo/redo](../../crates/voxelite-core/src/engine.rs:363),
[preview reads](../../crates/voxelite-app/src/history.rs:131).

**Real pain points are modest and local.** App/MCP each implement small capability
adapters. Admission paths must preserve the ordering “validate/commit, reconcile,
adopt transient feedback”; comments and palettes have similar plumbing. Grid state
is intentionally bridged between persistent preferences and host settings. Comments
views clone numbering/state and thread data. The transform host collects/sorts
bindings and clones settings for each dispatch
([host.rs](../../crates/voxelite-plugin-host/src/host.rs:500)).
These are inspectable costs and coordination points, not evidence that multiple
plugins maintain divergent scene copies. I found no reproduced synchronization bug
requiring ECS. Existing selection/mirror regression tests already cover relevant
boundaries ([editor tests](../../crates/voxelite-app/src/editor.rs:918),
[selection undo test](../../crates/voxelite-app/src/editor.rs:1092)).
Small targeted adapter/publication changes remain possible without an ECS migration.

**Prototype delivered.** [examples/ecs-state-spike](../../examples/ecs-state-spike/README.md)
is a standalone Rust 2024 workspace, outside the parent's `crates/*` membership
glob. Its own manifest/lockfile isolate the experiment. It pins the verified latest
stable standalone release, `bevy_ecs 0.20.0`, with `default-features = false` and
only `std`. The registry reports Rust 1.95 minimum; our build uses the pinned 1.99.1.
Primary references: [release metadata](https://docs.rs/crate/bevy_ecs/0.20.0),
[ECS API](https://docs.rs/bevy_ecs/0.20.0/bevy_ecs/).

The ECS backend stores mirror settings, discussion presentation state and active
layer/box selection as three components on an editor entity. An Authority resource
retains the real Engine. An explicitly chained Schedule transforms tool proposals,
produces comment proposals, admits edits, reconciles derived thread entities and
publishes an immutable read model. Thread entities store UUID/index/resolved flags;
message bodies and voxel chunks remain canonical document data. Undo removes and
redo recreates derived entities by stable UUID, without persisting Bevy Entity IDs.
The production MirrorPlugin and CommentsPlugin callbacks are reused unchanged.

GPUI integration is demonstrated as an owner-thread `dispatch()` followed by
`read()` of a published Arc; the eventual handler would notify GPUI after completion.
Rendering never runs a schedule or borrows a mutable World. MCP reads call the
existing pagination implementation against the live snapshot, even when the UI
shows history. Agent edits carry expected revision and actor and use core admission.
These are executable adapter seams, **not a running GPUI window or MCP transport**.
The same immutable publication API is implemented by the direct control.

The experiment deliberately excludes full registry metadata, timing/quarantine,
extension loading, save durability and OS integration. Its two known callbacks are
invoked directly with simple enablement flags. A production migration must preserve
the host's capability restrictions and panic isolation. Unrestricted plugin World
mutation would violate that boundary, and Schedule execution does not provide
transaction rollback. Native non-Send plugin/UI objects must remain owner-thread
objects. Moving canonical content into entities would additionally require replacing
core snapshot/history/format machinery; this spike supplies no evidence for that.

**Code size comparison.** Counts are rustfmt-formatted physical lines, with exact
source intervals and nonblank/noncomment counts in
[loc.json](../../examples/ecs-state-spike/results/loc.json).

| Comparable code | Physical lines |
| --- | ---: |
| Existing mirror callback and recursive rewrite | 25 |
| Existing comments panel behavior/listing, excluding tool targeting | 229 |
| Existing app comment read/admission/context + MCP comment adapter | 126 |
| Existing selection reconciliation + mirror accessors + undo/redo wrappers | 45 |
| Prototype shared behavior, types and read boundary (`model.rs`) | 370 |
| Direct control adapter (`direct.rs`) | 73 |
| ECS adapter, schedule and derived index (`ecs.rs`, including test accessor) | 170 |

Both prototype backends retain the same production callbacks and shared 370 lines:
**443 direct versus 540 ECS lines, +97 (+22%)**. Excluding blank/comment lines the
totals are 410 versus 497, +87. This is the like-for-like orchestration comparison.
The 425-line production slice is context, not a deletion estimate: it excludes
shared API types, Engine and general Host infrastructure, while the prototype adds
request/publication plumbing and omits complete host guarantees. The spike demonstrates
no code reduction for these features. Harness and tests are counted separately.

**Measurements.** Linux x86_64 KVM VM, Intel Xeon, 8 logical CPUs, Rust 1.99.1,
four Cargo jobs. Fresh target directories in `/tmp`, warmed registry/source cache,
`--locked --offline`; no rustc wrapper. Inherited Cargo config disables incremental
builds and uses line-table debug info (dependency debug info disabled). Both backends
have identical profiles/dependencies except optional Bevy; release uses thin LTO and
one codegen unit. These standalone debug profiles do not inherit the product's
per-package optimization overrides. Three cold debug samples, one cold release
sample, five alternating release runtime samples. Debug samples saw 1-minute load
averages from roughly 2.9 to 7.9; runtime samples 5.45–5.89. Other jobs were active;
differences near a few percent are noise, and the single release sample has no spread.

| Build/size | Direct | ECS | Change |
| --- | ---: | ---: | ---: |
| Cold debug median (range) | 5.278 s (4.567–5.391) | 15.330 s (13.499–20.862) | 2.90× |
| Cold release, one sample | 11.537 s | 28.689 s | 2.49× |
| Release ELF, unstripped | 1,073,864 B | 1,903,448 B | +829,584 B |
| Release ELF, stripped | 868,536 B | 1,511,512 B | +642,976 B / 628 KiB (+74%) |

Release binaries exercise the same feature paths to keep them linked. These size
deltas describe the prototype executable, **not the macOS bundle or whole app**.

| Hot path, median per operation | Direct | ECS |
| --- | ---: | ---: |
| Idle schedule | No work; loop timing floor | 170 ns |
| Clone published read model | 15.1 ns | 14.3 ns |
| Select layer and publish | 85.0 ns | 246.4 ns |
| Mirrored tool no-op through Engine | 1.047 µs | 1.286 µs |
| Count open threads, 128 | 21.6 ns | 29.5 ns |
| Count open threads, 1,024 | 329.8 ns | 208.7 ns |
| Comments view, 1,024 threads | 229.1 µs | 227.3 µs |
| MCP list, 1,024 threads, 32-row page | 61.25 µs | 60.35 µs |
| Resolve + commit, 1,024 threads | 438.4 µs | 545.4 µs |
| Undo + redo pair, 128 threads | 1.50 µs | 12.15 µs |
| Undo + redo pair, 1,024 threads | 12.53 µs | 145.86 µs |

ECS saves about 0.12 µs on the largest count query using compact derived rows, while
revision-based index reconciliation adds work to edits and undo/redo. An ordinary
compact index could also obtain that query locality. Bevy change ticks cannot infer
which inner Engine data changed; receipt-aware incremental reconciliation would add
more implementation. Skipping the schedule when there is no input avoids its idle
cost. No measured path demonstrates a meaningful application-level advantage.

Full per-run values, ranges and source hashes:
[measurements.json](../../examples/ecs-state-spike/results/measurements.json),
[comparison.md](../../examples/ecs-state-spike/results/comparison.md),
[build log](../../examples/ecs-state-spike/results/build.log).
Reproduce with [measure.py](../../examples/ecs-state-spike/measure.py).
Final source hashes were checked against the measured files. Failed fixture/build
attempts are excluded from this dataset; the final full measurement run succeeded.

**Dependency/licence impact.** Active Linux normal/build package-version pairs,
excluding the prototype itself: 41 direct (35 registry + 6 existing path crates)
versus 87 ECS (81 registry + the same 6 path crates): **46 additional packages**.
Of those, 18 package/version pairs are absent from the product lockfile. Counts use
`cargo tree -e normal,build --target x86_64-unknown-linux-gnu`; an earlier metadata
walk included unused optional packages and was corrected. No Bevy reflection,
async executor or multi-threaded feature is enabled. The core ECS package is
MIT OR Apache-2.0; additional resolved licences are permitted by the existing
`deny.toml`. `cargo deny` passed advisories, bans, licences and sources for both
the spike and root workspace, with allowed duplicate-version/unused-ignore warnings.
[Dependency inventory](../../examples/ecs-state-spike/results/dependencies.json),
[active ECS tree](../../examples/ecs-state-spike/results/ecs-tree.txt),
[audit log](../../examples/ecs-state-spike/results/spike-deny.log).

**Validation and limits.** Passed root `cargo fmt --all -- --check`, spike rustfmt,
spike Clippy `--all-targets --locked -- -D warnings` both with and without `ecs`,
spike tests (1 direct; 4 with ECS), `cargo xtask tidy`, both dependency audits,
`cargo machete`, plugin dependency rule and its Python regressions. Contract tests
cover atomic rollback after a valid earlier batch command, stale revisions/heads,
failed comment proposal state, no-op revisions, disable/enable, immutable captured
reads, history preview versus live MCP reads, actor attribution, layer reconciliation,
redo invalidation, ECS entity index reconstruction and backend document/view/wire
equivalence. Both release workloads completed five times. Check logs and
[checks.json](../../examples/ecs-state-spike/results/checks.json)
are retained beside the measurements. No existing public API changed, so there are
no affected dependent crates requiring an API migration test.

No macOS/AppKit/Metal execution, GPUI view integration, wasm build, real MCP socket
integration or full application build/size/frame measurement was performed. The
prototype is not a production parity claim: its direct baseline omits Host lookup,
timing and quarantine overhead on both sides, and its derived index is intentionally
simple. Those limits favor another measured experiment if the workload changes,
not adoption on these results.

Files added: only `examples/ecs-state-spike/{Cargo.toml,Cargo.lock,README.md,measure.py,src/*,results/*}`
and this report. Existing source and product manifests remain unchanged. Keep all
prototype code uncommitted; no adoption migration is recommended.
