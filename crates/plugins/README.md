# Plugin crates

Place plugin packages in `crates/plugins/<crate-name>/`. This directory is a
container, not a Cargo package. The workspace includes `crates/*` and
`crates/plugins/*`, excluding only `crates/plugins/Cargo.toml`: excluding the
whole directory would also exclude its plugin packages.

Plugins may depend on `voxelite-plugin-api` and external crates. No other workspace
crate is allowed in normal or build dependencies, including optional, renamed and
target-specific dependencies. In particular, plugins must not depend directly on
`voxelite-core`; any shared core types must be exposed through the API crate.
Dev-dependencies are exempt so plugin tests can use
`voxelite_plugin_host::conformance::check`.

Run the dependency rule and its fixture tests with:

```sh
python3 -B scripts/check-plugin-deps.py
python3 -B -m unittest discover -s scripts/tests -p 'test_plugin_deps.py'
```

`voxelite-plugin-example` is a minimal template with ID `dev.voxelite.example`,
`builtin = false` and one command declaration. It is not linked into the app and
implements no feature or command execution. The tools, mirror, palette, comments
and agent built-ins live alongside it and currently contribute command metadata
only; execution remains in the app. They ship enabled and can be disabled by the
host, though the shell does not expose enablement preferences yet. This layout
does not add third-party loading, WASM or scripting.

The palette plugin owns the headless palette manager: typed choices with stable
identities and read-only origins, automatic custom copies on source edits, stable
slot/display-order edits, cancellable RGB previews, and sixteen persisted recent
colors. Legacy `PaletteAction::Edit` operations remain supported. All mutations
return `PaletteOutcome`; the host persists `library_bytes` before adopting state.
No palette plugin callback reads or writes files or mutates a document directly.

`PaletteView::formats` and `voxelite_plugin_palette::FORMATS` list HEX, GPL and
JASC-PAL for import/export grouping. `ExportLibrary` returns text in the outcome;
the PAL exporter is also registered as `dev.voxelite.palette.pal`. Existing HEX/GPL
exporters remain with the exporters plugin. See [the manager contract](../../docs/palette-manager.md)
for preview, naming, selection and persistence details.
