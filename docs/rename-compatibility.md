# Quantum rename compatibility

Quantum now uses `co.worldwidewebb.quantum` for its macOS bundle and windows,
`quantum.v1` for MCP envelopes, `quantum` for the project discriminator and
`application/vnd.quantum.project+json` for the media type. This supersedes the
previous decision to freeze those identifiers. `brand.toml` records the current
values and the exhaustive legacy-string allowlist. `cargo xtask check brand`
rejects unlisted occurrences and stale exception paths/literals, including built
site output when `QUANTUM_BRAND_SITE` is set.

## Existing macOS installations: direct, authenticated update

A normal Quantum.app payload with the new ID alone **cannot** update every old
installation. Sparkle 2.10.1's
[SUInstaller](https://github.com/sparkle-project/Sparkle/blob/2.10.1/Autoupdate/SUInstaller.m#L23-L155)
searches recursively for the installed filename or the installed host's bundle
name plus extension; bundle identifier is only the fallback. A Voxelite host
matches neither Quantum.app nor the new ID. Symlinks and Finder aliases are
explicitly ignored.

The packager now puts these entries in every new Quantum.dmg:

- `Quantum.app`: the canonical signed, notarized bundle.
- `Update Compatibility/Voxelite.app`: a real, identical copy of that bundle,
  including the **new** ID, Quantum bundle name, executable, version, public key
  and canonical feed URL. Nothing inside the signed bundle is edited.
- `Update Compatibility/README.txt`: tells manual installers to use the root app.
- `Applications`: the existing convenience symlink.

The compatibility directory is deliberately visible.
[SUDiskImageUnarchiver](https://github.com/sparkle-project/Sparkle/blob/2.10.1/Autoupdate/SUDiskImageUnarchiver.m#L195-L278)
skips hidden top-level entries, aliases and symlinks but copies ordinary
directories recursively. The
[feed generator](https://github.com/sparkle-project/Sparkle/blob/2.10.1/generate_appcast/ArchiveItem.swift#L202-L221)
requires only one bundle at the archive root; the nested compatibility copy
therefore avoids an ambiguous feed. Both staged copies undergo strict nested
code-signature verification before the DMG is built and notarized.

Installed Voxelite.app (including a user-renamed file whose bundle name remains
Voxelite) finds the nested copy by filename. Quantum.app under the old ID finds
the root copy. Sparkle installs only the selected bundle **in place**; it does
not install a second app. After that first update, both payloads carry the new
ID and future updates can also match by identifier. The installed filename can
remain Voxelite.app; users may rename it to Quantum.app in Finder after quitting.
The updater does not move a running app or rewrite Launch Services registrations.
Keep the compatibility payload in future releases so users can skip versions.

`SUNormalizeInstalledApplicationName` is not a runtime plist option. The pinned
framework's normalization build setting is off; even when enabled it derives a
name from the installed host, not the incoming payload. This design needs no
bridge release, custom Sparkle framework, package installer or name normalization.

### Authentication and release qualification

[Sparkle's update validator](https://github.com/sparkle-project/Sparkle/blob/2.10.1/Sparkle/SUUpdateValidator.m)
requires an installable payload independently of authentication. For this path,
retain the committed EdDSA public key and sign the entire new DMG with its
existing private key. With `SUVerifyUpdateBeforeExtraction`, Sparkle validates
the archive against the installed key before extraction, then checks the new
bundle's code signature and prohibits removal of signing or public keys.
Without prevalidation, a valid old-key EdDSA signature permits an Apple signing
identity change, but a corrupted new code signature is still rejected. The
[code-signing verifier](https://github.com/sparkle-project/Sparkle/blob/2.10.1/Autoupdate/SUCodeSigningVerifier.m)
checks the old designated requirement on its matching-identity path; changing
bundle ID can fail that requirement. Do not rely on that fallback or rotate the
EdDSA key during this migration. Continue Developer ID signing, notarization,
stapling and signed-feed/notes verification exactly as before.

No release, tag, deployment or existing signed entry is changed by this work.
Before authorizing the first release, the owner must qualify a newly signed test
DMG/feed on macOS with the existing production keys, on disposable installations:

1. Start from the last released Voxelite.app under the old ID; repeat with a
   user-renamed copy and Quantum.app under the old ID. Seed updater preferences,
   including disabled automatic checks and a skipped version, and local state.
2. Serve the test signed feed through a controlled test endpoint. Trigger the
   installed updater, verify it installs exactly one bundle at the original
   path, relaunches with the new ID and retains preferences/state. Test a second
   update and both manual and automatic update checks.
3. Verify the two DMG copies have identical metadata/content and valid nested
   signatures, run Gatekeeper/stapler checks, and verify a tampered archive or
   bundle is rejected. Confirm manual installation uses only the root app.
4. Separately authorize the normal stable version/tag/release workflow. It
   generates and signs a new feed entry while preserving previous entries.
   Deploy those exact signed XML bytes on both hosts; verify HTTP 200 and byte
   equality at both `/appcast.xml` endpoints. Keep both DNS/Vercel bindings.

Linux tests cover packaging layout, metadata equality, discovery cases and
migration failure contracts; they do not prove AppKit, cfprefsd, notarization,
Gatekeeper or a real Sparkle installation. macOS CI compiles the native adapter.
Real signed update qualification remains the owner's release gate, not a claim
made from the Linux tests.

## Websites and immutable history

New bundles use `https://quantum.worldwidewebb.co/appcast.xml`. Existing defaults
can retain `https://voxelite.worldwidewebb.co/appcast.xml`; both hosts must serve
identical signed XML directly with HTTP 200. Other old-host pages redirect to
canonical paths with HTTP 308. `site/vercel.json` and its regression test retain
this exception. No separate legacy feed or release channel is needed.

New releases use the Quantum repository and Quantum.dmg / Quantum.html names.
The repository rename (#91) is complete: workflow guards and release-fetch
commands accept only `0x63616c/quantum`. GitHub redirects historical repository
URLs; no published release was changed.
Validation still accepts historical GitHub URLs and Voxelite.dmg / Voxelite.html
names without editing signed XML, release notes, tags or published assets.

## macOS local data

Startup first retains the Application Support migration introduced with the
product rename, then migrates bundle-keyed state before GPUI/Sparkle starts.
The old `co.worldwidewebb.voxelite` persistent defaults domain is read through
NSUserDefaults, merged into `co.worldwidewebb.quantum`, and synchronized before
recording completion. Every existing destination key wins, including false,
zero and empty values. All other keys retain native plist types, including
Sparkle's `SU*` preferences (checks, intervals, skipped versions, last-check dates
and any feed override); registered/global defaults are not copied. The original
domain remains intact. Copying the plist file behind cfprefsd is avoided.

Source inspection found these stores relevant to this unsandboxed app:

- GPUI/AppKit can persist defaults and `Library/Saved Application State/<id>.savedState`.
- Sparkle's `SPULocalCacheDirectory` uses `Library/Caches/<id>/org.sparkle-project.Sparkle`.
- Sparkle's downloader uses a default NSURLSession, and its release-notes UI uses
  WKWebView with the default website data store. Migrate existing
  `Library/HTTPStorages/<id>`, `Library/WebKit/<id>`, and bundle-keyed
  `Library/Application Support/<id>` directories, plus `<id>.binarycookies`
  files under `Library/HTTPStorages` and `Library/Cookies`.
- The app itself stores settings, palettes, recents, thumbnails and sockets under
  `Library/Application Support/quantum`, not the bundle ID. It has no sandbox,
  app-group, keychain access-group or host-specific preferences configuration to
  migrate. User projects and signed application contents are never moved.

The filesystem adapter migrates only existing paths; it does not assume every
OS version created every store. An existing destination, even empty, wins as a
whole. Directory moves use no-replace rename; cross-device moves stage a complete
copy and publish without replacement, preserving the source. Cookie files use
an atomic no-replace copy. Failures stop startup and leave completion unmarked;
retry preserves successful earlier work and current destination values. The
`.bundle-migration-v1` marker is written under the new Application Support data
directory only after all steps succeed. Subsequent launches do not reimport
intentionally removed preferences or caches. Filesystem and defaults interfaces
are exercised on Linux, including missing stores and injected failures.

The earlier `voxelite` → `quantum` data-directory migration still covers macOS
Application Support, older macOS XDG data, and other platforms' XDG/fallback data.
`QUANTUM_DATA_DIR` takes precedence over deprecated `VOXELITE_DATA_DIR`; explicit
overrides suppress both migrations. Unique session sockets avoid stale endpoints.

## Projects, MCP and plugins

The `.voxl` extension and v1 data model stay unchanged. Writers emit
`"format": "quantum"` and MIME `application/vnd.quantum.project+json`. Readers
accept `"voxelite"` as a legacy discriminator, with the same bounded validation.
One legacy fixture loads and re-saves byte-for-byte to the current canonical
fixture. Unknown supported metadata, IDs, content, comments and versions survive.
Old applications requiring the former discriminator cannot read newly saved
files; keep an original copy for such applications. The old media type
`application/vnd.voxelite.project+json` describes historical files only: no MIME
input is validated by the file reader. Examples and benchmark files use the new
discriminator. Renderer scene hashes cover decoded snapshots, not format labels;
unchanged scene hashes/goldens remain honest. Format semantic hashes include the
canonical discriminator, so the demo's format hash changes with the writer.

MCP emits `quantum.v1` throughout code, discovery schemas, snapshots, docs and
skills. `spec/mcp/v1/compatibility.json` lists `voxelite.v1` as the equivalent
legacy envelope alias for consumers of older servers. No request or initialize
parameter validates an API version; it is output-only. Clients with exact old
response validators must update before upgrading. Rediscover `quantum_` tools
through `tools/list` and configure the `quantum-mcp` executable.

Persisted plugin declarations and settings still accept `dev.voxelite.*` as
read-side aliases and store canonical `dev.quantum.*` IDs. Other namespaces and
opaque extension data are preserved; this does not introduce dynamic plugins
or a plugin enablement UI.
