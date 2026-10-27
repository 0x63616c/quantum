# Plugins

Built-in features. One shared core.

Voxelite's plugins are statically linked Rust crates shipped with the editor. They declare features through `voxelite-plugin-api`; `voxelite-plugin-host` validates and enables them. This page describes the implemented boundaries in this development build, including the parts of the plugin architecture still to come.

## What stays in core

The document engine, validation, atomic transactions, undo/redo, native `.voxl` loading and saving, renderer and camera remain core responsibilities. The plugin host and its registry also belong to core. Disabling an optional feature does not remove its existing document content or weaken saving and history guarantees.

The shell owns windows, menus, command search (Cmd-K), settings surfaces and the shortcuts sheet. It derives command labels, icons, menu placement and keybindings from enabled registry contributions. The app still maps command IDs to its `Op` dispatch enum and supplies availability checks; a declaration alone does not implement an action. Settings have declarative schemas, but the existing settings surfaces are still shell adapters. GPUI presentation uses the shared `voxelite-ui` components and natural text editing; plugins receive no GPUI context.

## Built-ins in this tree

`app_host()` in `crates/voxelite-app/src/core_plugin.rs` registers the required `dev.voxelite.core` command provider, followed by these seven optional plugins. Each package lives under `crates/plugins/`.

| Feature    | Implementation and responsibility                                                                                                                                                                                                                                                                                   |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tools      | `voxelite-plugin-tools` (`dev.voxelite.tools`). Place, Erase, Paint, Fill, Box and Picker (eyedropper). Tools return previews and typed command proposals; the host handles picking and the engine admits edits.                                                                                                    |
| Mirror     | `voxelite-plugin-mirror` (`dev.voxelite.mirror`). Mirror X/Y/Z settings and a command transform applied to pending tool edits before transaction admission. Axes default off.                                                                                                                                       |
| Gizmo/grid | `voxelite-plugin-gizmo-grid` (`dev.voxelite.gizmo-grid`). Ground grid and camera orientation widget drawing and interaction. The gizmo orbits or snaps the camera; it is not a voxel transform tool.                                                                                                                |
| Palette    | `voxelite-plugin-palette` (`dev.voxelite.palette`). Presets, local library and palette manager behavior, swatch choices and edit proposals, HEX/GPL/JASC-PAL import, and the PAL exporter. The host handles storage; palette invariants and edit execution stay in core.                                            |
| Comments   | `voxelite-plugin-comments` (`dev.voxelite.comments`). Discussion panel behavior, placement, filtering, pin overlays and MCP comment requests. Stored comments remain core snapshot content, including history and named versions.                                                                                   |
| Agent/MCP  | `voxelite-plugin-agent` (`dev.voxelite.agent`). Metadata for the Agent connection and Extensions shell commands. Their execution remains in the app; both have `agent_visible = false`. MCP transport and tool dispatch remain in `voxelite-mcp`, with palette, comments and exports using enabled plugin bindings. |
| Exporters  | `voxelite-plugin-exporters` (`dev.voxelite.exporters`). GLB and PNG byte encoders plus HEX/GPL palette exports. The host supplies snapshots/rendering and writes the output. Native `.voxl` saving remains available when exporters are disabled.                                                                   |

**Shadows, lighting and FPS/stats are not separate plugins yet.** The architecture calls for them to become optional built-ins, but this tree has no such plugin packages or registrations. Shadow and lighting implementation remains in `voxelite-render`; viewport frame measurement, statistics display and its preference remain in the app. The agent plugin likewise does not yet own the whole MCP implementation. See [MCP tools](https://voxelite.worldwidewebb.co/docs/mcp.md) for the current wire interface.

## Manifests and contribution points

A plugin implements the `Plugin` trait. `Plugin::manifest` returns a `Manifest` with a stable reverse-DNS `PluginId`, `name_key` and `description_key`, implementation `version`, required `api_version`, and the `builtin`, `default_enabled` and `can_disable` flags. `API_VERSION` is currently `0.7.0`; it is independent of plugin versions, the `.voxl` format version and section schema versions.

A contribution point is a supported kind of declaration the host can discover and present or dispatch. `Plugin::contribute` fills a `Registrar` with data-only declarations:

*   `CommandContribution`: command ID, label, JSON argument schema, optional shortcut/context, icon, menu, presentation order and agent visibility.
*   `ToolContribution` and `CommandTransformContribution`: interactive tools and ordered rewrites of pending commands.
*   `PanelContribution` and `FormatContribution`: host-rendered panels and import/export format metadata.
*   `RenderPassContribution`: a declared insertion slot, `BeforeScene`, `AfterScene` or `Overlay`; no renderer resources are handed to the plugin.
*   `SettingContribution`: a namespaced Bool, Choice or Number schema and typed default.
*   `DataSectionContribution`: a plugin-local section name and positive schema version for project data.

Declarations do not automatically supply behavior. Bind implemented features with `Plugin::tool`, `command_transform`, `overlay`, `palette_panel`, `comments_panel` or `exporter`. Host-provided contexts such as `ToolCx`, `PaletteCx`, `CommentsCx`, `DataCx` and `ExportCx` expose only the relevant queries or proposals. Plugins do not receive the engine, app or an unrestricted service locator. `CommandCx` and `ActivateCx` are currently stubs.

## Enablement and lifecycle

The seven optional built-ins ship with `builtin = true`, `default_enabled = true` and `can_disable = true`. The host can disable them with `Host::disable` and re-enable them with `Host::enable`. The required `dev.voxelite.core` provider has `can_disable = false`. There is no plugin enablement preferences UI yet.

`Host::new` uses manifest defaults. `Host::with_enabled` accepts an explicit `EnabledSet` instead, while still selecting required plugins. Registration validates manifests, API compatibility and declarations. Enablement checks contribution-ID and default-keybinding conflicts before calling `activate`; a conflict leaves the candidate disabled. `Registry` exposes only successfully enabled declarations. Disabling calls `deactivate` and removes those declarations from the live registry; repeated requests for the same state are no-ops.

Lifecycle hooks and feature callbacks run behind `catch_unwind`. A panic in contribution, lifecycle or feature callbacks faults the plugin, removes its declarations and prevents retries for that host's lifetime. A panic while reading the manifest rejects registration because identity has not been established. `HostEvent` reports faults and transitions; `Timings` records callback costs and advisory budget overruns. Ordinary callback errors return to the caller without automatically faulting the plugin.

Panic isolation requires unwinding: it cannot recover from an abort or roll back arbitrary external effects performed by plugin code. This is an in-process API boundary, not a sandbox. Hosts stage document proposals and plugin data before committing them through core validation.

## The dependency rule

Plugin packages may depend on `voxelite-plugin-api` and external crates in normal and build dependencies. Every other workspace crate is forbidden there, including `voxelite-core`, `voxelite-app`, `voxelite-render` and `voxelite-ui`. The rule also covers optional, renamed and target-specific dependencies. Shared core types and export encoders are exposed through the API crate.

Dev-dependencies are exempt so tests can use `voxelite_plugin_host::conformance::check`. `scripts/check-plugin-deps.py` enforces the boundary from Cargo metadata.

## Versioned project data

A plugin may declare `DataSectionContribution { section_name, schema_version }` and store data at `extensions[plugin_id][section_name]` in `.voxl`. Each section envelope contains a positive `schema_version`, an `encoding` of `json` or `bytes`, and a `payload`. `DataCx` can read or write only the enabled owner's declared sections.

`Host::load_data` stages forward `DataMigration` callbacks and `Plugin::load_data` validation for enabled owners. Missing migration paths, newer stored schemas and callback errors reject the operation without changing the engine; panics also fault the owner. Successful changes commit together as one undoable edit. Hosts must run this load step when re-enabling a plugin against an open document too.

Unknown owners, disabled plugins and undeclared sections retain their raw envelope bytes, including whitespace and key order. Current sections retain their bytes unless changed. The format remains 1.0, and empty extensions are omitted. Existing palette and comment data stay in their core fields; they have not been migrated into extensions. See [plugin extension data in the .voxl format](https://voxelite.worldwidewebb.co/docs/format/#plugin-data) for the wire example, limits and preservation rules.

## Write a built-in plugin

Start with `crates/plugins/voxelite-plugin-example`. Its `ExamplePlugin` uses ID `dev.voxelite.example` and declares one `CommandContribution`, `dev.voxelite.example.command`, with an empty-object argument schema. It implements no feature or command execution, has `builtin = false` and `default_enabled = false`, and is not linked into the app.

1.  **Check the template.** Read its `Cargo.toml` and `src/lib.rs`, then run the commands below from the repository root.
2.  **Create a package under `crates/plugins/`.** Copy the example into a new directory, choose a unique package name and reverse-DNS plugin ID, and update contribution IDs and label keys. The workspace already includes `crates/plugins/*`. Keep `voxelite-plugin-api` as the only workspace dependency outside tests.
3.  **Declare and bind the feature.** Implement `manifest` and `contribute`, then the relevant behavior lookup. Return typed proposals through the API; leave transaction admission, file I/O and GPUI presentation to the host. For an optional shipped built-in, set `builtin`, `default_enabled` and `can_disable` to `true`.
4.  **Test the contract and behavior.** Keep the example's conformance test, replacing `ExamplePlugin` with your type. Add feature tests for errors, disable/re-enable behavior and unchanged core document guarantees. Use the dependency checker to catch accidental workspace coupling.
5.  **Wire it into the app.** Add the new package to `crates/voxelite-app/Cargo.toml` and its instance to `app_host()`. For shell commands, extend the `Op` ID mapping in `crates/voxelite-app/src/actions.rs` and the app's execution/availability adapters. A new panel or capability also needs its shell adapter. Rebuild and test the app; merely adding a crate or declaration does not make it a working feature.

```sh
cargo test --locked -p voxelite-plugin-example
python3 -B scripts/check-plugin-deps.py
python3 -B -m unittest discover -s scripts/tests -p 'test_plugin_deps.py'
```

The example's conformance test is:

```rust
#[test]
fn conforms_to_plugin_contract() {
    voxelite_plugin_host::conformance::check(|| Box::new(ExamplePlugin));
}
```

The harness checks declarations and exercises an enable/disable/enable cycle, including implementation availability for supported bindings. It complements feature-specific tests; it does not supply missing command execution.

## What is not supported yet

There is no third-party plugin loading, dynamic library loading, WASM plugin runtime, scripting API or plugin marketplace. The Rust API is not a stable binary ABI. The browser editor is compiled to WebAssembly, but that does not make it a WASM plugin loader. Built-in plugins are developed and reviewed as part of this repository and linked at build time.
