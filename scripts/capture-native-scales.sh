#!/usr/bin/env bash
# Additional native density checks; preserves logical 1440x900 layout.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
cargo build -p quantum-app
export QUANTUM_CAPTURE_ROOT="$PWD"
export VK_ICD_FILENAMES="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
xvfb-run -a -s '-screen 0 2880x1800x24 -nolisten tcp' dbus-run-session -- bash <<'CAPTURE'
set -euo pipefail
export XDG_RUNTIME_DIR
XDG_RUNTIME_DIR="$(mktemp -d)"
chmod 700 "$XDG_RUNTIME_DIR"
export XDG_CURRENT_DESKTOP=GNOME
unset WAYLAND_DISPLAY
app_pid=''
cleanup() {
  [[ -z "$app_pid" ]] || kill "$app_pid" 2>/dev/null || true
  rm -rf "$XDG_RUNTIME_DIR"
}
trap cleanup EXIT
mkdir -p screenshots/polish-2
for density in 150 200; do
  if [[ "$density" == 150 ]]; then
    export GPUI_X11_SCALE_FACTOR=1.5
    width=2160 height=1350
  else
    export GPUI_X11_SCALE_FACTOR=2
    width=2880 height=1800
  fi
  for theme in dark light; do
    export QUANTUM_DATA_DIR="$XDG_RUNTIME_DIR/$density-$theme"
    target/debug/quantum --example --theme "$theme" --exit-after 60 >"$XDG_RUNTIME_DIR/app.log" 2>&1 &
    app_pid=$!
    window=''
    for attempt in {1..100}; do
      window="$(xdotool search --all --onlyvisible --pid "$app_pid" --name '.*Quantum$' 2>/dev/null | head -1 || true)"
      [[ -z "$window" ]] || break
      kill -0 "$app_pid"
      sleep 0.1
    done
    [[ -n "$window" ]] || { cat "$XDG_RUNTIME_DIR/app.log"; exit 1; }
    xdotool windowmove "$window" 0 0 windowsize "$window" "$width" "$height"
    xdotool mousemove 2879 1799
    sleep 6
    kill -0 "$app_pid"
    timeout 15s import -window "$window" "screenshots/polish-2/main-$theme-$density.png"
    kill "$app_pid"
    wait "$app_pid" || true
    app_pid=''
  done
done
CAPTURE
