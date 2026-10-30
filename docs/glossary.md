# Domain glossary

Shared context for architecture discussions and code changes. The current terms
below describe the implementation; the planned seams name proposed boundaries,
not existing types. [ADR 0001](adr/0001-record-architecture-decisions.md)
records the architecture review's recommended order.

## Current terms

### Document

`Document` is the portable model: a stable UUID, live `Snapshot` content (name,
bounds, palette, layers and comments), durable named versions and optional branch
lineage in [crates/voxelite-core/src/model.rs](../crates/voxelite-core/src/model.rs).
The file-format `Project` wrapper adds format metadata, an optional view and build
history in [crates/voxelite-format/src/lib.rs](../crates/voxelite-format/src/lib.rs).
“Project” in the UI and “model” in MCP's `model_id` refer to this domain, but avoid
equating `Document` with the `Project` wrapper or its runtime `Engine`.

### Layer

A `Layer` has a stable UUID, name, visibility and lock flags, and sparse voxel
contents; `Snapshot.layers` orders layers from bottom to top in
[crates/voxelite-core/src/model.rs](../crates/voxelite-core/src/model.rs).
Hidden layers retain their contents but do not contribute to the visible
composite, and locked layers reject voxel edits and deletion.
Use “layer” rather than “mesh” or “object”; an active layer is the editor's selected
layer identity, not a separate kind of layer.

### Slot

A slot is a stable document-palette identity in `1..=256`, represented by
`PaletteSlot.index` and referenced by voxel cells; zero means air and does not
resolve to a swatch in
[crates/voxelite-core/src/palette.rs](../crates/voxelite-core/src/palette.rs).
“Palette slot” and “slot index” are aliases; avoid using display position, RGB
value or swatch name as synonyms for slot identity.

### Palette

`Palette` holds opaque `Swatch` colors in stable slots plus an independent display
`order` in [crates/voxelite-core/src/palette.rs](../crates/voxelite-core/src/palette.rs).
The separate `PaletteLibrary` collection is defined in
[crates/voxelite-plugin-api/src/palette.rs](../crates/voxelite-plugin-api/src/palette.rs);
selecting it alone does not mutate the document.
Use “document palette” when disambiguating; avoid conflating it with a library
palette, a single swatch or the command palette.

### Voxel

A voxel is an occupied integer cell in a layer, carrying a palette-slot reference;
`Pos` is an `[i32; 3]` world coordinate with Y up in
[crates/voxelite-core/src/model.rs](../crates/voxelite-core/src/model.rs), and `Volume`
stores cells in [crates/voxelite-core/src/chunk.rs](../crates/voxelite-core/src/chunk.rs).
The `Voxel` struct in
[crates/voxelite-core/src/command.rs](../crates/voxelite-core/src/command.rs) is an
explicit `at`/`slot` write, including slot zero to erase through `SetVoxels`.
“Cell” is the grid term and may also mean air; avoid “pixel” or treating a voxel as
an RGB color independent of its palette.

### Transaction

`Transaction` wraps a typed `Command` with an expected revision, caller-supplied
history-entry UUID, actor and label in
[crates/voxelite-core/src/engine.rs](../crates/voxelite-core/src/engine.rs).
Execution prepares and validates privately, then commits atomically with one
history entry unless it is a no-op, including when the command is a `Batch` from
[crates/voxelite-core/src/command.rs](../crates/voxelite-core/src/command.rs).
Use “edit transaction”; avoid equating it with a UI action, a disk save or an MCP
request, which can also perform reads and lifecycle operations.

### Receipt

The core `Receipt` summarizes validation or execution with revision, optional
entry identity, actual changes and charged work in
[crates/voxelite-core/src/engine.rs](../crates/voxelite-core/src/engine.rs); validation
returns a receipt without committing or advancing the live revision.
MCP's operation receipts are separately cached tool results keyed by
`operation_id` in
[crates/voxelite-mcp/src/session.rs](../crates/voxelite-mcp/src/session.rs), allowing
identical retries and rejecting reuse with different arguments.
Qualify “core receipt” versus “operation receipt”; neither term alone means the
document was saved to disk.

### Checkpoint

`Checkpoint` is a temporary named bookmark into retained runtime history, using a
history head UUID or `None` for the initial cursor in
[crates/voxelite-core/src/engine.rs](../crates/voxelite-core/src/engine.rs).
It can expire when its branch or retained history disappears, and restoration is
a new reversible edit guarded by the current revision and head.
“History bookmark” is an alias; avoid “named version”, “saved snapshot” or “backup”.

### Version / revision

A **named version** is a durable `NamedVersion` with a UUID, name, optional parent,
author role, content hash and nonrecursive snapshot in
[crates/voxelite-core/src/model.rs](../crates/voxelite-core/src/model.rs).
A **revision** is the `Engine`'s runtime `u64` concurrency counter, initially zero
and increasing on committed changes and undo/redo in
[crates/voxelite-core/src/engine.rs](../crates/voxelite-core/src/engine.rs); it is not
persisted as document content, so avoid calling it a version or history position.
The **format version** is the separate major/minor `Version` declaration in
[crates/voxelite-format/src/lib.rs](../crates/voxelite-format/src/lib.rs); qualify
“version” to distinguish this declaration from a named snapshot or checkpoint.

### Build history

`BuildHistory` is an optional bounded portable recipe of ordered `BuildStep`
commands that must replay from an empty default document to reproduce the entire
saved document in
[crates/voxelite-format/src/build_history.rs](../crates/voxelite-format/src/build_history.rs).
Opening a valid recipe reconstructs an engine timeline marked saved, while saving
changed content omits the original recipe if it no longer matches.
“Build recipe” and “build provenance” are aliases; avoid calling it a serialization
of the host's undo/redo session or a list of named versions.

### Attached session

`AttachedSession` serves MCP over a private Unix socket, with owner-thread `poll`
temporarily lending the editor's live `Engine` to the MCP session so human and
agent edits share revision and history in
[crates/voxelite-mcp/src/attached.rs](../crates/voxelite-mcp/src/attached.rs).
Document creation, opening and saving remain app responsibilities, and
[crates/voxelite-app/src/ui/agent.rs](../crates/voxelite-app/src/ui/agent.rs) polls
events and reconciles editor state after agent changes.
“Live attachment” or “agent attachment” are aliases; avoid conflating this with the
standalone MCP `Session` in
[crates/voxelite-mcp/src/session.rs](../crates/voxelite-mcp/src/session.rs) or the
planned `DocumentSession`.

### Engine

`Engine` is the shared human/agent authority for one validated document, owning
atomic command execution, revision guards, bounded undo/redo history, checkpoints
and comparison with the last marked-saved state in
[crates/voxelite-core/src/engine.rs](../crates/voxelite-core/src/engine.rs).
Hosts supply persistence and session/epoch guards.
Use “core engine” or “document engine”; avoid “renderer”, “editor” or “session” as
synonyms.

### Editor

`Editor` holds one open project and its `Engine` alongside app state such as the
active layer, tool, brush color, camera, path and saved-file fingerprint in
[crates/voxelite-app/src/editor.rs](../crates/voxelite-app/src/editor.rs), with timeline
preview and playback methods in
[crates/voxelite-app/src/history.rs](../crates/voxelite-app/src/history.rs).
Its library state is separate from GPUI's `EditorView`, which composes widgets,
render scheduling, dialogs and input in
[crates/voxelite-app/src/ui.rs](../crates/voxelite-app/src/ui.rs).
Use “editor state” for `Editor` and “editor view” for `EditorView`; avoid using
either as an alias for the portable document or core engine.

### Op / Action

`Op` is the app enum deriving GPUI's `Action`, with labels, icons, shortcuts and
menu membership in
[crates/voxelite-app/src/actions.rs](../crates/voxelite-app/src/actions.rs).
`EditorView` supplies context-dependent enablement, selected state and dispatch in
[crates/voxelite-app/src/ui.rs](../crates/voxelite-app/src/ui.rs), and many operations
only change UI state.
“UI action” and “app command” are aliases; distinguish these from the typed
document-edit `Command` in
[crates/voxelite-core/src/command.rs](../crates/voxelite-core/src/command.rs) and MCP's
`operation_id` retry identity.

### UI component library

`voxelite-ui` is the shell-only GPUI presentation crate in
[crates/voxelite-ui](../crates/voxelite-ui/README.md). It owns theme and geometry
tokens, controls, grouped forms, dialog shells and fixed-width status slots.
The app owns editor entities, domain state, command dispatch and modal lifecycle.
Plugins depend on `voxelite-plugin-api`, never on the UI component library.

## Planned seams

These names come from the architecture review; the linked files identify current
responsibilities to extract, not implementations of the proposed types.

### DocumentSession

Planned shared owner of the engine and document lifecycle: path, saved-file
fingerprint, dirty state, open/save, atomic writes and external-change detection,
with app `Editor` and MCP `Session` acting as adapters.
These responsibilities currently span
[crates/voxelite-app/src/editor.rs](../crates/voxelite-app/src/editor.rs),
[crates/voxelite-app/src/persistence.rs](../crates/voxelite-app/src/persistence.rs),
[crates/voxelite-mcp/src/session.rs](../crates/voxelite-mcp/src/session.rs) and
[crates/voxelite-mcp/src/files.rs](../crates/voxelite-mcp/src/files.rs).
“Document session” is the prose alias; avoid treating it as an existing type or
renaming the current multi-document MCP `Session` by implication.

### Viewport

Planned owner of viewport render scheduling, pending work, dimensions, frame
identity and GPU/CPU fallback currently held by `EditorView` in
[crates/voxelite-app/src/ui.rs](../crates/voxelite-app/src/ui.rs) and its GPU helper in
[crates/voxelite-app/src/ui/viewport/gpu_viewport.rs](../crates/voxelite-app/src/ui/viewport/gpu_viewport.rs).
“Viewport module” names the proposed boundary; avoid equating it with the renderer
it invokes or the existing lowercase `viewport` field that stores screen bounds.

### CommandPalette

Planned owner of command search state and filtering, currently `command_open`,
`tools_only`, `query`, `command_index`, `command_scroll` and `filtered_actions` in
[crates/voxelite-app/src/ui.rs](../crates/voxelite-app/src/ui.rs), using `Op` metadata
from [crates/voxelite-app/src/actions.rs](../crates/voxelite-app/src/actions.rs).
“Command search” is the current UI wording; avoid confusing this seam with the
document palette, swatch picker or `palette_*` color-picker state.

### SaveFlow

Planned state machine for pending operations, unsaved-change choices, dialogs and
continuing after a successful save, currently represented by `pending`,
`pending_choice`, `after_save`, `dialog_open` and related methods in
[crates/voxelite-app/src/ui.rs](../crates/voxelite-app/src/ui.rs).
It would coordinate the UI flow around persistence, whose current `SaveTask` and
`SavedProject` types live in
[crates/voxelite-app/src/persistence.rs](../crates/voxelite-app/src/persistence.rs).
“Save flow” is the prose alias; avoid using it for atomic file writing itself or
for the broader proposed `DocumentSession`.

### Command registry

Planned consolidation of app command metadata, enablement and execution behind
one interface, extending the existing metadata registry in
[crates/voxelite-app/src/actions.rs](../crates/voxelite-app/src/actions.rs) and dispatch
in [crates/voxelite-app/src/ui.rs](../crates/voxelite-app/src/ui.rs).
The review proposes deriving menus, command search, the shortcut sheet and README
shortcut documentation from it; current shortcut formatting lives in
[crates/voxelite-app/src/shortcuts.rs](../crates/voxelite-app/src/shortcuts.rs) and
[crates/voxelite-app/src/ui/shortcut_hint.rs](../crates/voxelite-app/src/ui/shortcut_hint.rs).
“Action registry” is an alias already used by `actions.rs`; qualify it as the app
registry to distinguish it from core edit commands and the MCP tool catalog in
[crates/voxelite-mcp/src/catalog.rs](../crates/voxelite-mcp/src/catalog.rs).
