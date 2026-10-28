# Quantum UI library

This crate owns GPUI presentation and field editing. The app owns domain state,
actions, selection, open/close state, focus restoration, and tooltip placement.
Use the public re-exports in `src/lib.rs`; icon assets remain in the app.

## Files and tokens

- `src/tokens.rs`: `Theme` owns semantic colors, including scrims, artwork colors,
  selection, hover, disabled, axis and contrast colors. Dynamic document/palette
  colors are data, not theme literals.
- `space`: spacing scale, including documented optical and compatibility insets.
  `NONE` is zero inset or a flexible minimum, not a new control size.
- `radius`: `CONTROL = MD` (6 logical pixels) is the default for all four corners
  of buttons, tabs, segments, and rail controls. `SM`, `LG`, `XL`, `PREVIEW`,
  `APPEARANCE`, and `WINDOW` describe existing nonstandard artwork/surface geometry.
  `concentric(outer, inset)` aligns inner and outer corner centers; inset includes
  border width and padding.
- `type_size`: UI and mono families, font sizes, line heights, font weights, and
  mono advance. Rail axis badges use `MONO` and `BADGE`.
- `ControlSize`: Sm/Md/Lg heights (24/32/40). Icon buttons use the same value for
  both dimensions.
- `metrics`: named control, tab, rail, sidebar, panel, menu, popover, dialog,
  settings, palette, thumbnail, timeline, toolbar, status and window dimensions.
  Render viewport sizing must use the same metrics as native layout.
- `focus_ring` and `panel_shadow`: shared elevation without layout impact.
- `src/components.rs`: action and form controls, navigation rows, cards,
  toolbar, dialogs, tooltip and keycap presentation.
- `src/navigation.rs`: underline tabs and segmented choices.
- `src/surfaces.rs`: panel and popover chrome, including concentric inner clipping.
- `src/menu.rs`: menu triggers, surfaces and entries.
- `src/text_field.rs`, `numeric_field.rs`, `editing/`: controlled editors, IME,
  selection, clipboard, undo, numeric validation and platform key bindings.
- `src/color.rs`, `color_picker.rs`, `slider.rs`: color math, color square,
  hue slider and controlled numeric slider. Pointer geometry stays with the control.
- `src/status.rs`: fixed-cell status/coordinate telemetry.
- `src/ascii_reveal/`: MIT port of ascii-reveal v0.3.0 at `f602e20`. Preserve
  upstream constants, Float32 timing rounding and Node-derived inline goldens.
  Keep `core`, `presets`, `skeleton`, and `media` free of GPUI imports.

## Components

- `Button`: Primary, Secondary, Ghost, Icon (compatibility), Danger; size,
  icon, selected, disabled and tab order. `build()` exposes GPUI event hooks.
  `custom` and `tile` frame rich content while inheriting its typography and
  preserving caller layout. Use `IconButton` for ordinary icon-only actions.
- `IconButton`: a real square builder. `new`/compatibility `button` uses
  `ControlSize`; `rail` always uses `metrics::RAIL_BUTTON` (36), even if `size`
  is called later. Selected rail borders are shared with ordinary tool buttons.
  `badge(text)` is absolutely positioned at bottom-right and cannot shift the
  icon. Mirror tooltip anchor corrections preserve their original placement.
  `custom` frames independently colored glyph content at a square size.
- `Tab` / `TabBar`: underline tabs with `Role::Tab`, `aria_selected`, and tab
  traversal. All highlight corners use `CONTROL`. The underline is a separate
  absolute child with 2px horizontal insets and 2px thickness. Bottom padding
  retains the old text baseline; never use a bottom border to draw the underline.
- `Segment` / `SegmentedControl`: selected choices with button interaction.
  `row` provides grouped chrome; `inline` is unframed. `ButtonGroup` and
  `Segmented` are compatibility names for `SegmentedControl`.
- `Panel`: section, rail, sidebar and grouped-card surfaces; `new().header()`
  adds an optional title. `section` inherits its parent background and leaves
  layout to the caller; `rail` and `sidebar` own their shared dimensions/edges.
- `Popover`: XL raised surface, border, background, shadow and clipping.
  `canvas`, `menu`, and `tooltip` retain the existing surface palettes/geometries.
  `inner(inset)` builds a concentric inner clip. GPUI's overflow mask is rectangular:
  a flush child background must also round its exposed corners explicitly.
- `Dialog`: thin modal wrapper over `Popover`, adding focus tracking, role,
  sizing, typography, title/footer, scrim, and `trap_tab`. App closes/restores focus.
- `Menu`: thin `Popover` wrapper with menu role, entry presentation and trigger.
  App owns dismissal, navigation and callbacks.
- `Card` / `Group`: grouped `Panel` surface, optional heading and inset dividers.
- `FieldFrame`: native field/editor chrome and search rows.
- `Swatch`: palette/recents swatches; `canvas` retains sidebar selection borders;
  `previous` handles the exposed edge of a joined previous/current color sample.
- `FormRow`: wrapping labels/descriptions and a nonshrinking control.
- `SidebarItem`, `SidebarSection`: navigation/results rows and section labels;
  `disclosure` retains unrounded document-list separators.
- `Toolbar`, `ToolbarSeparator`: fixed toolbar geometry and short dividers.
- `Toggle`: accessible switch; circular track/thumb are intentional.
- `Badge` / `Pill`: informational labels, not buttons.
- `Kbd`: keycap, shaped-glyph `frame`, and settings shortcut `well`.
- `tooltip`: compact `Popover` chrome. Native timing/placement belongs to GPUI/app.
- `row`, `mono`, `icon`: alignment, fixed-advance text, and asset glyph helpers.
- `Spinner` in `feedback.rs`: fixed-size eight-dot loading indicator, respecting
  GPUI reduced motion by default plus explicit host preferences and snapshot clocks.
- `AsciiSkeleton`: stable keyed ID, paragraph `lines`/GPUI `widths`, block `rows`,
  `variant` (dither/scan/braille/terminal), `seed`, `animate`, `reduced_motion`.
- `AsciiReveal`: real text and accessible label, decode/dither/terminal/blueprint
  preset, fast/cinematic timing, seed/speed, digits/flash, direction and loading.
  Reserve final shaped layout. `AsciiImageReveal` accepts a shared, precomputed
  `Media` RGBA grid and the caller's already fitted GPUI image element.
  All three accept `started_at(Instant)` or timer-free `elapsed(Duration)`.
  Pass the app's reduced-motion preference: skeletons freeze; reveal uses the
  original 220 ms fade; hide is instant. Cache patterns/timings/shaping; never
  allocate an element per glyph or accumulate animation state per frame.
  Rearm at most one 33 ms timer per component only while visible and animating;
  stop on completion, removal, static snapshots, or hidden windows.
  Keep [NOTICE](NOTICE) and the app acknowledgement with the port.

### Deviations from the original

The complete list is in [README.md](README.md#deviations-from-the-original) and
the module docs; update both when changing these contracts. Typed GPUI builders
replace DOM/SSR/CSS traversal, surface effects, lazy sections and mascots. Theme
tokens/Geist Mono and native line metrics replace browser fonts and CSS;
explicit newlines work without soft wrapping. Native glyph caches replace
canvases, rectangular masks clip blocks with library radii, and one-shot timers
replace rAF/watchdog scheduling. Media is bounded caller-fitted RGBA at DPR 1,
precomputed at all five sizes. The caller owns image I/O and reduced motion.
Loading switches directly to the skeleton, with explicit hide for scramble-out;
stable explicit clocks replace random browser seeds and the mount delay.
Seed zero works consistently, non-finite speed falls back to one, and presets
are a typed built-in enum. Accessible real labels replace DOM text; GPUI has
no `aria-busy` builder. Do not claim full browser controller or macOS coverage.

## Rules and compatibility

Never hardcode px/radius/colors for controls in the app; add a documented token.
Use one corner radius (`radius::CONTROL`) for new buttons, tabs, segments and rail
buttons. Keep behavior in the app and chrome in the library. Preserve accessible
labels, shortcuts, selection, disabled states and keyboard traversal when migrating.
Do not attach an enabled callback to a disabled control.

The #134 migration uses the typed design palette, spacing, type and corner scales.
Old semantic color accessors resolve this same palette. App text fields use
`TextField::design`; full typography uses `TextSize::apply` or `TextStyle::text_style`.
Do not attach a second hover, active or focus-visible style to a design component:
GPUI rejects duplicate handlers. Use its controlled state or rich `ListRow::frame`.
The palette grid’s cell/pitch metrics must also drive drag hit testing and scrolling.
Keep document colors, world geometry and viewport/window bounds independent of
control appearance. Numeric field widths include border and caret clearance.
`Tooltip::decorate` preserves host builder hooks with an absolute hitbox; it must
never introduce an in-flow child. Each window owns its shared `OverlayState`.

## Adding or changing a component

1. Check whether a current component can express the desired presentation.
2. Put reusable geometry/colors in documented tokens, keeping existing values.
3. Add a controlled builder, return a GPUI element for native event hooks, and
   re-export it from `lib.rs`. Include accessibility and keyboard focus states.
4. Test actual built styles without a window where possible. Avoid tests that
   only compare token constants with themselves. Use screenshots for layout,
   compositing, hover, press, focus and clipping.
5. Update this inventory and migrate callers; avoid duplicate chrome in the app.

## Verification and screenshot diffs

Source `/workspace/cargo-env.sh` when using the shared workspace build cache.
Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
and `cargo test -p quantum-ui -p quantum-app`. Follow applicable CI checks;
this checkout has `ci-macos.yml` and `render-checks.yml`, not `ci.yml`.
The plugin dependency check and Python packaging tests run on Linux. Apple builds,
DMG signing and Metal golden rendering require the macOS runner.

Build with `cargo build --release -p quantum-app`. Preserve a baseline binary
before edits. Capture both binaries with the same Xvfb geometry, data and theme:
`/workspace/mirror-buttons/capture.sh BIN OUTDIR` is a copy of the original
`/workspace/palette-corners/capture.sh`, extended for hover/focus, comments,
settings, About and menus. Keep captures in `/workspace/mirror-buttons/codex-shots/`.
Use `compare -metric AE BEFORE AFTER DIFF`, inspect the diff, and verify every
changed pixel is within the authorized regions. Check panel tab focus separately
from hover. Native click automation can need a frame after pointer movement;
keyboard traversal/activation is a useful deterministic fallback.

Migration checklist: inventory literal counts; record baseline screenshots;
map controls/surfaces to components; preserve old metric values; audit radii and
colors; verify focus/labels; run tests and lint; compare identical capture states;
record per-file raw-px counts and explain every retained literal/visual exception.
