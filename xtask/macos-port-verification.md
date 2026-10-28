# macOS xtask port verification

This records the PR 3 migration of packaging, feeds and release notes. The
production signing commands and asset names remain unchanged. Apple tools are
injected in tests; Linux results do not qualify real signatures, notarization,
Gatekeeper, Sparkle updates, AppKit or Metal.

## Check results

- `cargo fmt --all -- --check`: passed. The standalone Rust tool fixture also
  passes `rustfmt --edition 2024 --check`.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo test --workspace --locked`: passed, 651 tests and six existing ignored
  tests. After final xtask refinements, `cargo test -p xtask --locked` passed all
  81 tests, including real subprocess credential removal and both signals.
- `cargo deny check`: passed (advisories, licenses, bans and sources).
- `cargo machete`: passed, no unused dependencies.
- `cargo xtask check appcast` and `xtask version --check-tag v0.3.1`: passed.
- `cargo xtask tidy`: the ordinary unstaged index still names the deleted
  scripts, so that invocation fails on the first missing file. The same tidy
  implementation passes against a temporary `GIT_INDEX_FILE` containing the
  proposed file set, including all new files and deletions. The real index,
  Git configuration and commits were not changed.
- Deleted-script reference search and `git diff --check`: passed.

Signing/notarization/Sparkle/DMG behavior still requires the trusted macOS CI and
real Apple qualification described in the release guide. These tests validate
the command protocol, data validation, secret isolation and cleanup, without
asserting that Linux mocks prove Apple-service behavior. No assets were published.

## External command mapping

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

## Regression coverage mapping

All 37 original test methods are covered below. Rust names omit the Python
`test_` prefix. No test case was dropped. Fixtures use an injectable Rust runner,
real temporary Git repositories, and a small Rust executable compiled solely by
integration tests. The workflow tests execute the actual existing workflow run
blocks in Bash; they do not create Bash/Python/Node fixture scripts.

| Original suite and test | Rust destination |
| --- | --- |
| Appcast: empty_feed_is_explicit_and_has_no_installable_item | `macos::tests::appcast`, same name |
| Appcast: cli_rejects_wrong_local_dmg_length | `tests/macos_workflows.rs`, same name, real CLI |
| Appcast: offline_validation_matches_actual_files_and_history | `macos::tests::appcast`, same name |
| Appcast: url_signature_length_and_metadata_rejection | `macos::tests::appcast`, same name; every URL/signature/length/missing-node case |
| Appcast: malformed_xml_and_unstable_or_unordered_versions | `macos::tests::appcast`, same name |
| Appcast: asset_metadata_must_cover_every_asset | `macos::tests::appcast`, same name |
| Fetch: paginated_releases_select_numeric_highest_feed_and_copy_verbatim | `macos::tests::appcast`, same name |
| Fetch: no_releases_or_no_feed_uses_empty_feed | `macos::tests::appcast`, same name |
| Fetch: api_download_and_invalid_asset_errors_preserve_destination | `macos::tests::appcast`, same name |
| Packaging: release_notes_match_packaging_notarization | `macos::tests::release_notes`, same name, all four credential/marker transitions |
| Packaging: no_secrets_reach_build_or_tools | `macos::tests::packaging`, same name; environment removals, redacted display and independent shasum verification |
| Packaging: default_public_key_and_matching_override | `macos::tests::packaging`, same name |
| Packaging: mismatched_public_override_and_missing_default_fail | `macos::tests::packaging`, same name |
| Packaging: appcast_requires_credentials_and_notarization | `macos::tests::appcast`, same name |
| Packaging: skip_build | merged into `packaging::skip_build_and_missing_mcp_executable` |
| Packaging: bundled_executables_signed_before_bundle | `packaging::bundled_executables_and_sparkle_signed_inside_out` |
| Packaging: skip_build_requires_mcp_executable | merged into `packaging::skip_build_and_missing_mcp_executable` |
| Packaging: adhoc_and_partial_credentials | `macos::tests::packaging`, same name |
| Packaging: empty_keychain_list_and_lowercase_fingerprint | `macos::tests::packaging`, same name |
| Packaging: malformed_keychain_list_stops_before_mutation | `macos::tests::packaging`, same name |
| Packaging: private_failures_clean_keychain_and_hide_diagnostics | `macos::tests::packaging`, same name, expanded to additional security operations |
| Packaging: term_cleans_keychain | `packaging::int_and_term_clean_keychain`, isolated real SIGINT and SIGTERM |
| Packaging: failure_cannot_reuse_notarized_marker | `macos::tests::packaging`, same name |
| Packaging: invalid_inputs_and_notarization_rejection | `macos::tests::packaging`, same name |
| Packaging: sparkle_signing_and_feed_validation | `appcast::sparkle_signing_and_feed_validation` plus `packaging::bundled_executables_and_sparkle_signed_inside_out`; all eight generation outcomes, HTML escaping, preserved history, missing notes, downgrade and checksum corruption |
| Release: release_tags | `tests/macos_workflows.rs`, same name |
| Release: self_hosted_guards_and_pins | `tests/macos_workflows.rs`, same name |
| Release: release_build_precedes_credentials | `tests/macos_workflows.rs`, same name; also requires prebuilt xtask in secret-bearing steps |
| Release: tag_must_match_cargo_before_building | `tests/macos_workflows.rs`, same name; real xtask with injected Cargo metadata |
| Release: site_release_trigger_and_shared_concurrency | `tests/macos_workflows.rs`, same name |
| Release: release_requires_all_apple_credentials_before_packaging | `tests/macos_workflows.rs`, same name |
| Release: publication_is_staged_and_immutable | `tests/macos_workflows.rs`, same name |
| Release notes: installation_notes_follow_current_notarization_marker | `macos::tests::release_notes`, same name |
| Release notes: previous_stable_tag_and_release_boundary | `macos::tests::release_notes`, same name |
| Release notes: first_release_on_root_commit | `macos::tests::release_notes`, same name |
| Release notes: no_previous_stable_tag_includes_all_commits | `macos::tests::release_notes`, same name |
| Release notes: merged_history_and_annotated_previous_tag | `macos::tests::release_notes`, same name |

Additional coverage checks panic cleanup, cleanup failures, directory/file modes,
identity selection, quoted keychain paths, dry-run immutability, malformed feed
metadata, missing history, nonincreasing updates, actual secret stdin and
interrupting/reaping an active child while allowing restoration commands.

## Linux dry-run transcripts

The following are complete stdout/stderr transcripts from the prebuilt xtask
binary, with `VERSION=1.2.3`. All credentials are fake fixture strings. `TMPDIR`
is `/tmp`; deterministic dry-run directory names never exist on disk. Every
command explicitly shows the seven credential environment removals. The
certificate identity, generated password, certificate password, Apple IDs and
Sparkle stdin are redacted.

### Ad-hoc mode

```text
[dry-run] create private temporary directory /tmp/voxelite-package.DRY-RUN (0700)
[dry-run] remove dist/ and its old notarization marker
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) cargo build --release --locked -p voxelite-app --bin voxelite
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) cargo build --release --locked -p voxelite-mcp
[dry-run] assemble dist/Voxelite.app from target/release/voxelite and voxelite-mcp; write Info.plist version 1.2.3 with the committed Sparkle public key
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 16 16 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_16x16.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 32 32 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_16x16@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 32 32 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_32x32.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 64 64 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_32x32@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 128 128 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_128x128.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 256 256 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_128x128@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 256 256 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_256x256.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 512 512 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_256x256@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 512 512 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_512x512.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 1024 1024 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_512x512@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) iconutil -c icns /tmp/voxelite-package.DRY-RUN/Voxelite.iconset -o /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Resources/Voxelite.icns
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) curl --fail --location --retry 3 --proto =https --tlsv1.2 https://github.com/sparkle-project/Sparkle/releases/download/2.10.1/Sparkle-2.10.1.tar.xz --output /tmp/voxelite-package.DRY-RUN/Sparkle.tar.xz
[dry-run] verify SHA-256 /tmp/voxelite-package.DRY-RUN/Sparkle.tar.xz = 1a856d2a9edb8e628de88121d7be04cf0e2167f8641650b78fca00b14244cf27 before extracting
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) tar -xf /tmp/voxelite-package.DRY-RUN/Sparkle.tar.xz -C dist/sparkle ./Sparkle.framework ./bin ./LICENSE
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) ditto dist/sparkle/Sparkle.framework /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework
[dry-run] copy dist/sparkle/LICENSE to dist/Voxelite.app/Contents/Resources/Sparkle-LICENSE.txt
NOTICE: Developer ID credentials unavailable or ad-hoc mode requested; using ad-hoc app signing and an unsigned DMG. Notarization is skipped.
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/XPCServices/Installer.xpc
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none --preserve-metadata=entitlements /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/XPCServices/Downloader.xpc
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/Autoupdate
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/Updater.app
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/MacOS/voxelite-mcp
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none --entitlements packaging/macos/Voxelite.entitlements /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/MacOS/Voxelite
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none --entitlements packaging/macos/Voxelite.entitlements /workspace/vx-xtask-macos/dist/Voxelite.app
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --verify --deep --strict /workspace/vx-xtask-macos/dist/Voxelite.app
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) ditto /workspace/vx-xtask-macos/dist/Voxelite.app /tmp/voxelite-package.DRY-RUN/stage/Voxelite.app
[dry-run] symlink /tmp/voxelite-package.DRY-RUN/stage/Applications -> /Applications
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) hdiutil create -volname Voxelite -srcfolder /tmp/voxelite-package.DRY-RUN/stage -fs "HFS+" -format UDZO -ov /workspace/vx-xtask-macos/dist/Voxelite.dmg
[dry-run] write dist/Voxelite.dmg.sha256 using SHA-256 and the relative filename Voxelite.dmg
[dry-run] remove temporary directory /tmp/voxelite-package.DRY-RUN
```

### Automatic mode without credentials

```text
[dry-run] create private temporary directory /tmp/voxelite-package.DRY-RUN (0700)
[dry-run] remove dist/ and its old notarization marker
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) cargo build --release --locked -p voxelite-app --bin voxelite
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) cargo build --release --locked -p voxelite-mcp
[dry-run] assemble dist/Voxelite.app from target/release/voxelite and voxelite-mcp; write Info.plist version 1.2.3 with the committed Sparkle public key
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 16 16 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_16x16.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 32 32 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_16x16@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 32 32 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_32x32.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 64 64 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_32x32@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 128 128 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_128x128.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 256 256 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_128x128@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 256 256 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_256x256.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 512 512 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_256x256@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 512 512 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_512x512.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 1024 1024 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_512x512@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) iconutil -c icns /tmp/voxelite-package.DRY-RUN/Voxelite.iconset -o /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Resources/Voxelite.icns
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) curl --fail --location --retry 3 --proto =https --tlsv1.2 https://github.com/sparkle-project/Sparkle/releases/download/2.10.1/Sparkle-2.10.1.tar.xz --output /tmp/voxelite-package.DRY-RUN/Sparkle.tar.xz
[dry-run] verify SHA-256 /tmp/voxelite-package.DRY-RUN/Sparkle.tar.xz = 1a856d2a9edb8e628de88121d7be04cf0e2167f8641650b78fca00b14244cf27 before extracting
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) tar -xf /tmp/voxelite-package.DRY-RUN/Sparkle.tar.xz -C dist/sparkle ./Sparkle.framework ./bin ./LICENSE
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) ditto dist/sparkle/Sparkle.framework /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework
[dry-run] copy dist/sparkle/LICENSE to dist/Voxelite.app/Contents/Resources/Sparkle-LICENSE.txt
NOTICE: Developer ID credentials unavailable or ad-hoc mode requested; using ad-hoc app signing and an unsigned DMG. Notarization is skipped.
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/XPCServices/Installer.xpc
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none --preserve-metadata=entitlements /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/XPCServices/Downloader.xpc
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/Autoupdate
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/Updater.app
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/MacOS/voxelite-mcp
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none --entitlements packaging/macos/Voxelite.entitlements /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/MacOS/Voxelite
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign - --timestamp=none --entitlements packaging/macos/Voxelite.entitlements /workspace/vx-xtask-macos/dist/Voxelite.app
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --verify --deep --strict /workspace/vx-xtask-macos/dist/Voxelite.app
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) ditto /workspace/vx-xtask-macos/dist/Voxelite.app /tmp/voxelite-package.DRY-RUN/stage/Voxelite.app
[dry-run] symlink /tmp/voxelite-package.DRY-RUN/stage/Applications -> /Applications
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) hdiutil create -volname Voxelite -srcfolder /tmp/voxelite-package.DRY-RUN/stage -fs "HFS+" -format UDZO -ov /workspace/vx-xtask-macos/dist/Voxelite.dmg
[dry-run] write dist/Voxelite.dmg.sha256 using SHA-256 and the relative filename Voxelite.dmg
[dry-run] remove temporary directory /tmp/voxelite-package.DRY-RUN
```

### Developer ID without notarization

```text
[dry-run] create private temporary directory /tmp/voxelite-package.DRY-RUN (0700)
[dry-run] remove dist/ and its old notarization marker
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) cargo build --release --locked -p voxelite-app --bin voxelite
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) cargo build --release --locked -p voxelite-mcp
[dry-run] assemble dist/Voxelite.app from target/release/voxelite and voxelite-mcp; write Info.plist version 1.2.3 with the committed Sparkle public key
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 16 16 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_16x16.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 32 32 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_16x16@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 32 32 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_32x32.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 64 64 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_32x32@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 128 128 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_128x128.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 256 256 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_128x128@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 256 256 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_256x256.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 512 512 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_256x256@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 512 512 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_512x512.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 1024 1024 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_512x512@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) iconutil -c icns /tmp/voxelite-package.DRY-RUN/Voxelite.iconset -o /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Resources/Voxelite.icns
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) curl --fail --location --retry 3 --proto =https --tlsv1.2 https://github.com/sparkle-project/Sparkle/releases/download/2.10.1/Sparkle-2.10.1.tar.xz --output /tmp/voxelite-package.DRY-RUN/Sparkle.tar.xz
[dry-run] verify SHA-256 /tmp/voxelite-package.DRY-RUN/Sparkle.tar.xz = 1a856d2a9edb8e628de88121d7be04cf0e2167f8641650b78fca00b14244cf27 before extracting
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) tar -xf /tmp/voxelite-package.DRY-RUN/Sparkle.tar.xz -C dist/sparkle ./Sparkle.framework ./bin ./LICENSE
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) ditto dist/sparkle/Sparkle.framework /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework
[dry-run] copy dist/sparkle/LICENSE to dist/Voxelite.app/Contents/Resources/Sparkle-LICENSE.txt
[dry-run] decode MACOS_CERTIFICATE_P12_BASE64 into /tmp/voxelite-package.DRY-RUN/certificate.p12 (0600; value redacted)
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security list-keychains -d user
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) openssl rand -hex 32
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security create-keychain -p [REDACTED] /tmp/voxelite-package.DRY-RUN/signing.keychain-db
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security set-keychain-settings -lut 21600 /tmp/voxelite-package.DRY-RUN/signing.keychain-db
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security unlock-keychain -p [REDACTED] /tmp/voxelite-package.DRY-RUN/signing.keychain-db
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security import /tmp/voxelite-package.DRY-RUN/certificate.p12 -k /tmp/voxelite-package.DRY-RUN/signing.keychain-db -P [REDACTED] -T /usr/bin/codesign -T /usr/bin/security
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security set-key-partition-list -S "apple-tool:,apple:,codesign:" -s -k [REDACTED] /tmp/voxelite-package.DRY-RUN/signing.keychain-db
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security list-keychains -d user -s /tmp/voxelite-package.DRY-RUN/signing.keychain-db
[dry-run] remove /tmp/voxelite-package.DRY-RUN/certificate.p12
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security find-identity -v -p codesigning /tmp/voxelite-package.DRY-RUN/signing.keychain-db
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/XPCServices/Installer.xpc
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp --preserve-metadata=entitlements /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/XPCServices/Downloader.xpc
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/Autoupdate
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/Updater.app
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/MacOS/voxelite-mcp
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp --entitlements packaging/macos/Voxelite.entitlements /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/MacOS/Voxelite
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp --entitlements packaging/macos/Voxelite.entitlements /workspace/vx-xtask-macos/dist/Voxelite.app
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --verify --deep --strict /workspace/vx-xtask-macos/dist/Voxelite.app
NOTICE: App Store Connect API credentials incomplete; Developer ID signing succeeded, but notarization and Gatekeeper assessment are skipped.
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) ditto /workspace/vx-xtask-macos/dist/Voxelite.app /tmp/voxelite-package.DRY-RUN/stage/Voxelite.app
[dry-run] symlink /tmp/voxelite-package.DRY-RUN/stage/Applications -> /Applications
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) hdiutil create -volname Voxelite -srcfolder /tmp/voxelite-package.DRY-RUN/stage -fs "HFS+" -format UDZO -ov /workspace/vx-xtask-macos/dist/Voxelite.dmg
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --sign [REDACTED] --timestamp /workspace/vx-xtask-macos/dist/Voxelite.dmg
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --verify --strict /workspace/vx-xtask-macos/dist/Voxelite.dmg
[dry-run] write dist/Voxelite.dmg.sha256 using SHA-256 and the relative filename Voxelite.dmg
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security delete-keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security list-keychains -d user -s
[dry-run] remove temporary directory /tmp/voxelite-package.DRY-RUN
```

### Developer ID with notarization

```text
[dry-run] create private temporary directory /tmp/voxelite-package.DRY-RUN (0700)
[dry-run] remove dist/ and its old notarization marker
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) cargo build --release --locked -p voxelite-app --bin voxelite
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) cargo build --release --locked -p voxelite-mcp
[dry-run] assemble dist/Voxelite.app from target/release/voxelite and voxelite-mcp; write Info.plist version 1.2.3 with the committed Sparkle public key
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 16 16 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_16x16.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 32 32 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_16x16@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 32 32 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_32x32.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 64 64 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_32x32@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 128 128 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_128x128.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 256 256 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_128x128@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 256 256 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_256x256.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 512 512 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_256x256@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 512 512 packaging/macos/icon-1024.png --out /tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_512x512.png
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) sips -z 1024 1024 packaging/macos/icon-1024.png --out "/tmp/voxelite-package.DRY-RUN/Voxelite.iconset/icon_512x512@2x.png"
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) iconutil -c icns /tmp/voxelite-package.DRY-RUN/Voxelite.iconset -o /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Resources/Voxelite.icns
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) curl --fail --location --retry 3 --proto =https --tlsv1.2 https://github.com/sparkle-project/Sparkle/releases/download/2.10.1/Sparkle-2.10.1.tar.xz --output /tmp/voxelite-package.DRY-RUN/Sparkle.tar.xz
[dry-run] verify SHA-256 /tmp/voxelite-package.DRY-RUN/Sparkle.tar.xz = 1a856d2a9edb8e628de88121d7be04cf0e2167f8641650b78fca00b14244cf27 before extracting
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) tar -xf /tmp/voxelite-package.DRY-RUN/Sparkle.tar.xz -C dist/sparkle ./Sparkle.framework ./bin ./LICENSE
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) ditto dist/sparkle/Sparkle.framework /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework
[dry-run] copy dist/sparkle/LICENSE to dist/Voxelite.app/Contents/Resources/Sparkle-LICENSE.txt
[dry-run] decode MACOS_CERTIFICATE_P12_BASE64 into /tmp/voxelite-package.DRY-RUN/certificate.p12 (0600; value redacted)
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security list-keychains -d user
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) openssl rand -hex 32
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security create-keychain -p [REDACTED] /tmp/voxelite-package.DRY-RUN/signing.keychain-db
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security set-keychain-settings -lut 21600 /tmp/voxelite-package.DRY-RUN/signing.keychain-db
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security unlock-keychain -p [REDACTED] /tmp/voxelite-package.DRY-RUN/signing.keychain-db
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security import /tmp/voxelite-package.DRY-RUN/certificate.p12 -k /tmp/voxelite-package.DRY-RUN/signing.keychain-db -P [REDACTED] -T /usr/bin/codesign -T /usr/bin/security
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security set-key-partition-list -S "apple-tool:,apple:,codesign:" -s -k [REDACTED] /tmp/voxelite-package.DRY-RUN/signing.keychain-db
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security list-keychains -d user -s /tmp/voxelite-package.DRY-RUN/signing.keychain-db
[dry-run] remove /tmp/voxelite-package.DRY-RUN/certificate.p12
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security find-identity -v -p codesigning /tmp/voxelite-package.DRY-RUN/signing.keychain-db
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/XPCServices/Installer.xpc
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp --preserve-metadata=entitlements /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/XPCServices/Downloader.xpc
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/Autoupdate
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework/Versions/B/Updater.app
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/Frameworks/Sparkle.framework
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/MacOS/voxelite-mcp
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp --entitlements packaging/macos/Voxelite.entitlements /workspace/vx-xtask-macos/dist/Voxelite.app/Contents/MacOS/Voxelite
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --sign [REDACTED] --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --options runtime --timestamp --entitlements packaging/macos/Voxelite.entitlements /workspace/vx-xtask-macos/dist/Voxelite.app
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --verify --deep --strict /workspace/vx-xtask-macos/dist/Voxelite.app
[dry-run] decode APPLE_API_KEY_P8_BASE64 into /tmp/voxelite-package.DRY-RUN/AuthKey.p8 (0600; value redacted)
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) ditto -c -k --keepParent /workspace/vx-xtask-macos/dist/Voxelite.app /tmp/voxelite-package.DRY-RUN/Voxelite.zip
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) xcrun notarytool submit /tmp/voxelite-package.DRY-RUN/Voxelite.zip --key /tmp/voxelite-package.DRY-RUN/AuthKey.p8 --key-id [REDACTED] --issuer [REDACTED] --wait --timeout 30m --output-format json
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) xcrun stapler staple /workspace/vx-xtask-macos/dist/Voxelite.app
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) xcrun stapler validate /workspace/vx-xtask-macos/dist/Voxelite.app
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) spctl --assess --type execute /workspace/vx-xtask-macos/dist/Voxelite.app
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) ditto /workspace/vx-xtask-macos/dist/Voxelite.app /tmp/voxelite-package.DRY-RUN/stage/Voxelite.app
[dry-run] symlink /tmp/voxelite-package.DRY-RUN/stage/Applications -> /Applications
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) hdiutil create -volname Voxelite -srcfolder /tmp/voxelite-package.DRY-RUN/stage -fs "HFS+" -format UDZO -ov /workspace/vx-xtask-macos/dist/Voxelite.dmg
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --force --keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db --sign [REDACTED] --timestamp /workspace/vx-xtask-macos/dist/Voxelite.dmg
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) codesign --verify --strict /workspace/vx-xtask-macos/dist/Voxelite.dmg
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) xcrun notarytool submit /workspace/vx-xtask-macos/dist/Voxelite.dmg --key /tmp/voxelite-package.DRY-RUN/AuthKey.p8 --key-id [REDACTED] --issuer [REDACTED] --wait --timeout 30m --output-format json
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) xcrun stapler staple /workspace/vx-xtask-macos/dist/Voxelite.dmg
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) xcrun stapler validate /workspace/vx-xtask-macos/dist/Voxelite.dmg
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) spctl --assess --type open --context context:primary-signature /workspace/vx-xtask-macos/dist/Voxelite.dmg
[dry-run] write dist/Voxelite.dmg.sha256 using SHA-256 and the relative filename Voxelite.dmg
[dry-run] write dist/.notarized only after all validation succeeds
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security delete-keychain /tmp/voxelite-package.DRY-RUN/signing.keychain-db
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) security list-keychains -d user -s
[dry-run] remove temporary directory /tmp/voxelite-package.DRY-RUN
```

### Appcast signing

```text
[dry-run] create private temporary directory /tmp/voxelite-appcast.DRY-RUN (0700)
[dry-run] require dist/.notarized, embedded Sparkle, matching bundle version, and a valid older dist/previous-appcast.xml
[dry-run] verify dist/Voxelite.dmg against dist/Voxelite.dmg.sha256
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) xcrun stapler validate dist/Voxelite.dmg
[dry-run] copy final DMG and previous feed into private workspace; HTML-escape dist/RELEASE_NOTES.md to archives/Voxelite.html
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) dist/sparkle/bin/generate_appcast --ed-key-file - --maximum-deltas 0 --maximum-versions 0 --download-url-prefix https://github.com/0x63616c/voxelite/releases/download/v1.2.3/ --release-notes-url-prefix https://github.com/0x63616c/voxelite/releases/download/v1.2.3/ --link https://voxelite.worldwidewebb.co -o /tmp/voxelite-appcast.DRY-RUN/appcast.xml /tmp/voxelite-appcast.DRY-RUN/archives < [REDACTED stdin]
[dry-run] (in /workspace/vx-xtask-macos) (unset APPLE_API_ISSUER_ID) (unset APPLE_API_KEY_ID) (unset APPLE_API_KEY_P8_BASE64) (unset MACOS_CERTIFICATE_P12_BASE64) (unset MACOS_CERTIFICATE_PASSWORD) (unset MACOS_SIGNING_IDENTITY) (unset SPARKLE_ED_PRIVATE_KEY) dist/sparkle/bin/sign_update --ed-key-file - --verify /tmp/voxelite-appcast.DRY-RUN/appcast.xml < [REDACTED stdin]
[dry-run] copy signed Voxelite.html to dist; validate every feed item, local asset length, bundle metadata and preserved history
[dry-run] copy signed XML bytes unchanged to site/appcast.xml; chmod XML/HTML 0644; append generated=true to GITHUB_OUTPUT when set
[dry-run] remove temporary directory /tmp/voxelite-appcast.DRY-RUN
Would generate site/appcast.xml for upload as a release asset. The site deploy must copy this file unchanged to /appcast.xml.
```
