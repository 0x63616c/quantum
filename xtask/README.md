# Repository tasks

Run from the repository with `cargo xtask --help`.

```sh
cargo xtask validate --strict
cargo xtask check --list
cargo xtask check tidy fmt
cargo xtask check plugin-deps brand docs-links docs-assets
cargo xtask validate --skip test
cargo xtask validate --only tidy fmt
cargo xtask bump-version patch --dry-run
cargo xtask version --check-tag v0.3.1
cargo xtask macos package --dry-run
cargo xtask macos check-appcast site/appcast.xml --allow-empty
cargo xtask bench --help
```

`validate` runs every registered check in order and continues after failures.
Missing Cargo tools skip unless `--strict` is set; broken installed tools fail.
Rustfmt and Clippy are probed because Cargo also lists rustup proxies for
components missing from the selected toolchain.
Filters accept multiple names or repeated flags. `tidy` remains a standalone
command. Remaining macOS, site and PII scan script checks, GPU checks and macOS
qualification remain separate.

The repository checks run in-process from `src/checks/`:

- `plugin-deps`: plugin and MCP dependency rules, including optional, renamed,
  target-specific, build and patched registry dependencies. Reads locked Cargo
  metadata offline first and retries online if that command fails. Preserves the
  success message on stdout and full failure diagnostics on stderr; the registry
  adds its timed summary on stdout. Failures exit with status 1.
- `brand`: `brand.toml` stability and consistency with packaging, package names,
  app IDs, MCP names, site metadata, project format, storage paths and skills.
- `docs-links`: local links in the glossary and product direction, resolved
  relative to each document with URL escapes decoded.
- `docs-assets`: local image links in the root guides, docs and crate READMEs;
  images must exist outside `screenshots/`. Checks Git ignores captures while
  keeping `docs/images/*.png` available.

Run their fixture and parsing tests with `cargo test -p xtask --locked checks::`.
Tests inject the shared command runner for Cargo metadata and Git ignore output;
no external-tool fixture scripts are needed. TOML, plist and source-pattern
parsing use `toml`, `plist` and `regex`; `glob` expands the original file scopes.

`bump-version` accepts `major`, `minor`, `patch` or a stable `X.Y.Z`. It checks
all current version references before writing, preserves their formatting and
runs `cargo update --workspace --offline`. If that command or a write fails,
it restores the original files and lockfile. It rejects downgrades, prereleases
and build metadata; an equal version does nothing. It never tags or publishes.

The library provides these public modules:

- `cmd`: argument-vector command builder, redacted diagnostics and an injectable
  `Runner`. `run` inherits stdio, `output` captures bytes, `read` returns trimmed
  UTF-8, and `status` leaves exit-code policy to the caller.
- `workspace`: checked root, common paths, tracked filenames and package versions
  from Cargo metadata. `target()` is the repository output path, independent of
  Cargo's build-cache override.
- `check`: named checks, selection, pass/fail/skip outcomes and timed reporting.
- `version`: stable version planning, current references and lockfile refresh.
- `macos`: credential-free child commands, private temporary files/keychain guards,
  codesign/notarization/stapling, DMGs, SHA-256, signed feeds and release notes.
  `Context` accepts a runner and explicit environment; packaging's platform and
  archive digest are injectable for Linux tests. CLI uses the pinned digest.

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

`Command::secret_stdin` sends private input over a pipe and redacts it in errors.
`SystemRunner::with_interruption` kills and reaps the active child when its atomic
exit-code flag is set. Only restoration commands use `during_cleanup` to finish
after interruption. The macOS CLI registers SIGINT/SIGTERM handlers for 130/143;
Drop guards restore keychains and remove temporary files on errors and unwinds.

The `macos` group includes `package [--skip-build] [--dry-run]`, `appcast
[--dry-run]`, `fetch-appcast <out>`, `check-appcast <file> [--allow-empty]` and
`release-notes`. Configuration still uses the documented release environment
variables. `check-appcast` also accepts paired `--release-dir` and `--previous`.
The registered `appcast` check validates `site/appcast.xml`, allowing an empty feed.

Build xtask before providing credentials. In release CI, credential-bearing
steps execute `target/release/xtask` directly; invoking Cargo there could expose
credentials to Cargo and build scripts before xtask can sanitize its children.
See the [release guide](../docs/releasing-macos.md) and
[port verification](macos-port-verification.md) for dry runs, command mapping and
the complete regression coverage map.
