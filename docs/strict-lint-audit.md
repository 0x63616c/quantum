# Strict lint rollout audit (#138)

The requested lint rollout is **not implemented**. This audit stops at the
user-requested high-volume checkpoint rather than suppressing existing library
violations or committing a CI gate that fails immediately.

## Reproduction

On starting revision `e420729`, temporarily create `clippy.toml` with:

```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
```

Run:

```sh
cargo clippy --workspace --all-targets --locked --message-format=json -- \
  -W clippy::unwrap_used -W clippy::expect_used -W clippy::panic \
  -W clippy::todo -W clippy::dbg_macro -W clippy::print_stdout
```

Deduplicate compiler diagnostics by primary source filename, line and lint
because Cargo checks both library and test targets. This reports 263 distinct
sites: 132 `unwrap_used`, 93 `expect_used`, 6 `panic`, and 32 `print_stdout`.
There are no `todo` or `dbg_macro` findings in this Linux configuration.
These are diagnostic counts, **not production-only counts**: helper functions
in test modules and test infrastructure can still trigger the lints despite the
three test exemptions. Targets behind macOS configuration are not audited.
Remove the temporary configuration after the audit.

## Blocking volume and contracts

| Location | Findings outside ordinary test functions | Contract to resolve |
| --- | --- | --- |
| `crates/voxelite-plugin-host/src/conformance.rs`, `check` | 24 expects, 11 unwraps | Exported assertion harness, compiled into the library; its documented behavior deliberately panics on failed plugin conformance. Test exemptions do not cover it. |
| `crates/voxelite-core/src/` | 13 expects | Engine transactions, entity insertion, shapes and extension history rely on internal invariants. Changes must preserve atomicity and history semantics. |
| `crates/voxelite-render/src/` (excluding test output) | 14 unwraps, 1 expect | Rasterization, occupancy, chunk meshing and GPU resource setup need explicit failure or invariant handling while preserving pixels. |
| `crates/voxelite-app/src/` | 3 expects in the library; 13 expects and 2 unwraps in the native binary | Construction, command contributions, shortcuts, thumbnails, settings and viewport failure handling cross several UI paths. |

The 35 sites in the exported conformance harness alone require a deliberate
contract decision. Replacing them mechanically with defaults would weaken
assertions; changing the function to return a result changes how every plugin's
conformance tests report failure. Adding blanket exceptions would bypass the
requested policy. The remaining library/runtime findings also span document
invariants and rendering, beyond a small lint configuration change.

A coordinator can stage a separately scoped cleanup of the conformance harness
and runtime invariants, then resume #138. No follow-up issue was created because
this task prohibits GitHub writes. Legitimate CLI output can receive individual,
reasoned `#[expect(clippy::print_stdout)]` annotations during that cleanup; this
audit does not propose exceptions for runtime unwraps or expects.

## Acceptance audit

All five issue checklist items remain unmet:

- Workspace lint declarations: unchanged; enabling them now fails CI.
- Transitive core/format/mesh boundary check: not added.
- App styling and string-ID ratchet: not added; migration baseline still needed.
- File/function size ratchet: not added; limits and baseline still needed.
- `cargo xtask lint`: not added or wired into CI/validation.

No production code, workflow, allowlist or lint policy changed. This document is
partial investigation evidence and must not be described as closing #138.

## Baseline verification

Commands ran on Linux with `CARGO_BUILD_JOBS=3` and the provided shared target
directory. The temporary audit configuration was removed before these checks.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed under the existing workspace lint policy. |
| `cargo test -p xtask --locked` | Failed: 103 passed, 1 failed in the library suite. `checks::brand::tests::repository_brand_consistency` expects an environment/path inventory that differs from `local_settings.rs`. Cargo stops before later test binaries. |
| `cargo xtask validate` | Failed: 9 checks passed, 2 failed, 0 skipped. The brand check and workspace tests fail on that same inventory mismatch. Tidy, plugin-deps, docs-links, docs-assets, appcast, fmt, clippy, deny and machete passed. |

No workflows changed, so actionlint is not applicable. No native macOS, Metal,
AppKit or packaging behavior was verified. The baseline failure is unrelated to
this documentation-only change and was not repaired as part of the audit.
