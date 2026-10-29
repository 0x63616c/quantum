# Preparing reusable crates for publication

Issue #112 prepares `quantum-ui` and `quantum-format`. No publish is authorized.
The packages inherit the workspace version, MIT license, edition and MSRV.
Their archives include local license files, public source, README and independent
examples. UI also includes its upstream NOTICE and design-system reference.
Repository-dependent galleries, demos and fixture suites remain checkout-only;
Cargo removes excluded example/test targets from the normalized manifest.

## Review on Linux

Use the pinned Rust toolchain from the repository root:

```sh
cargo test -p quantum-format -p quantum-ui --locked
cargo run -p quantum-ui --locked --example editing
cargo package -p quantum-core --list
cargo package -p quantum-format --list
cargo package -p quantum-ui --list
cargo publish -p quantum-core --dry-run --locked
cargo publish -p quantum-format --dry-run --locked
cargo publish -p quantum-ui --dry-run --locked
cargo xtask validate --strict
cargo xtask lint
```

Dry runs do not upload. Format's registry dry run requires the matching
`quantum-core` release to exist; the workspace dependency carries both a path
and a version so Cargo can normalize it for publication. Never remove that
version when changing workspace dependency declarations. Keep it aligned with
workspace bumps. Check normalized manifests and archive lists for external paths,
missing examples, license notices and accidental assets before publication.

## Owner release steps

1. Verify crates.io ownership and availability of all three `quantum-*` names,
   choose the release version, and explicitly authorize publishing. Credentials
   stay in the owner's credential store; never add them to this repository.
2. Validate on a physical Mac: the gallery in both themes, native text input/IME,
   focus, menu undo routing and accessibility. Linux tests do not prove these.
3. Resolve the workspace-only `gpui-apple` patch described in
   [vendor/README.md](../vendor/README.md). Downstream Cargo builds do not inherit
   `[patch.crates-io]`. Check upstream GPUI 1.25.3 behavior and document or ship an
   approved fix before recommending macOS use; issue #114 tracks upstreaming.
4. Run clean archive builds/tests and registry dry runs after dependencies become
   available. Publish `quantum-core` first, then `quantum-format`; UI can publish
   independently after its macOS dependency review. Wait for registry indexing
   and repeat format's dry run before publishing it.
5. Build docs from the unpacked archives, check links and examples, then approve
   and perform uploads manually. This change does not publish or create tags.

The existing format snapshot suite freezes canonical bytes and trailing newlines
against the versioned spec fixtures. The archive-contained portable test also
checks slice/reader equivalence, document identity, bytes and semantic hashes.
Do not rebaseline snapshots to accommodate unintended wire changes.
