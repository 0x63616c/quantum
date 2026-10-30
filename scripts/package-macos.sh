#!/usr/bin/env bash
# Build Voxelite.app (arm64) and package dist/Voxelite.dmg. Runs on macOS with Xcode
# (Metal toolchain installed). Signs with $MACOS_SIGN_IDENTITY when set, else ad-hoc.
set -euo pipefail
cd "$(dirname "$0")/.."
VERSION="${VERSION:-$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys;print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="voxelite-app"))')}"
IDENTITY="${MACOS_SIGN_IDENTITY:--}"
cargo build --release --locked -p voxelite-app --bin voxelite
APP=dist/Voxelite.app
rm -rf dist && mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/voxelite "$APP/Contents/MacOS/Voxelite"
sed "s/@VERSION@/$VERSION/g" packaging/macos/Info.plist > "$APP/Contents/Info.plist"
ICONSET=$(mktemp -d)/Voxelite.iconset; mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  sips -z $s $s packaging/macos/icon-1024.png --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
  sips -z $((s*2)) $((s*2)) packaging/macos/icon-1024.png --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/Voxelite.icns"
codesign --force --deep --options runtime --timestamp=none --sign "$IDENTITY" "$APP"
codesign --verify --strict --verbose=2 "$APP"
STAGE=$(mktemp -d); cp -R "$APP" "$STAGE/"; ln -s /Applications "$STAGE/Applications"
hdiutil create -volname Voxelite -srcfolder "$STAGE" -fs HFS+ -format UDZO -ov dist/Voxelite.dmg
[ "$IDENTITY" = "-" ] || codesign --force --sign "$IDENTITY" dist/Voxelite.dmg
shasum -a 256 dist/Voxelite.dmg | tee dist/Voxelite.dmg.sha256
