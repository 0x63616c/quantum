#!/usr/bin/env bash
# Capture PR screenshot screens from one built `voxelite` binary on a virtual X display.
# Called by `cargo xtask screenshots`; see .agents/skills/ui-pr-screenshots/SKILL.md.
#
# usage: capture-pr-screens.sh BINARY OUT_DIR SPEC_FILE
# SPEC_FILE has one tab-separated line per screen:
#   name <TAB> window-title-regex <TAB> width <TAB> height <TAB> app args (space separated)
# Rendering is deterministic: Mesa lavapipe (CPU Vulkan), a fresh data directory per
# screen (no recents or settings), the pointer parked outside the window, fixed size.
set -euo pipefail
[[ $# == 3 ]] || { echo "usage: $0 BINARY OUT_DIR SPEC_FILE" >&2; exit 2; }
for tool in xvfb-run xdotool import dbus-run-session; do
  command -v "$tool" >/dev/null || { echo "missing $tool (apt install xvfb xdotool imagemagick dbus mesa-vulkan-drivers)" >&2; exit 2; }
done
export PR_SHOTS_BIN PR_SHOTS_OUT PR_SHOTS_SPEC PR_SHOTS_ROOT
PR_SHOTS_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PR_SHOTS_BIN="$(realpath "$1")"
mkdir -p "$2"
PR_SHOTS_OUT="$(realpath "$2")"
PR_SHOTS_SPEC="$(realpath "$3")"
export VK_ICD_FILENAMES="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
export LANG=C.UTF-8 TZ=UTC
xvfb-run -a -s "-screen 0 ${PR_SHOTS_DISPLAY:-1600x1000x24} -nolisten tcp" dbus-run-session -- bash <<'CAPTURE' 2> >(grep -v -E 'xdg-desktop-portal|pw\.conf|PipeWire|dbus-daemon|^$' >&2)
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
while IFS=$'\t' read -r name title width height args; do
  [[ -z "$name" || "$name" == \#* ]] && continue
  export VOXELITE_DATA_DIR="$XDG_RUNTIME_DIR/data-$name"
  mkdir -p "$VOXELITE_DATA_DIR"
  if [[ "$name" == welcome-recents-* ]]; then
    python3 "$PR_SHOTS_ROOT/scripts/prepare-welcome-capture.py" \
      "$PR_SHOTS_ROOT" "$(dirname "$PR_SHOTS_OUT")/fixtures" "$VOXELITE_DATA_DIR"
  fi
  # shellcheck disable=SC2086 # args are a space-separated list by design
  "$PR_SHOTS_BIN" $args --exit-after 300 --session-socket "$XDG_RUNTIME_DIR/$name.sock" \
    >"$XDG_RUNTIME_DIR/$name.log" 2>&1 </dev/null &
  app_pid=$!
  window=''
  for _ in {1..150}; do
    window="$(xdotool search --all --onlyvisible --pid "$app_pid" --name "$title" 2>/dev/null | head -1 || true)"
    [[ -n "$window" ]] && break
    kill -0 "$app_pid" 2>/dev/null || break
    sleep 0.2
  done
  if [[ -z "$window" ]]; then
    echo "screen $name: no window titled /$title/ appeared" >&2
    tail -40 "$XDG_RUNTIME_DIR/$name.log" >&2 || true
    kill "$app_pid" 2>/dev/null || true
    app_pid=''
    continue
  fi
  xdotool windowmove "$window" 0 0 windowsize "$window" "$width" "$height" || true
  xdotool mousemove 0 9999
  # Let the CPU renderer finish full-quality frames and any entrance animation.
  sleep "${PR_SHOTS_SETTLE:-6}"
  import -window "$window" "$PR_SHOTS_OUT/$name.png"
  echo "captured $name"
  kill "$app_pid" 2>/dev/null || true
  wait "$app_pid" 2>/dev/null || true
  app_pid=''
done <"$PR_SHOTS_SPEC"
CAPTURE
