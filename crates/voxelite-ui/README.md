# UI component library

`voxelite-ui` is the shell's shared GPUI presentation library. It depends only on
GPUI for UI/runtime services (default features disabled) and the already locked
`unicode-segmentation` crate for the pure editor. It has no app or domain
dependencies. The application selects native platform support. Plugins
must not depend on this crate.

Use `tokens` for Theme colors, spacing, corner radii, Geist/Geist Mono typography,
control heights, focus rings and panel shadows. `Theme::surface`, `card`,
`surface_raised` and `separator` reproduce the grouped Settings appearance.

Use Button / IconButton for actions, ButtonGroup (Segmented) for related choices,
SidebarItem and SidebarSection for navigation, Card (Group) and FormRow for
settings, and Toolbar / ToolbarSeparator for editor chrome. Button's `build()`
exposes GPUI event, tooltip and accessibility hooks. Disabled buttons have no tab
stop; attach action listeners only while enabled. Keep domain dispatch in the app.

`FieldFrame` supplies field and search-row chrome. `TextField` owns editing,
as described below. Slider supports a gradient background, a value-label slot
and arrow/Home/End keys. Swatch supports selection and a lock badge. Toggle,
Badge (Pill) and Kbd cover small form and informational elements.

`Hsb` provides bounded sRGB ↔ HSB (HSV) conversion, byte rounding and achromatic
hue preservation. `rgb_hex` / `parse_rgb_hex` format and validate opaque colors.
`ColorSquare` paints the saturation/brightness plane at the supplied hue;
`HueSlider` paints the hue spectrum. Both are controlled, focusable inputs with
arrow keys (Shift for larger steps), accessible values and pointer tracking beyond
their bounds. `ColorChange::commit` is false during a drag and true on release or
keyboard changes. The owner supplies separate `ColorGesture`s, cancels them before
rolling back on Escape, and routes previews/commits through its domain. The square
takes its parent's width and has equal height; the hue track is 24 px high.

`SidebarItem::custom` provides the same selection, hover and accessibility frame
as a standard item, with caller-provided content such as a palette thumbnail.
`Menu::button`, `panel` and `item` provide consistent menu presentation. The owner
manages open state, arrow-key selection, activation, dismissal and focus. These
helpers perform no domain operations.

Dialog supplies a centered scrim, focusable panel, title and optional footer.
The caller owns Escape, open state and focus restoration; `trap_tab` keeps focus
inside the panel. Small About panels use the quiet scrim. Tooltips share the
`tooltip` frame, while the app retains placement and shortcut rendering.

StatusBar and StatusSegment reserve physical widths using Geist Mono's advance.
`fixed_slot` uses nonbreaking padding, an em-dash placeholder, and `+` on overflow;
coordinates use five left-aligned cells (sign plus four digits), with exact values
in a tooltip. Other segments remain right-aligned; dimensions and full box counts
have explicit tested cell budgets.
Keep telemetry in fixed slots and the message in a middle `flex_1` region.

## TextField

`TextField::new(label, cx)` creates the shared single-line entity. Its placeholder
is also its accessible label. The entity owns GPUI `EntityInputHandler`, native
UTF-16 conversion, clipboard, focus, blink/scroll tasks and painting. The
`editing` module has no GPUI imports: it owns grapheme/word boundaries, directional
selection, transactions, numeric filtering/parsing and scrub arithmetic. All text
mutation goes through `Editor`; callers read `text()`, use `set()` for model
replacement, and subscribe to `TextFieldEvent` for domain effects.

The app adapter re-exports the entity and routes native Edit-menu Undo/Redo to
its local actions. The inventory covers Settings search, command
palette/tool search, palette name/hex, RGB boxes beside sliders, comment drafts
and version naming. These all use this entity. There are no separate editable
status fields or other rename-dialog editors in this revision; status values are
read-only. OS file dialogs retain their platform-owned filename controls.

### Bindings

`editing::bindings` is the authoritative table used to install `TextFieldAction`
bindings in the `TextField` context. Tests list every chord and its action, check
GPUI parsing/context resolution, and exercise the pure commands. Here `⌫` means
Backspace, `⌦` means forward Delete, and Option is Alt on other keyboards.

| Operation | macOS | Linux / Windows |
| --- | --- | --- |
| Previous / next grapheme | ← / →; ⌃B / ⌃F | ← / → |
| Extend by grapheme | ⇧← / ⇧→; ⇧⌃B / ⇧⌃F | ⇧← / ⇧→ |
| Previous / next word | ⌥← / ⌥→ | Ctrl← / Ctrl→ |
| Extend by word | ⇧⌥← / ⇧⌥→ | Shift Ctrl← / Shift Ctrl→ |
| Line start / end | Home / End; ⌘← / ⌘→; ⌃A / ⌃E | Home / End; Ctrl Home / Ctrl End |
| Extend to line start / end | ⇧Home / ⇧End; ⇧⌘← / ⇧⌘→; ⇧⌃A / ⇧⌃E | Shift Home / Shift End; Shift Ctrl Home / Shift Ctrl End |
| Line start / end (text fields) | ↑ / ↓; ⌥↑ / ⌥↓ | ↑ / ↓; Alt↑ / Alt↓ |
| Extend to line start / end (text fields) | ⇧↑ / ⇧↓; ⇧⌥↑ / ⇧⌥↓ | Shift↑ / Shift↓; Shift Alt↑ / Shift Alt↓ |
| Delete previous / next grapheme | ⌫ / ⌦; ⌃H / ⌃D | Backspace / Delete |
| Delete previous / next word | ⌥⌫ / ⌥⌦ | Ctrl Backspace / Ctrl Delete |
| Delete to line start / end | ⌘⌫ / ⌘⌦; ⌃K for end | Select with Shift Home/End, then Delete |
| Cut / Copy / Paste / Select All | ⌘X / ⌘C / ⌘V / ⌘A | Ctrl X / C / V / A |
| Undo / Redo | ⌘Z / ⇧⌘Z | Ctrl Z / Shift Ctrl Z or Ctrl Y |
| Commit | Enter | Enter |
| Cancel according to field policy, then blur | Escape | Escape |
| Next / previous focus stop | Tab / ⇧Tab | Tab / Shift Tab |
| Editing context menu | Menu or ⇧F10; right-click | Menu or Shift F10; right-click |
| Numeric increment / decrement | ↑ / ↓ | ↑ / ↓ |
| Numeric coarse increment / decrement | ⇧↑ / ⇧↓ | Shift↑ / Shift↓ |
| Numeric fine increment / decrement | ⌥↑ / ⌥↓; ⇧⌥↑ / ⇧⌥↓ | Alt↑ / Alt↓; Shift Alt↑ / Shift Alt↓ |

Plain Left/Right collapse a selection to its corresponding edge, without an
extra step. Modified arrows move from the active end; Shift always preserves
the anchor. Deletion with any selected range deletes only that range. Unicode
word segmentation keeps combining sequences and emoji intact, uses individual
CJK boundaries, skips adjacent whitespace, and groups punctuation runs.
Transposition (⌃T) is not implemented. Character Viewer (⌃⌘Space), dead keys and
IME input remain with GPUI/platform input; the component does not synthesize
characters from key-down events. Marked composition is underlined.

### Focus, history and numeric policies

`EscapePolicy::Blur` keeps the current draft and blurs. `EscapePolicy::Revert`
restores the last value established by `set()` or Enter, then blurs. Hex and
numeric fields use Revert. Enter commits the draft and emits `Committed`. Ordinary
text blur retains its draft without establishing a new commit; numeric blur
validates, formats/clamps a valid value or restores the last committed value.
The shell connects Enter to command invocation, custom palette name/color apply,
named-version save and comment submission. A commit validator rejects invalid
hex without losing the previous Escape baseline; domain validation stays in
the app adapter.
The first Escape is consumed by the field. The next belongs to the surrounding
dialog (an open context menu consumes Escape to close itself first).

`focus_policy(scope, return_focus)` supplies a modal Tab scope and Escape focus
destination. Tab/Shift-Tab follow GPUI's rendered focus order, including other
controls; modal traversal wraps inside the supplied scope. Keyboard/programmatic
focus selects all. Pointer focus places the caret, Shift-click extends,
double-click selects a word, triple-click selects the line, and double-click
drag extends by whole words. Dragging outside an overflowing field autoscrolls
from the current window pointer until release. Outside clicks blur.

The command palette uses the same text-editing bindings, including line-end
Up/Down while its field is focused. After Escape blurs the input, Up/Down navigate
results and Enter invokes the selected command; results can also be clicked.

Undo and redo are **always consumed while a field has focus**, even with empty
history. Empty history is a no-op, never a fallback to document undo. Adjacent
typing within 750 ms coalesces; movement, selection, clipboard operations, commit
and focus changes break the run. Each paste and boundary deletion is one step;
a composition or numeric scrub is one step. History retains at most 100
transactions, and the line is limited to 8192 UTF-8 bytes. Paste normalizes
CRLF/newlines, tabs and Unicode line separators to spaces and drops other control
characters. New edits invalidate redo. Model `set()` establishes a fresh history.

`NumericField` is a controlled wrapper around the same persistent entity, using
a stable element ID. `NumericPolicy` defines bounds, decimal permission and the
fine step. Up/Down step by 1, Shift by 10, and Option by the fine step (including
when Shift is also held). Integer RGB fields use a fine step of 1; decimal
fields usually use 0.1. All steps clamp and quantize. Only digits, a leading sign
and one permitted decimal point are accepted; paste filters against the whole
candidate. Incomplete prefixes such as `-` can be typed but revert on blur.
Option-drag anywhere in a numeric field scrubs horizontally, four logical pixels
per fine step; Shift multiplies scrub speed by ten. The cursor changes from
I-beam to horizontal resize for the gesture.

The caret resets and stays visible while editing, then blinks every 500 ms; its
task stops on blur/window deactivation. There is no reduced-motion setting in
this app revision. Window deactivation preserves the draft and selection and
does not commit numeric input. Selection uses `Theme::text_selection`, and field/search
focus uses token rings. The adapter exposes GPUI `TextInput` role, label,
placeholder and value. Platform screen-reader announcements, native macOS IMEs,
Character Viewer and NSTextInputClient edge cases still need macOS validation;
pure UTF-16/composition tests cannot establish those integration behaviors.
GPUI 1.25.3 has no disabled-state setter for these menu elements: disabled items
have no activation handler and announce an “Unavailable” description, matching
the library's existing button convention.

## Native visual checks

From the repository root, run `BIN=/path/to/voxelite OUT=/path/to/shots
scripts/capture-ui-polish.sh`. It requires Xvfb, xdotool, ImageMagick and Python
with Pillow; xfwm4 enables the real fullscreen transition check. The same script
accepts the baseline binary and tolerates missing features. It records full and
compact windows, About and acknowledgements, Updates, View menu, and both
fullscreen labels.

The explicit `VOXELITE_CAPTURE_STATUS` switch enables Linux-only status fixtures
for this script. They affect only the readout, never voxel tool input or document
state. `statusbar-stability.png` compares positive, negative, three-digit, i32
extreme and placeholder values; `statusbar-separators.json` records the painted
separator columns so identical positions can be checked independently of text.
