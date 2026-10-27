# 0002: Built-in plugin architecture

Date: 2026-10-26

Status: Accepted; the shell reads command metadata and the editor dispatches tools
and command transforms through the host registry. Camera gizmo and grid drawing
and interaction also dispatch through enabled overlay bindings. Export formats
and byte encoding dispatch through the enabled exporters registry.

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
command transforms, panels, formats, render passes, settings and `.voxl` data
section declarations into `Contributions`, preserving registration order across kinds.

Commands declare JSON Schema arguments, optional default keybindings and contexts,
icons, menus and agent visibility. Shell presentation order and optional Apple
application-menu sections preserve placement across plugin boundaries without
changing registry registration order. Settings declare Bool, Choice or Number schemas
with typed defaults. Render passes declare insertion slots; data sections declare
their stored names and schema revisions. Declaring a data section does not change
the current `.voxl` format. Local validation checks identifier syntax, duplicate
contribution identifiers within a plugin and setting defaults. IDs and setting
keys are namespaced; data section names may also name existing `.voxl` fields.

Plugins receive capability-scoped, host-implemented `CommandCx`, `ToolCx`,
`TransformCx`, `OverlayCx` and `ExportCx` traits, never the app, a GPUI context or an unrestricted
service locator.
`ActivateCx` scopes activation capabilities separately. `CommandCx` and
`ActivateCx` remain stubs. `ToolCx` supplies read-only queries for the hovered
voxel and face normal, bounded work-plane intersection, visible composite sample,
layer voxel lookup, palette, selected color, active layer, visibility/lock state
and document bounds. It exposes no engine, mutations, mirror settings or GUI.
The API depends on `voxelite-core` and `voxelite-export` and re-exports the core
command/value types and the shared GLB/PNG byte encoders;
plugin crates still depend only on the API.

`Plugin` exposes `manifest`, `contribute`, default-unbound `tool(id)` and
`command_transform(id)`, `overlay(id)` and `exporter(id)` implementation lookups, and default no-op
`activate`/`deactivate` hooks. The host owns API compatibility checks, cross-plugin
conflicts, keybinding conflicts, enabled sets and lifecycle management. Tool lookup and invocation are admitted
only for declared IDs belonging to an enabled plugin, as are command transforms.

Manifest `version` and `api_version` are `semver::Version` values. `API_VERSION`
is `0.5.0`, independently of plugin implementation versions and `.voxl` or
data section schema versions. Before 1.0, breaking API changes bump the minor
version and compatible additions or fixes bump patch. From 1.0, breaking changes
bump major, compatible additions bump minor and fixes bump patch. Required trait
methods and changes to exhaustive enums or public struct construction count as
breaking changes. Host compatibility checks respect the pre-1.0
minor boundary and reject newer required API versions.
The `0.2.0` bump adds command presentation fields to public struct construction.
The `0.3.0` bump adds required `ToolCx` queries and the tool behavior contract.
The `0.4.0` bump adds the command transform contribution kind and behavior contract.
The compatible `0.4.1` addition binds portable overlays to existing render-pass
contributions through a default-unbound lookup, without changing existing exhaustive
API enums or required methods on existing capability traits.
The `0.5.0` bump adds required snapshot/render queries to `ExportCx` and a
`FormatContent` category to public `FormatContribution` construction.

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

Tools propose unmirrored commands. The host runs registered `CommandTransform`s
on each outcome in ascending declared `order`, with registration order for ties.
Each transform receives pending commands and a read-only `TransformCx` exposing
live document bounds and only its owner's setting values. The mirror plugin owns
the three boolean axis declarations and recursive centered command rewrite.
The host validates and retains per-plugin setting values across disable/enable;
mirror axes remain transient per-editor state, defaulting off when opening a
document, with no change to preferences or `.voxl` persistence. Existing mirror
command IDs, menus and shortcuts are unchanged. The editor wraps the transformed
commands in the same ordered `Command::Batch` and calls the existing `Engine`
transaction path with the same label and human attribution. Core validation,
atomicity, history, undo and redo remain invariants
of the engine. Feedback state is adopted only after a successful transaction;
failure preserves the box anchor and rolls back any proposed palette insertion.

Missing, disabled or faulted tools produce no edits or preview. Disabling the
active tools plugin cancels the pending box; re-enabling starts a fresh gesture.
Core navigation, layers, persistence and undo remain usable without tools. This
step adds no enablement UI: the shell's immutable metadata view and installed
bindings still need rebuilding when runtime enablement is exposed to users.
Disabling mirror removes its transform and declarations while tools remain usable.
The editor's live registry check disables stale shell commands and toggles.

## Gizmo and grid overlays

`voxelite-plugin-gizmo-grid` (`dev.voxelite.gizmo-grid`) owns the existing six-ended
camera orientation widget and ground grid. The repository's gizmo is a camera
navigation widget, not a voxel transform tool: dragging orbits, clicking snaps to
one of six signed axes, and double-clicking invokes the core isometric fit action.
This extraction adds no new editing tool or voxel-transform behavior. Core retains
camera ownership and navigation operations, including the existing view commands.

The API already has render-pass insertion slots, so `Plugin::overlay(id)` binds an
`Overlay` to a declared `RenderPassContribution` instead of adding another registry
kind. The plugin contributes its grid at `BeforeScene` and orientation widget at
`Overlay`. `OverlayCx` exposes displayed bounds, the displayed camera basis, only
the owner's settings, and a primitive emission sink. It grants no document mutation,
GUI, renderer resources, filesystem or access to other plugins. Drawing emits a
world-space grid lattice or screen-space line/circle widget primitives. The lattice
stays analytic for the GPU resolve; the CPU adapter paints the same ordered paths.
Integer plane height and offset remain separate to preserve each backend's original
floating-point operations. The shell resolves semantic colors through its theme.
Changes to GPU-composited overlay data schedule a new frame and reject stale
in-flight frames, including disablement without a document revision change.

Pointer input uses local coordinates for circle hit-testing and window coordinates
for incremental drag deltas. Outcomes contain transient drag/hover state and core
camera requests. Any document edits can only be returned as typed core `Command`s;
the host runs command transforms for nonempty edits and the editor admits the ordered
batch through its existing human transaction path before adopting camera or gesture
feedback. The built-in gizmo returns no document commands, so navigation never adds
undo entries or changes `.voxl` bytes.

The grid command retains `dev.voxelite.core.grid` as an explicit compatibility ID,
now owned by the plugin. Its menu placement, label and lack of default binding are
unchanged. The declared boolean `dev.voxelite.gizmo-grid.grid_enabled` defaults to
true. The shell bridges it to the existing app-owned `Preferences.grid` JSON field;
there is no settings-file or `.voxl` migration. Settings rows read the enabled
registry. Disabling the plugin removes its command, setting, widget and grid while
core navigation, editing, saving and undo remain available. The shell cancels a
pending overlay gesture when its declaration disappears; enabling restores the
host-retained setting and starts a fresh interaction.

Overlay binding, drawing and interaction use the host's timing and panic isolation.
A draw panic discards all output from that owner, including earlier passes, while
other plugins continue drawing. Interaction errors expose no partial outcome;
panics quarantine the plugin. The conformance harness exercises supplied overlay
bindings across enable/disable/enable, alongside tools and command transforms.

## Export formats and persistence boundary

`voxelite-plugin-exporters` (`dev.voxelite.exporters`) contributes GLB, PNG, HEX
and GPL formats and their stateless byte callbacks. Document/view formats appear
in the Save/Export dialog; palette formats appear in the palette panel. This
category preserves the existing dialog choices, labels, extension handling and
palette controls. The existing browser export command IDs and menu placement are
retained as compatibility IDs owned by the exporters plugin. Native `.voxl` saves
remain core operations and are never dependent on a format contribution.

`ExportCx` exposes an immutable captured `Snapshot` (content, palette and bounds)
and rendering of only the host-selected view or contact sheet to RGBA8 pixels.
It supplies no engine, undo history, GUI, filesystem or general service access.
The core-side `voxelite-export` encoders remain unchanged. The plugin accesses GLB
and PNG encoding only through `voxelite-plugin-api::encoding`; HEX/GPL use the
API's existing palette type. No extra workspace dependency is allowed for plugins.
The dependency-rule tests explicitly reject direct `voxelite-export` dependencies
and permit the API to depend on it, without widening the allowed plugin set.

The host admits callback lookup only for enabled declarations with `export = true`.
A sendable `ExportJob` captures the stateless callback so native rendering/encoding
can continue on a worker while plugin instances stay on their owner thread. The
worker returns an opaque completion, with panic isolation and elapsed time. Only
the originating host can release its bytes, after rechecking enablement. Callback
panics quarantine the owner and remove its declarations; encoder errors return
without faulting the plugin. Binding and encoding have host timing records.
Callers retain their existing filesystem policy: native dialog writes the confirmed
path, browser downloads return bytes, MCP creates files atomically, and palette
files use their existing atomic replacement path. Exporting never marks the project
saved or completes a deferred New/Open/Quit requiring a project save.

The native save panel receives a fresh registry list on opening and execution checks
the live registry again. Disabling exporters removes document formats and palette
export controls; editor menus omit the plugin's commands. Core save/open,
undo and navigation remain available. There is still no plugin enablement UI.

MCP retains its public GLB, PNG and palette tool names and argument schemas, but
resolves their formats and callbacks through the registry. Attached requests borrow
the editor's actual host, including disablement and panic state, and refresh the
transport's tool list. Standalone MCP constructs the standard exporter host on its
request thread; stateless callbacks need no plugin state transferred between MCP
workers. Capabilities list enabled registry extensions plus core `.voxl`.
Read-only render feedback remains a core renderer service, independent of file
export enablement. PNG export keeps the existing camera, image metadata, labeled
contact sheets and pixel budgets, passing the composed pixels through `ExportCx`.
Palette export captures the selected palette and its name, which can differ from
the document palette, without changing the document.

Pre-migration characterization fixtures pin exact GLB, rendered PNG, straight-alpha
RGBA PNG, HEX/GPL and MCP contact-sheet bytes. The migrated plugin and MCP paths
compare against these bytes. GLB pins both pre-existing JSON chunk orders: the
standalone encoder uses sorted maps, while native GPUI enables
`serde_json/preserve_order` through feature unification. Neither layout is changed.
Conformance requires every declared exporter to bind
through enable/disable/enable. Regression tests cover missing/import-only bindings,
callback errors and panics, worker completion after disablement or host replacement,
registry-driven lists, unchanged `.voxl`/history, and native saving with exporters
disabled.

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

Plugin entry points, including tool, transform, overlay and export binding and invocation, are
guarded with `catch_unwind`. A hook panic marks the plugin
`Faulted`, discards its declarations and prevents retries for that host's lifetime;
a manifest panic is a registration rejection because no identity is established.
Shell notices are drainable `HostEvent` values. `Timings` records count/total/max by
plugin and label, including failed hooks, and reports advisory budget overruns.
Tool errors return to the editor without faulting the plugin; binding or interaction
panics quarantine it and remove its declarations. Tool calls use the same timings
and fault notices as lifecycle calls.
Transform panics abort the pending outcome before transaction admission and
quarantine the plugin; later editing can proceed without that transform.
The public `conformance::check` harness validates a plugin and exercises a full
enable/disable/enable cycle, recollecting metadata to detect residue and checking
that every declared tool, transform and exporter and each supplied overlay binding is available
only while enabled. Required
plugins also have their cleanup hooks exercised without exposing a public bypass
of their disablement policy. Panic isolation requires unwinding and cannot recover
from aborts or roll back arbitrary plugin-owned external effects.

## Consequences

The API and host have independent unit tests and a reusable conformance harness;
the app's `app_host()` registers required `dev.voxelite.core`, then tools, mirror,
gizmo/grid, palette, comments, agent and exporters built-ins, all enabled. These seven
plugin crates depend only on the API outside their tests. Core currently owns the
remaining shell commands, including camera navigation and native saving.

The shell takes an immutable view of enabled command contributions, sorted by
presentation order, for menus, Cmd-K, the shortcuts sheet and keybindings. English
label keys are used directly for now. An exhaustive ID mapping retains `Op` as the
GPUI dispatch type; existing availability checks now also consult the live registry. There is no
plugin enablement UI yet; editor menus filter or rebuild from the live registry,
while future enablement changes still need to rebuild the command-search/shortcut
view and installed bindings. Agent connection and Extensions remain UI-only commands,
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
Mirror characterization tests cover place/box/fill for every axis combination,
translated and centered even/odd bounds, exact voxels and byte-identical undo/redo.
Transform tests cover ordering, scoped settings, nested commands, disablement and
panic isolation before palette, gesture or document changes are adopted.

Gizmo/grid characterization passed on the original app path before migration:
endpoint hit tests, overlap ordering, signed snap directions, incremental drag
thresholds, unchanged document/history bytes, preference persistence and shortcut
IDs. The migrated tests invoke registered behavior, and cover grid geometry/style,
widget drawing, cancellation, disablement, restart, typed edit atomicity and
byte-identical undo/redo. The unchanged shell golden still covers command placement,
icons and shortcuts. Host tests exercise scoped settings, missing bindings, timing,
partial-draw disposal, error handling and panic quarantine.
