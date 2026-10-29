# Quantum design system

`quantum-ui` owns presentation. App crates compose these controls and own domain
state, actions, focus restoration and open/close state. They must not choose their
own colors, type sizes, spacing, radii or control geometry.

Issue #133 supplies the API; #134 migrates every application surface. App actions,
fields, tabs, segments, rows and swatches use `::design` (or the component’s `::new`)
and the typed tokens below. Shared compatibility color accessors now resolve the
same semantic palette; they no longer define a second set of screen colors. Layout
metrics retain viewport/window sizes and platform artwork geometry. See the
[migration audit](../../docs/design-system-migration.md) for coverage and verification.
Do not add new compatibility colors. `Theme::palette()` is the only new color system.

## Tokens

All values live in `src/tokens.rs`, with frozen old values in `src/tokens/legacy.rs`.

| API | Values in logical pixels |
| --- | --- |
| `Space::S0..S8` | 0, 2, 4, 8, 12, 16, 24, 32, 48 |
| `Radius::{Xs, Sm, Md, Lg, Full}` | 4, 6, 10, 14, 9999 |
| `ControlHeight::{Dense, Default, Large}` | 24, 28, 32; rail remains 36 |
| `TextSize::{Caption, Small, Body, Title, Heading, Display, Mono}` | 11/16 w500, 12/16 w400, 13/20 w400, 15/20 w600, 20/28 w600, 28/34 w600, 12/16 w400 tabular |

`TextSize::apply(element)` applies the whole type contract, including Geist/Geist
Mono and the mono tabular feature. Do not apply a size without its line height.

| `ColorToken` | Dark | Light |
| --- | --- | --- |
| Canvas | #0b0b0c | #f5f5f7 |
| Surface | #111113 | #ffffff |
| Raised | #18181b | #f4f4f5 |
| Overlay | #1f1f23 | #ffffff |
| BorderSubtle | #232326 | #e4e4e7 |
| Border | #2e2e33 | #d4d4d8 |
| TextPrimary | #ededef | #18181b |
| TextSecondary | #a1a1aa | #52525b |
| TextTertiary | #8b8b94 | #65656f |
| Accent | #2563eb | #2563eb |
| AccentText | #93c5fd | #1d4ed8 |
| Danger | #f87171 | #b91c1c |
| OnAccent | #ffffff | #ffffff |
| Success | #86efac | #166534 |

The final two entries are semantic extensions for primary text and success feedback,
not another palette. Tests pin all values and require 4.5:1 contrast for text on all
four surfaces and white on primary/rest, primary/hover and primary/pressed. Disabled controls are
intentionally exempt from normal text contrast.

State tokens: neutral hover 5%, pressed 8% (white in dark, primary text ink in light), selected accent 16% with primary
text and accent icon, navigation/tab indicator 2px, keyboard focus ring 2px with
1px offset, disabled opacity 40%, scrim black 50%, primary hover 6% white mixed
into accent in sRGB. `ControlState` allows deterministic state previews; ordinary
controls use an absolute painted wash and GPUI focus-visible hooks. Pointer hover/release
ease out over 120 ms, press over 80 ms, retargeting from the current painted value.
No geometry or text moves. Reduced motion applies immediately; idle, hidden and
removed controls schedule no ongoing frames. Focus is immediate. Selection never implies focus. `Button::on_surface(ColorToken)` sets the surrounding
surface used for the focus offset; menus, segments and dialogs supply it automatically.

## Components

| Component | New API / contract |
| --- | --- |
| Button | `Button::design(id, label, theme).style(ButtonStyle::Primary).height(ControlHeight::Default)`; Primary/Secondary/Ghost/Danger, selected, disabled, loading, guarded `on_click` |
| IconButton | `IconButton::design(id, icon, tooltip_label, theme, &overlays)`; label required; square, selected; render the builder to include the live tooltip |
| Panel | `Panel::design(theme).header(title).actions(...).body(scroll_id, ...).footer(...)`; 16px padding; constrain height for scrolling |
| ListRow | Leading icon/thumb, ellipsized title/meta, fixed trailing slot, selected, focus, disabled; `navigation(true)` adds the selection bar |
| SettingsRow | `SettingsRow::row(title, description, control, theme)` and `SectionLabel::label` |
| EmptyState | One icon/line, `primary` and `secondary` action slots; a third action is unrepresentable |
| Tabs | `Tab::design` in `Tabs::design`; keep the strip outside a scroll body for sticky layout |
| SegmentedControl | `Segment::design` in `SegmentedControl::design`; fill indicator only |
| Tooltip / Kbd | Required label, optional single chip, 500ms default dwell; `Tooltip::attach` adds lifecycle listeners; `Kbd::design` draws exactly one border |
| Menu | `Menu::design`, 28px `MenuItem`, `Separator`, measured `Submenu`, `destructive_group`; host owns arrow navigation/dispatch |
| Popover | `Popover::design` surface plus `Popover::anchored(trigger_bounds, side, content)`; measured flip, 4px gap, viewport clamping |
| Dialog | `Dialog::design(id, title, theme, focus, cancel)`; cancel callback required, Cancel always supplied, Escape capture and tab trap; `danger` left, `primary` right |
| TextField | `TextField::design(placeholder, cx)` uses the existing editor; `appearance` and `error` control theme and inline validation; keyboard focus ring only |
| Swatch | `Swatch::design(theme, document_color)`; selection changes the outer ring only; `interactive(id, label)` adds Tab traversal and a contrasting inner focus stroke |
| FocusGroup | Focusable list/grid frame with an inset ring that survives scroll clipping; host owns selection and arrow navigation |
| StatusBarItem | Proportional label, fixed-width tabular value, overflow/padding, optional separator |
| Toast / InlineError | `message(text, FeedbackKind::{Info, Success, Danger}, theme)`; glyph plus color |
| Spinner / Shimmer | Eight-dot inline spinner; automatically respects GPUI reduced motion, with additional host-preference/snapshot controls; `Shimmer` aliases the existing `AsciiSkeleton` |

Use the builder's guarded `on_click` for new controls. Adding a callback directly
to a raw `build()` result transfers disabled/loading enforcement to the caller.
The app uses `ButtonStyle` directly. Rich rows use `ListRow::frame` so hover,
pressed and keyboard focus styles remain owned by the library. Do not add a second
GPUI hover/active/focus-visible style to a component that already supplies one.

## Overlay lifecycle

Create one `OverlayState::new(cx)` entity per window and pass it to tooltip triggers.
Call `state.set_open(id, layer, open, cx)` in the same update that changes the
host's popover/dialog state. Separate IDs allow nested overlays to close independently.
Opening or closing a layer invalidates pending and visible tooltips; while a popover
or dialog is open hints are suppressed. A new hover is required after dismissal.
Mouse presses (including capture-phase presses on children) cancel pending timers.

`OverlayLayer` is the single deferred ordering contract: Tooltip < Popover < Dialog.
The new tooltip does not use GPUI's native topmost tooltip slot. App tooltips share one overlay state per window. `Tooltip::decorate` keeps host
event hooks and places its hitbox outside layout; opening editor overlays suppresses
those hints before they paint. `Popover::anchored` measures content before positioning;
submenus and tooltips use the same placement function. The host owns trigger bounds
and dialog focus restoration. `Dialog::build()` is an inline preview; render the
builder or call `overlay()` for a modal scrim and proper layer ordering.

## Gallery and verification

```sh
cargo run --locked -p quantum-ui --example ui_kit
cargo run --locked -p quantum-ui --example ui_kit -- --section buttons
cargo test --locked -p quantum-ui
cargo clippy --locked -p quantum-ui -p quantum-app --all-targets -- -D warnings
cargo xtask screenshots
```

Sections: tokens, buttons, rows, navigation, menus, fields, feedback. Both themes
are shown together. Tab through native controls, edit the fields, hover/press the
live tooltip trigger, open the anchored popover and open/cancel the dialog. The
state matrix includes rest, hover, pressed, keyboard focus, disabled and loading.
The gallery's development-only GPUI platform dependency launches the window; normal
library dependencies remain GPUI and Unicode segmentation. Font/icon assets are
read from the existing app assets with their original license notices unchanged.

Linux exercises the builder contracts, editing tests, gallery and screenshot harness.
AppKit, Metal, native macOS menu routing and IMEs still require macOS verification.
