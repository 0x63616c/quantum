# Sourced inside capture-native.sh's Xvfb session. Real GPUI pointer/key events.
xdotool windowsize "$window" 1440 900 windowfocus "$window"
sleep 3
capture_ui2() {
  sleep 0.6
  kill -0 "$app_pid"
  timeout 15s import -window "$window" "$VOXELITE_CAPTURE_DIR/$1-$theme.png"
}
xdotool mousemove --window "$window" 700 60
capture_ui2 main
# Hover opposite corners of Fill. Both captures must keep the tooltip fixed.
xdotool mousemove --window "$window" 16 240
sleep 1
capture_ui2 tooltip-tool-a
xdotool mousemove --window "$window" 700 60
sleep 0.3
xdotool mousemove --window "$window" 40 264
sleep 1
capture_ui2 tooltip-tool-b
convert "$VOXELITE_CAPTURE_DIR/tooltip-tool-b-$theme.png" -crop 220x80+4+215 +repage -filter point -resize 400% "$VOXELITE_CAPTURE_DIR/keycap-tooltip-$theme.png"
xdotool mousemove --window "$window" 700 60
capture_ui2 keycap-search-source
convert "$VOXELITE_CAPTURE_DIR/keycap-search-source-$theme.png" -crop 280x64+1160+32 +repage -filter point -resize 400% "$VOXELITE_CAPTURE_DIR/keycap-search-$theme.png"
# Below the top toolbar / above the bottom controls.
xdotool mousemove --window "$window" 1370 64
sleep 1
capture_ui2 tooltip-top
xdotool mousemove --window "$window" 1020 836
sleep 1
capture_ui2 tooltip-bottom
# Pressed controls expose a distinct state without moving their layout.
xdotool mousemove --window "$window" 28 252 mousedown 1
capture_ui2 tool-pressed
xdotool mouseup 1
xdotool key b
# Gizmo positive X, negative X, orbit from empty space, drag out, double-click.
xdotool mousemove --window "$window" 1047 180
sleep 0.2
capture_ui2 gizmo-hover
convert "$VOXELITE_CAPTURE_DIR/gizmo-hover-$theme.png" -crop 128x128+956+100 +repage -filter point -resize 400% "$VOXELITE_CAPTURE_DIR/gizmo-zoom-$theme.png"
xdotool click 1
sleep 5
capture_ui2 gizmo-right
xdotool key f
sleep 5
xdotool mousemove --window "$window" 993 148 click 1
sleep 5
capture_ui2 gizmo-left
xdotool key f
sleep 5
xdotool mousemove --window "$window" 1020 164 mousedown 1
xdotool mousemove --window "$window" 1035 175
sleep 0.1
xdotool mousemove --window "$window" 1080 220
sleep 0.1
xdotool mousemove --window "$window" 1120 250
capture_ui2 gizmo-dragging
xdotool mouseup 1
sleep 5
capture_ui2 gizmo-orbited
xdotool mousemove --window "$window" 1020 164 click --repeat 2 --delay 120 1
sleep 5
capture_ui2 gizmo-reset
# Status telemetry tooltip anchors above its source.
xdotool mousemove --window "$window" 1400 886
sleep 1
capture_ui2 tooltip-status
xdotool mousemove --window "$window" 700 60
# Command palette, Comments and settings still use live controls.
xdotool key --clearmodifiers ctrl+k
xdotool type --clearmodifiers 'save'
capture_ui2 commands
xdotool key Escape
# Native in-window menu shortcuts (the Mac uses its system menu bar).
xdotool mousemove --window "$window" 120 16 click 1
capture_ui2 menu
xdotool key Escape
# Tab traverses the rail, and Enter selects without editing the model.
xdotool key Tab Tab Tab Tab Tab Tab Tab
capture_ui2 focus-tool
xdotool key Return
[[ "$(xdotool getwindowname "$window")" != *"•"* ]]
xdotool key Escape
xdotool mousemove --window "$window" 1250 191 click 1
capture_ui2 comments
# Open the real thread and post through keyboard focus, then undo the reply.
xdotool mousemove --window "$window" 1240 440 click 1
sleep 0.5
capture_ui2 comments-thread
xdotool mousemove --window "$window" 1240 780 click 1
xdotool type --clearmodifiers 'Checked the axis views.'
xdotool key Tab
capture_ui2 focus-comment-send
xdotool key Return
sleep 1
[[ "$(xdotool getwindowname "$window")" == *"•"* ]]
capture_ui2 comments-reply
xdotool key Escape ctrl+z
sleep 1
[[ "$(xdotool getwindowname "$window")" != *"•"* ]]
xdotool mousemove --window "$window" 1145 191 click 1
xdotool key --clearmodifiers ctrl+comma
settings_window=''
for attempt in {1..50}; do
  settings_window="$(xdotool search --all --onlyvisible --pid "$app_pid" --name '^Voxelite Settings$' 2>/dev/null | head -1 || true)"
  [[ -n "$settings_window" ]] && break
  sleep 0.2
done
[[ -n "$settings_window" ]]
xdotool windowfocus "$settings_window"
sleep 1
timeout 15s import -window "$settings_window" "$VOXELITE_CAPTURE_DIR/settings-general-$theme.png"
xdotool mousemove --window "$settings_window" 100 207 click 1
sleep 0.5
timeout 15s import -window "$settings_window" "$VOXELITE_CAPTURE_DIR/settings-appearance-$theme.png"
xdotool mousemove --window "$settings_window" 100 285 click 1
sleep 0.5
timeout 15s import -window "$settings_window" "$VOXELITE_CAPTURE_DIR/settings-viewport-$theme.png"
xdotool mousemove --window "$settings_window" 100 323 click 1
sleep 0.5
timeout 15s import -window "$settings_window" "$VOXELITE_CAPTURE_DIR/settings-shortcuts-$theme.png"
xdotool key Escape
xdotool windowfocus "$window"

# Compare the actual captured pixels, including hover entry at different corners.
python3 - "$VOXELITE_CAPTURE_DIR" "$theme" <<'PYTEST'
from pathlib import Path
from PIL import Image, ImageChops
import os, sys
root, theme = Path(sys.argv[1]), sys.argv[2]
def image(name):
    return Image.open(root / f"{name}-{theme}.png").convert("RGB")
def same(a, b, crop):
    return ImageChops.difference(image(a).crop(crop), image(b).crop(crop)).getbbox() is None
assert same("tooltip-tool-a", "tooltip-tool-b", (48, 218, 230, 287)), "tooltip moved with the pointer"
art = (120, 220, 940, 775)
assert not same("main", "gizmo-right", art), "positive axis did not snap"
assert not same("gizmo-right", "gizmo-left", art), "negative axis did not show the opposite view"
assert not same("main", "gizmo-orbited", art), "gizmo drag did not orbit"
assert same("main", "gizmo-reset", art), "double-click did not reset to isometric"
if os.environ.get("VOXELITE_CAPTURE_MAC_UI"):
    source = image("keycap-search-source")
    columns = {}
    for x in range(1382, 1401):
        rows = [y for y in range(56, 70) if
                (source.getpixel((x, y))[0] > 95 if theme == "dark" else
                 source.getpixel((x, y))[0] < 160)]
        if rows: columns[x] = rows
    groups = []
    for x, rows in columns.items():
        if not groups or x > groups[-1][-1][0] + 1: groups.append([])
        groups[-1].append((x, rows))
    assert len(groups) == 2, "expected separate Command and K glyphs"
    boxes = [(g[0][0], min(min(y) for _, y in g), g[-1][0] + 1,
              max(max(y) for _, y in g) + 1) for g in groups]
    assert abs((boxes[0][3] - boxes[0][1]) - (boxes[1][3] - boxes[1][1])) <= 1
    assert 1 <= boxes[1][0] - boxes[0][2] <= 2, "modifier spacing is not compact"
    print(f"{theme}: Command/K ink bounds {boxes}")
print(f"{theme}: anchored tooltips, opposite axis snaps, drag and double-click reset verified")
PYTEST
