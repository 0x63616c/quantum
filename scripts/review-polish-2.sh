#!/usr/bin/env bash
# Exercise native palette interchange through actual desktop portal dialogs.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
cargo build -p voxelite-app
export VK_ICD_FILENAMES="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
xvfb-run -a -s '-screen 0 1440x900x24 -nolisten tcp' dbus-run-session -- bash <<'REVIEW'
set -euo pipefail
export XDG_RUNTIME_DIR
XDG_RUNTIME_DIR="$(mktemp -d)"
chmod 700 "$XDG_RUNTIME_DIR"
export VOXELITE_DATA_DIR="$XDG_RUNTIME_DIR/data"
export XDG_CURRENT_DESKTOP=GNOME
unset WAYLAND_DISPLAY
app_pid=''
cleanup() {
  result=$?
  if [[ "$result" != 0 ]]; then
    cat "$XDG_RUNTIME_DIR/app.log" || true
    import -window root /tmp/voxelite-polish-2-dialog-failure.png || true
  fi
  [[ -z "$app_pid" ]] || kill "$app_pid" 2>/dev/null || true
  rm -rf "$XDG_RUNTIME_DIR"
}
trap cleanup EXIT
mkdir -p "$VOXELITE_DATA_DIR"
target/debug/voxelite --theme dark --exit-after 150 >"$XDG_RUNTIME_DIR/app.log" 2>&1 &
app_pid=$!
window=''
for attempt in {1..100}; do
  window="$(xdotool search --all --onlyvisible --pid "$app_pid" --name '.*Voxelite$' 2>/dev/null | head -1 || true)"
  [[ -z "$window" ]] || break
  sleep 0.1
done
[[ -n "$window" ]]
xdotool windowmove "$window" 0 0 windowsize "$window" 1440 900 windowfocus "$window"
sleep 5
xdotool key ctrl+k
xdotool type --clearmodifiers 'manage palettes'
xdotool key Return
sleep 0.5
chooser() {
  local title="$1" path="$2"
  local dialog=''
  for attempt in {1..100}; do
    dialog="$(xdotool search --onlyvisible --name "^$title" 2>/dev/null | head -1 || true)"
    [[ -z "$dialog" ]] || break
    sleep 0.1
  done
  [[ -n "$dialog" ]]
  xdotool windowfocus "$dialog" key ctrl+l
  sleep 0.2
  xdotool key ctrl+a
  xdotool type --clearmodifiers --delay 4 "$path"
  sleep 0.3
  local width height
  width="$(xdotool getwindowgeometry --shell "$dialog" | sed -n 's/WIDTH=//p')"
  height="$(xdotool getwindowgeometry --shell "$dialog" | sed -n 's/HEIGHT=//p')"
  xdotool mousemove --window "$dialog" "$((width-80))" "$((height-24))" click 1
  sleep 0.8
  xdotool windowfocus "$window"
}
xdotool mousemove 680 659 click 1
chooser Save "$VOXELITE_DATA_DIR/export.hex"
[[ -s "$VOXELITE_DATA_DIR/export.hex" ]]
xdotool mousemove 785 659 click 1
chooser Save "$VOXELITE_DATA_DIR/export.gpl"
[[ -s "$VOXELITE_DATA_DIR/export.gpl" ]]
xdotool mousemove 590 659 click 1
chooser Open "$VOXELITE_DATA_DIR/export.gpl"
python3 - "$VOXELITE_DATA_DIR" <<'PY'
import json, pathlib, sys
root = pathlib.Path(sys.argv[1])
palette = json.loads((root/'palettes.json').read_text())['entries'][0]['palette']
source = json.loads(pathlib.Path('examples/autumn-workshop.voxl').read_text())['palette']
assert [s['rgba'] for s in palette['slots']] == [s['rgba'] for s in source['slots']]
assert len((root/'export.hex').read_text().splitlines()) == 36
assert (root/'export.gpl').read_text().startswith('GIMP Palette\n')
PY
xdotool key Escape
xdotool key --clearmodifiers ctrl+s
chooser Save "$VOXELITE_DATA_DIR/working-project.voxl"
python3 - "$VOXELITE_DATA_DIR/working-project.voxl" <<'PY'
import json, sys
project = json.load(open(sys.argv[1]))
assert project['document']['name'] == 'Autumn Workshop', project.keys()
assert sum(len(layer['voxels']) for layer in project['layers']) == 27897
PY
# Save As opens another destination chooser and preserves the project contents.
xdotool key --clearmodifiers ctrl+shift+s
chooser Save "$VOXELITE_DATA_DIR/renamed-project.voxl"
cmp "$VOXELITE_DATA_DIR/working-project.voxl" "$VOXELITE_DATA_DIR/renamed-project.voxl"
printf 'Native palette interchange, Save and Save As passed.\n'
REVIEW
