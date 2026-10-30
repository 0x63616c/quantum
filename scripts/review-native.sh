#!/usr/bin/env bash
# Real native keyboard/dialog regression checks; artifacts contain only fixture data.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
cargo build -p voxelite-app --bin voxelite
export VOXELITE_REVIEW_ROOT="$PWD"
export VK_ICD_FILENAMES="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
export VOXELITE_REVIEW_ARTIFACTS="${VOXELITE_REVIEW_ARTIFACTS:-$PWD/screenshots/review-1}"
mkdir -p "$VOXELITE_REVIEW_ARTIFACTS"
xvfb-run -a -s '-screen 0 2880x1800x24 -nolisten tcp' dbus-run-session -- bash <<'REVIEW'
set -euo pipefail
export XDG_RUNTIME_DIR
XDG_RUNTIME_DIR="$(mktemp -d)"
chmod 700 "$XDG_RUNTIME_DIR"
export XDG_CURRENT_DESKTOP=GNOME
unset WAYLAND_DISPLAY
app_pid=''
cleanup() {
  if [[ -n "$app_pid" ]]; then kill "$app_pid" 2>/dev/null || true; fi
  rm -rf "$XDG_RUNTIME_DIR"
}
trap cleanup EXIT
artifact="$VOXELITE_REVIEW_ARTIFACTS"
project="$XDG_RUNTIME_DIR/review.voxl"
cp examples/autumn-workshop.voxl "$project"
start_app() {
  GPUI_X11_SCALE_FACTOR="$1" target/debug/voxelite --theme "$2" --exit-after 180 "${@:3}" >"$XDG_RUNTIME_DIR/app.log" 2>&1 &
  app_pid=$!
  window=''
  for attempt in {1..100}; do
    window="$(xdotool search --all --onlyvisible --pid "$app_pid" --name '.*Voxelite$' 2>/dev/null | head -1 || true)"
    [[ -n "$window" ]] && break
    kill -0 "$app_pid" || { cat "$XDG_RUNTIME_DIR/app.log"; exit 1; }
    sleep 0.1
  done
  [[ -n "$window" ]]
  xdotool windowmove "$window" 0 0 windowfocus "$window"
}
stop_app() {
  kill "$app_pid"
  wait "$app_pid" || true
  app_pid=''
}
wait_title() {
  for attempt in {1..100}; do
    [[ "$(xdotool getwindowname "$window")" == "$1" ]] && return
    sleep 0.1
  done
  timeout 10s import -window root /tmp/voxelite-review-failure.png || true
  printf 'Expected title: %s; got: %s\n' "$1" "$(xdotool getwindowname "$window")" >&2
  exit 1
}
search() {
  xdotool key --clearmodifiers ctrl+k
  xdotool type --clearmodifiers --delay 20 "$1"
  xdotool key Return
  sleep 0.3
}
capture() {
  xdotool mousemove 2879 1799
  sleep 0.3
  kill -0 "$app_pid"
  timeout 15s import -window "$window" "$XDG_RUNTIME_DIR/capture.png"
  mv "$XDG_RUNTIME_DIR/capture.png" "$artifact/$1.png"
}
start_app 1 dark "$project"
xdotool windowsize "$window" 1024 640
sleep 6
# Searching must not apply the letters as editing shortcuts.
xdotool key ctrl+k
xdotool type --clearmodifiers 'paint'
capture command-search
xdotool key Escape
wait_title 'Autumn Workshop — Voxelite'
search 'choose tool'
capture tools
xdotool type --clearmodifiers 'paint'
xdotool key Return
wait_title 'Autumn Workshop — Voxelite'
# Camera-axis controls must navigate without placing a voxel through the overlay.
xdotool mousemove 632 174 click 1
sleep 4
capture axis-right
wait_title 'Autumn Workshop — Voxelite'
xdotool mousemove 608 126 click 1
sleep 4
capture axis-top
wait_title 'Autumn Workshop — Voxelite'
search 'frame model'
sleep 4
# Mirror controls disclose only while active; command search remains the entry.
search 'mirror x'
capture mirror-active
search 'mirror x'
wait_title 'Autumn Workshop — Voxelite'
search 'choose palette'
xdotool key Down Down Down Down Down Down Down
capture palettes
xdotool key Return
wait_title 'Autumn Workshop — Voxelite'
# A keyboard edit, then Save & continue must save once and finish New.
xdotool key b Return
wait_title 'Autumn Workshop • — Voxelite'
xdotool key ctrl+n
capture unsaved
xdotool key Return
wait_title 'Untitled — Voxelite'
python3 - "$project" <<'PY'
import json,sys
project=json.load(open(sys.argv[1]))
assert sum(len(layer['voxels']) for layer in project['layers']) == 27898
PY
# The native portal is modal even if keys are deliberately sent to the app.
xdotool key ctrl+o
file_dialog=''
for attempt in {1..100}; do
  file_dialog="$(xdotool search --onlyvisible --name '^Open' 2>/dev/null | head -1 || true)"
  [[ -n "$file_dialog" ]] && break
  sleep 0.1
done
[[ -n "$file_dialog" ]] || { cat "$XDG_RUNTIME_DIR/app.log"; exit 1; }
xdotool key --window "$window" b Return Right Return
wait_title 'Untitled — Voxelite'
xdotool windowfocus "$file_dialog" key Escape
sleep 0.5
xdotool windowfocus "$window"
wait_title 'Untitled — Voxelite'
# Open through the real chooser, then exercise Cancel and Discard by keyboard.
xdotool key ctrl+o
sleep 0.6
file_dialog="$(xdotool search --onlyvisible --name '^Open' | head -1)"
xdotool windowfocus "$file_dialog" key ctrl+l
sleep 0.3
xdotool type --clearmodifiers --delay 5 "$project"
sleep 0.5
# GTK's portal can keep Location entry focus after Return under bare Xvfb.
# Activate the actual Open button to complete the chooser consistently.
dialog_width="$(xdotool getwindowgeometry --shell "$file_dialog" | sed -n 's/WIDTH=//p')"
dialog_height="$(xdotool getwindowgeometry --shell "$file_dialog" | sed -n 's/HEIGHT=//p')"
xdotool mousemove --window "$file_dialog" "$((dialog_width - 80))" "$((dialog_height - 24))" click 1
wait_title 'Autumn Workshop — Voxelite'
xdotool windowfocus "$window" key Right Return
wait_title 'Autumn Workshop • — Voxelite'
xdotool key ctrl+n Left Return
wait_title 'Autumn Workshop • — Voxelite'
xdotool key ctrl+n Left Left Return
wait_title 'Untitled — Voxelite'
stop_app
# Inspect preset previews and the truthful local-session guide in both themes.
for theme in dark light; do
  start_app 1 "$theme"
  xdotool windowsize "$window" 1440 900
  sleep 6
  search 'choose palette'
  capture "palette-$theme"
  xdotool key Escape
  xdotool mousemove 1200 302 click 1
  capture "guide-$theme"
  xdotool key Escape
  wait_title 'Autumn Workshop — Voxelite'
  stop_app
done
# Boundary layout: 128-byte names, 64 layers and all 256 palette slots.
python3 - "$XDG_RUNTIME_DIR/boundary.voxl" <<'PYBOUNDARY'
import json,sys,uuid
project=json.load(open('examples/autumn-workshop.voxl'))
project['document']['name']='Autumn Workshop ' + 'extended-name-' * 8
for index in range(37,257):
    project['palette']['slots'].append({'index':index,'rgba':[index-1,256-index,(index-1)^7,255]})
    project['palette']['order'].append(index)
for index in range(4,64):
    project['layers'].append({'id':str(uuid.UUID(int=1000+index)), 'name':f'Layer {index+1}: '+ 'long layer name ' * 6, 'locked':False, 'visible':True, 'voxels':[]})
json.dump(project,open(sys.argv[1],'w'))
PYBOUNDARY
for theme in dark light; do
  start_app 1 "$theme" "$XDG_RUNTIME_DIR/boundary.voxl"
  xdotool windowsize "$window" 1024 640
  sleep 6
  capture "boundary-$theme"
  stop_app
done
# Large logical windows at fractional and high pixel density, both themes.
for scale in 1.5 2; do
  for theme in dark light; do
    start_app "$scale" "$theme"
    read -r width height < <(python3 - "$scale" <<'PY'
import sys
s=float(sys.argv[1]); print(int(1440*s), int(900*s))
PY
)
    xdotool windowsize "$window" "$width" "$height"
    sleep 6
    capture "app-$theme-${scale}x"
    stop_app
  done
done
printf 'Native review workflows passed.\n'
REVIEW
