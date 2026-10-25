#!/usr/bin/env bash
# Real native keyboard/dialog regression checks; artifacts contain only fixture data.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
cargo build -p voxelite-app --bin voxelite
export VOXELITE_REVIEW_ROOT="$PWD"
export VK_ICD_FILENAMES="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
mkdir -p screenshots/review-1
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
artifact="$VOXELITE_REVIEW_ROOT/screenshots/review-1"
project="$XDG_RUNTIME_DIR/review.voxl"
cp examples/autumn-workshop.voxl "$project"
start_app() {
  GPUI_X11_SCALE_FACTOR="$1" target/debug/voxelite --theme "$2" --exit-after 180 "${@:3}" >"$XDG_RUNTIME_DIR/app.log" 2>&1 &
  app_pid=$!
  window=''
  for attempt in {1..100}; do
    window="$(xdotool search --name '.*Voxelite$' 2>/dev/null | head -1 || true)"
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
  import -window "$window" "$artifact/$1.png"
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
xdotool type --clearmodifiers --delay 5 "$project"
xdotool key Return
wait_title 'Autumn Workshop — Voxelite'
xdotool windowfocus "$window" key Right Return
wait_title 'Autumn Workshop • — Voxelite'
xdotool key ctrl+n Left Return
wait_title 'Autumn Workshop • — Voxelite'
xdotool key ctrl+n Left Left Return
wait_title 'Untitled — Voxelite'
stop_app
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
