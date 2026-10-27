# Sourced by capture-native.sh. Exercise the real settings window under Xvfb.
xdotool windowsize "$window" 1440 900 windowfocus "$window"
# The always-visible cog opens Settings. The shortcut is checked below.
xdotool mousemove --window "$window" 96 836 click 1
settings_window=''
for attempt in {1..50}; do
  settings_window="$(xdotool search --all --onlyvisible --pid "$app_pid" --name '^Quantum Settings$' 2>/dev/null | head -1 || true)"
  [[ -n "$settings_window" ]] && break
  sleep 0.2
done
[[ -n "$settings_window" ]] || { echo 'Settings window did not open' >&2; exit 1; }
xdotool windowfocus "$settings_window"
sleep 2
capture_settings() {
  xdotool mousemove --window "$settings_window" 870 630
  sleep 0.4
  kill -0 "$app_pid"
  timeout 15s import -window "$settings_window" "$VOXELITE_CAPTURE_DIR/settings-$1-$theme.png"
}
capture_settings general
# One settings window per editor, including repeated shortcut invocation.
xdotool key --clearmodifiers ctrl+comma
[[ "$(xdotool search --all --onlyvisible --pid "$app_pid" --name '^Quantum Settings$' | wc -l)" == 1 ]]
# Search supports text selection and native clipboard actions.
xdotool mousemove --window "$settings_window" 110 90 click 1
xdotool type --clearmodifiers 'appearance'
xdotool key --clearmodifiers ctrl+a ctrl+x ctrl+v
sleep 0.4
capture_settings search
# Individual matches are indented beneath their section and navigate on click.
xdotool key --clearmodifiers ctrl+a
xdotool type --clearmodifiers 'smooth edges'
capture_settings search-setting
xdotool mousemove --window "$settings_window" 110 165 click 1
xdotool mousemove --window "$settings_window" 816 246 click 1
sleep 0.4
python3 - "$VOXELITE_DATA_DIR/settings.json" <<'PYTEST'
import json, sys
assert json.load(open(sys.argv[1]))['antialiasing'] is False
PYTEST
xdotool mousemove --window "$settings_window" 816 246 click 1
capture_settings search-setting-selected
# Clear through the actual clear button, then exercise section history.
xdotool mousemove --window "$settings_window" 203 90 click 1
xdotool mousemove --window "$settings_window" 66 31 click 1
capture_settings history-back
xdotool mousemove --window "$settings_window" 98 31 click 1
capture_settings history-forward
xdotool mousemove --window "$settings_window" 34 31 click 1
capture_settings sidebar-hidden
xdotool mousemove --window "$settings_window" 34 31 click 1
xdotool mousemove --window "$settings_window" 110 207 click 1
capture_settings appearance
# Change the appearance, confirm persistence, then restore this capture's theme.
if [[ "$theme" == dark ]]; then opposite=light; opposite_x=555; else opposite=dark; opposite_x=744; fi
xdotool mousemove --window "$settings_window" "$opposite_x" 250 click 1
sleep 0.5
python3 - "$VOXELITE_DATA_DIR/settings.json" "$opposite" <<'PYTEST'
import json, sys
assert json.load(open(sys.argv[1]))['appearance'] == sys.argv[2]
PYTEST
capture_settings changed-appearance
xdotool mousemove --window "$settings_window" 370 250 click 1
sleep 0.3
python3 - "$VOXELITE_DATA_DIR/settings.json" <<'PYTEST'
import json, sys
assert json.load(open(sys.argv[1]))['appearance'] == 'system'
PYTEST
if [[ "$theme" == dark ]]; then restore_x=744; else restore_x=555; fi
xdotool mousemove --window "$settings_window" "$restore_x" 250 click 1
sleep 0.3
xdotool mousemove --window "$settings_window" 110 285 click 1
capture_settings viewport
# Toggle the grid; check this is a real persisted preference.
xdotool mousemove --window "$settings_window" 816 180 click 1
sleep 0.5
python3 - "$VOXELITE_DATA_DIR/settings.json" <<'PYTEST'
import json, sys
assert json.load(open(sys.argv[1]))['grid'] is False
PYTEST
xdotool mousemove --window "$settings_window" 816 180 click 1
# Hiding navigation help must leave the cog available.
xdotool mousemove --window "$settings_window" 110 169 click 1
xdotool mousemove --window "$settings_window" 816 177 click 1
sleep 0.4
python3 - "$VOXELITE_DATA_DIR/settings.json" <<'PYTEST'
import json, sys
assert json.load(open(sys.argv[1]))['navigation_hints'] is False
PYTEST
# Closing settings keeps the project window alive and allows reopening.
xdotool key --clearmodifiers Escape
sleep 0.5
xdotool windowfocus "$window"
timeout 15s import -window "$window" "$VOXELITE_CAPTURE_DIR/cog-without-guide-$theme.png"
xdotool mousemove --window "$window" 96 836 click 1
sleep 0.5
settings_window="$(xdotool search --all --onlyvisible --pid "$app_pid" --name '^Quantum Settings$' | head -1)"
xdotool windowfocus "$settings_window"
capture_settings reopened
xdotool mousemove --window "$settings_window" 816 177 click 1
xdotool key --clearmodifiers Escape
xdotool windowfocus "$window"
# Hover coordinates and an inclusive box preview in the bottom bar.
xdotool key u
xdotool mousemove --window "$window" 510 510 click 1
xdotool mousemove --window "$window" 560 540
sleep 1
import -window "$window" "$VOXELITE_CAPTURE_DIR/status-selection-$theme.png"
xdotool key Escape b
# Show Palette focuses the live widget; Enter must not paint into the document.
xdotool key --clearmodifiers ctrl+k
xdotool type --clearmodifiers 'show palette'
xdotool key Return
sleep 0.3
xdotool key Right Return
sleep 0.3
[[ "$(xdotool getwindowname "$window")" != *"•"* ]]
import -window "$window" "$VOXELITE_CAPTURE_DIR/palette-focused-$theme.png"
xdotool key Escape
