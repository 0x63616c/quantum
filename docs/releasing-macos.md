# macOS CI, signing, and updates

Calum configures the credentials below in this repository's **Settings → Secrets
and variables → Actions → Secrets**. No credential values belong in source,
release notes, issue comments, or workflow YAML. The workflows use GitHub's
provided token; a personal access token is not needed.

## Secrets to create

| Secret | Value and source |
| --- | --- |
| `MACOS_CERTIFICATE_P12_BASE64` | Base64 of a password-protected Developer ID **Application** `.p12` export, including its private key. |
| `MACOS_CERTIFICATE_PASSWORD` | The nonempty password chosen when exporting that `.p12`. |
| `MACOS_SIGNING_IDENTITY` | Optional: the certificate's SHA-1 fingerprint. Omit when the export contains exactly one valid Developer ID Application identity; the script auto-detects it. |
| `APPLE_API_KEY_P8_BASE64` | Base64 of the downloaded App Store Connect **team** API `.p8` key. Assign the key the **Developer** role. |
| `APPLE_API_KEY_ID` | The API key's Key ID from App Store Connect. |
| `APPLE_API_ISSUER_ID` | The team API Issuer ID from App Store Connect. |
| `SPARKLE_ED_PRIVATE_KEY` | The **contents** of the private key file exported by Sparkle `generate_keys -x`. It is already base64; do not base64-encode it again. |

The public Sparkle key belongs in
`packaging/macos/sparkle_public_ed_key.txt`, as one base64 line. It is deliberately
absent until Calum generates a key pair. Alternatively, set the Actions **variable**
`SPARKLE_PUBLIC_ED_KEY`; it overrides the file and is not a secret. Local packaging
accepts that same environment variable. No placeholder key is shipped.

### Developer ID certificate

1. In the Apple Developer account, create a **Developer ID Application**
   certificate using a certificate signing request from Keychain Access, or use
   an existing valid certificate. Install it on the Mac holding its private key.
   Developer ID Installer and Apple Distribution certificates are not suitable
   for this app. See [Apple's Developer ID certificate guide](https://developer.apple.com/help/account/certificates/create-developer-id-certificates/).
2. In **Keychain Access → My Certificates**, expand the certificate and confirm
   the private key is present. Select the certificate/private-key pair, choose
   **Export**, and save as **Personal Information Exchange (.p12)** with a strong,
   nonempty export password. Keep the export outside the checkout.
3. Set `MACOS_CERTIFICATE_PASSWORD` using the GitHub secrets UI (or `gh secret set`
   interactively). On macOS, encode and upload the export without printing it:

   ```sh
   base64 -i /secure/path/developer-id.p12 | tr -d '\n' |
     gh secret set MACOS_CERTIFICATE_P12_BASE64 --repo 0x63616c/voxelite
   ```

4. Normally omit `MACOS_SIGNING_IDENTITY`. If the export contains multiple
   Developer ID identities, inspect `security find-identity -v -p codesigning`
   locally and save the selected fingerprint in that secret. Do not copy the
   identity listing into the repository or CI logs.

### App Store Connect notarization key

In **App Store Connect → Users and Access → Integrations → App Store Connect API
→ Team Keys**, generate a team key with **Developer** access. An Account Holder or
Admin creates the team key; API access may first need approval. Download its `.p8`
file when offered, and save the displayed Key ID and Issuer ID in the corresponding
secrets. This workflow supplies an issuer and requires a team key. See Apple's
[API key instructions](https://developer.apple.com/documentation/AppStoreConnectAPI/creating-api-keys-for-app-store-connect-api)
and [notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow).

```sh
base64 -i /secure/path/AuthKey.p8 | tr -d '\n' |
  gh secret set APPLE_API_KEY_P8_BASE64 --repo 0x63616c/voxelite
```

Keep the downloaded key and certificate export in secure storage outside the
checkout. The packaging script decodes them into a private temporary directory,
imports the certificate into a temporary keychain, grants noninteractive
`codesign` access, and restores the runner's keychain search list on exit. Cleanup
also runs on failures, `INT`, and `TERM`. Do not run packaging with shell tracing.

### Sparkle update keys

Use the same pinned Sparkle distribution as `scripts/package-macos.sh`:
**2.10.1**, SHA-256
`1a856d2a9edb8e628de88121d7be04cf0e2167f8641650b78fca00b14244cf27`.
On a Mac, from the repository root:

```sh
sparkle_setup=$(mktemp -d)
curl --fail --location --proto '=https' --tlsv1.2 \
  https://github.com/sparkle-project/Sparkle/releases/download/2.10.1/Sparkle-2.10.1.tar.xz \
  --output "$sparkle_setup/Sparkle.tar.xz"
printf '%s  %s\n' \
  1a856d2a9edb8e628de88121d7be04cf0e2167f8641650b78fca00b14244cf27 \
  "$sparkle_setup/Sparkle.tar.xz" | shasum -a 256 --check
# Continue only after the digest check succeeds.
tar -xf "$sparkle_setup/Sparkle.tar.xz" -C "$sparkle_setup"
"$sparkle_setup/bin/generate_keys"
"$sparkle_setup/bin/generate_keys" -p > packaging/macos/sparkle_public_ed_key.txt
(umask 077; "$sparkle_setup/bin/generate_keys" -x "$sparkle_setup/private-key.txt")
gh secret set SPARKLE_ED_PRIVATE_KEY --repo 0x63616c/voxelite \
  < "$sparkle_setup/private-key.txt"
rm -rf "$sparkle_setup"
```

Only the public key file is committed. `generate_keys` also retains the private
key in the Mac's login keychain; maintain a secure backup. Use the same key for
subsequent releases. See [Sparkle's setup and key guidance](https://sparkle-project.org/documentation/).

## CI behavior

`.github/workflows/ci-macos.yml` runs on pushes to `main` and same-repository pull
requests. Its job-level guard is:

```yaml
if: github.event_name != 'pull_request' || github.event.pull_request.head.repo.full_name == github.repository
```

Fork pull requests never schedule this self-hosted job. Do not change the event to
`pull_request_target` or remove the guard. The release job has the same guard.

Both workflows use `[self-hosted, macOS, ARM64, xcode]`, targeting the
`homelab-voxelite` runner, with
`DEVELOPER_DIR=/Applications/Xcode-27.0.app/Contents/Developer`. There are no
GitHub-hosted macOS jobs. The runner needs that Xcode installation and its Metal
toolchain, the Rust toolchain from `rust-toolchain.toml` including rustfmt/Clippy,
Python 3, curl, and GitHub CLI for releases. The runner must support
the v4 checkout and artifact actions, pinned to reviewed commit SHAs in the
workflows.

CI checks formatting, runs `cargo clippy --workspace --locked -- -D warnings`,
runs `cargo test --workspace --locked`, builds the release app, and packages a
DMG. Strict Clippy passed on Linux when this workflow was added; the first macOS
run must confirm the platform-specific code too. `MACOS_SIGNING_MODE=adhoc`
forces an ad-hoc app and unsigned DMG even if the runner has signing credentials.
No signing/notarization secrets are passed to CI. Its DMG and checksum artifacts
expire after **3 days**. New runs cancel older CI runs for the same ref.

## Release behavior

The release workflow is triggered by `v*` tags or manual dispatch with an
existing tag. It accepts stable `vMAJOR.MINOR.PATCH` tags only and checks out the
fully qualified `refs/tags/` ref, so a same-named branch cannot supply the build.
The tag without `v` supplies both bundle version fields. Use an
increasing version for each public update and keep published release tags/assets
immutable; the update feed depends on their exact bytes. Releases serialize and
are not cancelled in progress. Publication creates a draft, uploads the DMG,
checksum, and optional feed together, then publishes only after every upload
succeeds. An existing release (including a draft) makes publication fail; assets
are never overwritten. A failed upload leaves a draft for inspection. Remove
that unpublished draft manually before retrying, or use a new version tag.

The packager:

1. Builds `voxelite-app` with the locked dependencies and assembles the app/icon.
2. If a public Sparkle key exists, downloads the pinned archive, verifies SHA-256,
   preserves framework symlinks, and embeds Sparkle plus its license. It sets
   `SUFeedURL=https://voxelite.worldwidewebb.co/appcast.xml`, `SUPublicEDKey`, and
   `SUEnableAutomaticChecks=true` in the built Info.plist. Without the public key,
   it omits those keys and the framework and prints a notice.
3. With a certificate and password, signs the Sparkle XPC services, helper tool,
   updater app, framework, and outer app in that order, using the same Developer
   ID, hardened runtime, and secure timestamps. The app uses
   `packaging/macos/Voxelite.entitlements`; the empty entitlement dictionary keeps
   library validation enabled. Downloader entitlements are preserved. See
   [Sparkle's manual signing guidance](https://sparkle-project.org/documentation/sandboxing/#code-signing).
4. With all three API credentials, notarizes a ZIP of the app, requires an
   `Accepted` result, staples and validates the app, and checks it with `spctl`.
5. Builds the DMG containing that stapled app and the Applications shortcut, signs
   and notarizes the DMG, staples/validates it, and runs the DMG Gatekeeper
   assessment. It computes SHA-256 **after** the final staple.
6. Deletes its temporary keychain and working files on exit.

Without the certificate/password pair, the build succeeds with ad-hoc app signing
and an unsigned DMG. With Developer ID but incomplete API credentials, it signs
without notarizing. Both cases print clear notices and skip the public appcast.
Invalid credentials, rejected notarization, bad checksums, or failed signature
verification fail the release; they do not silently fall back. Ad-hoc builds omit
hardened runtime so they can load the ad-hoc Sparkle framework during development.

The runtime updater loads only `Contents/Frameworks/Sparkle.framework/Sparkle`
from the app bundle, through `dlopen`. It retains the Objective-C controller and
framework for the process lifetime. Non-bundle `cargo run` and bundles without
Sparkle work normally. The macOS menu integration should invoke
`crate::updater::check_for_updates()` on GPUI's main thread for **Check for
Updates…**; this change exposes that API but leaves menu wiring to the menu job.
Only this macOS module permits the unsafe Objective-C calls; the app's other code
denies unsafe code, and other crates retain the workspace forbid lint.

## Appcast and site deployment

After a notarized build, `scripts/generate-macos-appcast.sh` passes
`SPARKLE_ED_PRIVATE_KEY` to Sparkle's `generate_appcast --ed-key-file -` through
stdin. It creates a single-version `site/appcast.xml`, with no delta updates,
pointing at:

```text
https://github.com/0x63616c/voxelite/releases/download/<tag>/Voxelite.dmg
```

The script rejects an unsigned enclosure, wrong URL, incorrect size, or stale
bundle version. Sparkle compares the signing key with the bundled public key;
a mismatch cannot be published as an unsigned update. The feed itself is also
signed by the pinned Sparkle tool. Missing private key, missing framework, or
non-notarized output skips generation with a notice.

The release workflow uploads `Voxelite.dmg`, `Voxelite.dmg.sha256`, and, when
generated, **`appcast.xml` as GitHub release assets**. It never commits the feed,
pushes a branch, or deploys the site. No PR-less push is used.

**The site deployment must copy the release's appcast asset to `site/appcast.xml`
after building the site and before publishing it.** The repository's site build
script does not fetch this asset automatically. For an explicitly selected
release, the deployment preparation step is:

```sh
# Set RELEASE_TAG to the release selected for this deployment.
gh release download "$RELEASE_TAG" --repo 0x63616c/voxelite \
  --pattern appcast.xml --dir site --clobber
```

Serve those exact bytes at `https://voxelite.worldwidewebb.co/appcast.xml`; do not
rewrite/minify the signed XML. A release without an appcast must leave the last
published feed intact. Confirm its enclosure URL serves the released DMG before
publishing the feed. Site publication is a separate, explicit operation.

## Verification and remaining Mac checks

Local static checks:

```sh
bash -n scripts/package-macos.sh scripts/generate-macos-appcast.sh
shellcheck scripts/package-macos.sh scripts/generate-macos-appcast.sh
actionlint
cargo fmt --all -- --check
cargo clippy --workspace --locked -- -D warnings
cargo check --workspace --locked
cargo test --workspace --locked
```

The implementation was checked on Linux, including a separate Apple ARM64
cross-target check of the updater module and packaging control-flow tests with
mock macOS tools. These do not validate Apple services or actual signatures.
Before a public signed release, verify on the Mac: the Xcode/Metal setup, full
workspace build/tests/Clippy, nested `codesign --verify --deep --strict`, both
notarization submissions and stapled tickets, `spctl` assessments, and an app
launch from `/Applications`. With the menu integration and hosted feed in place,
test a real update from an older version, including relaunch and an offline
launch of the stapled app. No macOS execution, release, or deployment was done
while implementing this change.
