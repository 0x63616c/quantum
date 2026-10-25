#!/usr/bin/env bash
# Capture actual GPUI windows with the Mesa CPU Vulkan driver.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
cargo build -p voxelite-app --bin voxelite
mkdir -p screenshots
export VOXELITE_CAPTURE_ROOT="$PWD"
export VK_ICD_FILENAMES="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
xvfb-run -a -s '-screen 0 1440x900x24 -nolisten tcp' dbus-run-session -- bash <<'CAPTURE'
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
for theme in dark light; do
  "$VOXELITE_CAPTURE_ROOT/target/debug/voxelite" --theme "$theme" --exit-after 60 >"$XDG_RUNTIME_DIR/$theme.log" 2>&1 &
  app_pid=$!
  window=''
  for attempt in {1..50}; do
    window="$(xdotool search --name 'Autumn Workshop.*Voxelite' 2>/dev/null | head -1 || true)"
    [[ -n "$window" ]] && break
    kill -0 "$app_pid"
    sleep 0.2
  done
  [[ -n "$window" ]] || { cat "$XDG_RUNTIME_DIR/$theme.log"; exit 1; }
  xdotool windowmove "$window" 0 0 windowsize "$window" 1440 900
  xdotool mousemove 1439 899
  # Allow the full-quality CPU render and compositor upload to finish.
  sleep 6
  kill -0 "$app_pid"
  import -window "$window" "$VOXELITE_CAPTURE_ROOT/screenshots/app-$theme.png"
  xdotool windowsize "$window" 1024 640
  sleep 6
  import -window "$window" "$VOXELITE_CAPTURE_ROOT/screenshots/app-$theme-small.png"
  kill "$app_pid"
  wait "$app_pid" || true
  app_pid=''
done
CAPTURE
"$VOXELITE_CAPTURE_ROOT/target/debug/voxelite" --viewport-png screenshots/viewport.png
