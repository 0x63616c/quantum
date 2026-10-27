#!/usr/bin/env bash
# Build Voxelite.app (arm64) and dist/Voxelite.dmg on macOS with Xcode + Metal.
set -euo pipefail
# Never trace credentials, even if invoked with bash -x.
set +x
# Keep credentials out of Cargo build scripts and unrelated child processes.
# Release CI also builds in a separate step that receives no signing secrets.
export -n MACOS_CERTIFICATE_P12_BASE64 MACOS_CERTIFICATE_PASSWORD MACOS_SIGNING_IDENTITY \
  APPLE_API_KEY_P8_BASE64 APPLE_API_KEY_ID APPLE_API_ISSUER_ID SPARKLE_ED_PRIVATE_KEY
cd "$(dirname "$0")/.."

BUILD_APP=true
case "$#:${1:-}" in
  0:) ;;
  1:--skip-build) BUILD_APP=false ;;
  *) echo "Usage: scripts/package-macos.sh [--skip-build]" >&2; exit 1 ;;
esac
if [[ $(uname -s) != Darwin ]]; then
  echo "macOS packaging requires macOS and Xcode." >&2
  exit 1
fi
case "${MACOS_SIGNING_MODE:-auto}" in
  auto|adhoc) ;;
  *) echo "MACOS_SIGNING_MODE must be auto or adhoc." >&2; exit 1 ;;
esac

VERSION="${VERSION:-$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys;print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="voxelite-app"))')}"
if [[ ! "$VERSION" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
  echo "VERSION must be a stable version such as 0.1.0." >&2
  exit 1
fi
export VERSION
# A stale Actions variable must not silently change the key shipped to users.
export SPARKLE_PUBLIC_ED_KEY="${SPARKLE_PUBLIC_ED_KEY:-}"
SPARKLE_PUBLIC_ED_KEY=$(python3 - <<'PY'
import base64
import os
from pathlib import Path

committed = Path("packaging/macos/sparkle_public_ed_key.txt").read_text().strip()
override = os.environ["SPARKLE_PUBLIC_ED_KEY"].strip()
for key in (committed, override or committed):
    try:
        decoded = base64.b64decode(key, validate=True)
        if len(decoded) != 32 or base64.b64encode(decoded).decode() != key:
            raise ValueError()
    except ValueError:
        raise SystemExit("Sparkle public key must be canonical base64 of 32 bytes.") from None
if override and override != committed:
    raise SystemExit("SPARKLE_PUBLIC_ED_KEY differs from packaging/macos/sparkle_public_ed_key.txt; update the file and variable together.")
print(override or committed)
PY
)
APP=dist/Voxelite.app
DMG=dist/Voxelite.dmg
IDENTITY=-
KEYCHAIN=
RESTORE_KEYCHAINS=false
ORIGINAL_KEYCHAINS=()
WORK_DIR=$(mktemp -d "${TMPDIR:-/tmp}/voxelite-package.XXXXXX")
cleanup() {
  local status=$?
  trap - EXIT
  if [[ -n "$KEYCHAIN" ]]; then
    if ! security delete-keychain "$KEYCHAIN" >/dev/null 2>&1; then
      echo "Failed to delete the temporary signing keychain." >&2
      status=1
    fi
  fi
  if [[ "$RESTORE_KEYCHAINS" == true ]]; then
    # The conditional expansion supports empty arrays with macOS Bash 3.2 + -u.
    security list-keychains -d user -s ${ORIGINAL_KEYCHAINS[@]+"${ORIGINAL_KEYCHAINS[@]}"} || status=1
  fi
  if [[ "$status" != 0 ]]; then rm -f dist/.notarized; fi
  rm -rf "$WORK_DIR"
  exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# Decode secrets from the environment so they never appear in command arguments.
decode_secret() (
  # Only this decoder receives this one value in its environment.
  export "${1?}"
  python3 - "$1" "$2" <<'PY'
import base64
import os
import pathlib
import sys

try:
    value = base64.b64decode("".join(os.environ[sys.argv[1]].split()), validate=True)
except (ValueError, KeyError):
    raise SystemExit(f"Invalid base64 in {sys.argv[1]}") from None
pathlib.Path(sys.argv[2]).write_bytes(value)
PY
)

# Error output from Apple tools can contain certificate subjects and identities.
# Keep it private even on failure; GitHub cannot mask a decoded certificate name.
private_tool() {
  local operation=$1
  shift
  if ! "$@" > "$WORK_DIR/tool-output.log" 2>&1; then
    echo "$operation failed; raw signing diagnostics suppressed. Reproduce locally to inspect them." >&2
    return 1
  fi
}

rm -rf dist
if [[ "$BUILD_APP" == true ]]; then
  cargo build --release --locked -p voxelite-app --bin voxelite
  cargo build --release --locked -p voxelite-mcp
fi
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/voxelite "$APP/Contents/MacOS/Voxelite"
cp target/release/voxelite-mcp "$APP/Contents/MacOS/voxelite-mcp"
python3 - "$APP/Contents/Info.plist" <<'PY'
import os
import plistlib
import sys

with open("packaging/macos/Info.plist", "rb") as source:
    info = plistlib.load(source)
info["CFBundleVersion"] = info["CFBundleShortVersionString"] = os.environ["VERSION"]
info.update(
    SUFeedURL="https://voxelite.worldwidewebb.co/appcast.xml",
    SUPublicEDKey=os.environ["SPARKLE_PUBLIC_ED_KEY"],
    SUEnableAutomaticChecks=True,
    SUAutomaticallyUpdate=False,
    SUVerifyUpdateBeforeExtraction=True,
    SURequireSignedFeed=True,
)
with open(sys.argv[1], "wb") as output:
    plistlib.dump(info, output)
PY
ICONSET="$WORK_DIR/Voxelite.iconset"
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  sips -z "$s" "$s" packaging/macos/icon-1024.png --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
  sips -z "$((s*2))" "$((s*2))" packaging/macos/icon-1024.png --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/Voxelite.icns"

# Pin both the release and its archive digest; never execute unverified tools.
SPARKLE_VERSION=2.10.1
SPARKLE_SHA256=1a856d2a9edb8e628de88121d7be04cf0e2167f8641650b78fca00b14244cf27
curl --fail --location --retry 3 --proto '=https' --tlsv1.2 \
  "https://github.com/sparkle-project/Sparkle/releases/download/$SPARKLE_VERSION/Sparkle-$SPARKLE_VERSION.tar.xz" \
  --output "$WORK_DIR/Sparkle.tar.xz"
printf '%s  %s\n' "$SPARKLE_SHA256" "$WORK_DIR/Sparkle.tar.xz" | shasum -a 256 --check
mkdir -p dist/sparkle "$APP/Contents/Frameworks"
tar -xf "$WORK_DIR/Sparkle.tar.xz" -C dist/sparkle ./Sparkle.framework ./bin ./LICENSE
# ditto preserves the framework's versioned symlinks and executable modes.
ditto dist/sparkle/Sparkle.framework "$APP/Contents/Frameworks/Sparkle.framework"
cp dist/sparkle/LICENSE "$APP/Contents/Resources/Sparkle-LICENSE.txt"

if [[ ${MACOS_SIGNING_MODE:-auto} != adhoc && -n ${MACOS_CERTIFICATE_P12_BASE64:-} && -n ${MACOS_CERTIFICATE_PASSWORD:-} ]]; then
  # Restrict secret files without changing permissions on the distributable app.
  umask 077
  decode_secret MACOS_CERTIFICATE_P12_BASE64 "$WORK_DIR/certificate.p12"
  security list-keychains -d user > "$WORK_DIR/keychains.txt"
  # Parse before changing keychains; process-substitution failures are otherwise
  # ignored by Bash, and an empty list must not become one empty path.
  python3 -c 'import shlex,sys; [print(path) for path in shlex.split(open(sys.argv[1]).read())]' \
    "$WORK_DIR/keychains.txt" > "$WORK_DIR/keychain-paths.txt"
  while IFS= read -r keychain; do
    ORIGINAL_KEYCHAINS+=("$keychain")
  done < "$WORK_DIR/keychain-paths.txt"
  RESTORE_KEYCHAINS=true
  KEYCHAIN="$WORK_DIR/signing.keychain-db"
  KEYCHAIN_PASSWORD=$(openssl rand -hex 32)
  security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
  security set-keychain-settings -lut 21600 "$KEYCHAIN"
  security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
  private_tool 'Certificate import' security import "$WORK_DIR/certificate.p12" -k "$KEYCHAIN" -P "$MACOS_CERTIFICATE_PASSWORD" -T /usr/bin/codesign -T /usr/bin/security
  private_tool 'Keychain access configuration' security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
  security list-keychains -d user -s ${ORIGINAL_KEYCHAINS[@]+"${ORIGINAL_KEYCHAINS[@]}"} "$KEYCHAIN"
  rm -f "$WORK_DIR/certificate.p12"
  unset KEYCHAIN_PASSWORD
  # Accept an optional identity name or fingerprint; auto-detect only Developer ID.
  security find-identity -v -p codesigning "$KEYCHAIN" > "$WORK_DIR/identities.txt"
  IDENTITY=$(MACOS_SIGNING_IDENTITY="${MACOS_SIGNING_IDENTITY:-}" python3 - "$WORK_DIR/identities.txt" <<'PY'
import os
import re
import sys

identities = re.findall(r'\b([A-Fa-f0-9]{40}) "(Developer ID Application: [^"]+)"', open(sys.argv[1]).read())
requested = os.environ.get("MACOS_SIGNING_IDENTITY")
if requested:
    identities = [(digest, name) for digest, name in identities if requested == name or requested.lower() == digest.lower()]
if len(identities) != 1:
    raise SystemExit("Expected one valid Developer ID Application identity in the imported p12; set MACOS_SIGNING_IDENTITY to select one.")
print(identities[0][0])
PY
  )
  umask 022
else
  echo "NOTICE: Developer ID credentials unavailable or ad-hoc mode requested; using ad-hoc app signing and an unsigned DMG. Notarization is skipped."
fi
unset MACOS_CERTIFICATE_P12_BASE64 MACOS_CERTIFICATE_PASSWORD MACOS_SIGNING_IDENTITY

SIGN_ARGS=(--force --sign "$IDENTITY")
if [[ "$IDENTITY" != - ]]; then
  SIGN_ARGS+=(--keychain "$KEYCHAIN" --options runtime --timestamp)
else
  # Ad-hoc development builds cannot satisfy hardened runtime library validation.
  SIGN_ARGS+=(--timestamp=none)
fi
if [[ -d "$APP/Contents/Frameworks/Sparkle.framework" ]]; then
  FRAMEWORK="$APP/Contents/Frameworks/Sparkle.framework"
  # Sparkle 2.10.1's nested code, signed inside-out. Never use --deep to sign.
  private_tool 'Installer signing' codesign "${SIGN_ARGS[@]}" "$FRAMEWORK/Versions/B/XPCServices/Installer.xpc"
  private_tool 'Downloader signing' codesign "${SIGN_ARGS[@]}" --preserve-metadata=entitlements "$FRAMEWORK/Versions/B/XPCServices/Downloader.xpc"
  private_tool 'Autoupdate signing' codesign "${SIGN_ARGS[@]}" "$FRAMEWORK/Versions/B/Autoupdate"
  private_tool 'Updater signing' codesign "${SIGN_ARGS[@]}" "$FRAMEWORK/Versions/B/Updater.app"
  private_tool 'Framework signing' codesign "${SIGN_ARGS[@]}" "$FRAMEWORK"
fi
# Sign nested executables before sealing the bundle, with the same identity and
# hardened runtime options as the app. The stdio server needs no UI entitlements.
private_tool 'MCP signing' codesign "${SIGN_ARGS[@]}" "$APP/Contents/MacOS/voxelite-mcp"
private_tool 'Main executable signing' codesign "${SIGN_ARGS[@]}" --entitlements packaging/macos/Voxelite.entitlements "$APP/Contents/MacOS/Voxelite"
private_tool 'App signing' codesign "${SIGN_ARGS[@]}" --entitlements packaging/macos/Voxelite.entitlements "$APP"
private_tool 'App signature verification' codesign --verify --deep --strict "$APP"

NOTARIZE=false
if [[ "$IDENTITY" != - ]]; then
  if [[ -n ${APPLE_API_KEY_P8_BASE64:-} && -n ${APPLE_API_KEY_ID:-} && -n ${APPLE_API_ISSUER_ID:-} ]]; then
    NOTARIZE=true
    umask 077
    decode_secret APPLE_API_KEY_P8_BASE64 "$WORK_DIR/AuthKey.p8"
    umask 022
  else
    echo "NOTICE: App Store Connect API credentials incomplete; Developer ID signing succeeded, but notarization and Gatekeeper assessment are skipped."
  fi
fi
unset APPLE_API_KEY_P8_BASE64
notarize() {
  if ! xcrun notarytool submit "$1" --key "$WORK_DIR/AuthKey.p8" \
    --key-id "$APPLE_API_KEY_ID" --issuer "$APPLE_API_ISSUER_ID" \
    --wait --timeout 30m --output-format json > "$WORK_DIR/notary-result.json" 2> "$WORK_DIR/notary-error.log"; then
    echo "Notarization submission failed; raw authentication diagnostics suppressed." >&2
    return 1
  fi
  python3 - "$WORK_DIR/notary-result.json" <<'PY'
import json
import sys

result = json.load(open(sys.argv[1]))
if result.get("status") != "Accepted":
    raise SystemExit(f"Notarization was not accepted (submission {result.get('id', 'unknown')}). Inspect it with notarytool log.")
PY
}
if [[ "$NOTARIZE" == true ]]; then
  ditto -c -k --keepParent "$APP" "$WORK_DIR/Voxelite.zip"
  notarize "$WORK_DIR/Voxelite.zip"
  xcrun stapler staple "$APP"
  xcrun stapler validate "$APP"
  # Avoid verbose spctl output: it includes the certificate owner's identity.
  private_tool 'App Gatekeeper assessment' spctl --assess --type execute "$APP"
fi

# Package the stapled app so offline installs receive its ticket too.
STAGE="$WORK_DIR/stage"
mkdir -p "$STAGE"
ditto "$APP" "$STAGE/Voxelite.app"
ln -s /Applications "$STAGE/Applications"
hdiutil create -volname Voxelite -srcfolder "$STAGE" -fs HFS+ -format UDZO -ov "$DMG"
if [[ "$IDENTITY" != - ]]; then
  private_tool 'DMG signing' codesign --force --keychain "$KEYCHAIN" --sign "$IDENTITY" --timestamp "$DMG"
  private_tool 'DMG signature verification' codesign --verify --strict "$DMG"
fi
if [[ "$NOTARIZE" == true ]]; then
  notarize "$DMG"
  xcrun stapler staple "$DMG"
  xcrun stapler validate "$DMG"
  private_tool 'DMG Gatekeeper assessment' spctl --assess --type open --context context:primary-signature "$DMG"
fi
# The checksum must work when both release assets are downloaded side by side.
(cd dist && shasum -a 256 Voxelite.dmg > Voxelite.dmg.sha256)
cat "$DMG.sha256"
if [[ "$NOTARIZE" == true ]]; then
  # Only a fully notarized result is eligible for the public update feed.
  touch dist/.notarized
fi
