# Repository tasks

Run from the repository with `cargo xtask --help`.

```sh
cargo xtask validate --strict
cargo xtask check --list
cargo xtask check tidy fmt
cargo xtask validate --skip test
cargo xtask validate --only tidy fmt
cargo xtask bump-version patch --dry-run
cargo xtask bench --help
```

`validate` runs every registered check in order and continues after failures.
Missing Cargo tools skip unless `--strict` is set; broken installed tools fail.
Rustfmt and Clippy are probed because Cargo also lists rustup proxies for
components missing from the selected toolchain.
Filters accept multiple names or repeated flags. `tidy` remains a standalone
command. Script checks, GPU checks and macOS qualification remain separate.

`bump-version` accepts `major`, `minor`, `patch` or a stable `X.Y.Z`. It checks
all current version references before writing, preserves their formatting and
runs `cargo update --workspace --offline`. If that command or a write fails,
it restores the original files and lockfile. It rejects downgrades, prereleases
and build metadata; an equal version does nothing. It never tags or publishes.

The library has four public modules:

- `cmd`: argument-vector command builder, redacted diagnostics and an injectable
  `Runner`. `run` inherits stdio, `output` captures bytes, `read` returns trimmed
  UTF-8, and `status` leaves exit-code policy to the caller.
- `workspace`: checked root, common paths, tracked filenames and package versions
  from Cargo metadata. `target()` is the repository output path, independent of
  Cargo's build-cache override.
- `check`: named checks, selection, pass/fail/skip outcomes and timed reporting.
- `version`: stable version planning, current references and lockfile refresh.

Add a check with one `Check::new(...)` or `Check::cargo(...)` entry in
`check::CHECKS`. In-process checks receive a workspace, runner and strict policy;
return `Outcome::Pass`, `Fail(messages)` or `Skip(reason)`. Register checks in
execution order, with cheap checks first. Add unit tests beside the module.

Add a command to the clap-derived `Task` enum and its dispatch in `main.rs`, and
include it in the grouped help template. Keep its work in a library module.
Use `workspace.cargo()` or `cmd::Command` instead of constructing shell strings.

`SystemRunner::dry_run()` prints suppressed commands and returns success with
empty output. `with_dry_run_output` supplies another fake result. Read-only
planning commands can use `.allow_in_dry_run()`. Filesystem operations must also
check `Runner::is_dry_run()`. Mark credentials with `secret_arg` or `secret_env`;
diagnostics redact their values, including captured error output. Inherited child
stdio is unfiltered; use capture for tools that may echo secrets. Tests can
implement `Runner` to record calls and return outputs without spawning tools.
