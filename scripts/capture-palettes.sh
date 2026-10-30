#!/usr/bin/env bash
# Capture real palette interactions with an isolated library and Linux window manager.
set -euo pipefail
cd "$(dirname "$0")/.."
: "${BIN:?Set BIN to the built voxelite executable}"
BIN="$(realpath "$BIN")"
OUT="${OUT:-/workspace/ui-polish-shots/after-palette}"
mkdir -p "$OUT"
OUT="$(realpath "$OUT")"
export BIN OUT
export THEME="${THEME:-dark}"
export VK_ICD_FILENAMES="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
xvfb-run -a -s '-screen 0 1600x1000x24 -nolisten tcp' dbus-run-session -- bash <<'CAPTURE'
set -euo pipefail
export XDG_RUNTIME_DIR
XDG_RUNTIME_DIR="$(mktemp -d)"
chmod 700 "$XDG_RUNTIME_DIR"
export VOXELITE_DATA_DIR="$XDG_RUNTIME_DIR/data" XDG_CURRENT_DESKTOP=GNOME
export XDG_CACHE_HOME="$XDG_RUNTIME_DIR/cache" XDG_CONFIG_HOME="$XDG_RUNTIME_DIR/config" GSETTINGS_BACKEND=memory
unset WAYLAND_DISPLAY
app_pid=''
wm_pid=''
cleanup() {
  if [[ -n "$app_pid" ]]; then kill "$app_pid" 2>/dev/null || true; wait "$app_pid" 2>/dev/null || true; fi
  if [[ -n "$wm_pid" ]]; then kill "$wm_pid" 2>/dev/null || true; fi
  rm -rf "$XDG_RUNTIME_DIR"
}
trap cleanup EXIT
xfwm4 --compositor=off --sm-client-disable >"$OUT/window-manager.log" 2>&1 &
wm_pid=$!
"$BIN" --theme "$THEME" --exit-after 240 --session-socket "$XDG_RUNTIME_DIR/app.sock" >"$OUT/app.log" 2>&1 &
app_pid=$!
window=''
for attempt in {1..150}; do
  window="$(xdotool search --all --onlyvisible --pid "$app_pid" --name 'Autumn Workshop.*Voxelite' 2>/dev/null | head -1 || true)"
  [[ -n "$window" ]] && break
  kill -0 "$app_pid"
  sleep 0.2
done
[[ -n "$window" ]]
xdotool windowmove "$window" 0 0 windowsize "$window" 1440 900 windowfocus "$window"
sleep 5
shot() { sleep 1; timeout 15s import -window "$window" "$OUT/$1.png"; }
click() { xdotool mousemove --window "$window" "$1" "$2" click 1; sleep .2; }
xdotool key --clearmodifiers ctrl+k
xdotool type --clearmodifiers --delay 50 'manage palettes'
xdotool key Return
sleep 2
xdotool mousemove --window "$window" 1320 820
shot default
# Sidebar keyboard navigation from the project palette to the first built-in.
click 390 186
xdotool key Down
sleep 2
xdotool mousemove --window "$window" 1320 820
shot built-in-read-only
# Preview and rollback while the pointer remains down; releasing after Escape must not commit.
xdotool mousemove --window "$window" 675 490 mousedown 1
xdotool mousemove --sync --window "$window" 705 470
shot picker-mid-drag
xdotool key Escape
xdotool mouseup 1
shot cancelled-preview
if [[ -e "$VOXELITE_DATA_DIR/palettes.json" ]]; then
  printf '%s\n' 'Unexpected persistence after cancelled source preview' >&2
  exit 1
fi
click 695 480
xdotool mousemove --window "$window" 1320 820
shot after-copy-on-edit
cp "$VOXELITE_DATA_DIR/palettes.json" "$OUT/copied-library.json"
# Arrow navigation and duplicate shortcut keep focus in the grid.
click 560 230
xdotool key Right ctrl+d
shot keyboard-duplicate
cp "$VOXELITE_DATA_DIR/palettes.json" "$OUT/duplicated-library.json"
# A real swatch drag shows the insertion marker before dropping.
xdotool mousemove --window "$window" 596 230 mousedown 1
xdotool mousemove --sync --window "$window" 620 230
sleep .2
xdotool mousemove --sync --window "$window" 745 230
shot swatch-drag
xdotool mouseup 1
sleep .5
cp "$VOXELITE_DATA_DIR/palettes.json" "$OUT/reordered-library.json"
# Recent colors come from committed picker changes.
for x in 575 610 645 680 715 750; do click "$x" 666; done
xdotool mousemove --window "$window" 1320 820
shot recents
# Apply and add are explicit targets for a recent swatch.
click 1075 722
click 555 759
sleep 1
python3 - "$VOXELITE_DATA_DIR/palettes.json" <<'VERIFY'
import json, sys
assert len(json.load(open(sys.argv[1]))['entries'][0]['palette']['order']) == 18
VERIFY
click 970 722
click 585 759
click 965 94
xdotool mousemove --window "$window" 1320 820
shot import-export-menu
# Import a full 256-color cube through the native file chooser.
python3 - "$XDG_RUNTIME_DIR/Cube.hex" <<'FIXTURE'
from pathlib import Path
import sys
Path(sys.argv[1]).write_text(''.join(f'{r:02X}{g:02X}{b:02X}\n' for r in (0,85,170,255) for g in (0,85,170,255) for b in range(0,256,17)))
FIXTURE
click 965 147
sleep 2
xdotool key --clearmodifiers ctrl+l
xdotool type --clearmodifiers --delay 20 "$XDG_RUNTIME_DIR/Cube.hex"
xdotool key Return
sleep 1
xdotool key Return
sleep 2
xdotool windowfocus "$window"
xdotool mousemove --window "$window" 1320 820
shot many-colors
cp "$VOXELITE_DATA_DIR/palettes.json" "$OUT/library.json"
python3 - "$OUT" <<'VERIFY'
from pathlib import Path
import json, sys
out = Path(sys.argv[1])
load = lambda name: json.loads((out / name).read_text())
copied = load('copied-library.json')['entries']
assert len(copied) == 1 and copied[0]['name'] == 'PICO-8 copy'
assert len(copied[0]['palette']['order']) == 16
duplicated = load('duplicated-library.json')['entries'][0]['palette']
reordered = load('reordered-library.json')['entries'][0]['palette']
assert len(duplicated['order']) == 17
assert duplicated['slots'] == reordered['slots']
assert duplicated['order'] != reordered['order']
library = load('library.json')
assert len(library['entries'][-1]['palette']['order']) == 256
assert len(library['recent_colors']) >= 6
VERIFY
# Invalid fields must remain visible and leave persisted data untouched.
click 825 473
xdotool key --clearmodifiers ctrl+a
xdotool type --clearmodifiers '#zzzzzz'
xdotool key Return
xdotool mousemove --window "$window" 1320 820
shot invalid-hex
cmp "$VOXELITE_DATA_DIR/palettes.json" "$OUT/library.json"
xdotool key Escape
# Enter and blur both commit exact numeric values through the host.
expect_recent() {
  sleep 1
  python3 - "$VOXELITE_DATA_DIR/palettes.json" "$1" <<'VERIFY'
import json, sys
actual = json.load(open(sys.argv[1]))['recent_colors'][0]
assert actual == json.loads(sys.argv[2]), actual
VERIFY
}
edit_field() {
  click "$1" "$2"
  xdotool key --clearmodifiers ctrl+a
  xdotool type --clearmodifiers "$3"
  xdotool key "${4:-Return}"
}
edit_field 825 473 '#12AB34'
expect_recent '[18,171,52]'
edit_field 815 546 '200' Tab
expect_recent '[200,171,52]'
edit_field 815 619 '210'
edit_field 925 619 '0'
expect_recent '[200,200,200]'
xdotool mousemove --window "$window" 1320 820
shot grey-preserves-hue
edit_field 925 619 '100'
expect_recent '[0,100,200]'
xdotool mousemove --window "$window" 1320 820
shot numeric-edit
# Scroll the selection to the last cell and surface the capacity error inline.
click 560 238
xdotool key End ctrl+d
xdotool mousemove --window "$window" 1320 820
shot capacity-error
# Last-swatch removal is disabled in the toolbar and rejected by the keyboard path.
click 400 690
click 560 238
xdotool key BackSpace
xdotool mousemove --window "$window" 1320 820
shot last-swatch-error
click 400 766
xdotool mousemove --window "$window" 1320 820
shot delete-confirmation
CAPTURE
