# 0002: Built-in plugin architecture

Date: 2026-10-26

Status: Accepted; the plugin API foundation is implemented, host and registry follow.

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
icons, menus and agent visibility. Settings declare Bool, Choice or Number schemas
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
will own implementation binding, API compatibility checks, cross-plugin conflicts,
keybinding conflicts, enabled sets and lifecycle management in the next PR.

Manifest `version` and `api_version` are `semver::Version` values. `API_VERSION`
starts at `0.1.0`, independently of plugin implementation versions and `.voxl` or
data section schema versions. Before 1.0, breaking API changes bump the minor
version and compatible additions or fixes bump patch. From 1.0, breaking changes
bump major, compatible additions bump minor and fixes bump patch. Required trait
methods and changes to exhaustive enums or public struct construction count as
breaking changes. Future host compatibility checks must respect the pre-1.0
minor boundary and reject newer required API versions.

Third-party plugins and WASM are deferred to a later design; dynamic loading and
scripting are also out of scope. This is a statically linked Rust API, not a stable
binary ABI or an external code loader.

## Consequences

This step adds the API and its unit tests only; it does not implement a registry,
wire plugins into the app or change runtime behavior. Later steps move built-in
feature registration and shell metadata behind this boundary while preserving
existing behavior.
