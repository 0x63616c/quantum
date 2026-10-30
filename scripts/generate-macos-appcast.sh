#!/usr/bin/env bash
# Create a feed for the final, stapled DMG. Does not publish, commit, or push.
set -euo pipefail
set +x
# Only stdin should expose the key to generate_appcast, not its child processes.
export -n SPARKLE_ED_PRIVATE_KEY
cd "$(dirname "$0")/.."
if [[ -n ${GITHUB_OUTPUT:-} ]]; then
  echo 'generated=false' >> "$GITHUB_OUTPUT"
fi
if [[ -z ${SPARKLE_ED_PRIVATE_KEY:-} ]]; then
  echo "NOTICE: SPARKLE_ED_PRIVATE_KEY is unavailable; skipping appcast generation."
  exit 0
fi
if [[ ! -f dist/.notarized ]]; then
  echo "NOTICE: The DMG was not notarized; skipping the public update feed."
  exit 0
fi
if [[ ! -d dist/Voxelite.app/Contents/Frameworks/Sparkle.framework ]]; then
  echo "NOTICE: Sparkle was not embedded (no public key); skipping appcast generation."
  exit 0
fi
: "${TAG:?Set TAG to the GitHub release tag}"
export TAG
export GITHUB_REPOSITORY="${GITHUB_REPOSITORY:-0x63616c/voxelite}"
WORK_DIR=$(mktemp -d "${TMPDIR:-/tmp}/voxelite-appcast.XXXXXX")
trap 'rm -rf "$WORK_DIR"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

DOWNLOAD_PREFIX=$(python3 - <<'PY'
import os
import re
from urllib.parse import quote

repo = os.environ["GITHUB_REPOSITORY"]
if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repo):
    raise SystemExit("Invalid GitHub repository")
print(f"https://github.com/{repo}/releases/download/{quote(os.environ['TAG'], safe='')}/")
PY
)
export DOWNLOAD_PREFIX
(cd dist && shasum -a 256 --check Voxelite.dmg.sha256)
xcrun stapler validate dist/Voxelite.dmg
mkdir -p "$WORK_DIR/archives"
cp dist/Voxelite.dmg "$WORK_DIR/archives/Voxelite.dmg"
# Pass the key only through stdin; never save it in the checkout or login keychain.
# The pinned tool compares it with SUPublicEDKey inside the archived app.
printf '%s\n' "$SPARKLE_ED_PRIVATE_KEY" | dist/sparkle/bin/generate_appcast \
  --ed-key-file - --maximum-deltas 0 \
  --download-url-prefix "$DOWNLOAD_PREFIX" \
  --link 'https://voxelite.worldwidewebb.co' \
  -o "$WORK_DIR/appcast.xml" "$WORK_DIR/archives"

# The enclosure signature alone does not authenticate the feed's update metadata.
# Verify the embedded feed signature with the pinned tool before publishing bytes.
# sign_update can echo malformed key input; never expose its raw diagnostics.
if ! printf '%s\n' "$SPARKLE_ED_PRIVATE_KEY" | dist/sparkle/bin/sign_update \
  --ed-key-file - --verify "$WORK_DIR/appcast.xml" > "$WORK_DIR/feed-verification.log" 2>&1; then
  echo "Appcast signature verification failed; raw key diagnostics suppressed." >&2
  exit 1
fi

# A mismatched key can produce an unsigned enclosure with only a tool warning.
# Reject it, wrong asset URLs, stale versions, and incorrect lengths before upload.
python3 - "$WORK_DIR/appcast.xml" <<'PY'
import base64
import os
import pathlib
import plistlib
import sys
import xml.etree.ElementTree as ET

sparkle = "{http://www.andymatuschak.org/xml-namespaces/sparkle}"
root = ET.parse(sys.argv[1]).getroot()
items = root.findall("channel/item")
with open("dist/Voxelite.app/Contents/Info.plist", "rb") as source:
    info = plistlib.load(source)
if len(items) != 1:
    raise SystemExit("Expected exactly one update in the generated appcast")
item = items[0]
enclosure = item.find("enclosure")
if enclosure is None:
    raise SystemExit("Missing appcast enclosure")
try:
    signature = base64.b64decode(enclosure.get(sparkle + "edSignature", ""), validate=True)
except ValueError:
    raise SystemExit("Invalid appcast Ed25519 signature") from None
if len(signature) != 64:
    raise SystemExit("Missing appcast signature; check that the private key matches the app's SUPublicEDKey")
if enclosure.get("url") != os.environ["DOWNLOAD_PREFIX"] + "Voxelite.dmg":
    raise SystemExit("Appcast does not reference the expected GitHub release asset")
if enclosure.get("length") != str(pathlib.Path("dist/Voxelite.dmg").stat().st_size):
    raise SystemExit("Appcast DMG length does not match")
if item.findtext(sparkle + "version") != info["CFBundleVersion"]:
    raise SystemExit("Appcast version does not match the bundle")
if os.environ["TAG"] != "v" + info["CFBundleVersion"]:
    raise SystemExit("Release tag does not match the packaged bundle version")
PY
mkdir -p site
# Preserve the feed's own Sparkle signature; do not reformat generated XML.
cp "$WORK_DIR/appcast.xml" site/appcast.xml
if [[ -n ${GITHUB_OUTPUT:-} ]]; then
  echo 'generated=true' >> "$GITHUB_OUTPUT"
fi
echo "Generated site/appcast.xml for upload as a release asset. The site deploy must copy this file unchanged to /appcast.xml."
