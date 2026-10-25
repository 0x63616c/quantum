# Sourced by capture-native.sh inside its Xvfb / D-Bus session.
artifact="$VOXELITE_CAPTURE_ROOT/screenshots/polish-2"
mkdir -p "$artifact"
search() {
  xdotool key --clearmodifiers ctrl+k
  xdotool type --clearmodifiers --delay 12 "$1"
  xdotool key Return
  sleep 0.4
}
capture_feature() {
  xdotool mousemove 1439 899
  sleep 0.5
  kill -0 "$app_pid"
  timeout 15s import -window "$window" "$artifact/$1-$theme.png"
}
xdotool windowsize "$window" 1440 900 windowfocus "$window"
sleep 3
capture_feature main
xdotool key p shift+x shift+z
capture_feature tools
xdotool key b shift+x shift+z
search 'manage palettes'
# Presets stay immutable; duplicate PICO-8 and author a custom brush palette.
xdotool mousemove 420 307 click 1
xdotool mousemove 590 298 click 1
sleep 0.3
xdotool mousemove 700 251 click 1
sleep 0.25
xdotool key ctrl+a
xdotool type --clearmodifiers --delay 15 'Studio warm'
sleep 0.25
xdotool mousemove 1040 251 click 1
sleep 0.25
xdotool mousemove 620 429 click 1
sleep 0.25
xdotool key ctrl+a
xdotool type --clearmodifiers --delay 15 '#CC8844'
sleep 0.25
xdotool mousemove 780 429 click 1
sleep 0.3
python3 - "$VOXELITE_DATA_DIR/palettes.json" <<'PYTEST'
import json, sys
p = json.load(open(sys.argv[1]))['entries'][0]
assert p['name'] == 'Studio warm', p
assert p['palette']['slots'][0]['rgba'] == [204, 136, 68, 255], p
assert len(p['palette']['slots']) == 16
PYTEST
capture_feature palettes
xdotool key Escape
search 'share project'
capture_feature share
xdotool key Escape
# Populate live history via the actual stdio bridge, not seeded UI data.
timeout 30s python3 scripts/lib/capture-mcp.py "$XDG_RUNTIME_DIR/$theme.sock" seed > "$artifact/mcp-$theme.json"
sleep 4
capture_feature agent
# Open the discussion panel and its first real thread.
xdotool mousemove 1260 193 click 1
sleep 0.3
xdotool mousemove 1230 425 click 1
sleep 0.4
# Reply with native text input. The field contains tool shortcut letters.
xdotool mousemove 1230 779 click 1
sleep 0.25
xdotool type --clearmodifiers --delay 12 'The warm highlights look balanced.'
xdotool mousemove 1260 831 click 1
sleep 3
capture_feature comments
xdotool key Escape
# Named versions and a draggable history preview.
xdotool mousemove 1370 193 click 1
sleep 0.4
xdotool mousemove 1230 829 click 1
sleep 0.25
xdotool type --clearmodifiers --delay 12 'Human review'
xdotool mousemove 1395 829 click 1
sleep 0.4
xdotool mousemove 1230 318 mousedown 1 mousemove 1190 318 mouseup 1
sleep 3
capture_feature history
xdotool key Escape

# Read through a new MCP connection to verify the UI edits and preview isolation.
timeout 30s python3 scripts/lib/capture-mcp.py "$XDG_RUNTIME_DIR/$theme.sock" verify > "$artifact/ui-$theme.json"
