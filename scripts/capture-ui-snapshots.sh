#!/usr/bin/env bash
# Native 1x/2x captures. Never synthesize a density by resizing an image.
set -euo pipefail
[[ $# == 3 && "$(uname -s)" == Darwin ]] || { echo 'usage (macOS): capture-ui-snapshots.sh BINARY OUT SPEC' >&2; exit 2; }
binary="$1"
out="$2"
spec="$3"
helper="$(cd "$(dirname "$0")" && pwd)/ui-snapshots/window.swift"
work="$(mktemp -d)"
app_pid=''
cleanup() {
  if [[ -n "$app_pid" ]]; then kill "$app_pid" 2>/dev/null || true; wait "$app_pid" 2>/dev/null || true; fi
  rm -rf "$work" # Only the temporary directory created by this invocation.
}
trap cleanup EXIT
export LANG=en_US.UTF-8 TZ=UTC
swiftc "$helper" -o "$work/window"
while IFS=$'\t' read -r name title width height args; do
  for scale in 1 2; do
    export VOXELITE_DATA_DIR="$work/$name-${scale}x"
    mkdir -p "$VOXELITE_DATA_DIR"
    (
      cd "$VOXELITE_DATA_DIR"
      # Registry-owned arguments, never user input.
      # shellcheck disable=SC2086
      exec "$binary" $args --exit-after 300 --session-socket ./session.sock
    ) >"$out/$name-${scale}x.log" 2>&1 &
    app_pid=$!
    window_id="$("$work/window" "$app_pid" "$title" "$width" "$height" "$scale")"
    screencapture -x -o -l "$window_id" "$out/$name-${scale}x.png"
    kill "$app_pid" 2>/dev/null || true
    wait "$app_pid" 2>/dev/null || true
    app_pid=''
  done
done < "$spec"
