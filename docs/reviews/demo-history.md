# Demo history self-review

Reviewed against `5afad1f` on `feat/demo-history`.

- **Loading was the real integration gap.** The original format had no persisted
  history and both editor and standalone MCP discarded any runtime timeline.
  The format now validates a bounded typed command recipe; those loaders replay
  it into the real engine. The attached MCP shadow still uses content only,
  because requests swap in the editor's authoritative engine.
- **Comparing two regenerated copies would not protect the artwork.** Tests pin
  both original snapshot and full-project hashes, independently of the new asset.
  A direct comparison with the original file also confirmed that removing only
  `build_history` leaves identical document JSON, including all layers, palette,
  comments and named versions.
- **A smaller history budget initially rejected the demo.** The full 49-step
  timeline fits 32 MiB (and the editor's default 128 MiB). A 16 MiB host now
  evicts early entries normally while reaching the exact same final document.
  Both cases are covered. Operation IDs are checked across the entire recipe,
  including entries that a smaller host evicts.
- **A stale recipe must never overwrite edited content on reopen.** Unchanged
  saves preserve the recipe. Changed content omits it, including when the input
  metadata template still contains the old recipe. This scope is documented;
  arbitrary interactive undo/redo remains session-local.
- **Repeatability needs more than seeded colors.** Geometry uses integer
  ellipsoid tests, ordered cell maps, fixed IDs, and logical minute offsets.
  Two builds and the checked-in file must match byte for byte. The old export
  example shares the same builder, so it cannot silently erase the history.
- **Stored commands are executable data.** Replay rejects mismatched final
  content, no-op steps, invalid commands, duplicate IDs, non-increasing times,
  unknown command fields, noncanonical UUIDs and exceeded resource budgets.
  Legacy projects retain their previous behavior.

Verification covers every undo/redo step back to the empty default scene,
read-only scrubbing, actual place/erase/paint effects, original content hashes,
format round trips, and editor save/reopen. Required checks:

```sh
cargo fmt --check
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
```

The generator and export examples also pass Clippy with `--all-targets`.
