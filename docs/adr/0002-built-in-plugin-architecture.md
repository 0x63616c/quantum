# 0002: Built-in plugin architecture

Date: 2026-10-26

Status: Accepted; the shell reads command metadata and the editor dispatches tool
behavior through the host registry.

## Context

Voxelite needs one source of metadata for commands, settings and formats so the
shell can present installed features consistently and users can disable optional
features without weakening document or save guarantees.

## Decision

Keep the engine/document model, undo, `.voxl` and saving, renderer/camera, plugin
host and registry in core. The shell owns the window, menus, Cmd-K, settings and
shortcuts sheet, with their feature entries generated from the registry.

Ship tools (paint, erase, box, fill, eyedropper), mirror X/Y/Z, gizmo/grid, exact
shadows and lighting, palette/colors, comments/pins, `.glb`/`.png` exporters,
MCP/agent tools and FPS/stats as standard built-in plugins that can be disabled.
This splits feature exposure and orchestration; document invariants and shared
engine operations remain core responsibilities.

`voxelite-plugin-api` is pure Rust with no GPUI or app dependency. Built-ins use
only this crate as their interface to the host (dogfooding the same API), without
privileged access to app internals. Statically linked plugins declare manifests
with stable reverse-DNS IDs such as `dev.voxelite.tools`, localization keys,
versions and enablement policy. A `Registrar` collects data-only commands, tools,
panels, formats, render passes, settings and `.voxl` data section declarations
into `Contributions`, preserving registration order across kinds.

Commands declare JSON Schema arguments, optional default keybindings and contexts,
icons, menus and agent visibility. Shell presentation order and optional Apple
application-menu sections preserve placement across plugin boundaries without
changing registry registration order. Settings declare Bool, Choice or Number schemas
with typed defaults. Render passes declare insertion slots; data sections declare
their stored names and schema revisions. Declaring a data section does not change
the current `.voxl` format. Local validation checks identifier syntax, duplicate
contribution identifiers within a plugin and setting defaults. IDs and setting
keys are namespaced; data section names may also name existing `.voxl` fields.

Plugins receive capability-scoped, host-implemented `CommandCx`, `ToolCx` and
`ExportCx` traits, never the app, a GPUI context or an unrestricted service locator.
`ActivateCx` scopes activation capabilities separately. `CommandCx`, `ExportCx`
and `ActivateCx` remain stubs. `ToolCx` supplies read-only queries for the hovered
voxel and face normal, bounded work-plane intersection, visible composite sample,
layer voxel lookup, palette, selected color, active layer, visibility/lock state
and document bounds. It exposes no engine, mutations, mirror settings or GUI.
The API depends on `voxelite-core` and re-exports the core command and value types;
plugin crates still depend only on the API.

`Plugin` exposes `manifest`, `contribute`, a default-unbound `tool(id)` implementation
lookup, and default no-op `activate`/`deactivate` hooks. The host
owns API compatibility checks, cross-plugin conflicts, keybinding conflicts,
enabled sets and lifecycle management. Tool lookup and invocation are admitted
only for declared tool IDs belonging to an enabled plugin.

Manifest `version` and `api_version` are `semver::Version` values. `API_VERSION`
is `0.3.0`, independently of plugin implementation versions and `.voxl` or
data section schema versions. Before 1.0, breaking API changes bump the minor
version and compatible additions or fixes bump patch. From 1.0, breaking changes
bump major, compatible additions bump minor and fixes bump patch. Required trait
methods and changes to exhaustive enums or public struct construction count as
breaking changes. Host compatibility checks respect the pre-1.0
minor boundary and reject newer required API versions.
The `0.2.0` bump adds command presentation fields to public struct construction.
The `0.3.0` bump adds required `ToolCx` queries and the tool behavior contract.

Third-party plugins, WASM plugins, dynamic loading and scripting are out of scope.
This is a statically linked Rust API, not a stable
binary ABI or an external code loader.

## Tool behavior and transaction boundary

`voxelite-plugin-tools` implements Place, Erase, Paint, Fill, Box and Picker through
the API's `Tool` trait. `ToolInput` describes hover, preview, pointer click or an
explicit keyboard coordinate, plus the pending box anchor. Stateless tools return
`ToolOutcome`: ordered typed core `Command`s, transaction label, optional preview
region, optional original sampled color, resolved target and next anchor.
The plugin owns face-versus-ground targeting, layer admission, lazy palette slot
selection, no-op detection, box corners, preview regions and picker sampling.

The editor retains its `Tool` enum as stable IDs and UI selection, implements
`ToolCx` with a private read-only adapter, and retains a host for dispatch and
lifecycle state. Ray construction, DDA picking and work-plane intersection remain
host responsibilities. The viewport draws the returned region; pointer and
keyboard handlers continue through the editor. Alt-click dispatches the registered
Picker implementation without changing the active tool or pending box.
Background file opening returns a portable `DocumentSession`; the UI thread then
attaches the editor's plugins, preserving the API's lack of a `Send` requirement.

Tools propose unmirrored commands. For this step the editor still applies its
centered mirror settings, wraps the commands in the same ordered `Command::Batch`,
and calls the existing `Engine` transaction path with the same label and human
attribution. Core validation, atomicity, history, undo and redo remain invariants
of the engine. Feedback state is adopted only after a successful transaction;
failure preserves the box anchor and rolls back any proposed palette insertion.

Missing, disabled or faulted tools produce no edits or preview. Disabling the
active tools plugin cancels the pending box; re-enabling starts a fresh gesture.
Core navigation, layers, persistence and undo remain usable without tools. This
step adds no enablement UI: the shell's immutable metadata view and installed
bindings still need rebuilding when runtime enablement is exposed to users.

## Crate layout and dependency rule

Core crates live in `crates/*`; plugin packages live in `crates/plugins/*`.
`crates/plugins` is a tracked container with a README, not a package. The workspace
includes both globs and excludes only `crates/plugins/Cargo.toml`, because excluding
the container directory would also exclude the plugin packages beneath it.

Plugin packages may depend only on `voxelite-plugin-api` and external crates in
normal and build dependencies. This includes optional, renamed and target-specific
dependencies. Direct dependencies on any other workspace crate, including
`voxelite-core`, are forbidden; shared core types must be exposed through the API
crate. Dev-dependencies are exempt so tests can use the host's conformance harness.
`scripts/check-plugin-deps.py` enforces this boundary from Cargo metadata in CI,
trying locked offline resolution first and retrying without offline mode if needed.

`voxelite-plugin-example` is a minimal template (`dev.voxelite.example`,
`builtin = false`) with one command declaration and a conformance test. It is not
linked into the app and adds no built-in feature or command execution.

## Host

`voxelite-plugin-host` is pure Rust and depends on the API without GPUI. It validates
manifests and collects declarations once per plugin. The read-only `Registry`
exposes only successfully enabled plugins, preserving plugin and declaration
registration order. Catalog conflicts include disabled plugins; duplicate IDs
across contribution kinds and duplicate canonical default keybindings in the same
opaque context are structured `Conflict` values. Enablement checks the candidate
against enabled plugins and its own bindings, leaving the whole candidate disabled
on conflict so it can be retried after the conflicting plugin is disabled.

`EnabledSet` converts to and from `Vec<String>` or `BTreeSet<String>` for later
app-owned persistence. A supplied set overrides defaults, but `can_disable = false`
plugins are always selected, subject to validation and successful activation.
Repeated enable/disable requests are no-ops; transitions call their hook once.
API compatibility ignores build metadata and requires exact version precedence
for prereleases, which have no compatibility guarantee.

Plugin entry points, including tool binding and interaction, are guarded with
`catch_unwind`. A hook panic marks the plugin
`Faulted`, discards its declarations and prevents retries for that host's lifetime;
a manifest panic is a registration rejection because no identity is established.
Shell notices are drainable `HostEvent` values. `Timings` records count/total/max by
plugin and label, including failed hooks, and reports advisory budget overruns.
Tool errors return to the editor without faulting the plugin; binding or interaction
panics quarantine it and remove its declarations. Tool calls use the same timings
and fault notices as lifecycle calls.
The public `conformance::check` harness validates a plugin and exercises a full
enable/disable/enable cycle, recollecting metadata to detect residue and checking
that every declared tool binds only while enabled. Required
plugins also have their cleanup hooks exercised without exposing a public bypass
of their disablement policy. Panic isolation requires unwinding and cannot recover
from aborts or roll back arbitrary plugin-owned external effects.

## Consequences

The API and host have independent unit tests and a reusable conformance harness;
the app's `app_host()` registers required `dev.voxelite.core`, then tools, mirror,
palette, comments and agent built-ins, all enabled. These five plugin crates depend
only on the API outside their conformance tests. Core currently owns the remaining
shell commands, including grid and the existing browser export entries.

The shell takes an immutable view of enabled command contributions, sorted by
presentation order, for menus, Cmd-K, the shortcuts sheet and keybindings. English
label keys are used directly for now. An exhaustive ID mapping retains `Op` as the
GPUI dispatch type; availability and execution paths are unchanged. There is no
plugin enablement UI yet; future enablement changes must rebuild the shell view
and installed bindings. Agent connection and Extensions remain UI-only commands,
with `agent_visible = false`.

A golden test captured the pre-migration Apple and non-Apple menus, icons,
shortcuts, search order and bindings. Disablement tests verify that optional
plugins disappear from shell lists without affecting the remaining commands.

App characterization tests passed against the original tool path before dispatch
was switched. They assert exact command batches, labels and actor attribution,
preview regions, palette/no-op behavior, centered mirrors, rejected transactions,
and byte-identical `.voxl` content through undo/redo for every tool. Plugin unit
tests use a fake `ToolCx`; host tests cover enabled-only dispatch, missing bindings,
gesture errors and panic isolation. App tests also exercise disabled and absent
tools, reactivation and replacement registered behavior.
