# Sourced inside capture-polish-2.sh; exercise the real local Share backend.
search 'share project'
# Copy the actual URI and paste through native clipboard/text input. A temporary
# participant makes the exact clipboard contents observable in persisted state.
xdotool mousemove 1355 330 click 1
xdotool key Tab ctrl+v Return
sleep 0.4
python3 - "$VOXELITE_DATA_DIR/shares.json" <<'PY'
import json, sys
records = json.load(open(sys.argv[1]))
assert len(records) == 1, records
share = next(iter(records.values()))
assert share['people'][0]['name'] == 'voxelite://share/' + share['id'], share
assert share['access'] == 'view', share
PY
xdotool mousemove 1375 511 click 1
sleep 0.25
# Add participants through the visible field; single-letter tool shortcuts must
# remain suppressed while typing their names.
xdotool key Tab
xdotool type --clearmodifiers --delay 12 'Designer'
xdotool key Return
sleep 0.3
xdotool mousemove 1280 511 click 1
sleep 0.2
xdotool mousemove 1200 550 click 1
sleep 0.3
xdotool key Tab
xdotool type --clearmodifiers --delay 12 'Canopy agent'
xdotool key Return
sleep 0.3
xdotool mousemove 1280 551 click 1
sleep 0.2
xdotool mousemove 1330 590 click 1
sleep 0.3
xdotool mousemove 1330 244 click 1
sleep 0.3
python3 - "$VOXELITE_DATA_DIR/shares.json" <<'PY'
import json, sys
share = next(iter(json.load(open(sys.argv[1])).values()))
assert share['access'] == 'remix', share
assert [(p['name'], p['access']) for p in share['people']] == [
    ('Designer', 'comment'), ('Canopy agent', 'remix')], share
PY
# Dismiss and reopen to reload persisted settings; capture Copy feedback as well.
xdotool key Escape
search 'share project'
xdotool mousemove 1355 330 click 1
capture_feature share
cp "$artifact/share-$theme.png" "$VOXELITE_CAPTURE_ROOT/screenshots/share-$theme.png"
xdotool windowsize "$window" 1024 640
sleep 3
capture_feature share-small
xdotool mousemove 920 510 click --repeat 8 --delay 30 5
sleep 0.3
capture_feature share-small-scrolled
# Outside click restores editor focus even if a text field owned it previously.
xdotool key Tab
xdotool mousemove 200 150 click 1
xdotool key p b
xdotool windowsize "$window" 1440 900
sleep 3
