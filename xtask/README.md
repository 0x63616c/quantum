# Repository tasks

Run from the repository with `cargo xtask --help`. Tasks discover the workspace
from the current directory (including subdirectories), so shared Cargo build
artifacts do not redirect checks to the worktree where xtask was compiled.

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
command. Remaining site and PII scan script checks, GPU checks and macOS qualification
remain separate.

The repository checks run in-process from `src/checks/`:

- `plugin-deps`: plugin and MCP dependency rules, including optional, renamed,
  target-specific, build and patched registry dependencies. Reads locked Cargo
  metadata offline first and retries online if that command fails. Preserves the
  success message on stdout and full failure diagnostics on stderr; the registry
  adds its timed summary on stdout. Failures exit with status 1.
- `brand`: `brand.toml` stability and consistency with packaging, package names,
  app IDs, MCP names, site metadata, project format, storage paths and skills.
  Storage checks match the production path-selection functions, verifying macOS
  Application Support, XDG/fallback paths and the environment override without
  counting migration code or test assertions.
- `docs-links`: local file and directory links in every tracked `docs/**/*.md`,
  resolved relative to each document with URL escapes decoded. Parses inline and
  reference Markdown links and images, skipping code examples and external URLs;
  fragments and queries are stripped (anchors are not validated). Generated and
  untracked documents are excluded.
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

- `cmd`: xshell-backed argument-vector commands, redacted diagnostics and an injectable
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
Use `workspace.cargo()` for Cargo and `Command::from(cmd!(sh, "git status"))`
for ordinary command vectors (`use xshell::{cmd, Shell}`). Interpolation preserves
argument boundaries without invoking a shell. `Context::script(cmd!(sh, "git ..."))`
also applies release credential removal and the repository working directory.
Use `Command::new` for dynamic programs and append credentials with `secret_arg`
or `secret_env`. Keep execution through `Runner`, including in simple scripts.

The xshell adapter imports argv, cwd and environment overrides only. Apply stdin,
secrecy, dry-run and execution policy on `Command` after conversion; do not pass
xshell execution flags or `env_clear` to the adapter. `SystemRunner` uses xshell's
quiet, secret, ignore-status capture, then applies our exit policy and bounded,
redacted errors. Inherited stdio/status uses xshell's `std::process::Command`
adapter because xshell's `run` discards the exit status and closes stdin.
Interruptible and piped-input commands use that same adapter with our process
supervisor: xshell exposes neither a child handle nor process-group cancellation,
and cannot observe interruption while descendants retain output pipes. These
paths share the same injectable `Runner` and diagnostics.

`SystemRunner::dry_run()` prints suppressed commands and returns success with
empty output. `with_dry_run_output` supplies another fake result. Read-only
planning commands can use `.allow_in_dry_run()`. Filesystem operations must also
check `Runner::is_dry_run()`. Mark credentials with `secret_arg` or `secret_env`;
diagnostics redact their values, including captured error output. Inherited child
stdio is unfiltered; use capture for tools that may echo secrets. Tests can
implement `Runner` to record calls and return outputs without spawning tools.

`Command::secret_stdin` sends private input over a pipe and redacts it in errors.
`SystemRunner::with_interruption` kills and reaps the active child when its atomic
exit-code flag is set. On Unix it isolates each tool in a process group and stops
its descendants too, so inherited pipes cannot block cleanup. Only restoration commands use `during_cleanup` to finish
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
See the [release guide](../docs/releasing-macos.md) for dry runs and macOS
qualification. Portable release regression tests live under `src/macos/tests/`
and `tests/macos_workflows.rs`. The latter compiles its Rust tool fixture only
during tests; no fake-tool mode or fixture is included in the production binary.

## Release command ownership

Every external operation in the former packaging shell has an explicit owner.
File operations and parsers now run in-process; commands still use argument
vectors through `Context::command`, which removes all seven signing variables.
Private-tool failures suppress raw output rather than trying to identify every
possible certificate subject or authentication detail in it.

| Former command / operation | Rust implementation |
| --- | --- |
| `uname -s` | `macos::package::Options::is_macos`, production `cfg!(target_os = "macos")`; dry runs bypass execution |
| `cargo metadata ...` + Python JSON version selector | `macos::version`, using cleaned Cargo metadata; dry runs read `version::workspace_version` |
| `cargo build --release --locked -p voxelite-app --bin voxelite` | `package::build`, omitted by `--skip-build` |
| `cargo build --release --locked -p voxelite-mcp` | `package::build`, omitted by `--skip-build` |
| `mktemp -d` | `files::WorkDir`, tempfile with explicit 0700 directory permissions |
| Python canonical public-key validation | `files::public_key`, canonical 32-byte base64 and committed-file equality |
| Python secret decoding | `files::decode_secret`, in-process base64 into create-new 0600 files |
| Python plist read/update/dump | `package::bundle_plist`, sorted plist keys, identical fields and trailing newline |
| `mkdir`, `cp`, `rm`, `touch` | `mkdir`, `copy`, `write`, `files::remove`, `std::fs` and Drop guards |
| Ten `sips -z ...` calls | `package::build`, 16/32/128/256/512 points at 1× and 2× |
| `iconutil -c icns ... -o ...` | `package::build` |
| `curl --fail --location --retry 3 --proto '=https' --tlsv1.2 ...` | `package::build`, pinned Sparkle 2.10.1 URL |
| `shasum -a 256 --check` for Sparkle | `macos::sha256` before extraction, same pinned digest |
| `tar -xf ... -C dist/sparkle ./Sparkle.framework ./bin ./LICENSE` | `package::build` |
| `ditto` framework copy | `package::build`, retaining symlinks and modes |
| `security list-keychains -d user` + Python shlex parser | `sign::Keychain::prepare` and `sign::keychain_paths`, before mutations |
| `openssl rand -hex 32` | `sign::Keychain::import`, captured and never printed |
| `security create-keychain -p ...` | `sign::Keychain::import`, redacted secret argument |
| `security set-keychain-settings -lut 21600 ...` | `sign::Keychain::import` |
| `security unlock-keychain -p ...` | `sign::Keychain::import`, redacted secret argument |
| `security import ... -k ... -P ... -T /usr/bin/codesign -T /usr/bin/security` | `sign::Keychain::import`, redacted password, suppressed diagnostics |
| `security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k ...` | `sign::Keychain::import`, redacted password, suppressed diagnostics |
| `security list-keychains -d user -s ... temporary-keychain` | `sign::Keychain::import` |
| `security find-identity -v -p codesigning ...` + Python identity selector | `sign::Keychain::import` and `select_identity`, name/fingerprint matching |
| Nested/app `codesign --force --sign ...` | `sign::Signing::command` and `package::build`, Installer → Downloader (preserve entitlements) → Autoupdate → Updater → framework → MCP → main executable → app; runtime/timestamp only with Developer ID |
| `codesign --verify --deep --strict` app | `package::build`, private diagnostics |
| `ditto -c -k --keepParent` app ZIP | `package::build`, before app submission |
| `xcrun notarytool submit ... --key ... --key-id ... --issuer ... --wait --timeout 30m --output-format json` (ZIP and DMG) | `sign::notarize`, exact arguments, private diagnostics and Accepted-only result |
| Python notarization JSON status check | `sign::notarize` |
| `xcrun stapler staple`, `xcrun stapler validate` (app and DMG) | `sign::staple` |
| `spctl --assess --type execute` app | `package::build`, private diagnostics |
| `ditto` stapled app into stage | `package::build`, after app ticket validation |
| `ln -s /Applications` | `std::os::unix::fs::symlink` in `package::build` |
| `hdiutil create -volname Voxelite -srcfolder ... -fs HFS+ -format UDZO -ov ...` | `package::create_dmg` |
| DMG `codesign --force --keychain ... --sign ... --timestamp` and `--verify --strict` | `package::build`, private diagnostics |
| `spctl --assess --type open --context context:primary-signature` DMG | `package::build`, private diagnostics |
| `(cd dist && shasum -a 256 Voxelite.dmg > Voxelite.dmg.sha256)` and `cat` | `macos::sha256`, `write` and `print!`; same relative filename and two-space separator |
| EXIT/INT/TERM traps; `security delete-keychain`, `security list-keychains -d user -s ...`, `rm -rf` | `macos::Signals`, interruptible `SystemRunner`, `Keychain::finish`/Drop, `Marker` and `WorkDir` guards |

The other migrated commands map as follows:

| Former operation | Rust implementation |
| --- | --- |
| `gh api --paginate --slurp repos/0x63616c/voxelite/releases?per_page=100` | `appcast::fetch` |
| `gh release download ... --pattern appcast.xml --dir ...` | `appcast::fetch`; numeric stable selection and complete API asset-size validation |
| XML parsing/validation | `feed::validate`, `feed::validate_release`; original XML copied without serialization |
| DMG checksum check and `xcrun stapler validate` before signing feed | `appcast::generate` |
| HTML escaping release notes | `appcast::release_notes_html`, exact `html.escape` replacements |
| `generate_appcast --ed-key-file - --maximum-deltas 0 --maximum-versions 0 ...` | `appcast::generate`, secret stdin pipe and suppressed diagnostics |
| `sign_update --ed-key-file - --verify ...` | `appcast::generate`, secret stdin pipe and suppressed diagnostics |
| GitHub `generated=false` / `generated=true` outputs | `appcast::github_output`, unchanged ordering, skipped during dry run |
| `chmod 644` final XML/HTML | `files::public_permissions` |
| `git rev-parse --verify refs/tags/...^{commit}` | `release_notes::generate` |
| `git tag --merged ... --no-contains ...` | `release_notes::generate`, strict ancestors only |
| `git describe --tags --abbrev=0 --match=...` | `release_notes::generate`, stable tags only |
| `git log --no-merges --no-show-signature --no-color --pretty=- %s (%h) ... --` | `release_notes::generate`, fully qualified tag range |
