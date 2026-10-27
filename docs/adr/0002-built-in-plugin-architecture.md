# 0002: Built-in plugin architecture

Date: 2026-10-26

Status: Accepted; the registry foundation is implemented, app integration follows.

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

`voxelite-plugin` is pure Rust with no GPUI dependency. Statically linked plugins
declare manifests and contribute data-only commands, settings and formats. Hosts
bind implementations by ID and resolve defaults and disable policy into an
explicit enabled set. The registry validates unique plugin IDs and, among
enabled plugins, unique contribution IDs across all kinds and unique shortcuts
within each context. It preserves registration order and retains disabled
manifests for host management UI; disabled plugins do not register contributions.
Shortcut and context strings use canonical host spelling and exact comparison.

Third-party plugins, dynamic loading, WASM plugins and scripting are out of scope
for now. This is an internal built-in feature boundary, not a public extension ABI.

## Consequences

This step adds the registry and tests only; it does not wire plugins into the app
or change runtime behavior. Later steps move built-in feature registration and
shell metadata behind this boundary while preserving existing behavior.
