# 0002: Built-in plugin architecture

Date: 2026-10-26

Status: Accepted; the app shell reads built-in command metadata from the host registry.

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
`ActivateCx` scopes activation capabilities separately. These are trait stubs in
this step; concrete operations arrive with host integration. `Plugin` exposes
`manifest`, `contribute`, and default no-op `activate`/`deactivate` hooks. The host
owns API compatibility checks, cross-plugin conflicts, keybinding conflicts,
enabled sets and lifecycle management. Implementation binding follows with app
integration.

Manifest `version` and `api_version` are `semver::Version` values. `API_VERSION`
is `0.2.0`, independently of plugin implementation versions and `.voxl` or
data section schema versions. Before 1.0, breaking API changes bump the minor
version and compatible additions or fixes bump patch. From 1.0, breaking changes
bump major, compatible additions bump minor and fixes bump patch. Required trait
methods and changes to exhaustive enums or public struct construction count as
breaking changes. Host compatibility checks respect the pre-1.0
minor boundary and reject newer required API versions.
The `0.2.0` bump adds command presentation fields to public struct construction.

Third-party plugins and WASM are deferred to a later design; dynamic loading and
scripting are also out of scope. This is a statically linked Rust API, not a stable
binary ABI or an external code loader.

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

Plugin entry points are guarded with `catch_unwind`. A hook panic marks the plugin
`Faulted`, discards its declarations and prevents retries for that host's lifetime;
a manifest panic is a registration rejection because no identity is established.
Shell notices are drainable `HostEvent` values. `Timings` records count/total/max by
plugin and label, including failed hooks, and reports advisory budget overruns.
The public `conformance::check` harness validates a plugin and exercises a full
enable/disable/enable cycle, recollecting metadata to detect residue. Required
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
