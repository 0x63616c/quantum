#!/usr/bin/env bash
# Build Voxelite.app (arm64) and dist/Voxelite.dmg on macOS with Xcode + Metal.
set -euo pipefail
# Never trace credentials, even if invoked with bash -x.
set +x
cd "$(dirname "$0")/.."

if [[ $(uname -s) != Darwin ]]; then
  echo "macOS packaging requires macOS and Xcode." >&2
  exit 1
fi
case "${MACOS_SIGNING_MODE:-auto}" in
  auto|adhoc) ;;
  *) echo "MACOS_SIGNING_MODE must be auto or adhoc." >&2; exit 1 ;;
esac

VERSION="${VERSION:-$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys;print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="voxelite-app"))')}"
export VERSION
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
    security list-keychains -d user -s "${ORIGINAL_KEYCHAINS[@]}" || status=1
  fi
  rm -rf "$WORK_DIR"
  exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# Decode secrets from the environment so they never appear in command arguments.
decode_secret() {
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
}

cargo build --release --locked -p voxelite-app --bin voxelite
rm -rf dist
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/voxelite "$APP/Contents/MacOS/Voxelite"
python3 - "$APP/Contents/Info.plist" <<'PY'
import os
import plistlib
import sys

with open("packaging/macos/Info.plist", "rb") as source:
    info = plistlib.load(source)
info["CFBundleVersion"] = info["CFBundleShortVersionString"] = os.environ["VERSION"]
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

if [[ ${MACOS_SIGNING_MODE:-auto} != adhoc && -n ${MACOS_CERTIFICATE_P12_BASE64:-} && -n ${MACOS_CERTIFICATE_PASSWORD:-} ]]; then
  # Restrict secret files without changing permissions on the distributable app.
  umask 077
  decode_secret MACOS_CERTIFICATE_P12_BASE64 "$WORK_DIR/certificate.p12"
  security list-keychains -d user > "$WORK_DIR/keychains.txt"
  while IFS= read -r keychain; do
    ORIGINAL_KEYCHAINS+=("$keychain")
  done < <(python3 -c 'import shlex,sys; print("\n".join(shlex.split(open(sys.argv[1]).read())))' "$WORK_DIR/keychains.txt")
  RESTORE_KEYCHAINS=true
  KEYCHAIN="$WORK_DIR/signing.keychain-db"
  KEYCHAIN_PASSWORD=$(openssl rand -hex 32)
  security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
  security set-keychain-settings -lut 21600 "$KEYCHAIN"
  security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
  security import "$WORK_DIR/certificate.p12" -k "$KEYCHAIN" -P "$MACOS_CERTIFICATE_PASSWORD" -T /usr/bin/codesign -T /usr/bin/security >/dev/null
  security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN" >/dev/null
  security list-keychains -d user -s "${ORIGINAL_KEYCHAINS[@]}" "$KEYCHAIN"
  rm -f "$WORK_DIR/certificate.p12"
  unset KEYCHAIN_PASSWORD
  # Accept an optional identity name or fingerprint; auto-detect only Developer ID.
  security find-identity -v -p codesigning "$KEYCHAIN" > "$WORK_DIR/identities.txt"
  IDENTITY=$(python3 - "$WORK_DIR/identities.txt" <<'PY'
import os
import re
import sys

identities = re.findall(r'\b([A-Fa-f0-9]{40}) "(Developer ID Application: [^"]+)"', open(sys.argv[1]).read())
requested = os.environ.get("MACOS_SIGNING_IDENTITY")
if requested:
    identities = [(digest, name) for digest, name in identities if requested in (digest, name)]
if len(identities) != 1:
    raise SystemExit("Expected one valid Developer ID Application identity in the imported p12; set MACOS_SIGNING_IDENTITY to select one.")
print(identities[0][0])
PY
  )
  umask 022
else
  echo "NOTICE: Developer ID credentials unavailable or ad-hoc mode requested; using ad-hoc app signing and an unsigned DMG. Notarization is skipped."
fi

SIGN_ARGS=(--force --sign "$IDENTITY")
if [[ "$IDENTITY" != - ]]; then
  SIGN_ARGS+=(--keychain "$KEYCHAIN" --options runtime --timestamp)
else
  # Ad-hoc development builds cannot satisfy hardened runtime library validation.
  SIGN_ARGS+=(--timestamp=none)
fi
codesign "${SIGN_ARGS[@]}" --entitlements packaging/macos/Voxelite.entitlements "$APP"
codesign --verify --deep --strict --verbose=2 "$APP"

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
notarize() {
  xcrun notarytool submit "$1" --key "$WORK_DIR/AuthKey.p8" \
    --key-id "$APPLE_API_KEY_ID" --issuer "$APPLE_API_ISSUER_ID" \
    --wait --timeout 30m --output-format json > "$WORK_DIR/notary-result.json"
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
  spctl --assess --type execute --verbose=2 "$APP"
fi

# Package the stapled app so offline installs receive its ticket too.
STAGE="$WORK_DIR/stage"
mkdir -p "$STAGE"
ditto "$APP" "$STAGE/Voxelite.app"
ln -s /Applications "$STAGE/Applications"
hdiutil create -volname Voxelite -srcfolder "$STAGE" -fs HFS+ -format UDZO -ov "$DMG"
if [[ "$IDENTITY" != - ]]; then
  codesign --force --keychain "$KEYCHAIN" --sign "$IDENTITY" --timestamp "$DMG"
  codesign --verify --strict --verbose=2 "$DMG"
fi
if [[ "$NOTARIZE" == true ]]; then
  notarize "$DMG"
  xcrun stapler staple "$DMG"
  xcrun stapler validate "$DMG"
  spctl --assess --type open --context context:primary-signature --verbose=2 "$DMG"
  # Only a fully notarized result is eligible for the public update feed.
  touch dist/.notarized
fi
shasum -a 256 "$DMG" | tee "$DMG.sha256"
