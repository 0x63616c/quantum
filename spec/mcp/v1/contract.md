# voxelite.v1 — headless MCP stage contract

The implemented catalog is [tools.json](tools.json): 38 sorted tools, descriptions,
strict JSON Schema 2020-12 inputs, output envelopes, and annotations. The MCP
crate owns this committed catalog; the site reads it without linking the server.
Regenerate with
`UPDATE_TOOLS_JSON=1 cargo test -p voxelite-mcp --locked --test tools_catalog`.
The test compares exact bytes against `VoxeliteServer::tools()`, using pretty JSON,
stable tool/key order and a trailing newline. Its `serde_json/preserve_order`
dev-dependency keeps the site's existing key order in standalone and workspace
test runs. Commit the refreshed catalog; the snapshot test must pass without the
update flag. Breaking changes require a new API major. Application
version, MCP protocol version, and `.voxl` version evolve independently.

This is the standalone **MCP stage** requested for `voxelite-mcp`, not completion
of PLAN.md's full application/session/plugin inventory. Tool names follow the
stage's document/voxels/layers naming and have the `voxelite_` prefix. The response
API version is `voxelite.v1`. It builds exclusively on core, format, export, render,
and rmcp. There is no GUI, HTTP listener, live editor attachment, plugin runtime,
upload/artifact store, durable session journal, or recovery daemon in this stage.

## Protocol

- Newline-delimited UTF-8 JSON-RPC 2.0 over stdin/stdout, one message per line.
  Blank lines are ignored; CRLF is accepted. Maximum line size including newline
  is 16 MiB. Batch JSON-RPC arrays are rejected. No banners/logs on stdout.
- MCP `2025-11-25`: initialize, capability/version negotiation, initialized
  notification, tools/list, tools/call, ping, cancellation notifications.
- MCP `2026-07-28`: fresh requests with protocolVersion, clientInfo, and
  clientCapabilities in namespaced `_meta`, without initialize. Result types and
  unsupported-profile validation are supplied by the pinned rmcp implementation.
- Only tools capability is advertised. The fixed catalog has one page; a supplied
  tools/list cursor rejects. No sampling, resources, prompts, or subscriptions.
- Parse errors are `-32700`; invalid JSON-RPC requests are `-32600`; unknown
  methods are `-32601`; unknown tools/invalid protocol parameters are `-32602`.
  Uncorrelated errors include `id: null`. Malformed known method parameters
  (including nonobject arguments and nonstring cursors) are rejected before SDK
  dispatch; task-augmented calls reject because no task support is advertised.
  Invalid tool arguments and domain errors are normal tool results with
  `isError:true`, never an apparent success or an opaque internal error.
- Notifications have no response. Oversized lines are discarded through newline
  before returning an invalid-request error, allowing subsequent requests.

Both profiles are tested with spawned binaries and raw JSON. See
[docs/mcp.md](../../../docs/mcp.md) for complete envelopes and examples.

## Authority, coordinates, and atomicity

`capabilities_get` exposes the process session ID and epoch. All model reads use
`model_id`; edits additionally require `expected_epoch`, decimal-string
`expected_revision`, and UUID `operation_id`. Creation/open require epoch and
operation ID, but no existing revision. UUIDs are lowercase, hyphenated. Optional
read `revision` must equal the live revision. Returned IDs are canonical UUIDs;
layer and swatch names must resolve uniquely. No operation uses UI selection,
active layer, mirror settings, or camera.

All content edits become core commands with `Origin::Agent`. Batches apply
ordered voxel operations in one core transaction, with whole-batch validation,
one revision and undo entry; any error leaves the model unchanged. Construction
initializes a revision-zero document, and opening installs a fully validated
project with empty undo history. Undo/redo are the core's guarded history actions.

Successful mutations retain their exact receipts, including no-ops, for the
process lifetime. Identical operation ID/name/arguments returns the original
result even after later edits. Different arguments with that ID reject. Failed
operations do not reserve IDs. Receipt admission is checked before side effects;
when the count/byte budget is exhausted, save documents and start a new process.
Admission reserves capacity for saving every dirty document, so an edit
refused for receipt exhaustion does not prevent final saves. A save cannot spend
another dirty document's reservation, including repeated saves of a clean model.
Failed saves retain the reservation and consume no receipt. Receipts are not
retained across restart. Use the original request when
a response is lost; do not generate a new ID for an uncertain mutation.

An admitted blocking worker checks cancellation after acquiring the authority.
Once synchronous core preparation or filesystem saving starts, it completes
atomically even if the client cancels. Cancellation does not undo a committed
edit. Read current state or retry the identical operation to recover its receipt.
Rendering is sequential CPU work; it does not have a preemptive execution deadline.

Coordinates are right-handed, Y up, integer cells in `[-32768,32768)`, bounded by
the document. A region has inclusive `min`, exclusive `max`, positive extents
no larger than 256. Palette slots 1–256 are opaque RGBA; slot zero means air.
Stored layer voxels count overlaps separately. The highest visible layer wins in
the composite; hidden layers remain stored. Layer array order is bottom to top.

Mirror is `{"axes":["x"],"plane2":[32,0,0]}`: reflection maps cell coordinate `c`
to `plane2 - c - 1`, with explicit independent doubled planes. Axes are unique;
omission means no mirroring. Box defaults are solid, place mode. Place/paint
require slot; erase forbids slot. Fill is six-connected, including air, within an
explicit region or document bounds, and rejects on its visit/work cap.

## Results and limits

Every tool result contains an object `structuredContent` and equivalent JSON in
its first text block. Success has `api_version`, `session_id`, `epoch`, `ok:true`,
and object `data`. Failure retains those identity fields, has `ok:false` and
`error`, and sets `isError:true`. Error fields are code, message, retryable,
recovery, and optional path, command_index, current_revision. Error messages and
paths are bounded, including UTF-8. No personal author or machine identity enters
portable content.

Changes report model ID, revision, operation ID, undo entry/head IDs, added,
removed, recolored and total changed voxels, changed bounds/layers, metadata and
palette flags, work units, and `durability:"memory"`. No-ops keep the revision and
have no undo entry. Save success reports bytes/SHA-256 and clears dirty state;
exports leave dirty/history state unchanged. Query pages return `next_offset` or
null. Include the captured revision on subsequent pages; stale reads reject.

| Limit | Value |
| --- | --- |
| Open documents | 8 per process |
| Retained undo history | 16 MiB per document, bounded core eviction |
| Concurrent admitted tool workers | 16, serialized authority |
| Successful operation receipts | 4096 / 8 MiB; 64 KiB per receipt reserved before writes, with capacity held for final dirty-document saves |
| Encoded request / tool result | 16 MiB / 12 MiB |
| Explicit writes / batch operations / charged work | 65,536 entries summed across the entire request / 256 / 1,048,576 |
| Voxel read page | default 1000, maximum 10,000 |
| Comments/history page; messages per thread | default 50, maximum 200 |
| Comment text | 4096 UTF-8 bytes per message |
| Palette import text | 64 KiB, 1–256 colors |
| Named versions | 64, aggregate 64 MiB canonical snapshots |
| `.voxl` file | 128 MiB |
| Render | 1–6 views, each at most 1024²; at most 4,194,304 view + sheet pixels |
| Contact sheet | at most 2048 per axis, two columns for multiple views |

Core geometry/chunk limits still apply. Renders use one immutable revision,
2x antialiasing, CPU lighting/AO/shadows, transparent background, and independent
cameras. Images return base64 `image/png` content; metadata identifies exact
cameras, bounds, dimensions, hashes and output order. Labels overlay the tile,
without adding unaccounted pixels. Default is four 512² views and one 1024² sheet.
`individual` returns only views; `both` returns the sheet first then views.
An explicit camera replaces named views and perspective fitting. PNG export
requires exactly one output image; a contact sheet counts as one.

Schemas encode box mode/slot requirements, camera projection fields and selector
exclusivity, and opaque palette alpha. Name/label limits are UTF-8 byte limits;
JSON Schema's character limits are supplemented by runtime byte validation.
Nested output rows have typed fields and accept additive fields within this API
major. Core batch errors report both `command_index` and an operation JSON Pointer.

## Files and persistence

File tools accept explicit relative paths within the existing `--root` directory.
Parents must already exist. Absolute paths, `..`, nonregular targets, final
symlinks, and parent symlinks escaping the root reject. There is no directory scan,
shell, URL fetch, implicit overwrite, or embedded executable evaluation.

Save writes a sibling temporary, flushes/fsyncs, then atomically installs it.
Existing destinations can be overwritten only when this document previously
opened/saved that same canonical path and its recorded SHA-256 still matches.
New paths and exports use create-only atomic installation. A post-rename
Unix directory-fsync failure reports `outcome_unknown`; inspect the destination.
Canonical open/save preserves unknown supported metadata, IDs, comments and named
versions. Raw history and runtime receipts reset on reopen.

The root is a cooperative local project boundary, not an operating-system sandbox
against concurrent hostile filesystem changes. Hash-check/rename has the usual
external-writer race; no multi-process filesystem lease is implemented. Do not run
two editing authorities writing the same file. No mutation is called journaled:
explicit save is required for persistence across process exit.
