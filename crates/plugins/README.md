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
implements no feature or command execution. Future built-in feature plugins live
alongside it; this layout does not add third-party loading, WASM or scripting.
