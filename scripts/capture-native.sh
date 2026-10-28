#!/usr/bin/env bash
# Capture actual GPUI windows with the Mesa CPU Vulkan driver.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
cargo build -p quantum-app -p quantum-mcp
mkdir -p "${QUANTUM_CAPTURE_DIR:-screenshots}"
export QUANTUM_CAPTURE_DIR
QUANTUM_CAPTURE_DIR="$(cd "${QUANTUM_CAPTURE_DIR:-screenshots}" && pwd)"
export QUANTUM_CAPTURE_ROOT="$PWD"
export QUANTUM_CAPTURE_TARGET
QUANTUM_CAPTURE_TARGET="$(cd "${CARGO_TARGET_DIR:-target}" && pwd)"
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
  result=$?
  if [[ "$result" != 0 ]]; then
    for log in "$XDG_RUNTIME_DIR"/*.log; do
      [[ ! -f "$log" ]] || tail -80 "$log" >&2
    done
  fi
  if [[ "$result" != 0 && -n "${window:-}" ]]; then
    timeout 10s import -window "$window" /tmp/quantum-polish-2-capture-failure.png || true
  fi
  if [[ -n "$app_pid" ]]; then kill "$app_pid" 2>/dev/null || true; fi
  rm -rf "$XDG_RUNTIME_DIR"
}
trap cleanup EXIT
for theme in dark light; do
  export QUANTUM_DATA_DIR="$XDG_RUNTIME_DIR/data-$theme"
  # Software rendering and loaded CI runners need headroom; cleanup owns normal exit.
  "$QUANTUM_CAPTURE_TARGET/debug/quantum" --example --theme "$theme" --exit-after 900 --session-socket "$XDG_RUNTIME_DIR/$theme.sock" >"$XDG_RUNTIME_DIR/$theme.log" 2>&1 &
  app_pid=$!
  window=''
  for attempt in {1..50}; do
    window="$(xdotool search --all --onlyvisible --pid "$app_pid" --name 'Autumn Workshop.*Quantum' 2>/dev/null | head -1 || true)"
    [[ -n "$window" ]] && break
    kill -0 "$app_pid"
    sleep 0.2
  done
  [[ -n "$window" ]] || { cat "$XDG_RUNTIME_DIR/$theme.log"; exit 1; }
  xdotool windowmove "$window" 0 0 windowsize "$window" 1440 900
  xdotool mousemove 400 60
  # Allow the full-quality CPU render and compositor upload to finish.
  sleep 6
  kill -0 "$app_pid"
  timeout 15s import -window "$window" "$XDG_RUNTIME_DIR/capture.png"
  mv "$XDG_RUNTIME_DIR/capture.png" "$QUANTUM_CAPTURE_DIR/app-$theme.png"
  xdotool windowsize "$window" 1024 640
  sleep 6
  kill -0 "$app_pid"
  timeout 15s import -window "$window" "$XDG_RUNTIME_DIR/capture.png"
  mv "$XDG_RUNTIME_DIR/capture.png" "$QUANTUM_CAPTURE_DIR/app-$theme-small.png"
  if [[ "${QUANTUM_CAPTURE_UI2:-0}" == 1 ]]; then
    source scripts/lib/capture-ui2.sh
  elif [[ "${QUANTUM_CAPTURE_UI_ONLY:-0}" == 1 ]]; then
    source scripts/lib/capture-ui.sh
  else
    source scripts/lib/capture-polish-2.sh
  fi
  kill "$app_pid"
  wait "$app_pid" || true
  app_pid=''
done
CAPTURE
"$QUANTUM_CAPTURE_TARGET/debug/quantum" --viewport-png "$QUANTUM_CAPTURE_DIR/viewport.png"
