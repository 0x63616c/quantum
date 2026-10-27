#!/usr/bin/env bash
# Capture real dialogs and a toolbar tooltip through pointer/key flows in old and new builds.
set -euo pipefail
cd "$(dirname "$0")/.."
export BIN OUT
if [[ -z "${BIN:-}" ]]; then
  BIN="$(cargo metadata --no-deps --format-version 1 | python3 -c \
    'import json, sys; print(json.load(sys.stdin)["target_directory"] + "/debug/voxelite")')"
fi
BIN="$(realpath "$BIN")"
mkdir -p "${OUT:-screenshots/dialogs}"
OUT="$(cd "${OUT:-screenshots/dialogs}" && pwd)"
[[ -x "$BIN" ]] || { echo "Not an executable: $BIN" >&2; exit 1; }
export VK_ICD_FILENAMES="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
xvfb-run -a -s '-screen 0 1440x900x24 -nolisten tcp' dbus-run-session -- bash <<'CAPTURE'
set -euo pipefail
export XDG_RUNTIME_DIR
XDG_RUNTIME_DIR="$(mktemp -d)"
chmod 700 "$XDG_RUNTIME_DIR"
export XDG_CURRENT_DESKTOP=GNOME
export VOXELITE_DATA_DIR="$XDG_RUNTIME_DIR/data"
unset WAYLAND_DISPLAY
app_pid=''
window=''
cleanup() {
  result=$?
  if [[ "$result" != 0 ]]; then
    tail -80 "$XDG_RUNTIME_DIR/app.log" >&2 || true
    if [[ -n "$window" ]]; then
      timeout 15s import -window "$window" "$OUT/failure.png" || true
    fi
  fi
  if [[ -n "$app_pid" ]]; then
    kill "$app_pid" 2>/dev/null || true
    wait "$app_pid" 2>/dev/null || true
  fi
  rm -rf "$XDG_RUNTIME_DIR"
}
trap cleanup EXIT
"$BIN" --theme dark --exit-after 300 --session-socket "$XDG_RUNTIME_DIR/app.sock" >"$XDG_RUNTIME_DIR/app.log" 2>&1 &
app_pid=$!
for attempt in {1..100}; do
  window="$(xdotool search --all --onlyvisible --pid "$app_pid" --name 'Autumn Workshop.*Voxelite' 2>/dev/null | head -1 || true)"
  [[ -n "$window" ]] && break
  kill -0 "$app_pid"
  sleep 0.2
done
[[ -n "$window" ]] || { echo 'Voxelite window did not appear' >&2; exit 1; }
xdotool windowmove "$window" 0 0 windowsize "$window" 1440 900 windowfocus "$window"
xdotool mousemove --window "$window" 700 60
# Wait for the full-quality software render, which gates viewport edits.
sleep 6
capture() {
  sleep 0.8
  kill -0 "$app_pid"
  timeout 15s import -window "$window" "$OUT/$1.png"
}
# Paint is the third tool in the left rail in both builds. Hover without
# clicking, then keep the button and the complete tooltip in the same crop.
xdotool mousemove --window "$window" 27 210
sleep 1.5
capture tooltip
convert "$OUT/tooltip.png" -crop 300x100+0+160 +repage "$OUT/tooltip.png"
# Bottom-left keyboard guide (view-help), to the right of the Settings cog.
xdotool mousemove --window "$window" 136 836 click 1
xdotool mousemove --window "$window" 700 60
capture help
xdotool key --clearmodifiers Escape ctrl+k
xdotool type --clearmodifiers --delay 100 'ip'
capture palette-ip
xdotool key --clearmodifiers Escape ctrl+k
xdotool type --clearmodifiers --delay 100 'zzzz'
capture palette-empty
xdotool key --clearmodifiers Escape b
# Place beside a visible voxel; retry while a slow CPU render finishes.
for attempt in {1..20}; do
  xdotool mousemove --window "$window" 550 480 click 1
  sleep 1
  [[ "$(xdotool getwindowname "$window")" == *"•"* ]] && break
done
[[ "$(xdotool getwindowname "$window")" == *"•"* ]] || { echo 'Viewport edit did not dirty the project' >&2; exit 1; }
xdotool mousemove --window "$window" 700 60
xdotool key --clearmodifiers ctrl+n
capture save
CAPTURE
