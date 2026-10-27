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
  echo "SPARKLE_ED_PRIVATE_KEY is required for a release appcast." >&2
  exit 1
fi
if [[ ! -f dist/.notarized ]]; then
  echo "A notarized DMG is required for a release appcast." >&2
  exit 1
fi
if [[ ! -d dist/Voxelite.app/Contents/Frameworks/Sparkle.framework ]]; then
  echo "The release must embed Sparkle." >&2
  exit 1
fi
: "${TAG:?Set TAG to the GitHub release tag}"
export TAG
export GITHUB_REPOSITORY="${GITHUB_REPOSITORY:-0x63616c/voxelite}"
umask 077
WORK_DIR=$(mktemp -d "${TMPDIR:-/tmp}/voxelite-appcast.XXXXXX")
trap 'rm -rf "$WORK_DIR"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

DOWNLOAD_PREFIX=$(python3 - <<'PY'
import os
import re
from urllib.parse import quote

repo = os.environ["GITHUB_REPOSITORY"]
if repo != "0x63616c/voxelite":
    raise SystemExit("Invalid GitHub repository")
if not re.fullmatch(r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", os.environ["TAG"]):
    raise SystemExit("Invalid release tag")
print(f"https://github.com/{repo}/releases/download/{quote(os.environ['TAG'], safe='')}/")
PY
)
export DOWNLOAD_PREFIX
# Fetch history in a separate workflow step without signing credentials. Local
# callers must do the same; a missing history file must never reset the feed.
python3 - dist/previous-appcast.xml <<'PY'
import os
import plistlib
import sys
sys.path.insert(0, "scripts")
from macos_appcast import SPARKLE, validate, version_tuple

with open("dist/Voxelite.app/Contents/Info.plist", "rb") as source:
    info = plistlib.load(source)
if os.environ["TAG"] != "v" + info["CFBundleVersion"]:
    raise SystemExit("Release tag does not match the packaged bundle version")
previous = validate(sys.argv[1], allow_empty=True)
if previous and version_tuple(info["CFBundleVersion"]) <= version_tuple(previous[0].findtext(SPARKLE + "version")):
    raise SystemExit("Release version must be newer than the published appcast")
PY
(cd dist && shasum -a 256 --check Voxelite.dmg.sha256)
xcrun stapler validate dist/Voxelite.dmg
mkdir -p "$WORK_DIR/archives"
cp dist/Voxelite.dmg "$WORK_DIR/archives/Voxelite.dmg"
cp dist/previous-appcast.xml "$WORK_DIR/appcast.xml"
# Sparkle discovers release notes beside the archive using the same basename.
# Escape all text, including commit subjects, before embedding it as HTML.
python3 - "$WORK_DIR/archives/Voxelite.html" <<'PY'
import html
from pathlib import Path
import sys

notes = Path("dist/RELEASE_NOTES.md").read_text(encoding="utf-8")
Path(sys.argv[1]).write_text(
    '<!doctype html><html><head><meta charset="utf-8"></head><body>\n'
    '<pre style="white-space: pre-wrap; font-family: system-ui">'
    + html.escape(notes) + '</pre>\n</body></html>\n', encoding="utf-8",
)
PY
# Pass the key only through stdin; never save it in the checkout or login keychain.
# The pinned tool compares it with SUPublicEDKey inside the archived app.
# Suppress both tools' output: malformed key diagnostics may contain key input.
if ! printf '%s\n' "$SPARKLE_ED_PRIVATE_KEY" | dist/sparkle/bin/generate_appcast \
  --ed-key-file - --maximum-deltas 0 --maximum-versions 0 \
  --download-url-prefix "$DOWNLOAD_PREFIX" \
  --release-notes-url-prefix "$DOWNLOAD_PREFIX" \
  --link 'https://voxelite.worldwidewebb.co' \
  -o "$WORK_DIR/appcast.xml" "$WORK_DIR/archives" > /dev/null 2>&1; then
  echo "Appcast generation failed; raw key diagnostics suppressed." >&2
  exit 1
fi

# The enclosure signature alone does not authenticate the feed's update metadata.
# Verify the embedded feed signature with the pinned tool before publishing bytes.
# sign_update can echo malformed key input; never expose its raw diagnostics.
if ! printf '%s\n' "$SPARKLE_ED_PRIVATE_KEY" | dist/sparkle/bin/sign_update \
  --ed-key-file - --verify "$WORK_DIR/appcast.xml" > /dev/null 2>&1; then
  echo "Appcast signature verification failed; raw key diagnostics suppressed." >&2
  exit 1
fi

# A mismatched key can produce an unsigned enclosure with only a tool warning.
# Check every item and the exact new asset lengths before publishing any bytes.
cp "$WORK_DIR/archives/Voxelite.html" dist/Voxelite.html
python3 scripts/check-macos-appcast.py "$WORK_DIR/appcast.xml" \
  --release-dir dist --previous dist/previous-appcast.xml
mkdir -p site
# Preserve the feed's own Sparkle signature; do not reformat generated XML.
cp "$WORK_DIR/appcast.xml" site/appcast.xml
chmod 644 site/appcast.xml dist/Voxelite.html
if [[ -n ${GITHUB_OUTPUT:-} ]]; then
  echo 'generated=true' >> "$GITHUB_OUTPUT"
fi
echo "Generated site/appcast.xml for upload as a release asset. The site deploy must copy this file unchanged to /appcast.xml."
