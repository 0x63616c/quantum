# quantum-format

Bounded, canonical `.voxl` JSON serialization and durable document sessions for
[Quantum](https://github.com/0x63616c/quantum). MIT licensed; Rust 2024, MSRV 1.99.
The library depends on `quantum-core`, without a GUI, renderer or plugin runtime.

## Read and round-trip

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let input = std::fs::File::open("scene.voxl")?;
let project = quantum_format::read(std::io::BufReader::new(input))?;
let canonical = project.to_bytes()?;
let reopened = quantum_format::from_bytes(&canonical)?;
assert_eq!(reopened.document, project.document);
assert_eq!(reopened.to_bytes()?, canonical);
# Ok(())
# }
```

`Project::new` wraps a validated core document. `read` streams a reader;
`from_bytes` parses an existing slice. `Project::write` streams canonical output.
All return typed `FormatError`s. A writer can receive partial output on failure;
use `storage::safe_write` for durable replacement, or `session::DocumentSession`
for live engine ownership, disk fingerprints and optimistic save receipts.
Hosts select `SavePolicy`; keep filesystem work off the UI thread and apply save
receipts to the originating session so intervening edits remain dirty.

Inputs are limited to 128 MiB and metadata to 8 MiB, with separate comment/other
budgets. Duplicate keys/voxel coordinates, incompatible major versions and unknown
required features are rejected. Optional inert metadata and supported minor
versions survive saves. Runtime revisions and raw undo history are not serialized.
`semantic_hash` excludes the portable view and optional build recipe. Build
recipes contain typed commands, are validated by replay, and are not arbitrary code.

The [format specification, schema and fixtures](https://github.com/0x63616c/quantum/tree/main/spec/voxl/1.0)
describe compatibility and canonical ordering. The legacy discriminator is read
for compatibility; output uses the Quantum discriminator.

## Examples and tests

From a repository checkout:

```sh
cargo run -p quantum-format --locked --example roundtrip -- input.voxl output.voxl
cargo test -p quantum-format --locked
cargo run -p quantum-format --locked --example demo-history -- --check
```

`roundtrip` refuses to overwrite an existing destination. It is included in the
package along with a portable integration test. Repository-only demo generation,
spec fixtures and snapshot suites stay in the checkout; they test canonical byte
snapshots, idempotence, metadata preservation, hostile inputs, history replay and
session failure contracts. No renderer is required.

## Publication

These crates are prepared for review, not published. See the repository's
[crate release checklist](https://github.com/0x63616c/quantum/blob/main/docs/publishing-crates.md).
`quantum-core` must be published at the matching version before this crate.
