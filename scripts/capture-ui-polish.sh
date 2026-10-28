#!/usr/bin/env bash
# Real Linux UI flows. BIN/OUT allow the same capture against the baseline build.
# The explicit Linux-only status fixture hook covers otherwise unreachable i32
# extremes through the real StatusBar render path. Normal runs never enable it.
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ -z "${BIN:-}" ]]; then
  BIN="$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"] + "/debug/quantum")')"
fi
BIN="$(realpath "$BIN")"
mkdir -p "${OUT:-/workspace/ui-polish-shots/after}"
OUT="$(realpath "${OUT:-/workspace/ui-polish-shots/after}")"
export BIN OUT
export VK_ICD_FILENAMES="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
xvfb-run -a -s '-screen 0 1600x1000x24 -nolisten tcp' dbus-run-session -- bash <<'CAPTURE'
set -euo pipefail
export XDG_RUNTIME_DIR
XDG_RUNTIME_DIR="$(mktemp -d)"
chmod 700 "$XDG_RUNTIME_DIR"
export QUANTUM_DATA_DIR="$XDG_RUNTIME_DIR/data" XDG_CURRENT_DESKTOP=GNOME
export XDG_CACHE_HOME="$XDG_RUNTIME_DIR/cache" XDG_CONFIG_HOME="$XDG_RUNTIME_DIR/config" GSETTINGS_BACKEND=memory
export QUANTUM_CAPTURE_STATUS=1
unset WAYLAND_DISPLAY
app_pid=''
wm_pid=''
cleanup() {
  if [[ -n "$app_pid" ]]; then kill "$app_pid" 2>/dev/null || true; wait "$app_pid" 2>/dev/null || true; fi
  if [[ -n "$wm_pid" ]]; then kill "$wm_pid" 2>/dev/null || true; fi
  rm -rf "$XDG_RUNTIME_DIR"
}
trap cleanup EXIT
# A real window manager lets GPUI complete the EWMH fullscreen transition.
if command -v xfwm4 >/dev/null; then
  xfwm4 --compositor=off --sm-client-disable >"$OUT/window-manager.log" 2>&1 &
  wm_pid=$!
  sleep 1
fi
"$BIN" --example --theme dark --exit-after 240 --session-socket "$XDG_RUNTIME_DIR/app.sock" >"$OUT/app.log" 2>&1 &
app_pid=$!
window=''
for attempt in {1..150}; do
  window="$(xdotool search --all --onlyvisible --pid "$app_pid" --name 'Autumn Workshop.*Quantum' 2>/dev/null | head -1 || true)"
  [[ -n "$window" ]] && break
  kill -0 "$app_pid"
  sleep 0.2
done
[[ -n "$window" ]] || { tail -60 "$OUT/app.log" >&2; exit 1; }
xdotool windowmove "$window" 0 0 windowsize "$window" 1440 900 windowfocus "$window"
xdotool mousemove --window "$window" 700 60
sleep 6
shot() {
  sleep 0.7
  timeout 15s import -window "${2:-$window}" "$OUT/$1.png"
}
shot full-window || true
convert "$OUT/full-window.png" -crop 1440x28+0+872 +repage -resize 200% "$OUT/statusbar.png" || true
xdotool windowsize "$window" 1000 700
sleep 1
shot compact-window || true
xdotool windowsize "$window" 1440 900
sleep 2
# Real pointer samples are useful in both binaries.
for sample in '420 420' '550 480' '690 580'; do
  read -r x y <<<"$sample"
  xdotool mousemove --window "$window" "$x" "$y"
  shot "hover-$x" || true
  convert "$OUT/hover-$x.png" -crop 1440x28+0+872 +repage "$OUT/hover-$x.png" || true
done
xdotool mousemove --window "$window" 700 60
for name in positive negative three-digit wide placeholder; do
  xdotool key --clearmodifiers ctrl+shift+F12
  shot "status-$name" || true
  convert "$OUT/status-$name.png" -crop 1440x28+0+872 +repage -resize 200% "$OUT/status-$name.png" || true
done
montage "$OUT/status-positive.png" "$OUT/status-negative.png" "$OUT/status-three-digit.png" "$OUT/status-wide.png" "$OUT/status-placeholder.png" -tile 1x -geometry +0+4 -background '#202020' "$OUT/statusbar-stability.png" || true
# Compare the painted vertical separator columns, independent of the text.
python3 - "$OUT" <<'VERIFY' || true
from pathlib import Path
from PIL import Image
import json, sys
out = Path(sys.argv[1])
columns = {}
for name in ("positive", "negative", "three-digit", "wide", "placeholder"):
    im = Image.open(out / f"status-{name}.png").convert("RGB")
    xs = []
    for x in range(im.width):
        c = im.getpixel((x, 25))
        if 10 < c[0] < 60 and c[0] == c[1] == c[2] and all(im.getpixel((x, y)) == c for y in range(20, 35)):
            xs.append(x)
    columns[name] = xs
(out / "statusbar-separators.json").write_text(json.dumps(columns, indent=2) + "\n")
assert columns["positive"] and all(xs == columns["positive"] for xs in columns.values())
VERIFY
xdotool key --clearmodifiers Escape ctrl+k
xdotool type --clearmodifiers --delay 70 'fullscreen'
shot command-palette-fullscreen || true
xdotool key --clearmodifiers ctrl+a
shot textfield-command-selection
xdotool key --clearmodifiers Return
sleep 1
xprop -id "$window" _NET_WM_STATE >"$OUT/fullscreen-state.txt" || true
xdotool key --clearmodifiers Escape ctrl+k
xdotool type --clearmodifiers --delay 70 'fullscreen'
shot command-palette-fullscreen-exit || true
xdotool key --clearmodifiers Escape F11
sleep 1
xdotool windowsize "$window" 1440 900 windowmove "$window" 0 0
xdotool key --clearmodifiers Escape
# View menu is at the same location in both shells.
xdotool mousemove --window "$window" 238 16 click 1
xdotool mousemove --window "$window" 700 60
shot view-menu || true
xdotool key --clearmodifiers Escape ctrl+k
xdotool type --clearmodifiers --delay 70 'about'
xdotool key --clearmodifiers Return
shot about || true
# Tab focus should enter the small modal, never the controls behind it.
xdotool key --clearmodifiers Tab
shot about-focus || true
xdotool mousemove --window "$window" 780 614 click 1
xdotool mousemove --window "$window" 700 60
shot acknowledgements || true
xdotool key --clearmodifiers Escape ctrl+comma
settings=''
for attempt in {1..50}; do
  settings="$(xdotool search --all --onlyvisible --pid "$app_pid" --name 'Quantum Settings' 2>/dev/null | head -1 || true)"
  [[ -n "$settings" ]] && break
  sleep 0.2
done
if [[ -n "$settings" ]]; then
  xdotool windowfocus "$settings"
  xdotool mousemove --window "$settings" 100 90 click 1
  xdotool type --clearmodifiers --delay 70 'updates'
  sleep 0.5
  xdotool mousemove --window "$settings" 100 134 click 1
  xdotool mousemove 1500 980
  shot settings-updates "$settings" || true
  xdotool mousemove --window "$settings" 100 90 click 1
  # Return through keyboard focus: the field must be a Tab stop and select all.
  xdotool key --clearmodifiers shift+Tab
  sleep 0.3
  xdotool key --clearmodifiers Tab
  shot textfield-settings-selection "$settings"
else
  printf '%s\n' 'Settings did not open; inspect app.log (the baseline may panic).' >"$OUT/settings-unavailable.txt"
fi
CAPTURE
