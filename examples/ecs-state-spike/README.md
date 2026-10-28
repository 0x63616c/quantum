# ECS state experiment

Throwaway prototype, **not a production migration**. This is an independent
workspace so the parent manifest, lockfile and existing crates remain untouched.
It uses standalone `bevy_ecs = 0.20.0` with only `std`; the default build has no
Bevy dependency. No Bevy renderer, app loop, reflection or worker pool is enabled.

Run from the repository root:

```sh
cargo test --manifest-path examples/ecs-state-spike/Cargo.toml --locked
cargo test --manifest-path examples/ecs-state-spike/Cargo.toml --features ecs --locked
cargo clippy --manifest-path examples/ecs-state-spike/Cargo.toml --all-targets --features ecs --locked -- -D warnings
cargo run --manifest-path examples/ecs-state-spike/Cargo.toml --release --locked
cargo run --manifest-path examples/ecs-state-spike/Cargo.toml --release --features ecs --locked
python3 -B examples/ecs-state-spike/measure.py
cargo deny --manifest-path examples/ecs-state-spike/Cargo.toml --all-features --config deny.toml check
cargo machete examples/ecs-state-spike
```

`measure.py` uses fresh, temporary target directories beneath
`/tmp/ecs-state-spike-measure`, four Cargo jobs, three
alternating debug builds, one release build per backend, and five alternating
runtime samples. It removes only its own temporary directories. Registry sources
must already be fetched (`cargo fetch --manifest-path ... --locked`). Timing
excludes downloads, includes compilation/linking, and records host load and source
hashes. These are Linux microbenchmarks, not application frame or macOS timings.

## Model

`model.rs` is shared by both controls. It calls the **existing** `MirrorPlugin`
and `CommentsPlugin` behavior, preserving recursive mirror transforms and full
comment view/pagination semantics. Selection means the existing active layer UUID
and pending box corner; the product does not have a general scene-object selection
model to migrate. `direct.rs` is a small direct struct-and-function control, not a
copy of the GPUI app. Both controls execute identical work and publish the same
owned read model. `main.rs` exercises all paths in both release binaries and
measures request dispatch, reads, comment iteration, commit and undo/redo.

`ecs.rs` owns a World and a single-threaded Schedule. One editor entity carries
`MirrorState`, `Discussion` and `Selection`. An `Authority` resource contains the
real core Engine and an optional immutable history preview. Five ordered systems
transform tool proposals, propose comments, admit transactions, reconcile thread
entities, and publish an immutable read model. Ordering is explicit with `chain()`;
Bevy's borrow scheduling alone cannot establish commit-before-publication semantics.
The schedule also applies deferred entity operations before returning.

Thread entities contain a stable document UUID, its current snapshot index and a
derived resolved flag. They demonstrate queried per-item components without copying
message bodies or voxel chunks. Index reconciliation runs on document revision
changes, including undo/redo; snapshot references and transient settings are never
separate document authorities. There is no entity per voxel. The resolved-count
microbenchmark uses these compact rows; this can also be achieved with an ordinary
compact index, without ECS.

All edits pass through Engine transactions. A rejected proposal leaves the document,
revision, transient state and published Arc intact. Undo/redo uses the original
Engine history. Agent edits use live state, explicit command mirrors and actor
attribution; they do not adopt UI comment selection. Human edits during preview are
rejected. Stable UUIDs, not Bevy Entity values, cross read and request boundaries.

## GPUI and MCP integration sketch

The UI owner would retain `Ecs` inside its editor model. An event handler dispatches
a typed request, the schedule finishes, then `read()` clones the published Arc and
the handler calls `cx.notify()`. Rendering reads that Arc without mutating the World
or running systems. Existing camera/render jobs can keep borrowing captured immutable
documents. Native UI objects and plugin instances with no Send requirement stay on
their owner thread; the experiment does not attempt to put the real Host in a Bevy
Resource requiring Send/Sync.

An attached MCP socket would retain its existing request queue and owner-thread
dispatch. `ReadModel::mcp_comments` uses the actual production pagination callback
against **live** state even during a UI history preview. A worker can hold an older
published Arc; writes still carry an expected revision, so Engine rejects stale
requests. No World mutex, Entity IDs, wire schema changes, socket implementation or
GPUI dependency is introduced here. The proposed publication boundary also works in
the direct control and is not an ECS-specific benefit.

## Deliberate limits

This prototype does not replace the registry, capability validation, callback timing,
panic quarantine, extension migration, file durability, menus or platform adapters.
It invokes two known built-ins directly and models their enablement flags. A real
adoption must retain all host guarantees; giving plugins arbitrary `&mut World`
access would bypass them. A Schedule panic is not an atomic plugin transaction or
an automatic rollback. Tests cover core rejection, not hostile systems/panic recovery.

The ECS thread index adds synchronization and storage. Both controls retain the
production callbacks and Engine; no deletion of their source lines is claimed.
`results/measurements.json` contains raw measurements, `dependencies.json` contains
the active normal/build dependency closure, and the job report at
[RECOMMENDATION.md](RECOMMENDATION.md) records the recommendation and limitations.
