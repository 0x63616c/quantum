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
  # Stay inside either window size so the previous control receives mouse-out.
  xdotool mousemove 80 200
  sleep 0.5
  kill -0 "$app_pid"
  timeout 15s import -window "$window" "$artifact/$1-$theme.png"
}
xdotool windowsize "$window" 1440 900 windowfocus "$window"
sleep 3
capture_feature main
# Pointer selection and keyboard reset share the same tool/mirror actions.
xdotool mousemove 28 211 click 1
xdotool mousemove 28 395 click 1
xdotool mousemove 28 435 click 1
xdotool mousemove 28 475 click 1
capture_feature tools
xdotool key b shift+x shift+y shift+z
search 'manage palettes'
# Presets stay immutable; duplicate PICO-8 and author a custom brush palette.
xdotool mousemove 420 307 click 1
xdotool mousemove 590 298 click 1
sleep 0.3
xdotool mousemove 700 251 click 1
sleep 0.25
xdotool key ctrl+a
xdotool type --clearmodifiers --delay 15 'Studio warm'
# Reversing a Shift selection must contract it, then insert at the caret.
xdotool key shift+Left shift+Right
xdotool type --clearmodifiers 'x'
xdotool key BackSpace
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
xdotool mousemove 943 478 click 1
sleep 0.2
xdotool click 1
sleep 0.2
python3 - "$VOXELITE_DATA_DIR/palettes.json" <<'PYTEST'
import json, sys
p = json.load(open(sys.argv[1]))['entries'][0]['palette']
assert p['slots'][2]['rgba'] == [204, 136, 68, 255], p
PYTEST
xdotool mousemove 825 478 click 1
sleep 0.2
xdotool click 1
sleep 0.3
capture_feature palettes
xdotool key Escape
# Populate live history via the actual stdio bridge, not seeded UI data.
timeout 30s python3 scripts/lib/capture-mcp.py "$XDG_RUNTIME_DIR/$theme.sock" seed > "$artifact/mcp-$theme.json"
sleep 4
capture_feature agent
xdotool windowsize "$window" 1024 640
sleep 3
capture_feature agent-small
# Reach the lower sidebar at the minimum window height.
xdotool mousemove 950 570 click --repeat 12 --delay 30 5
sleep 0.4
capture_feature agent-small-scrolled
xdotool mousemove 950 570 click --repeat 20 --delay 30 4
xdotool windowsize "$window" 1440 900
sleep 3
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
xdotool windowsize "$window" 1024 640
sleep 3
capture_feature comments-small
# The message viewport scrolls independently of the pinned reply controls.
xdotool mousemove 850 390 click --repeat 8 --delay 30 5
sleep 0.4
capture_feature comments-small-scrolled
xdotool windowsize "$window" 1440 900
sleep 3
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
xdotool windowsize "$window" 1024 640
sleep 3
capture_feature history-small
xdotool windowsize "$window" 1440 900
sleep 3
# Initial preview must not display later comment pins.
xdotool mousemove 1109 318 click 1
sleep 2
# Tab must not focus the hidden named-version field during preview. Right/Left
# must still navigate history; inspect the intermediate keyboard capture.
xdotool key Tab Right
sleep 2
capture_feature history-keyboard
xdotool key Left
sleep 2
capture_feature history-initial
xdotool key Escape

# Read through a new MCP connection to verify the UI edits and preview isolation.
timeout 30s python3 scripts/lib/capture-mcp.py "$XDG_RUNTIME_DIR/$theme.sock" verify > "$artifact/ui-$theme.json"
