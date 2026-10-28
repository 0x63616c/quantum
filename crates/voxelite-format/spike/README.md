# Payload loading experiment

Throwaway experiment, not an alternative `.voxl` specification. The nested
workspace and lockfile keep rkyv and the cache-eviction helper out of production
dependencies. No core derives, engine changes, unsafe code, or relaxed production
limits are needed. The only product change is using Serde's slice input adapter
for `from_bytes`; the bounded streaming reader remains available as the baseline.

From the repository root:

```sh
export CARGO_TARGET_DIR=/home/box/.cache/cargo-shared
cargo build --release --locked --manifest-path crates/voxelite-format/spike/Cargo.toml
taskset -c 2 "$CARGO_TARGET_DIR/release/payload-spike" /tmp/payload-spike 9
cargo test --locked --manifest-path crates/voxelite-format/spike/Cargo.toml
cargo clippy --all-targets --locked --manifest-path crates/voxelite-format/spike/Cargo.toml -- -D warnings
cargo fmt --manifest-path crates/voxelite-format/spike/Cargo.toml -- --check
cargo deny --manifest-path crates/voxelite-format/spike/Cargo.toml --config deny.toml check
```

CPU pinning is optional; choose an available CPU. Run without other builds.
Outputs go to the supplied directory, outside source control. `results.json`
records every sample, wall/CPU medians, range, file sizes, voxel/chunk counts, canonical
SHA-256 and Linux backing-read counters. `autumn-profile.json` contains separate
timings for the intact example including its recipe and named version. The
default is seven samples; the run count must be odd and at least three.

## Comparisons

- `json-reader`: unchanged `voxelite_format::read(&bytes[..])`, the previous
  `from_bytes` path: `BufReader` + bounded `Read` + Serde reader adapter.
- `json-slice`: same visitor, tuple/chunk construction, metadata, document and
  history checks, using the slice adapter. Output bytes are identical.
- `rkyv`: JSON metadata with empty voxel arrays plus sorted `Vec<Vec<[i32; 4]>>`.
  Explicit little endian, unaligned primitives, 32-bit offsets; schema 1 and a
  prototype marker. Every access calls the safe `rkyv::access`/bytecheck API.
- `packed-control`: the same metadata and 16-byte tuples, with manually bounded
  lengths and little-endian reads, using no archive framework. This distinguishes
  the benefit of binary numbers from a benefit specific to rkyv.

Both binary readers build the real `Volume` through its existing deserializer,
then validate the completed document. This preserves duplicate-coordinate,
palette, bounds, chunk and voxel checks; archive representation validation alone
is insufficient. There is no intermediate owned tuple vector on the load path.
The engine still requires mutable copy-on-write chunks, so this is **not a
zero-copy editor load**, even though archive access itself is zero-copy. Saving
includes validation, sorting/staging, metadata and encoding, not just serialization
of an already staged vector.

The prototype explicitly rejects build recipes and named versions. It preserves
the remaining project metadata through the format crate. Extending the wire
schema to version payloads and restoring recipe validation is deliberately left
to a future migration, if justified. The intact autumn file is profiled separately;
`autumn-content` deliberately removes its recipe and named version for codec
comparisons. It must not be described as a compatible roundtrip of that original.

`scenes.rs` copies the deterministic terrain generator from the render harness,
without its renderer dependency. Cases are 2,048 voxels, the 27,897-voxel autumn
content, and 1,040,000 voxels with 12 palette slots. Production permits at most
1,048,576 live voxels and 128 MiB of encoded data. A 10M project is outside that
contract; its tuple archive alone would be 160 MB, also above the byte cap. No
benchmark-only core feature is enabled to disguise those limits.

## Timing boundaries

- `decode-memory`: complete validated `Project` from resident bytes. Results are
  destroyed after the timer. Every codec is first warmed once and its order is
  rotated across samples. Canonical byte equality is checked outside timing.
- `open-warm`: bounded file read, codec, `DocumentSession::new`, and exact-byte
  fingerprint, matching the work of `DocumentSession::open`. The baseline map/path
  bookkeeping is omitted equally from all codecs. No rendering or UI work.
- `open-cold-requested`: same work after per-file `POSIX_FADV_DONTNEED`. Inspect
  `read_bytes`: nonzero page-rounded file lengths confirm guest backing reads.
  This cannot evict the hypervisor/storage cache and does not claim physical-device
  coldness. Warm reads should report zero. No global caches are dropped.
- `encode-memory`: validation, sorting, metadata and encoding, in a warm process.
- `save-durable-new`: encode, production atomic create-only persistence (including
  file and directory sync) and fingerprint. A fresh output is used every time.
  It excludes overwrite conflict detection, as does Save As to a new destination.
- `encode-first-process` and `save-durable-first-process`: first top-level save in
  a newly launched child. Process launch and loading the source model are outside
  the timer. The latter includes the former plus the same durable write. These
  report first-use/cold-process save behavior; there is no meaningful cold read
  cache for a new destination. They do not measure cold application startup.
- `phase/*`: diagnostic controls, not a sampling-profiler stack breakdown. Number
  parsing omits chunk construction and validation. Chunk construction uses native
  tuples from the first layer. Their isolated medians are not exactly additive.
  Build recipe replay is included by the actual project/engine open methods.

`rkyv-bytecheck-access` is structural validation/access only. Primitive arrays
have no invalid bit patterns, so this can check vector ranges without traversing
each voxel. Report the complete decode/open metrics when comparing editor load.

There is no compression in any variant, because canonical `.voxl` is uncompressed
JSON. File cache and process cache are different concepts; save timing labels
make that distinction explicit. CPU/allocator state, randomized chunk hash maps
and shared-VM scheduling still cause run spread. CPU time is additionally recorded
for in-process samples to expose descheduling; it excludes waiting on file IO.
Recheck on Apple silicon before
making product performance claims.
