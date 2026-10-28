# The .voxl format

One project. One readable JSON file.

`.voxl` stores a complete portable project as UTF-8 JSON. This reference follows `quantum-format` and the validated core model in this development build. Media type: `application/vnd.voxelite.project+json`. Current writer version: `{"major": 1, "minor": 0}`.

## Top-level structure

| Field               | Contract                                                                                                                |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `format`            | Required literal `"quantum"`.                                                                                           |
| `version`           | Required `major` and `minor` unsigned 16-bit integers. Major must be 1. A newer minor is preserved.                     |
| `required_features` | Required array of strings. Only an empty array is currently supported.                                                  |
| `document`          | Required `id`, `name`, `bounds`.                                                                                        |
| `palette`           | Required `slots` and `order`.                                                                                           |
| `layers`            | Required array, bottom to top; 1 to 64 entries.                                                                         |
| `comments`          | Optional thread array, omitted when empty.                                                                              |
| `versions`          | Optional named snapshot array, omitted when empty.                                                                      |
| `lineage`           | Optional source `document` and `version` UUIDs for an independent branch.                                               |
| `extensions`        | Optional plugin ID → section name map; each section has `schema_version`, `encoding` and `payload`. Omitted when empty. |
| `view`              | Optional portable view metadata, excluded from the project semantic hash.                                               |

## Coordinates and layers

Coordinates are integer `[x, y, z]` triples in a right-handed, Y-up space. A cell at `[x,y,z]` occupies a unit cube. `bounds.min` is inclusive and `bounds.max` exclusive. Each extent must be 1 to 256; cell coordinates must lie in `[-32768, 32768)`. The exclusive maximum may equal 32768.

Each layer has `id`, `name`, `visible`, `locked`, and `voxels`. A voxel row is `[x, y, z, slot]`. Omit air entirely: slot zero is invalid on disk. All rows must lie in the document bounds and reference existing palette slots. Duplicate coordinates within a layer reject; the same coordinate in different layers is valid. The highest visible occupied layer wins the composite. Hidden geometry is retained; lock state does not change rendering.

## Palette identity

A slot has `index` (1 to 256), `rgba` (four bytes, alpha exactly 255), and an optional `name`. There must be 1 to 256 slots, with unique indices. `order` is a full permutation of those indices and controls display order independently of identity. No translucent voxels are supported.

## Canonical bytes and hashes

*   Object keys sort lexicographically. Output uses two-space indentation, LF newlines, and one final newline, except raw plugin section envelopes described below.
*   Each voxel occupies one compact line, e.g. `[0, 0, 0, 1]`. Rows sort lexicographically by X, then Y, then Z.
*   Palette slots sort by index; comments and named versions sort by UUID. Layer order, palette display order and message order are meaningful and preserved.
*   UUIDs in validated ID fields use lowercase, hyphenated strings. Names are nonempty, at most 128 UTF-8 bytes, without control characters.
*   Outside `view` and plugin section envelopes, numbers must use integer representation; fractional/exponent/negative-zero representations reject. Finite floating-point values are permitted inside `view` and plugin section envelopes.
*   `Project::semantic_hash()` is SHA-256 of canonical file bytes with top-level `view` and `build_history` removed. Unknown optional metadata and exact extension section bytes remain in that hash.

The project semantic hash differs from a named version's `content_hash`, which hashes the canonical core snapshot serialization. Use the Rust API to compute these hashes; do not substitute a generic pretty-printed JSON hash.

## Comments and named versions

A comment contains `id`, `anchor`, optional `layer`, `resolved`, and ordered `messages`. Anchors are `{"kind":"voxel","at":[x,y,z]}` or `{"kind":"region","min":[x,y,z],"max":[x,y,z]}`. Each message has `id`, `origin` (`human` or `agent`), plain-text `text`, and optional public `tool` ID. Hidden/orphaned state is derived, not stored; an anchor survives deleted geometry.

A named version contains `id`, `name`, optional `parent`, `origin`, `content_hash`, and a nonrecursive `snapshot`. Snapshots contain `name`, `bounds`, `palette`, `layers`, and optional `comments`. Hashes must match. Parents must exist in the same library, and cycles reject. Session IDs, revision counters, undo/redo entries and retry receipts are not serialized by the writer.

## Parser and resource limits

| Resource                                       | Limit                                                                                  |
| ---------------------------------------------- | -------------------------------------------------------------------------------------- |
| Encoded file                                   | 128 MiB                                                                                |
| Live layer-voxels / allocated chunks           | 1,048,576 / 8,192; overlapping layers count separately                                 |
| Decoded voxels across live and version content | 4,194,304                                                                              |
| Metadata excluding voxel arrays                | 8 MiB total; 4 MiB top-level comments, 4 MiB other metadata                            |
| JSON nesting / optional metadata nesting       | 32 / 16                                                                                |
| Comment threads / messages                     | 1,024 / 10,000 per snapshot; each message 1 to 8,192 UTF-8 bytes; 4 MiB comment budget |
| Named versions                                 | 64; aggregate canonical snapshot bytes ≤64 MiB                                         |

Duplicate object keys reject at every level. Parsing validates all layers and snapshots, including hidden content. Unknown major versions and nonempty required-feature lists fail without installing the document. Limits may reject a file before any model becomes available.

## Forward compatibility

Bounded unknown optional fields are preserved as inert data and merged by stable IDs or palette indices, not array position. Generic optional values accept strings, booleans, null, signed 64-bit integers, arrays and objects within depth limits. They are never interpreted as executable code or remote resources. Known fields always come from current validated content.

## Plugin extension data

`extensions[plugin_id][section_name]` stores an envelope with a positive 32-bit `schema_version`, `encoding` (`json` or `bytes`), and required `payload`. JSON payloads may contain any bounded JSON value, including finite fractional numbers. Byte payloads use arrays of integer octets, 0–255.

```json
"extensions": {
  "dev.quantum.example": {
    "state": {"schema_version": 2, "encoding": "json", "payload": {"scale": 1.25}},
    "cache": {"schema_version": 1, "encoding": "bytes", "payload": [0, 255, 128]}
  }
}
```

Plugin IDs follow the lowercase reverse-DNS grammar (2+ labels, 1–63 bytes per label, 253 bytes total). Section names are plugin-local, at most 253 bytes, with dot-separated segments matching `[a-z][a-z0-9_-]*`. Raw envelopes, from opening to closing brace, retain their exact whitespace, key order, escapes and number spelling. Namespace keys sort and use compact JSON. Unknown owners, disabled plugins and undeclared sections remain opaque, including unknown envelope fields.

The namespace shares the existing 4 MiB non-comment metadata budget, counting raw whitespace and envelopes. Section nesting is at most 28 below its root, within the file's 32-level limit. Duplicate keys, malformed envelopes, invalid byte arrays and excessive size/depth reject with a section path. The previous editor and original file remain intact if an open fails.

Enabled owners declare their schema versions and forward migration callbacks. The host stages migrations and validation through `DataCx`, which reads/writes only that owner's declared sections. Missing paths, newer stored versions or callback errors reject without modifying the document. Successful changes form one undoable edit and are saved only on a subsequent save.

The format remains 1.0. Old files without extensions load unchanged; an empty namespace is omitted and preserves existing canonical output. Older builds do not guarantee raw-byte retention. Existing core comments and palette data stay in place. Extensions are project data outside named voxel snapshots; runtime history and build-recipe replay preserve them. See [Plugins](https://quantum.worldwidewebb.co/docs/plugins/#data) for declarations, lifecycle and host integration.

## A complete example

[Download a valid one-voxel project](https://quantum.worldwidewebb.co/assets/minimal.voxl). The site crate tests this fixture against the actual format reader and verifies canonical byte equality.

```json
{
  "document": {
    "bounds": {"max": [16, 16, 16], "min": [0, 0, 0]},
    "id": "00000000-0000-4000-8000-000000000001",
    "name": "One voxel"
  },
  "format": "voxelite",
  "layers": [{
    "id": "00000000-0000-4000-8000-000000000002",
    "locked": false, "name": "Base", "visible": true,
    "voxels": [[0, 0, 0, 1]]
  }],
  "palette": {
    "order": [1],
    "slots": [{"index": 1, "name": "clay", "rgba": [180, 140, 100, 255]}]
  },
  "required_features": [],
  "version": {"major": 1, "minor": 0}
}
```

This displayed example is compacted for readability. The downloadable file uses canonical whitespace. Open it in the editor or with `quantum_document_open` to validate it; the app preserves the existing document if an open fails.
