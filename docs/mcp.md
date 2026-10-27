# Voxelite MCP

`voxelite-mcp` is a Rust library and stdio binary for creating, editing, inspecting,
rendering and saving voxel projects without a GPU or display. Its 38 tools use the
same validated core commands, history, `.voxl` format and CPU renderer as the
other workspace crates. All content mutations are attributed to an agent.

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release -p voxelite-mcp
./target/release/voxelite-mcp --root /absolute/path/to/project
```

The project directory must exist. The process now reads MCP from stdin and writes
only MCP messages to stdout. Diagnostics use stderr. Close stdin to exit. For a
human-readable command summary use `--help`; `--schema` prints the complete tool
catalog and exits. Connect a host using the [Cursor, Claude and Codex configuration
examples](../integrations/README.md).

This stage has no GUI/plugin/web dependency and builds on stable Rust on Linux.
Edits are held in memory; call `voxelite_document_save` before disconnecting.
Interactive undo history and retry receipts are session-local. A saved build recipe
(such as the bundled demo) reconstructs its timeline when opened, subject to the
host's history budget. Comments and named versions persist inside saved `.voxl`
files. Comment lists page both threads and messages:
`limit` and `message_limit` default to 50 (maximum 200); use `offset` and
`message_offset` with the captured revision to read more.

## Agent workflow

1. Call `voxelite_capabilities_get` to capture the epoch and discover open model IDs.
2. Create or open a document. Keep its returned `model_id`, layer IDs and revision.
3. Apply palette changes or an atomic voxel batch. Each write uses the captured
   epoch/revision and a fresh UUID `operation_id`. Update your revision from the result.
4. Query counts/regions and render several views to inspect the result.
5. Use exact history entry IDs to undo/redo, or create comments and named versions.
6. Save `.voxl` and export PNG/GLB to explicit relative paths.

Layer/swatch names must be unique when used as selectors. Prefer returned UUIDs
and stable palette slots in repeatable recipes. Inputs reject unknown fields,
including inside batches, mirrors and cameras. Error text identifies a recovery
step; inspect the structured error rather than treating `tools/call` transport
success as edit success.

The 65,536 explicit-voxel limit applies to the entire request, summed across batch
members. Box mode/slot rules, camera projection fields and opaque palette colors
are expressed in discovery schemas. Names and labels have UTF-8 byte limits in
addition to JSON Schema character limits.

## Wire examples

For MCP `2025-11-25`, initialize and then notify:

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"voxelite-example","version":"1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"voxelite_capabilities_get","arguments":{}}}
```

Each line is a separate message; wait for each request's matching response. For
MCP `2026-07-28`, start directly with per-request metadata instead of initialize:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"voxelite-example","version":"1.0"},"io.modelcontextprotocol/clientCapabilities":{}},"name":"voxelite_capabilities_get","arguments":{}}}
```

Use that same `_meta` structure on subsequent requests in the newer profile. The
following examples show tool arguments; substitute the actual captured epoch,
model and revision values. UUID operation IDs must be fresh for each new write:

```json
{
  "name": "Small tower",
  "bounds": {"min": [0,0,0], "max": [32,32,32]},
  "expected_epoch": "<epoch from capabilities>",
  "operation_id": "00000000-0000-4000-8000-000000000001"
}
```

Pass those arguments to `voxelite_document_create`. Next call
`voxelite_voxels_batch` with the returned model ID and revision:

```json
{
  "model_id": "<created model UUID>",
  "expected_epoch": "<captured epoch>",
  "expected_revision": "0",
  "operation_id": "00000000-0000-4000-8000-000000000002",
  "label": "Block out tower",
  "operations": [
    {"kind":"box","layer":"Base","region":{"min":[10,0,10],"max":[22,2,22]},"slot":1},
    {"kind":"box","layer":"Base","region":{"min":[13,2,13],"max":[19,18,19]},"slot":1},
    {"kind":"box","layer":"Base","region":{"min":[11,18,11],"max":[21,20,21]},"slot":1}
  ]
}
```

Render it using `voxelite_render`:

```json
{"model_id":"<created model UUID>","views":["iso","front","right","back"],"width":512,"height":512,"image_mode":"contact_sheet"}
```

The render result contains text metadata plus an MCP image block:

```json
{"type":"image","mimeType":"image/png","data":"<base64 PNG>"}
```

For precise framing, replace `views` with a camera. `camera` and `views`/
`perspective` are mutually exclusive; projection-specific fields are required:

```json
{"model_id":"<created model UUID>","camera":{"eye":[40,32,40],"target":[16,10,16],"up":[0,1,0],"projection":"perspective","fov_y_degrees":45},"image_mode":"individual"}
```

A success envelope has the same JSON in `structuredContent` and the first text
block. Content-edit data includes these fields (UUID placeholders abbreviated):

```json
{
  "api_version": "voxelite.v1",
  "session_id": "<session UUID>",
  "epoch": "<epoch UUID>",
  "ok": true,
  "data": {
    "model_id": "<model UUID>",
    "revision": "1",
    "operation_id": "00000000-0000-4000-8000-000000000002",
    "undo_entry_id": "00000000-0000-4000-8000-000000000002",
    "head_id": "00000000-0000-4000-8000-000000000002",
    "changed_voxels": 1064,
    "added": 1064,
    "removed": 0,
    "recolored": 0,
    "changed_bounds": {"min":[10,0,10],"max":[22,20,22]},
    "changed_layers": ["<Base layer UUID>"],
    "palette_changed": false,
    "metadata_changed": false,
    "work_units": 1064,
    "durability": "memory"
  }
}
```

Actual charged work is reported by core. A domain failure returns `isError:true`
and an envelope such as:

```json
{
  "api_version":"voxelite.v1",
  "session_id":"<session UUID>",
  "epoch":"<epoch UUID>",
  "ok":false,
  "error":{
    "code":"revision_conflict",
    "message":"expected revision 0, current 1",
    "current_revision":"1",
    "retryable":false,
    "recovery":"Read current document and history state, rebase the intended change, then use a fresh operation_id."
  }
}
```

If a reply is lost, retry the **identical** tool name, arguments and operation ID
in the same process. Successful receipts replay exactly, including no-op results.
Different arguments with a used operation ID reject. Once an edit starts, it may
complete despite cancellation; cancellation never silently undoes a commit.

Receipt storage is bounded, but ordinary writes reserve enough capacity to save
each dirty document. If an edit hits the receipt limit, save dirty documents and
restart the process. Failed saves retain their reservation; redundant saves of a
clean document cannot consume another dirty document's reserved capacity.

## Tool catalog

Every name below starts with `voxelite_`. Exact defaults, limits and required fields
are in the discoverable schemas and the [frozen catalog](../spec/mcp/v1/tools.json).

| Tools | Behavior |
| --- | --- |
| `capabilities_get` | API/protocol versions, epoch, formats, CPU availability, presets, limits and open documents. |
| `document_create`, `document_open`, `document_save`, `document_info` | Up to eight identified projects; canonical, validated, atomic file persistence. Open preserves IDs and metadata; duplicate open IDs reject. |
| `voxels_get`, `voxels_set` | Explicit coordinates, integer or named slots. Read includes air; set defaults to place, supports paint and zero-slot erase. |
| `voxels_erase_region`, `voxels_fill`, `voxels_box`, `voxels_batch` | Half-open regions, six-connected fill, solid/shell boxes, atomic ordered batches. Mirror planes are explicit on each operation. |
| `query` | Bounds, occupied bounds, counts by slot, paged stored or visible-composite voxels. Include captured revision when continuing pages. |
| `layers_list`, `layers_add`, `layers_remove`, `layers_rename`, `layers_reorder`, `layers_visibility`, `layers_lock` | Stable IDs and bottom-to-top order. Voxel writes require visible/unlocked layers. Final-layer deletion rejects. |
| `palette_list`, `palette_set_slot`, `palette_apply_preset`, `palette_import`, `palette_export` | Opaque slots and exact presets; bounded `.hex`/`.gpl`/JASC-PAL `.pal` text. Imports/presets append into unused slots and return mappings, preserving existing geometry appearance. Explicit slot replacement recolors references. |
| `render` | Headless CPU PNGs with fitted named or explicit cameras, transparent background, AO/shadows, antialiasing, and labeled contact sheets. |
| `export_png`, `export_glb` | Deterministic new-file exports of visible content; existing paths reject. PNG may be one view or one sheet. |
| `comments_list`, `comments_add`, `comments_reply`, `comments_resolve`, `comments_reopen` | Anchored discussions with agent attribution, derived visible/hidden/orphaned state, persistence and undo. |
| `history_read`, `history_undo`, `history_redo` | Bounded applied timeline and guarded undo/redo. Undo/redo increase revision; new edits clear redo. |
| `versions_save`, `versions_list`, `versions_restore` | Immutable persistent snapshots; restore is an ordinary undoable edit and requires expected history head. |

Default render views are iso/front/right/back, 512² each, producing one 1024²
contact sheet. `image_mode:individual` returns view PNGs; `both` returns sheet
first and then views in request order. At most six views, 1024² each and 4,194,304
combined view/sheet pixels. A 12 MiB encoded result cap covers base64 plus all
metadata. Reduce resolution/view count if that cap rejects. Front looks from +Z,
right from +X, top from +Y with −Z up; all fits use Y-up coordinates.

Palette presets are `pico8`, `endesga32`, `sweetie16`, `grayscale`, `neutral`,
`warm`, and `vivid`. Imported slots may differ from their source indices: use the
returned `slot_mapping` instead of assuming append positions.

## Errors and file behavior

| Error code | Recovery |
| --- | --- |
| `invalid_argument`, `out_of_bounds`, `ambiguous_name` | Correct the indicated path/coordinates; use canonical IDs for ambiguous names. |
| `model_not_found`, `layer_not_found`, `not_found` | Refresh document/layer/comment/version information. |
| `model_already_open` | Continue with the existing model ID. |
| `epoch_conflict`, `revision_conflict` | Read current state and rebase the intended change. |
| `operation_id_reused` | Recover the original request; use a fresh ID only for a new operation. |
| `layer_blocked` | Explicitly show/unlock the intended layer before editing. |
| `history_conflict` | Refresh exact head/redo IDs; never assume a previous entry is still current. |
| `limit_exceeded` | Reduce page/batch/render size or model complexity; receipt exhaustion requires a new session after saving. |
| `file_exists`, `external_file_conflict` | Choose a new filename; preserve or reopen externally changed content. |
| `path_outside_root`, `storage_unavailable` | Correct the path or directory permissions. Parents must exist. |
| `invalid_format` | Correct unsupported/invalid `.voxl` input; the live document is unchanged. |
| `render_failed`, `export_failed` | Check camera/geometry validity and output budgets. |
| `model_busy`, `cancelled` | Request was not started; retry when appropriate with the original operation ID and identical arguments. |
| `outcome_unknown` | Rename succeeded but directory sync failed; inspect the destination before retrying. |

File tools operate inside `--root`. Save uses a sibling temporary file, fsync and
atomic installation. New paths are create-only. Overwriting a path this document
opened/saved requires its recorded hash to still match. Save failures before
rename preserve the previous file. A successful export never marks the document
saved. The full [stage contract](../spec/mcp/v1/contract.md) documents limits,
cancellation boundaries, filesystem races and the distinction from the later
application/session/plugin stages.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

The MCP tests verify the frozen schema catalog, transaction rollback, mirrored
writes, exact retries, conflicts, concurrent revision guards, locks/visibility,
palette formats, comment/version persistence, filesystem conflicts, camera and
render budgets, PNG decoding, exports, and raw JSON-RPC error framing. Process
tests run the complete editing loop independently under both supported MCP
profiles and verify a save/reopen/save produces identical canonical bytes.
Review regressions cover malformed protocol parameters and silent notifications,
null error IDs, saving at both receipt limits, concurrent exact retries, aggregate
batch input limits, conditional discovery schemas and typed nested output rows.

Verified on Linux with Rust 1.99.1 on 2026-10-25: workspace build, formatting,
Clippy with warnings denied, and all 74 workspace tests passed (26 MCP tests).
Host configuration files were parsed as JSON/TOML; host application launches
remain untested on this headless machine.

## Attach to a native editor window

The native app exposes its current document through a private Unix socket. Open
Agent → Agent connection to copy a stdio MCP configuration, or choose the socket
path explicitly:

```sh
voxelite --session-socket /tmp/voxelite-session.sock
voxelite-mcp --connect /tmp/voxelite-session.sock
```

`--connect` forwards the existing MCP protocol to the app; it does not open a
second document authority. Discover the app's model/epoch/revision with
`voxelite_capabilities_get`. Human and agent edits use the same engine and undo
history, and completed calls appear in the Session feed with actual differences.
Exact mutation retries retain their original receipts. New/Open in the app
rotates the epoch. Pause rejects new agent mutations while leaving reads enabled.

The attached catalog excludes document create/open/save: the app owns file
lifecycle and its external-file conflict checks. Other existing tools retain
the standalone validation and transaction behavior. Changes are in memory until
saved in the app, with no implied durable journal. Socket permissions are 0600;
existing paths are never replaced. The bridge is local to Unix platforms and
supports the same two MCP protocol profiles as the standalone server.

See [native editor controls](../crates/voxelite-app/README.md) and
[the executable round 2 review](reviews/app-polish-2.md). The process test
`cargo test -p voxelite-mcp --test attached` runs a real stdio bridge against the
app-side attachment and checks shared history, conflicts, pause, retries and
socket cleanup. `scripts/capture-native.sh` also exercises the live GPUI window.
