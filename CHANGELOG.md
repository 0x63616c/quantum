# Changelog

## [Unreleased]

## [0.4.0]

### Changed

- Voxelite is now Quantum. Crates, commands, integrations, skills, downloads and
  the canonical website use the new name. On first launch, settings, palettes,
  recent paths and thumbnails migrate to the `quantum` data directory without
  replacing an existing destination. User documents stay where they are, and
  existing `.voxl` projects still open. New saves use the Quantum discriminator
  and need a current reader. Bundle-keyed macOS state migrates once; automatic
  updates retain a compatible archive layout while adopting the new bundle ID. See
  [rename compatibility](docs/rename-compatibility.md) for update details.

### Added

- add opt-in crash report and feedback previews ([#207](https://github.com/0x63616c/quantum/pull/207))
- preflight automated release credentials ([#201](https://github.com/0x63616c/quantum/pull/201))
- rebuild the Welcome window as concept A (split layout) ([#151](https://github.com/0x63616c/quantum/pull/151))
- use the ASCII shimmer for loading states ([#152](https://github.com/0x63616c/quantum/pull/152))
- add Quantize app icon ([#185](https://github.com/0x63616c/quantum/pull/185))
- add cargo xtask release with an offline dry-run default ([#183](https://github.com/0x63616c/quantum/pull/183))
- add the Quantum design system to quantum-ui ([#177](https://github.com/0x63616c/quantum/pull/177))
- add opt-in crash and feedback reporting core crate ([#153](https://github.com/0x63616c/quantum/pull/153))
- before/after UI screenshots in PR descriptions ([#130](https://github.com/0x63616c/quantum/pull/130))

### Fixed

- validate tracked documentation links in the active worktree ([#182](https://github.com/0x63616c/quantum/pull/182))
- verify platform-specific brand storage paths ([#184](https://github.com/0x63616c/quantum/pull/184))
- write files atomically and store settings via JsonStore in the macOS settings dir ([#155](https://github.com/0x63616c/quantum/pull/155))
- show an error instead of quitting when a file can't be opened ([#158](https://github.com/0x63616c/quantum/pull/158))
- let Save As be cancelled instead of trapping the user ([#159](https://github.com/0x63616c/quantum/pull/159))
- never expose the session socket before it is 0600 ([#59](https://github.com/0x63616c/quantum/pull/59))
- use the bundle identifier as the window app_id ([#60](https://github.com/0x63616c/quantum/pull/60))
- step GPU supersampling down instead of dropping to CPU on large viewports ([#58](https://github.com/0x63616c/quantum/pull/58))
- describe non-voxel demo steps instead of +0 −0 ([#63](https://github.com/0x63616c/quantum/pull/63))
- root attached file tools at the project folder, not cwd ([#61](https://github.com/0x63616c/quantum/pull/61))

### Performance

- add opt-in Tracy frame and render tracing ([#210](https://github.com/0x63616c/quantum/pull/210))
- add divan benchmarks for greedy meshing and large scenes ([#147](https://github.com/0x63616c/quantum/pull/147))
- decode .voxl from a byte slice instead of a buffered reader ([#128](https://github.com/0x63616c/quantum/pull/128))
- re-record the M2 baseline on an idle runner ([#78](https://github.com/0x63616c/quantum/pull/78))

### Refactored

- migrate stable identifiers to Quantum ([#218](https://github.com/0x63616c/quantum/pull/218))
- split oversized app and GPU modules ([#212](https://github.com/0x63616c/quantum/pull/212))
- rename the app to Quantum ([#208](https://github.com/0x63616c/quantum/pull/208))
- enforce shared corner radius tokens ([#202](https://github.com/0x63616c/quantum/pull/202))
- split the editor shell into focused modules ([#198](https://github.com/0x63616c/quantum/pull/198))
- merge greedy meshers into quantum-mesh ([#199](https://github.com/0x63616c/quantum/pull/199))
- port macOS packaging, appcast and release scripts ([#154](https://github.com/0x63616c/quantum/pull/154))
- use slotmap typed IDs for layers, comments and history ([#162](https://github.com/0x63616c/quantum/pull/162))
- consolidate built-in manifests and conformance tests ([#160](https://github.com/0x63616c/quantum/pull/160))
- define each MCP tool once with typed arguments ([#142](https://github.com/0x63616c/quantum/pull/142))
- split command.rs and add undo/redo property tests ([#118](https://github.com/0x63616c/quantum/pull/118))

### Other

- add pinned graphify code-graph refresh ([#221](https://github.com/0x63616c/quantum/pull/221))
- drop the pre-rename repository slug after renaming to quantum ([#213](https://github.com/0x63616c/quantum/pull/213))
- bump the github-actions group with 3 updates ([#219](https://github.com/0x63616c/quantum/pull/219))
- show current editor and Welcome goldens ([#216](https://github.com/0x63616c/quantum/pull/216))
- generate site, docs, changelog and blog naming from brand.toml ([#211](https://github.com/0x63616c/quantum/pull/211))
- typos and brand checks driven by brand.toml ([#209](https://github.com/0x63616c/quantum/pull/209))
- add headless GPUI goldens for key screens ([#204](https://github.com/0x63616c/quantum/pull/204))
- limit per-change Mac checks to macOS-specific crates ([#175](https://github.com/0x63616c/quantum/pull/175))
- bump the github-actions group with 4 updates ([#197](https://github.com/0x63616c/quantum/pull/197))
- add a cheaper ci-dist cargo profile for PR DMG builds ([#174](https://github.com/0x63616c/quantum/pull/174))
- align grouped Dependabot updates with PR conventions ([#196](https://github.com/0x63616c/quantum/pull/196))
- enforce strict lints, crate boundaries and a shrink-only styling baseline ([#186](https://github.com/0x63616c/quantum/pull/186))
- run workspace tests with cargo-nextest and keep doctests ([#181](https://github.com/0x63616c/quantum/pull/181))
- scan secrets with gitleaks, pin every action by SHA and minimise workflow permissions ([#180](https://github.com/0x63616c/quantum/pull/180))
- build the macOS DMG nightly instead of on every main push ([#173](https://github.com/0x63616c/quantum/pull/173))
- explain the CARGO_BUILD_JOBS limit on the Mac mini runners ([#188](https://github.com/0x63616c/quantum/pull/188))
- cap CARGO_BUILD_JOBS at 4 on the Mac mini jobs ([#172](https://github.com/0x63616c/quantum/pull/172))
- add launch capture states for every screen and Settings section ([#187](https://github.com/0x63616c/quantum/pull/187))
- add insta snapshots for exporters, site output and .voxl round trips ([#164](https://github.com/0x63616c/quantum/pull/164))
- run rendering visual checks on either Mac runner, golden only for benchmarks ([#176](https://github.com/0x63616c/quantum/pull/176))
- drop Cargo.toml, Cargo.lock and most of xtask from the rendering-checks paths ([#171](https://github.com/0x63616c/quantum/pull/171))
- share one sccache server between the Mac mini runners ([#163](https://github.com/0x63616c/quantum/pull/163))
- build the DMG only when packaging changes and move script tests off the Mac mini ([#157](https://github.com/0x63616c/quantum/pull/157))
- remove stale code, assets and docs left after the browser removal ([#141](https://github.com/0x63616c/quantum/pull/141))
- add model labels, dependency and disk conventions ([#143](https://github.com/0x63616c/quantum/pull/143))
- run rendering checks only on the golden runner ([#77](https://github.com/0x63616c/quantum/pull/77))
- add issue-first project conventions, templates and PR checks ([#115](https://github.com/0x63616c/quantum/pull/115))
- record decision to keep the Linux GUI build ([#146](https://github.com/0x63616c/quantum/pull/146))
- keep a warm per-runner cargo target dir and skip macOS build on docs-only PRs ([#129](https://github.com/0x63616c/quantum/pull/129))
- Remove the browser (wasm) editor: code, build, CI and site page ([#116](https://github.com/0x63616c/quantum/pull/116))
- Add AsciiSkeleton and AsciiReveal to quantum-ui (port of ascii-reveal) ([#117](https://github.com/0x63616c/quantum/pull/117))
- Docs: remove mentions of the browser editor; Quantum is macOS-only ([#75](https://github.com/0x63616c/quantum/pull/75))
- Drop the browser editor line from macOS release notes ([#74](https://github.com/0x63616c/quantum/pull/74))
- Hoist shared dependencies into workspace.dependencies ([#72](https://github.com/0x63616c/quantum/pull/72))
- Display the website and browser editor as Quantum ([#73](https://github.com/0x63616c/quantum/pull/73))
- Move exact shadows and lighting into a render-pass plugin ([#34](https://github.com/0x63616c/quantum/pull/34))
- Display the app as Quantum ([#71](https://github.com/0x63616c/quantum/pull/71))
- Keep concrete plugins out of the quantum-mcp library ([#69](https://github.com/0x63616c/quantum/pull/69))
- Add brand.toml: inventory of brand-bearing identifiers before the Quantum rename ([#70](https://github.com/0x63616c/quantum/pull/70))
- Fold quantum-export into plugin-exporters; keep plugin-api renderer-free ([#66](https://github.com/0x63616c/quantum/pull/66))
- Add Linux CI with cargo-deny, cargo-machete and xtask tidy ([#65](https://github.com/0x63616c/quantum/pull/65))
- Build the site from the committed MCP tool catalog (drop quantum-site → quantum-mcp) ([#68](https://github.com/0x63616c/quantum/pull/68))
- Move design/ and screenshots/ out of the repository ([#67](https://github.com/0x63616c/quantum/pull/67))
- Debug overlay: FPS, frame timing, chunk, draw and memory stats (off by default) ([#56](https://github.com/0x63616c/quantum/pull/56))
- Stop committing generated site output ([#64](https://github.com/0x63616c/quantum/pull/64))
- remove stale planning and review notes from the repo root ([#62](https://github.com/0x63616c/quantum/pull/62))
