# UI component library

`quantum-ui` is the shell's shared GPUI presentation library. It depends only on
GPUI for UI/runtime services (default features disabled) and the already locked
`unicode-segmentation` crate for the pure editor. It has no app or domain
dependencies. The application selects native platform support. Plugins
must not depend on this crate.

## ASCII loading and reveals

`AsciiSkeleton`, `AsciiReveal` and `AsciiImageReveal` port **ascii-reveal 0.3.0**
by Quan Hoang (MIT), commit `f602e204232d2d31de2b67c66a166bcdbde51602`.
The original is at <https://github.com/quanhoangindex/ascii-reveal>; the complete
copyright and license are in [NOTICE](NOTICE), also included in the app's
acknowledgements.

```rust
use quantum_ui::{AsciiReveal, AsciiSkeleton, Theme};
use quantum_ui::ascii_reveal::{core::Timing, presets::Preset, skeleton::SkeletonVariant};

let placeholder = AsciiSkeleton::new("preview-loading", Theme::Dark)
    .lines(3)
    .variant(SkeletonVariant::Dither)
    .seed(7)
    .reduced_motion(false); // The app supplies its actual preference.
let title = AsciiReveal::new("project-name", "Quantum 42", Theme::Dark)
    .preset(Preset::Decode)
    .timing(Timing::Fast)
    .loading(false)
    .reduced_motion(false);
```

Give each instance a stable ID. Skeletons accept `lines`, repeating GPUI `widths`
(e.g. `gpui::relative(0.8)`), tightly stacked block `rows`, `variant`, `seed`,
and `animate`. Dither uses shaded blocks; scan uses dot-to-dense marks; braille
uses fine dots; terminal uses binary digits with an accent sweep. The base
pattern paints immediately. The two cached wave patterns alternate every 120 ms
inside the original 16-strip eased band, staggered 90 ms between paragraph lines.

Text builders accept `preset`, `timing`, `speed`, `seed`, `digits`, `flash`,
`direction`, `loading`, and `skeleton_variant`. Decode condenses light-to-dense
marks before scrambling; dither uses shaded blocks; terminal starts with a block
cursor; blueprint uses accent-colored construction marks. Fast and cinematic
use the original timing tables. Numeric strings automatically scramble through
0–9. The final shaped text reserves width and height through loading/decoding,
including emoji and combining characters. Its accessible label is always the
real text; decorative glyphs are painted outside the accessibility tree.

All builders accept `started_at(Instant)` for caller synchronization or replay,
and `elapsed(Duration)` for a deterministic snapshot with no timer. Without an
explicit clock, keyed state starts on mount and restarts text playback when
loading ends or text/options/direction change. Keep a supplied start instant
stable across renders and reset it when starting another reveal.

`reduced_motion(true)` freezes skeletons and replaces decoding with the original
220 ms CSS ease-out fade of real content; reduced-motion hides are immediate.
The app passes `quantum_app::reduce_motion::prefers_reduced_motion()`; this crate
does not inspect platform preferences.

For thumbnails, construct and reuse `Rc<ascii_reveal::media::Media>` with an
already fitted RGBA grid, dimensions, and `MediaOptions` (preset, timing, speed,
background RGB and optional origin). `AsciiImageReveal::new(id, media, image,
label, theme)` accepts the corresponding GPUI image element. Set the backdrop
to the actual color behind transparent pixels. Dimensions must be 1–1024 on
each edge and the pixel count must match; invalid input returns `MediaError`.
Sampling, luminance/ink, per-cell gates, source colors, coarse-to-fine resolution
and the source-image crossfade match upstream. Image loading stays with the app.

The public `ascii_reveal::{core,presets,skeleton,media}` modules contain no GPUI
imports. `Decode::new(text, DecodeOptions)` caches grapheme ranges and Float32
timings; `frame(elapsed_ms, Direction)` yields borrowed `Cell`s with glyph,
Pending/Cursor/Scramble/Landed state, opacity and flash weight. `duration_ms` and
`is_finished` expose completion. `Media::frame` yields borrowed image cells and
image/glyph opacity. Frame evaluation does not allocate or accumulate state.

CPU behavior: keyed GPUI caches retain text timing, three shaped runs per
skeleton row, real text and the small glyph alphabet. A skeleton row is one
shaped run per layer, never a tree of glyph elements. Its wave uses the same
16 clipped strips as upstream. Text and image glyphs reuse cached shaping and
stack-local decoration runs. One pending 33 ms timer per component requests
the next redraw; there is no vsync loop or catch-up work. Finished, static,
reduced skeletons and elapsed snapshots do not schedule. Clipped or hidden
windows do not rearm timers; a previously queued timer may wake once. Visible
windows animate even without keyboard focus. A visibility subscription resumes
an unfinished animation after hiding and is removed on completion. For long
lists use `animate(false)`.

Quantum uses these components for recent-project previews, project opening,
the first viewport frame, save/export I/O, and agent inspection images. Preview
pixels and both theme grids are prepared in background jobs; ordinary camera
changes and status updates do not replay a reveal. Run the native preset gallery
with `cargo run -p quantum-app --example ascii_loading --locked` (Space replays;
`-- --light` or `-- --reduced-motion` selects those modes).

### Deviations from the original

- Explicit text/image builders replace DOM traversal, CSS selectors, SSR,
  hydration, surface effects, lazy sections, pointer-origin discovery and mascots.
  `Timing::wave_delay` supplies target staggering for callers coordinating views.
  Local clocks omit the browser's 16 ms mount delay and random run/controller seeds.
- Library theme tokens and bundled Geist Mono replace CSS/custom fonts and colors.
  Text supports explicit newlines without soft wrapping; Unicode segmentation
  uses the crate's pinned Unicode version rather than the browser's ICU version.
  GPUI's native line metrics replace Canvas's text-baseline adjustments.
- GPUI's glyph atlas and cached shaped runs replace offscreen canvases. Skeleton
  block corners use the shared radius; their glyph clipping is rectangular,
  consistent with GPUI's content mask. Seed zero remains zero for the wave;
  upstream substitutes one for the wave alone. Widths are GPUI lengths, without
  CSS parsing, circle or arbitrary block-height props; callers compose layout.
- One-shot 33 ms timers replace the shared browser rAF and frame watchdog.
  Delayed frames immediately evaluate current time. No per-frame CPU percentage
  or browser benchmark result is claimed for GPUI.
- Media accepts bounded, already fitted RGBA thumbnails at DPR 1. Image I/O,
  decoding, object-fit, CORS fallback and device-pixel resampling belong to the
  caller. All five grids are computed up front instead of lazily during playback.
- Loading text switches directly to the skeleton; explicit `Direction::Hide`
  supplies scramble-out. Text/options changes restart automatic playback.
  Non-finite speed falls back to one. Built-in presets are exposed as a typed
  enum; arbitrary runtime CSS/preset objects are not part of this API.
- The app supplies reduced motion. GPUI supplies real accessible labels but has
  no `aria-busy` builder. These passive components add no focus stops.

Inline Node-derived goldens cover the upstream hash, easing LUT, skeleton text,
text frames (including numeric strings, terminal cursors and graphemes), and
image sampling/gates/colors. Rust tests also cover reverse time, completion,
determinism, reduced motion, invalid image grids and transparency. Run
`cargo test -p quantum-ui --locked`; native AppKit/Metal and VoiceOver still need
macOS verification.

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

`TextField::any_focused(window, cx)` checks live field focus handles, including
controlled numeric inputs. Use it for shell menu and keyboard routing: it is safe
before a window's first draw and does not query GPUI's previous-frame context
stack. Field registration ends when its entity is released.

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

From the repository root, run `BIN=/path/to/quantum OUT=/path/to/shots
scripts/capture-ui-polish.sh`. It requires Xvfb, xdotool, ImageMagick and Python
with Pillow; xfwm4 enables the real fullscreen transition check. The same script
accepts the baseline binary and tolerates missing features. It records full and
compact windows, About and acknowledgements, Updates, View menu, and both
fullscreen labels.

The explicit `QUANTUM_CAPTURE_STATUS` switch enables Linux-only status fixtures
for this script. They affect only the readout, never voxel tool input or document
state. `statusbar-stability.png` compares positive, negative, three-digit, i32
extreme and placeholder values; `statusbar-separators.json` records the painted
separator columns so identical positions can be checked independently of text.

App callers are checked by `cargo xtask lint`: literal `px`, hex `rgb`/`rgba`,
literal `hsl`/`hsla`, string patterns and literal string equality have a
shrink-only baseline. Add shared tokens/components and typed dispatch rather
than new exceptions. The shared UI crate owns styling and is exempt from the
app patterns. All Rust files/functions retain the 1,000/100-line size budgets.
See [the exact lint rules](../../xtask/README.md) for scope and baseline updates.

## Typed design system (#133)

See [DESIGN.md](DESIGN.md) for the palette, type/geometry scales, opt-in builders,
overlay lifecycle, headless tests and interactive dark/light gallery. The app migration and retained platform/layout boundaries are documented in the
[migration audit](../../docs/design-system-migration.md).

The [polish audit](../../docs/ui-polish.md) records the shared hover/press motion,
palette focus, empty/loading fixtures and visual review coverage for #135.
