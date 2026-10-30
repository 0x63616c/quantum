# MCP review round 1

Review date: 2026-10-25. Baseline: `a311a48`, branch `build-v1`.
Scope: `crates/voxelite-mcp`, its catalog, contract and tests. `design/` is read-only.

The baseline builds and passes formatting, workspace Clippy with warnings denied,
and 66 workspace tests (18 MCP). Passing tests do not establish protocol compliance:
the probes below expose cases missing from the existing suite.

The review uses PLAN.md §§4, 6 and 14 and the existing standalone-stage boundary
in `spec/mcp/v1/contract.md`. The 38-tool stage does not claim the full application's
52-tool inventory, HTTP, GUI, plugins, recovery journal or artifact service.
Those already documented later stages are not new defects in this crate.

## Findings

| ID | Severity | Finding and evidence | Required correction |
| --- | --- | --- | --- |
| MCP-01 | High | `Session::execute` applies the same exhausted receipt admission gate to edits and `document_save`. After 4096 successful mutations (or the byte limit), unsaved work cannot be persisted through MCP. Its recovery text tells the caller to perform the very save it rejects. | Reserve receipt capacity for saving every dirty document; preserve exact retries and the hard count/byte limits. Test count and byte exhaustion, multiple documents, redundant saves and recovery after save failures. |
| MCP-02 | Medium | The SDK's permissive decoding is not sufficient input validation. Raw `tools/call` with missing `name` or array `arguments` returns `-32601` instead of `-32602`. `tools/list` with numeric `cursor` silently lists tools. A malformed notification with scalar params receives a response. | Validate supported method parameters before SDK dispatch, distinguish framing from parameter errors, and never reply to notifications. Exercise both supported protocol profiles and continued operation after errors. |
| MCP-03 | Medium | Uncorrelated parse/invalid-request errors serialize without an `id` member because the SDK skips `None`. JSON-RPC requires an explicit null ID when no ID can be recovered. Existing tests only check error codes. | Normalize error serialization to include `id: null`; assert the raw wire member, valid string/numeric IDs and subsequent recovery. |
| MCP-04 | Medium | Discovery schemas admit invalid box mode/slot combinations, inconsistent camera projections, simultaneous camera/views and nonopaque palette entries. Nested output rows are largely untyped objects, so schema tests cannot catch broken voxel, layer, history, comment or render rows. | Express conditional inputs and nested result types, annotate byte limits, retain additive output compatibility, and test schema rejection against corrupted nested outputs. Regenerate the frozen catalog. |
| MCP-05 | Medium | `explicit_voxels: 65536` is advertised as a request limit, but each batch member independently allows 65536. Multiple members exceed the public explicit-input budget before core's separate work limit applies. | Check the sum of explicit voxel entries across the batch before decoding/applying commands; reject atomically with the offending member's path and index. |
| MCP-06 | Low | Core batch failures carry a command index but no JSON Pointer. Generic recovery text also recommends a new operation ID for `model_busy`, `cancelled` and uncertain filesystem outcomes, contradicting retry guidance. | Attach operation pointers and give error-specific recovery/retryability. Validate labels consistently on lifecycle/save/history operations as well as core edits. |

No user-input-triggered panic was found in the source audit: production
`expect`/`unwrap` sites rely on fixed catalog/JSON object construction or validated
palette invariants. The receipt-size `debug_assert` is now covered by a maximum
64-layer summary using names that expand during JSON escaping. Filesystem hash/rename races remain explicitly outside
the cooperative local-root guarantee; this is not a hostile-process sandbox.

## Standards checked

- [JSON-RPC 2.0](https://www.jsonrpc.org/specification): request IDs, notifications,
  error response shape and parameter/method error classification.
- [MCP 2025-11-25 tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools):
  discovery schemas, structured/text results, annotations and tool execution errors.
- [MCP 2026-07-28 stdio](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio)
  and [versioning](https://modelcontextprotocol.io/specification/2026-07-28/basic/versioning):
  newline framing, per-request metadata and cancellation behavior.

## Resolution and verification

All six findings are fixed. No medium-or-higher findings remain open in the
reviewed standalone MCP stage.

| Finding | Resolution and regression evidence |
| --- | --- |
| MCP-01 | Receipt admission reserves one bounded save result per open document before edits/create/open. Saves preserve reservations for other dirty documents. `exhausted_receipts_preserve_saves_for_every_dirty_document` exercises count and byte exhaustion, two dirty models, a failed save, blocked redundant saves, persisted files and exact replay at exhaustion. `maximum_layer_summary_fits_reserved_receipt_and_replays` checks the reservation against an escaped maximum-layer summary. |
| MCP-02 | Stdio validates parameter objects and supported method argument types before SDK decoding, rejects unsupported task requests, and silently ignores invalid notifications. `malformed_protocol_parameters_are_rejected_in_both_profiles` covers missing names, invalid argument/cursor types, task requests, string IDs and recovery under both MCP profiles. |
| MCP-03 | Error serialization restores the required null ID when the SDK omits it. Wire tests assert the member exists for parse errors, batch rejection and invalid null request IDs. |
| MCP-04 | Conditional schemas cover box mode/slot rules (including batches), projection fields, camera/view exclusivity, single-image exports, query selectors and opaque alpha. Names/labels document byte limits. Nested document, layer, voxel, palette, comment, history, version, camera and image output fields are typed while accepting additive fields. The frozen catalog matches generation; tests corrupt nested fields to prove validation is meaningful. |
| MCP-05 | Batch decoding sums explicit entries before core execution and reports the crossing operation's index and pointer. A 65,538-entry request rejects without revision/history/geometry changes; trimming it to 65,536 succeeds. |
| MCP-06 | Core/conversion batch failures include operation pointers. Busy/cancelled errors advise exact retries with the original ID; unknown filesystem outcomes advise inspection rather than blind replay. Lifecycle/save/history labels use the same byte/control-character validation as edits. Tests reject invalid save/create labels without creating files. |

Additional concurrency coverage submits eight identical mutations concurrently,
checks a single history entry, and verifies the original receipt still replays
after a later erase without resurrecting geometry. Existing tests continue to
cover symlink/root rejection, external-file conflicts, unknown metadata preservation,
history/lock guards, CPU rendering and save/reopen byte identity.

Final verification on Linux, stable Rust 1.99.1:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
git diff --check
```

All commands passed. The workspace has 74 passing tests, including 26 MCP tests
(3 unit, 18 contract, 5 stdio). Both end-to-end protocol workflows pass with the
normal parallel test runner. One earlier run timed out on both workflow tests;
an isolated diagnostic passed in 3.12 seconds and the final full stdio suite
passed in 3.29 seconds. No timeout increase or test exclusion was used.

The review and fixes are incremental commits; the final completion commit is
`MCP review round 1 fixes`. No files under `design/` were changed, and no remote,
push or deployment was performed. This was a source review plus automated Linux
verification, not an independent reviewer or a desktop-host interoperability run.
