# Live Tracy profiling

Quantum's `profiling` Cargo feature is off by default, including for release
packaging. It enables `tracing` spans in the app, renderer and mesh crate and
installs one global Tracy subscriber at app startup. Library consumers may enable
`quantum-render/profiling` or `quantum-mesh/profiling` and supply their own tracing
subscriber; the libraries never initialize a global subscriber.

## Attach

1. Install the [Tracy profiler](https://github.com/wolfpld/tracy/releases/tag/v0.14.1)
   v0.14.1, matching the locked `tracy-client-sys` 0.30.x protocol. Building the
   client requires a C++ compiler in addition to the normal Rust prerequisites.
2. From the repository root, start an optimized instrumented editor:

   ```sh
   cargo run --locked --release -p quantum-app --bin quantum --features profiling -- examples/autumn-workshop.voxl
   ```

3. Open Tracy and connect explicitly to `127.0.0.1` (default TCP port 8086).
   Discovery broadcast is disabled. The client accepts only loopback connections
   and records on demand while attached. Connect before reproducing the jank;
   no pre-attachment history is retained.
4. Orbit the camera and edit voxels. Capture and save the trace in Tracy before
   closing the editor. Disconnecting or quitting does not wait for a profiler.

`editor.frame` covers GPUI editor preparation and layout. Frame marks indicate
entry to this callback, so intervals include idle time in the event-driven editor;
they are not display refresh or presentation latency measurements.
`mesh.greedy` covers the portable mesher; `mesh.sync` and `mesh.chunk` cover the
incremental GPU mesh cache. `render.cpu` and `render.cpu.cached` cover software
rendering. `render.gpu.frame` (native Mac worker), `render.gpu.headless`,
`render.gpu.encode`, `render.gpu.occupancy` and `render.gpu.present` cover CPU
work involved in GPU rendering and native presentation. Async instrumentation enters spans only during future polling,
never holding an entered span across an await. These are CPU zones, not GPU timer
queries. Use the [rendering harness](rendering-harness.md) and its timestamp
metrics for GPU timings and authoritative before/after comparisons.

The app subscriber filters to Quantum app, render and mesh targets, excluding
dependency tracing. No document contents, paths or function arguments are recorded
by these spans.
Tracy still exposes timing and source locations to local profiling tools. Its
sampling, system tracing, crash handler and code transfer features are disabled.
Existing application logging and opt-in crash reporting remain independent.

## Verify the opt-in boundary

```sh
cargo tree --locked -p quantum-app -e normal --prefix none
cargo tree --locked -p quantum-app --features profiling -e normal --prefix none
cargo test --locked -p quantum-render --features profiling
cargo clippy --locked -p quantum-app --features profiling --all-targets -- -D warnings
```

The first tree must contain no `tracing-tracy`, `tracy-client`,
`tracy-client-sys` or `tracing-subscriber`. GPUI already uses `tracing` independently.
All Quantum span attributes and the subscriber startup are compiled out when
`profiling` is absent; release optimization and rendering quality are unchanged.
Do not enable `profiling` in distribution builds. Instrumented runs have profiling
overhead and should not replace the normal performance baseline.

On macOS, also build the feature and attach to the native Metal editor, then
verify frame, mesh and render zones while orbiting and editing. Linux verifies
portable code and capture plumbing but cannot establish AppKit/Metal behavior.
