# Repository tasks

Run from the repository with `cargo xtask --help`. Tasks discover the workspace
from the current directory (including subdirectories), so shared Cargo build
artifacts do not redirect checks to the worktree where xtask was compiled.

```sh
cargo xtask lint
cargo xtask validate --strict
cargo xtask check --list
cargo xtask check tidy fmt
cargo xtask check plugin-deps brand docs-links docs-assets
cargo xtask validate --skip test
cargo xtask validate --only tidy fmt
cargo xtask release patch # offline dry run is the default
cargo xtask release tag   # preview separately authorized tagging
cargo xtask bump-version patch --dry-run
cargo xtask version --check-tag v0.3.1
cargo xtask macos package --dry-run
cargo xtask macos check-appcast site/appcast.xml --allow-empty
cargo xtask icons
cargo xtask icons --check
cargo xtask bench --help
```

CI runs `cargo nextest run --workspace --locked --profile ci`, followed by
`cargo test --doc --workspace --locked` because nextest excludes doctests.
Install the CI version locally with `cargo install cargo-nextest --version 0.9.150 --locked`.
`cargo xtask validate` uses nextest when installed and always runs doctests;
otherwise it falls back to `cargo test --workspace --locked` (including doctests).
The `ci` profile disables fail-fast and retries. Set `CI=true` locally to match
CI snapshot verification; insta inherits that variable under nextest too.

## PR screenshots

For the independent display-free regression gate, run `cargo xtask ui-snapshots`.
Explicit `--rebaseline` updates reviewed references; `--screen 'settings-*'`
filters the key set. See [UI snapshots](../docs/ui-snapshots.md).

`cargo xtask screenshots` captures only the key set: `welcome`, `editor-dark` and
`editor-light`. The other registered surfaces are opt-in, keeping routine CI time
independent of the size of the capture catalog. Capture quality and sizes stay fixed.

```sh
cargo xtask screenshots --screen 'settings-*'
cargo xtask screenshots --screen 'panels-*' --screen 'dialogs-about-*'
cargo xtask screenshots --screen editor-dark
cargo xtask screenshots --screen '*' # explicitly capture the whole catalog
cargo xtask screenshots --publish 123
```

`--screen` selects exactly the union of its matches, deduplicated in registry order.
Selectors are exact names or prefixes ending in one `*`; quote patterns in the
shell. Unknown names, unsupported syntax and patterns matching nothing fail before
output is cleaned or either binary is built. The registry is `src/screenshots.rs`.

For migration PRs, add one plain line at column zero in the body, outside the
generated Screenshots section: `Screens: panels-* settings-general-*`. CI and
`--publish` read this whitespace-separated list and add its matches to the key set.
A missing or empty line uses only the defaults; duplicate lines fail. It is data,
not shell syntax. Explicit `--screen` options override PR-body selection locally.
To preview the CI selection locally:

```sh
gh pr view 123 --json body -q .body > target/screenshot-pr-body.md
cargo xtask screenshots --pr-body target/screenshot-pr-body.md
```

`--pr-body FILE` and `--screen` are mutually exclusive. The workflow fetches the
body with `gh pr view` into a file, then passes its path as a separate argument;
Rust validates each selector against the registry before using fixed capture args.
Dispatch's optional `screens` field accepts the same list and replaces the body's
extra selectors for that run; blank uses the body. Body edits alone do not rerun
the workflow: push or dispatch after changing the line.

Only changed/new/removed images and their zoom crops are pushed to `pr-assets`.
Unchanged screens appear in a collapsed text list; an entirely unchanged run makes
no assets commit. All captures remain in `target/pr-screenshots/` and CI artifacts.
See the [screenshot skill](../.agents/skills/ui-pr-screenshots/SKILL.md) for Linux
dependencies, publishing and visual review.

## Validation and task library

`validate` runs every registered check in order and continues after failures.
Missing Cargo tools skip unless `--strict` is set; broken installed tools fail.
Rustfmt and Clippy are probed because Cargo also lists rustup proxies for
components missing from the selected toolchain.
Filters accept multiple names or repeated flags. `tidy` remains a standalone
command. Remaining site and PII scan script checks, GPU checks and macOS qualification
remain separate.

`lint` runs `boundaries`, `source-lints` and workspace/all-target Clippy with
`--locked -- -D warnings`. It fails if Clippy is missing. These checks are also
registered in `validate`, which runs Clippy only once.

The repository checks run in-process from `src/checks/`:

- `boundaries`: locked `cargo metadata --all-features` graph traversal from
  `quantum-core`, `quantum-format` and `quantum-mesh`. Rejects direct/transitive
  dependencies on `gpui`, its published package `gpui-unofficial`, or
  `quantum-app`, including dev/build/optional and target-specific edges.
  Errors show the dependency path; cycles terminate.
- `source-lints`: syntax-aware rules described below.
- `workflows`: parse every workflow YAML file, require explicit top-level
  permissions and full commit SHAs for action/reusable workflow references. Local
  actions and Docker images pinned by SHA-256 digest are exempt. Runs in CI Linux
  and in `validate`.
- `plugin-deps`: plugin and MCP dependency rules, including optional, renamed,
  target-specific, build and patched registry dependencies. Reads locked Cargo
  metadata offline first and retries online if that command fails. Preserves the
  success message on stdout and full failure diagnostics on stderr; the registry
  adds its timed summary on stdout. Failures exit with status 1.
- `brand`: `brand.toml` stability and consistency with packaging, package names,
  app IDs, MCP names, site metadata, project format, storage paths and skills.
  Also scans tracked text and filenames for legacy branding, allowing only live,
  justified literal/path exceptions in `brand.toml` compatibility entries.
  Diagnostics include line/column; unused exceptions fail.
  Storage checks match the production path-selection functions, verifying macOS
  Application Support, XDG/fallback paths and the environment override without
  counting migration code or test assertions.
- `typos`: source/documentation spelling via typos-cli 1.51.1 and `_typos.toml`.
  Install with `cargo install typos-cli --version 1.51.1 --locked`; missing tools
  fail this gate. CI Linux installs that version and runs both brand and typos.
- `docs-links`: local file and directory links in every tracked `docs/**/*.md`,
  resolved relative to each document with URL escapes decoded. Parses inline and
  reference Markdown links and images, skipping code examples and external URLs;
  fragments and queries are stripped (anchors are not validated). Generated and
  untracked documents are excluded.
- `docs-assets`: local image links in the root guides, docs and crate READMEs;
  images must exist outside `screenshots/`. Checks Git ignores captures while
  keeping `docs/images/*.png` available.
- `icons`: exact regeneration of the Quantize SVG derivatives, including PNGs,
  ICNS, site marks and the design preview. See [artwork notes](../packaging/macos/ARTWORK.md).

Source rules use `syn` and token trees, including inactive platform code and
inline tests. Comments and string contents are not source violations. All tracked
Rust files under `crates/`, `xtask/` and `examples/` have a 1,000-line file budget
and a 100-line function budget (signature through closing brace, including blank
lines/comments). Free functions, methods and default trait methods count;
closures and macro-generated functions do not have separate function budgets.
Qualified module/impl/function names identify functions; mutually exclusive cfg
implementations with the same name share their maximum size budget.

In `crates/quantum-app/src/`, including submodules, the styling rules count:

- `px` calls with numeric literal, negated or parenthesized literal arguments;
- `rgb`/`rgba` calls with hexadecimal literal arguments, and `hsl`/`hsla` calls
  with numeric literal arguments;
- string literal patterns, including match arms, `if let` and `matches!`, and
  string literal `==`/`!=` comparisons. This deliberately includes protocol and
  setting dispatch as well as element/action IDs: use typed enums/IDs for new code.

Qualified calls and styling inside macro tokens count. Macro string dispatch is
checked for literal arms/alternatives and equality comparisons; `matches!` checks
all string literals. Arbitrary generated macro expansions and renamed imports
are not expanded. Dynamic document colors and `px(metrics::TOKEN)` are permitted;
put new control styling in `quantum-ui` components/tokens. Shared UI owns styling
and is exempt from the app styling rules, but retains the source size budgets.

Across `crates/`, `raw-radius` rejects numeric literals inside `rounded` and
`rounded_*` arguments, including corner-specific calls and macro tokens. Fixed
shortcuts such as `rounded_sm()` also fail. Use `quantum-ui` radius tokens or
token-based concentric calculations; `rounded_full()` remains the intentional
circular-shape exception. Only `quantum-ui/src/tokens.rs` and its `tokens/`
submodules may define literal radii. Numeric expected values in style assertions
remain independent checks of the rendered geometry.

`xtask/lint-allowlist.json` records existing debt by file/rule and qualified
function. Every entry must exactly match the remaining count or oversized line
count: growth and new entries fail; fixes also fail until the entry is reduced or
removed. Do not regenerate or increase the baseline. Stage new files before
running checks because the scope is Git's tracked files. The initial baseline
preserves current screens while #134 migrates them to shared components.

The baseline itself is checked against `HEAD` locally, or `LINT_BASE_REF` when
set. CI supplies the PR base SHA (previous revision for pushes), using full Git
history, so increasing the baseline alongside new violations fails. Only the
initial introduction may bootstrap when the base revision has no baseline file.
An invalid/missing base revision fails. Git-recognized file renames carry their
existing rule/function budgets to the new path; counts may still only shrink,
and new rules or functions gain no allowance. To reproduce the full PR comparison:
`LINT_BASE_REF=<base-sha> cargo xtask lint`.

Run their fixture and parsing tests with `cargo test -p xtask --locked checks::`.
Tests inject the shared command runner for Cargo metadata and Git ignore output;
no external-tool fixture scripts are needed. TOML, plist and source-pattern
parsing use `toml`, `plist` and `regex`; `glob` expands the original file scopes.

`bump-version` accepts `major`, `minor`, `patch` or a stable `X.Y.Z`. It checks
all current version references before writing, preserves their formatting and
runs `cargo update --workspace --offline`. If that command or a write fails,
it restores the original files and lockfile. It rejects downgrades, prereleases
and build metadata; an equal version does nothing. It never tags or publishes.

`release <X.Y.Z|patch|minor|major>` defaults to an offline plan with version diffs,
Conventional Commit changelog and release-notes previews. `--dry-run` explicitly
selects the same mode. `--execute` checks clean/up-to-date main, tag absence and
successful HEAD CI, then creates a local release branch and commit; PR creation
is printed only. After squash merge, `release tag --execute` verifies the release
commit on main and creates/pushes the annotated tag to trigger the existing macOS
workflow. No signing credentials are used by this command. See the
[release flow](../docs/releasing-macos.md#prepare-a-release) for safety boundaries,
changelog rules, failure recovery and macOS prerequisites.

The library provides these public modules:

- `cmd`: xshell-backed argument-vector commands, redacted diagnostics and an injectable
  `Runner`. `run` inherits stdio, `output` captures bytes, `read` returns trimmed
  UTF-8, and `status` leaves exit-code policy to the caller.
- `workspace`: checked root, common paths, tracked filenames and package versions
  from Cargo metadata. `target()` is the repository output path, independent of
  Cargo's build-cache override.
- `check`: named checks, selection, pass/fail/skip outcomes and timed reporting.
- `release`: offline release previews, local release commits and explicitly authorized tagging.
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

`screenshots` prepares its Welcome recents fixtures in-process via
`screenshot_fixtures::prepare_welcome` before invoking the capture script. Five
real example copies live beneath the output's disposable `home/`; the script
copies the shared `welcome-data/recents.json` into each screen's isolated data
directory. Both revisions use identical project paths, bytes and timestamps.

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
| `cargo build --release --locked -p quantum-app --bin quantum` | `package::build`, omitted by `--skip-build` |
| `cargo build --release --locked -p quantum-mcp` | `package::build`, omitted by `--skip-build` |
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
| `hdiutil create -volname Quantum -srcfolder ... -fs HFS+ -format UDZO -ov ...` | `package::create_dmg` |
| DMG `codesign --force --keychain ... --sign ... --timestamp` and `--verify --strict` | `package::build`, private diagnostics |
| `spctl --assess --type open --context context:primary-signature` DMG | `package::build`, private diagnostics |
| `(cd dist && shasum -a 256 Quantum.dmg > Quantum.dmg.sha256)` and `cat` | `macos::sha256`, `write` and `print!`; same relative filename and two-space separator |
| EXIT/INT/TERM traps; `security delete-keychain`, `security list-keychains -d user -s ...`, `rm -rf` | `macos::Signals`, interruptible `SystemRunner`, `Keychain::finish`/Drop, `Marker` and `WorkDir` guards |

The other migrated commands map as follows:

| Former operation | Rust implementation |
| --- | --- |
| `gh api --paginate --slurp repos/0x63616c/quantum/releases?per_page=100` | `appcast::fetch` |
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
