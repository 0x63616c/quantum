# Quantum rename compatibility

The product, Cargo packages, binaries, MCP tool names, plugin declarations,
installed skills, artwork filenames and new release assets use Quantum. The
exhaustive legacy-string allowlist lives in [brand.toml](../brand.toml), with
exact strings, path globs and reasons. It is intentionally separate from the
future enforcement work in #140. Repository renaming is #91; no remote or
published release is changed here.

## macOS updates

Keep `co.worldwidewebb.voxelite` as both CFBundleIdentifier and the window APP_ID.
The pinned Sparkle 2.10.1 installer finds an update by its installed filename or
bundle name, then falls back to matching bundle identifiers. Changing both the
filename and identifier would prevent older installations finding Quantum.app.
See [SUInstaller.m, lines 81–111](https://github.com/sparkle-project/Sparkle/blob/2.10.1/Autoupdate/SUInstaller.m#L81-L111).
The existing Sparkle public signing key is unchanged.

New downloads contain `Quantum.app`, executable `Quantum`, `quantum-mcp` and
`Quantum.icns`; new release assets are `Quantum.dmg` and `Quantum.html`. Sparkle
updates an existing `Voxelite.app` **in place**, so its on-disk filename can remain
old while all application content becomes Quantum. Quit the application and
rename that bundle to `Quantum.app` in Finder if desired; update discovery still
works through the retained bundle ID. Do not install a second copy alongside it.

`SUNormalizeInstalledApplicationName` is not a supported runtime plist switch.
The pinned framework's [build setting](https://github.com/sparkle-project/Sparkle/blob/2.10.1/Configurations/ConfigCommon.xcconfig#L94)
`SPARKLE_NORMALIZE_INSTALLED_APPLICATION_NAME` is off. Even a framework rebuilt
with it enabled computes the name from the **installed host**'s bundle metadata,
not the downloaded application's name; see
[SUNormalization.m](https://github.com/sparkle-project/Sparkle/blob/2.10.1/Sparkle/SUNormalization.m).
A new payload cannot change how an already installed 0.3.x updater installs that
payload. We do not move a running signed bundle behind Sparkle or Launch Services.
Real signed 0.3.x → Quantum installation remains a macOS release qualification
step; Linux packaging tests cannot prove it.

## Website and immutable release history

The canonical site and new SUFeedURL use `https://quantum.worldwidewebb.co`.
Old installations continue fetching
`https://voxelite.worldwidewebb.co/appcast.xml` directly with HTTP 200. Both hosts
must serve the **same signed XML bytes**. Every other old-host path redirects to
the matching canonical path with HTTP 308. Release enclosures and signed notes
use GitHub asset URLs, so no other site-host path needs an updater exception.

`site/vercel.json` is copied into the built site artifact, then deployed from
`site-dist` by Deploy site. The owner must attach the canonical hostname and DNS
to the existing Vercel project while retaining the old hostname. Until #91,
workflow guards and release-fetch commands accept both owner repository slugs;
new public links already use the canonical slug. Existing signed feeds retain
old GitHub URLs and `Voxelite.dmg` / `Voxelite.html` names. Validation accepts
both exact asset naming schemes and repository prefixes, including size metadata
returned by GitHub after its repository redirect. It never edits signed history.

## Local data and projects

Startup migrates the old `voxelite` directory to `quantum` on every platform:
Application Support on macOS, XDG_DATA_HOME when configured elsewhere, or
`~/.local/share`. Earlier macOS XDG data also remains eligible. Migration only
runs when the destination does not exist; even an empty destination wins.
Rename is atomic and refuses replacement. Cross-device migration copies into a
private staging directory, then publishes it without replacement, leaving the
source intact. Read/copy errors stop startup and discard only the incomplete
staging directory. Settings, palettes, absolute recent paths and thumbnails are
preserved; user project files are never moved.

`QUANTUM_DATA_DIR` takes precedence over the deprecated `VOXELITE_DATA_DIR`.
Explicit overrides are used as-is and suppress automatic migration. Sessions
use PID plus UUID socket names, so stale migrated sockets cannot block startup;
cross-device copies omit those transient socket entries. New untitled saves
start in `~/Documents/Quantum`; existing projects retain their parent folder.

The `.voxl` extension, JSON discriminator `"format": "voxelite"`, and MIME
`application/vnd.voxelite.project+json` stay frozen. Renaming these wire
identifiers would cause 0.3.x readers to reject otherwise identical projects.
Existing project fixtures and benchmark scenes remain byte-identical.

## MCP and plugin identities

MCP advertises server `quantum` and tool names beginning `quantum_`. Clients
should rediscover tools with `tools/list`; saved host configurations must use
`quantum-mcp` (or the new bundle's embedded command). The response-envelope
`api_version` remains `voxelite.v1` so external validators continue accepting
it. This field is output-only; no request schema takes an API-version string.

Built-in plugin declarations use `dev.quantum.*`. Loading persisted enabled IDs
and setting keys accepts `dev.voxelite.*` aliases and stores canonical IDs;
other namespaces remain unchanged. The current app has no persisted plugin
settings/enablement UI and built-ins declare no extension-data sections.
Comments are core project content, so there is no project extension namespace
to rewrite or migrate. Unknown third-party extension data still round-trips.
