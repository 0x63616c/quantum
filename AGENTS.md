# Working on Quantum

## Scope and naming

- Treat Quantum as the product name. It was formerly Voxelite; current crates,
  binaries, bundle IDs and the repo slug still use `voxelite`. Do not rename them
  piecemeal. Use real names in commands, paths and identifiers.
- Build an Aseprite-style 3D voxel editor with a Rust/GPUI native macOS shell,
  `.voxl` projects, GLB/PNG exports and MCP agent workflows.
- Verify behavior in code and manifests. [Product direction](docs/adr/0003-product-direction.md)
  and the [domain glossary](docs/glossary.md) distinguish current implementation
  from deferred work and proposed seams. Do not present those as implemented
  features.
- Keep changes scoped to the task. Preserve unrelated work, identifiers,
  file-format compatibility and published release assets.

Brand-bearing identifiers live in [brand.toml](brand.toml), with current values
and planned display/internal names. Stable ones must not change without a migration.

## Architecture and ownership

Read [ADR 0002](docs/adr/0002-built-in-plugin-architecture.md) before moving feature
ownership. Here `A → B` means A may depend on B, not the reverse.

| Owner | Boundary |
| --- | --- |
| `voxelite-crash` | Own opt-in crash/feedback data, redaction, local panic reports and relay request preparation. No other workspace crate, UI or network client dependency. |
| `voxelite-core` | Own document invariants, chunks, palettes/layers, snapshots, atomic commands and undo. No other workspace crate, GUI or host dependency. |
| `voxelite-format` → `voxelite-core` | Own canonical `.voxl`, bounded parsing, build recipes and `session::DocumentSession` persistence. No normal renderer/plugin dependencies. |
| `voxelite-mesh` → `voxelite-core` | Own deterministic greedy geometry and geometry errors. No renderer/GPU/host dependency. |
| `voxelite-render` → `voxelite-core`, `voxelite-mesh` | Render immutable snapshots; own cameras, rasterization, occupancy, shadows and GPU resources. No document mutations or host/encoder/plugin dependencies. |
| `voxelite-plugin-api` → `voxelite-core`, `voxelite-mesh` | Expose portable manifests, contributions, capabilities, export errors and selected shared types/geometry. No GPUI, app, registry lifecycle or renderer. |
| `voxelite-plugin-host` → `voxelite-core`, `voxelite-plugin-api` | Own registration, compatibility, enabled declarations, dispatch, fault quarantine, timings and conformance. No shell, GPUI or renderer. |
| `crates/plugins/*` → `voxelite-plugin-api` | Implement optional features through portable capabilities; return proposals without accessing app internals. |
| `voxelite-ui` | Own GPUI components, tokens and text editing. Depend on GPUI and Unicode segmentation, not app/domain crates. |
| `voxelite-mcp` | Adapt core/format/render and injected API/host capabilities to stdio and Unix-socket sessions. No app/GPUI dependency or concrete plugin imports in the library. |
| `voxelite-plugins-builtin` | Compose concrete built-ins for the app and MCP binary. Infrastructure outside `crates/plugins/`; API/host never depend on it. |
| `voxelite-app` | Compose editor state, plugins, native UI, dialogs, render scheduling and native MCP attachment. Keep domain invariants in the engine. |
| `voxelite-bench` | Exercise the production renderer; isolate benchmark-only features from production builds. |
| `voxelite-site` / `xtask` | Generate site/docs from the MCP catalog / launch the render harness. Keep outside product runtime dependencies. |

- Keep engine/document/undo, native `.voxl` saving, renderer/camera and host/registry
  in core infrastructure. “Core” here spans crates, not just `voxelite-core`.
- Keep tools in `voxelite-plugin-tools`, command mirroring in
  `voxelite-plugin-mirror`, camera gizmo/grid in `voxelite-plugin-gizmo-grid`,
  palettes in `voxelite-plugin-palette`, discussions/pins in
  `voxelite-plugin-comments`, bounded GLB/PNG encoders and export callbacks in
  `voxelite-plugin-exporters`, and agent command metadata in `voxelite-plugin-agent`.
- Keep `voxelite-plugin-example` a declaration-only template outside the app.
  Do not imply dynamic loading, scripting or a plugin enablement UI exists.
- Admit edits through `Engine` transactions. Preserve atomic failure, no-op,
  revision, actor, history, lock/visibility and palette-slot semantics. Playback
  inspects immutable previews; never replay commands into the live engine.
- Register features in [core_plugin.rs](crates/voxelite-app/src/core_plugin.rs).
  Append optional built-ins through `voxelite-plugins-builtin::register_all`.
  Keep labels, icons, menu placement, bindings and order in contributions.
  [actions.rs](crates/voxelite-app/src/actions.rs) maps IDs to `Op` and builds shell
  metadata from enabled registry entries. Derive menus, command search, shortcut
  help and settings entries from that registry; preserve live enablement checks.

### Render isolation

- Keep `voxelite-mesh` public API limited to `Quad`, `Mesh`, `greedy_mesh`,
  `greedy_mesh_with_limit`, `MAX_QUADS`, `MeshError` and `MeshResult`.
- Keep `voxelite-render` public API at cameras (`Camera`, `PresetView`,
  `Projection`, `visible_bounds`), CPU images (`View`, `RenderOptions`,
  `RenderedImage`, `RasterCache`, `render`) and `RenderError`/`RenderResult`.
  Its optional `gpu` module exposes `Style`, macOS `Worker`/`Frame`, and headless
  capture/profiling entry points. Keep `Viewport`, occupancy, traversal, chunk
  meshes, shaders and GPU resources private; do not re-export mesh internals.
- Supply renderer callbacks from app/MCP to exporters. Encoding fixtures may use
  renderer dev-dependencies; normal dependencies may not.
- Run `normal_workspace_dependencies_preserve_renderer_boundary` in
  [boundary.rs](crates/voxelite-render/tests/boundary.rs). It checks the renderer's
  two normal workspace dependencies and forbids normal renderer dependencies in
  format, plugin API/host and the exporters plugin:

```sh
cargo test -p voxelite-render --locked --test boundary normal_workspace_dependencies_preserve_renderer_boundary
```

### Plugin dependency rule

- Allow only `voxelite-plugin-api` plus external crates in plugin normal/build
  dependencies, including optional, renamed and target-specific dependencies.
  Keep GPUI out of plugins. Expose required domain types through the API rather
  than directly depending on core, mesh, UI or app crates.
- Use dev-dependencies for `voxelite_plugin_host::conformance::check`. Exercise
  enable/disable/enable, validation, errors and panic isolation as appropriate.
- MCP accepts a host factory from its composition root. Its default `standalone`
  feature links `voxelite-plugins-builtin` for the binary; library consumers disable
  default features. Concrete plugins are allowed only as MCP dev-dependencies.
  Verify library isolation with `cargo check -p voxelite-mcp --locked --lib --no-default-features`.
- Follow [the plugin guide](crates/plugins/README.md) and run:

```sh
cargo xtask check plugin-deps
cargo test -p xtask --locked checks::plugin_deps
```

## Rust and change conventions

- Use Rust 2024 and pinned Rust 1.99.1; the workspace MSRV is 1.99.
- Preserve workspace `unsafe_code = "forbid"`, `missing_docs = "warn"` and Clippy
  warnings. App/render use `deny` solely to permit existing scoped macOS FFI
  modules. Do not widen those exceptions or add unsafe code to pure crates.
  Read [vendor/README.md](vendor/README.md) before changing GPUI surface lifetime,
  synchronization or the vendored patch.
- Document public APIs and invariants. Run rustfmt and Clippy with `-D warnings`.
- Follow typed `Result`/`thiserror` errors in libraries and contextual `anyhow`
  errors at app entry points. Propagate recoverable failures; do not panic on
  invalid documents, plugin outcomes or I/O. Preserve live state on failure.
- Put unit tests beside implementation (`#[cfg(test)]`, `tests.rs` or focused
  modules), integration/contract tests under crate `tests/`, and script tests in
  `scripts/tests/`. Test behavior and failure contracts.
- Update relevant docs, schemas and fixtures with behavior changes. Explain
  platform verification limits; Linux cannot prove AppKit/Metal behavior.
- Update `Cargo.lock` deliberately with dependency/version changes and verify
  with `--locked`. Preserve pinned GPUI dependencies and asset license notices.

## Build, test and lint

`cargo xtask validate` runs the registered gates: tidy, plugin-deps, brand,
docs-links, docs-assets, appcast, fmt, clippy (all targets), workspace tests, deny and
machete. Missing tools skip with a reason; use `--strict` to fail instead.
Use `cargo xtask check --list` to list checks,
`cargo xtask check tidy fmt` for named checks, or `validate --only tidy fmt` /
`validate --skip test` to select gates. Script and platform checks below remain
separate. See [xtask/README.md](xtask/README.md) for the library and registry.
Build ordinary xtask command vectors with `xshell::cmd!` and convert them to
`cmd::Command` (or use `macos::Context::script`) before running through the
injectable `Runner`. Apply secret markers and execution policy after conversion;
keep process-group cancellation in the shared runner.

Run from the repository root. These are the general macOS CI checks:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked --no-fail-fast
cargo xtask check plugin-deps
cargo test -p xtask --locked
```

| Purpose | Command |
| --- | --- |
| All registered Rust validation gates | `cargo xtask validate --strict` |
| List / run named checks | `cargo xtask check --list` / `cargo xtask check tidy fmt` |
| Preview / apply a stable version bump | `cargo xtask bump-version patch --dry-run` / `cargo xtask bump-version patch` |
| Start native editor | `cargo run -p voxelite-app --bin voxelite` |
| Open sample in light mode | `cargo run -p voxelite-app --bin voxelite -- examples/autumn-workshop.voxl --theme light` |
| Standard workspace tests | `cargo test --workspace` |
| Focused core tests | `cargo test -p voxelite-core --locked` |
| Editor model without GPUI | `cargo test -p voxelite-app --no-default-features --locked` |
| Shared controls and editing | `cargo test -p voxelite-ui --locked` |
| Plugin lifecycle/dispatch | `cargo test -p voxelite-plugin-host --locked` |
| Repository consistency (plugins, branding, documentation) | `cargo xtask check plugin-deps brand docs-links docs-assets` |
| Remaining Python regressions (site, PII scan) | `python3 -B -m unittest discover -s scripts/tests -p 'test_*.py'` |
| Tracked root entries, 5 MiB size limit and junk files | `cargo xtask tidy` |
| Dependency licenses, advisories and sources | `cargo deny check` |
| Unused dependencies | `cargo machete` |
| Task / render harness help | `cargo xtask --help` / `cargo xtask bench --help` |
| Pinned visual check | `cargo xtask visual --determinism` |
| Performance comparison | `cargo xtask bench --scene autumn --runs 3 --compare` |
| Full site build | `scripts/build-site.sh` |
| Site generator tests | `cargo test --locked -p voxelite-site` |
| Built-site validation | `python3 scripts/check-site.py` |
| Offline feed validation | `cargo xtask macos check-appcast site/appcast.xml --allow-empty` |

- Core/MCP, Python tests and site builds can run on Linux. Full workspace and
  UI builds need GPUI system dependencies; use the native guide's Linux setup.
- Keep the Linux GUI build for CI and UI capture work; see
  [ADR 0004](docs/adr/0004-keep-linux-gui-build.md).
- Run Metal/AppKit/Sparkle checks and `cargo xtask macos package` on macOS. CI uses
  Apple silicon, Xcode 27.0 and its Metal toolchain. Do not claim Mac coverage
  from Linux checks or mocked packaging tools.
- In the documented Linux harness workspace, use
  `CARGO_TARGET_DIR=/home/box/.cache/cargo-shared`. For lavapipe the guide specifies
  `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json` and
  `XDG_RUNTIME_DIR=/tmp`; verify local installations before using these paths.
- Website checks need a server and Playwright: follow [site/README.md](site/README.md)
  for `node scripts/check-site-install.mjs` and
  `node scripts/capture-site.mjs screenshots/site/final`.

## Rendering and performance

- Make speedups pixel-identical on the same device/input. Never lower render
  scale, approximate exact shadows, disable AO or reduce AA for an optimization.
  Preserve production 4× MSAA and 2× supersampling.
- Measure first, change a demonstrated bottleneck and keep the solution scoped.
  Do not build speculative rendering infrastructure. Include before/after numbers
  and correctness evidence in the PR.
- Use the independent GPU visual/performance harnesses. `xtask` runs in release
  with `--locked`; headless timings exclude NV12/IOSurface/GPUI presentation.
- Rendering visual jobs run on either Mac mini runner; both were verified
  bit-exact against the committed goldens with `--determinism` (#165). `bench`
  and `bench-baseline` always require the `golden` runner. Before a new runner
  takes visual jobs, give it a neutral label, add it to the `runner_label`
  dispatch options and pass a `visual` run on it. Never put personal information
  in runner labels.
- Use [bench/pinned-machine.toml](bench/pinned-machine.toml)'s Apple M2 Metal owner
  for authoritative [goldens](bench/golden/apple-m2-metal/). Run
  `cargo xtask visual --determinism` at default tight tolerances. Driver tolerances
  are not permission to change quality.
- Update goldens only for an intentional, explained appearance/input change with
  reviewed before/after images. Run `cargo xtask visual --rebaseline --determinism`
  on the pinned machine or dispatch **Rendering checks** with `rebaseline-visual`.
  Review every changed PNG and JSON sidecar. CI uploads artifacts without pushing
  references. Do not use `--force` to substitute another machine's goldens.
- Use `--local-baseline` to create an absent local set on another machine, then
  add `--determinism` for comparison. Replace an existing local set only with
  explicit `--local-baseline --rebaseline`. Never commit Linux/software-GPU local
  references as Metal goldens.
- Keep performance references in [bench/baseline/](bench/baseline/). Generate them
  explicitly with `cargo xtask bench --runs 3 --update-baseline` or `bench-baseline`
  dispatch. Do not combine that flag with `--compare` or `--input`. The current M2
  baseline (run 40613358223) was recorded with the runner otherwise idle; see bench/README.md.
- Match scene hashes, dimensions, machine/driver, shadow path and camera sampling.
  Attach JSON, markdown metric deltas, run spread and machine load. Performance
  dispatch is diagnostic on shared CI; required visual comparison must pass.
- Keep `terrain10m` opt-in through `--scene terrain10m`. Its `large-scenes` /
  `unbounded-voxels` features are benchmark-only; never use an all-features app
  build as a production build.

Record before/after on their respective revisions with identical settings:

```sh
cargo xtask bench --scene autumn --runs 3 --out target/bench/before.json
cargo xtask bench --scene autumn --runs 3 --out target/bench/after.json --compare target/bench/before.json --markdown target/bench/comparison.md
```

## Shared UI and text editing

- Use [voxelite-ui](crates/voxelite-ui/README.md) for buttons, icon buttons,
  segmented controls, menus, fields, sliders, toggles, swatches, cards, dialogs
  and chrome. Add reusable controls under `crates/voxelite-ui/src/` and export
  through `lib.rs`; do not hand-style one-off replacements in the app.
- Build every control from the library: `Button`, `IconButton` (square 36 px
  rail button, `IconButton::rail`, optional corner badge such as the Mirror axis
  letter), `Tab`/`TabBar`, `Segment`/`SegmentedControl`, `Panel` and `Popover`
  from [navigation.rs](crates/voxelite-ui/src/navigation.rs),
  [surfaces.rs](crates/voxelite-ui/src/surfaces.rs) and
  [components.rs](crates/voxelite-ui/src/components.rs). Buttons, tabs, segments
  and rail controls all use `radius::CONTROL` on all four corners; selected and
  hover fills must be rounded too. Do not write raw `px(..)` sizes, radii or
  colors in `ui.rs` or `ui/*.rs`: add a token in
  [tokens.rs](crates/voxelite-ui/src/tokens.rs) (`space`, `radius`, `metrics`,
  `Theme`) and a component if none fits. Component rules and the documented
  radius exceptions live in [crates/voxelite-ui/AGENTS.md](crates/voxelite-ui/AGENTS.md).
  Any change to a shared control needs before/after captures of every screen
  that uses it.
- Use [tokens.rs](crates/voxelite-ui/src/tokens.rs) for theme, spacing, radii,
  typography, heights and focus rings. Keep domain dispatch, modal state, focus
  restoration and file dialogs in the app.
- Keep the status bar static as values change. Use `StatusBar`, `StatusSegment`
  and fixed slots from [status.rs](crates/voxelite-ui/src/status.rs); reserve
  physical widths, pad placeholders/signs and handle overflow within the slot.
  Only the middle message region flexes. Coordinates, counts and FPS must not
  shift separators or neighboring controls.
- Follow [the art checklist](design/ART-CHECKLIST.md): let the viewport dominate,
  keep chrome neutral, support both themes, use consistent spacing and bundled
  Geist/Geist Mono, and reserve shadows for floating surfaces. Put secondary
  actions in menus and keep destructive actions visually secondary.
- Prefer icons for compact actions with descriptive tooltips and shortcut hints.
  Keep Save in File and its shortcut, without a toolbar Save button. Do not
  restore “Not saved yet” copy: retain the native edited indicator and dirty-only
  `Edited` title treatment; clean files have no subtitle.

### Every app text field uses the shared editor

- Use `voxelite_ui::TextField` (app re-export:
  [text_field.rs](crates/voxelite-app/src/text_field.rs)) or its `NumericField`
  wrapper. Never hand-roll a text input, caret, selection or key-down editor.
  OS file dialogs retain their platform-owned filename controls.
- Keep pure editing in [editing/](crates/voxelite-ui/src/editing/) and native input,
  clipboard, UTF-16/IME adaptation and painting in
  [text_field.rs](crates/voxelite-ui/src/text_field.rs). Use `Editor` mutations,
  `set()` for model replacement and `TextFieldEvent` for domain effects.
- Preserve [bindings.rs](crates/voxelite-ui/src/editing/bindings.rs)'s natural
  macOS editing behavior:

| Key | Behavior |
| --- | --- |
| Option + Left/Right | Move by word. |
| Command + Left/Right | Move to line start/end. |
| Option + Delete (Backspace) | Delete the previous word. |
| Command + Delete (Backspace) | Delete to line start. |
| Shift + movement | Extend selection, preserving its anchor. |
| Command + Z / Command + Shift + Z | Undo/redo within the focused field. |

- Consume field undo/redo even with empty local history; never fall through to
  document undo. Preserve grapheme-safe movement, selection reversal, clipboard,
  IME composition, Escape/commit policies, numeric stepping and focus behavior.
- Run `cargo test -p voxelite-ui --locked`. The
  [editing tests](crates/voxelite-ui/src/editing/tests.rs) include
  `binding_commands_match_the_full_reviewed_table`,
  `every_movement_and_deletion_binding_executes_its_declared_action`, and
  `modified_moves_use_head_and_shift_keeps_anchor`. Adapter tests include
  `installed_bindings_parse_and_only_resolve_in_a_field` and
  `field_undo_bindings_take_precedence_over_document_bindings`. Check native Mac
  IMEs and menu routing on macOS when changing the adapter.

## Generated content

- Regenerate the **app crate README** shortcut table after contribution changes:

```sh
UPDATE_README=1 cargo test -p voxelite-app --bin voxelite readme_shortcuts_match_registry
```

- Edit site templates in `crates/voxelite-site/templates/`, article bodies in
  `site/content/`, generation/metadata in `crates/voxelite-site/src/`, and skills
  in `skills/`. Run `scripts/build-site.sh`.
- The site reads the committed `spec/mcp/v1/tools.json` catalog without linking
  `voxelite-mcp`. After changing MCP tool metadata or schemas, refresh and commit
  it with `UPDATE_TOOLS_JSON=1 cargo test -p voxelite-mcp --locked --test tools_catalog`.
  The MCP test checks exact bytes against `VoxeliteServer::tools()`; the site
  preserves that tool and JSON key order in its generated references.
- Generated site pages, Markdown/LLM exports, crawl files, MCP catalog copies and
  downloadable skill/integration copies are ignored. Do not commit them: CI
  builds the site on PRs and `main`, then publishes trusted `main` builds to
  `site-dist`. Keep authored assets, `site/content/`, `site/vercel.json` and the
  input `site/appcast.xml` tracked.
- The generator also updates the tracked root `integrations/` examples. Do not
  maintain them separately; their source is `crates/voxelite-site/src/install.rs`
  and its templates.
- Generate signed appcasts through the release workflow and
  `cargo xtask macos appcast`; fetch release feeds with
  `cargo xtask macos fetch-appcast <out>`. Preserve signed XML/HTML bytes. Never hand-edit
  an appcast, sign with an unrelated key or rewrite published assets.
- Review built output and relative links. Keep generated site output and
  local benchmark outputs out of source commits.

## Commits and PRs

- Follow [CONTRIBUTING.md](CONTRIBUTING.md) for the issue-first workflow, labels,
  board statuses, branch names, Conventional Commit PR titles, PR bodies and the
  Definition of Done. Agents also follow the
  [project-tracking skill](.agents/skills/project-tracking/SKILL.md): read the
  board first, file an issue per ask, link every PR, and audit **Done when** items
  before claiming anything is done.
- PRs that change anything visible follow the
  [ui-pr-screenshots skill](.agents/skills/ui-pr-screenshots/SKILL.md):
  `cargo xtask screenshots --publish <PR#>` (or the **PR screenshots** CI job) fills
  the description's Before | After Screenshots section. Never paste UI screenshots
  by hand, and check the rendered PR page before asking for review.
- Branch from `main` as `<type>/<issue#>-<slug>`; never push directly to `main`.
  Keep PRs small, focused and limited to one concern. Review harshly before merge: inspect correctness,
  scope, failure paths and actual UI evidence.
- Author all commits you create as `Calum <6991398+0x63616c@users.noreply.github.com>`:

```sh
git config user.name Calum
git config user.email 6991398+0x63616c@users.noreply.github.com
```

- Title PRs as Conventional Commits (`feat`, `fix`, `refactor`, `chore`, `docs`,
  `test`, `ci`, `perf`, optional scope) with an imperative summary;
  [PR conventions](.github/workflows/pr-conventions.yml) checks the title and a
  `Closes`/`Fixes`/`Refs #N` link. Do not add `Co-authored-by` trailers or
  tool/agent attribution. Squash merge as `Title (#N)`, then delete the branch.
- Explain the problem, resulting behavior and validation. Include before/after
  screenshots for UI changes, both themes where relevant, and before/after
  numbers for performance changes. Report untested platforms.
- Require green applicable checks before merging:

| Workflow | Required evidence |
| --- | --- |
| [CI macOS](.github/workflows/ci-macos.yml) | Plugin rule/tests and packaging task tests (GitHub-hosted Linux), then Clippy and workspace tests on the Mac mini. The ad-hoc DMG builds on `main` and on PRs that touch packaging inputs. Formatting runs in CI Linux. |
| [Rendering checks](.github/workflows/render-checks.yml) | Visual/determinism gate when path filters match; request explicitly for other rendering-impacting changes. |
| [Deploy site](.github/workflows/deploy-site.yml) | Ubuntu build/verification on PRs; production publication only for trusted `main` triggers. |

- Set `CARGO_BUILD_JOBS: 4` in every self-hosted macOS job env. Two runners
  share one 8 GB Mac mini; this limits CPU and memory contention and swap.
- Root `Cargo.toml` and `Cargo.lock` changes do not trigger **Rendering checks**.
  A nightly **Rendering checks** run on `main` covers dependency bumps; dispatch
  `visual` on the PR when a render dependency changes. Only `xtask/src/main.rs`, the
  visual/bench launcher, is included from `xtask/` in the rendering path filter.

- Respect owner-only, same-repository guards on self-hosted macOS jobs. Fork PRs
  do not run them; a skipped job is not verification. Arrange trusted verification
  before merge. Never remove guards or execute untrusted changes on the runner
  through `pull_request_target`.
- Never commit secrets, private keys or machine-specific credentials.
  `python3 scripts/scan-pii.py` scans source/assets and reachable Git history and
  prints redacted findings to stdout. Run it when relevant, review its report
  deliberately, and do not treat it as proof of no secrets.
  Keep private values out of logs and PRs.
- Write release-worthy subjects: `cargo xtask macos release-notes`
  includes non-merge commit subjects in public release notes.

## Releases

- Do not tag casually. Use a focused `chore(release): prepare vX.Y.Z` PR for version,
  lockfile and affected metadata, then a separately authorized stable
  `vMAJOR.MINOR.PATCH` tag matching the app version.
- Follow [docs/releasing-macos.md](docs/releasing-macos.md) and
  [Release macOS](.github/workflows/release-macos.yml). Validate real signing,
  notarization, installation and updating on macOS; mocked tests are insufficient.
- Keep the committed Sparkle public key stable. Optional `SPARKLE_PUBLIC_ED_KEY`
  must match `packaging/macos/sparkle_public_ed_key.txt`. Keep
  `SPARKLE_ED_PRIVATE_KEY` and Apple signing/notarization secrets in the configured
  secret store; never print, replace or commit them during routine work.
- Preserve released tags, DMGs, notes and signed feeds. Retry post-release site
  deployment through **Deploy site** on `main`, rather than republishing assets.
  That workflow deploys the built `site-dist` directory using `VERCEL_TOKEN`.

## Key references

| Reference | Use |
| --- | --- |
| [README.md](README.md) | Product overview, crate inventory, builds and entry points. |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Issue-first workflow, PR rules and Definition of Done. |
| [Product direction](docs/adr/0003-product-direction.md) | Product scope and deferred work; verify implementation. |
| [Domain glossary](docs/glossary.md) | Domain terminology; distinguish proposed seams from current code. |
| [ADRs](docs/adr/), [ADR 0001](docs/adr/0001-record-architecture-decisions.md) | Architecture decisions and review order. |
| [Core architecture](docs/architecture/core.md) | Document/transaction invariants. |
| [Plugin guide](crates/plugins/README.md), [ADR 0002](docs/adr/0002-built-in-plugin-architecture.md) | API, host and built-in ownership. |
| [Rendering harness](docs/perf/rendering-harness.md), [bench references](bench/README.md) | Golden/performance contracts and evidence. |
| [UI library](crates/voxelite-ui/README.md), [UI rules](crates/voxelite-ui/AGENTS.md), [art checklist](design/ART-CHECKLIST.md) | Controls, text editing and visual rules. |
| [Release guide](docs/releasing-macos.md) | Signing, Sparkle key custody, appcasts and qualification. |
