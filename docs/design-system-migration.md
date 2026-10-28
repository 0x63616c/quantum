# Design-system migration

Issue #134 migrates the application to the typed design system introduced in #133.
The owner requested one PR for the complete migration, superseding the issue’s
original one-PR-per-area plan. The presentation contract is in
[quantum-ui/DESIGN.md](../crates/quantum-ui/DESIGN.md).

## Scope and visual changes

All screen groups resolve the semantic palette and shared spacing, radius and
complete typography tokens. Actions, tabs, segments, text fields, swatches, list
rows and menu items use the design builders. Icon actions use the shared square
control; tool rail actions retain their 36 px geometry. Rich rows inherit control
states from `ListRow::frame`. Tooltip timing, measured placement, press cancellation
and overlay suppression belong to the library, with one state entity per window.

Intentional differences include neutral tinted surfaces instead of pure black/white,
blue primary actions and selection indicators, 28 px default controls, consistent
rounded corners, and the complete Geist/Geist Mono type scale. Settings rows have
16 px side insets; slider and inactive switch tracks remain visible on raised cards.
Numeric fields reserve border and caret clearance for a complete hex value.

Sidebar tabs remain outside scrolling panel content. The editor sidebar reserves
space for layers and palette controls while the history list scrolls independently.
Status labels use proportional type and values keep physical tabular slots, including
signs, missing values and overflow. Palette cell sizes, keyboard scrolling and drag
insertion use the same shared pitch. Document colors, command routing, file dialogs,
editing/undo, viewport dimensions and render quality remain owned by their existing
systems. The migration does not reduce AA, AO, render scale or shadow quality.

## Screen audit

The PR uses `Screens: *`, selecting the entire capture registry in both themes.
The [capture documentation](../crates/quantum-app/README.md#ui-capture-states)
explains fixtures and platform exclusions.

| Area | Before/after selectors | Shared presentation |
| --- | --- | --- |
| Welcome, empty and five recents | `welcome*` | Rows, thumbnails, keycaps, complete type scale, shared tooltips |
| Editor titlebar, tool rail, camera and status | `editor-*` | Buttons, square icons, segments, fixed status slots |
| Every application menu | `chrome-menu-*` | Menu surfaces and guarded menu items |
| Palette, layers, history and versions | `panels-palette-*`, `panels-history-*` | Rows, swatches, text fields and actions |
| Comment list, empty filter, thread, draft and pins | `panels-comment*` | Tabs, segments, empty state, rich rows and fields |
| Agent events and inspection | `panels-agent-*`, `popovers-connect-agent-*` | Panel chrome, actions, thumbnails and loading presentation |
| About, acknowledgements, guide and command/tool search | `dialogs-about-*`, `dialogs-acknowledgements-*`, `dialogs-keyboard-guide-*`, `dialogs-commands-*`, `dialogs-tools-*`, `dialogs-extensions-*` | Modal surface, menus, fields, keycaps and actions |
| Save confirmation and waiting | `dialogs-unsaved-changes-*`, `dialogs-save-waiting-*` | Explicit primary/secondary/destructive actions and modal surface |
| Palette manager, chooser, file menu and deletion | `dialogs-palettes-*`, `popovers-palette-*`, `dialogs-palette-delete-*` | List rows, swatches, numeric fields, menus and guarded actions |
| Feedback and crash report consent | `dialogs-feedback-*`, `dialogs-crash-report-*` | Fields, actions, grouped rows and modal typography |
| Every Settings page | `settings-*` | Shared settings rows, navigation, fields, toggles and slider |
| Debug overlay | `overlays-debug-*` | Shared surface and tabular typography |

## Source and regression audit

Production app raw `px(number)` styling falls from nine occurrences to zero.
The remaining twelve occurrences in `ui/viewport.rs` are numerical input fixtures
in tests, not control styling. All four production styling allowlist entries are
removed or reduced; function-size allowances only shrink. Dynamic document colors
and projected model coordinates remain data rather than theme tokens.

The existing app/editor, text editing and control tests remain in place. Additional
checks cover absolute tooltip hitboxes and complete numeric-field width. The
4.5:1 text-contrast test measures translucent selection after compositing onto both
canvas and surface backgrounds. This retains its threshold.

The independent [headless UI gate](ui-snapshots.md) from #124/#204 is rebaselined
with `cargo xtask ui-snapshots --rebaseline`, followed by an ordinary comparison.
The intentional golden changes are the palette, controls, typography and sidebar
layout described above. The before/after PR capture remains the review evidence;
headless references are not a substitute for that evidence.

## Platform handoff

Linux captures and tests cannot establish AppKit titlebar/traffic-light appearance,
native save-panel controls, native macOS menu routing, VoiceOver/IME behavior, or
the Metal surface handoff. The owner should inspect Welcome, editor, every Settings
page and the save/close flow in both themes on a physical Mac; check field-local
undo, an IME composition and native menu focus restoration. CI macOS compile/tests
are reported separately in the PR. No new signing credentials or product publication
are required. Project-board movement remains blocked by the missing `project` token
scope tracked in #148; the owner can move #134 to In review after granting that scope.
