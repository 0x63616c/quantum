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

No user-input-triggered panic was found in the initial source audit: production
`expect`/`unwrap` sites rely on fixed catalog/JSON object construction or validated
palette invariants. The receipt-size `debug_assert` needs boundary coverage as part
of admission verification. Filesystem hash/rename races remain explicitly outside
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

Implementation and regression verification are in progress. This section will
record the final changes and commands before the review is marked complete.
