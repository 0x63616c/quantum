# 0003: Product direction

Date: 2026-10-27

Status: Accepted.

## Context

The product needs a concise scope decision that reflects the current workspace.
Earlier planning mixed implemented features, proposed crate boundaries and future
extension systems. Code and manifests remain the authority for shipped behavior;
this decision does not certify platform support or release readiness.

## Decision

Build an Aseprite-style 3D voxel editor for people and agents. The product scope is
a Rust/GPUI native macOS application, the same editor compiled to WebAssembly,
and MCP workflows for creating, inspecting and editing models. Human editing must
remain useful without an agent or account. Keep canonical, versioned `.voxl`
projects and GLB/PNG exports portable across these workflows.

The shared document engine owns layers, palette slots, atomic commands, revision
checks and undo/redo. History playback inspects immutable previews; named versions
and anchored comments remain project content. Local human and agent edits use the
same engine and history. The current MCP transports are standalone stdio and a
private Unix-socket attachment to the native editor; the browser editor does not
provide a live MCP attachment.

Follow [ADR 0002](0002-built-in-plugin-architecture.md) for feature ownership:

- Core infrastructure owns document invariants and undo, `.voxl` persistence,
  renderer/camera, and the plugin host/registry. The app composes these services
  and owns platform UI and file dialogs.
- Statically linked built-ins own tools, mirrors, gizmo/grid, palettes,
  comments/pins, export callbacks and agent command metadata. They use
  `voxelite-plugin-api` capabilities and propose edits for engine admission.
  Shell command metadata comes from enabled contributions.
- Built-in registration and host enablement already exist. There is no external
  plugin loader or user-facing plugin enablement UI. The example plugin is a
  declaration-only template outside the app.

These boundaries are implemented in the [shared app entry points](../../crates/voxelite-app/src/lib.rs),
[built-in registration](../../crates/voxelite-app/src/core_plugin.rs),
[engine](../../crates/voxelite-core/src/engine.rs),
[document session](../../crates/voxelite-format/src/session.rs) and
[MCP attachment](../../crates/voxelite-mcp/src/attached.rs).

## Deferred beyond v1

- User scripting, script/WASM plugin runtimes and agent-authored executable tools.
- Third-party editor plugins, dynamic loading, a stable external plugin ABI and
  plugin marketplace/distribution.
- Hosted sharing, cloud relays, multi-user collaboration and presence.
- Hosted accounts, sign-in and cloud authentication. Local transport access
  controls remain part of the existing MCP implementation.

Agent-host configuration packages and skills connect external assistants to MCP;
they are distinct from loading third-party code into the editor.

## Consequences

Keep the local editor, portable files and shared human/agent transaction model
independent of hosted services. Extend built-ins through the existing portable
API without weakening engine or persistence guarantees. Deferred systems require
their own architecture and verification decisions before being advertised as
implemented features.
