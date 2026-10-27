# UI component library

`voxelite-ui` is the shell's shared GPUI presentation library. It depends only on
GPUI, with default features disabled; the application selects native or web
platform support. Plugins must not depend on this crate.

Use `tokens` for Theme colors, spacing, corner radii, Geist/Geist Mono typography,
control heights, focus rings and panel shadows. `Theme::surface`, `card`,
`surface_raised` and `separator` reproduce the grouped Settings appearance.

Use Button / IconButton for actions, ButtonGroup (Segmented) for related choices,
SidebarItem and SidebarSection for navigation, Card (Group) and FormRow for
settings, and Toolbar / ToolbarSeparator for editor chrome. Button's `build()`
exposes GPUI event, tooltip and accessibility hooks. Disabled buttons have no tab
stop; attach action listeners only while enabled. Keep domain dispatch in the app.

TextField is a presentation wrapper around the existing native editor's canvas
or entity. The app editor continues to own clipboard, selection, IME and its
behavior tests. `numeric(cells)` selects a fixed-width Geist Mono field for hex,
RGB and HSB values; callers own validation. Slider supports a gradient background,
a value-label slot and arrow/Home/End keys. Swatch supports selection and a lock
badge. Toggle, Badge (Pill) and Kbd cover small form and informational elements.

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
