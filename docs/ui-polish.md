# UI polish audit (#135)

This pass completes the interaction details following the [design-system
migration](design-system-migration.md). It preserves the viewport, document colors,
layout, control dimensions, renderer settings and the bundled Lucide icon family.

## Issue acceptance evidence

| Requirement | Implementation and evidence |
| --- | --- |
| Consistent icons | Existing Lucide SVGs retain their 24-unit, 2-unit rounded stroke and runtime tint. The asset test verifies that contract and resolves every registered action icon. Shortcut symbols, axis letters and numbered comment pins retain their distinct semantic purpose. |
| Hover and press animation | Shared buttons, icon buttons, tabs, segments, list rows and toggles use a paint-only wash. Hover/release takes 120 ms; press takes 80 ms. Cubic ease-out retargets from the current painted value. Primary hover retains its 6% white tint; other controls use 5% neutral hover and 8% pressed tint, darkening light surfaces. |
| Keyboard focus | Sidebar colors now participate in Tab traversal. Interactive recent/previous colors have a contrasting inner focus stroke as well as the outer ring, so focus is distinct from selection. Palette library/grid frames gain inset focus rings that survive scrolling. Existing buttons, fields, tabs, switches, sliders and color pickers retain immediate keyboard focus. |
| Empty and loading states | Empty command search uses the shared EmptyState and a working Clear search action that returns focus to the field. Scene loading pairs the existing Shimmer with a readable status label. Welcome, history, comments, agent invitation, cached previews and save waiting retain their existing empty/loading paths. |
| Dark theme | All registered surfaces are captured in both themes with `Screens: *`; the neutral palette and artwork remain unchanged. The additional control matrix covers all six states and long-label truncation in both themes. |

Motion changes only the painted background, never geometry, text or input dispatch.
The wash observes hit-tested pointer events without consuming them, respects overlay
occlusion, clears presses after releases outside the control/window, and requests
frames only while transitioning and visible. Disabled and loading controls install
no wash. Native Reduce Motion is synchronized at startup and window activation
to GPUI's shared preference; motion becomes immediate. Headless snapshots freeze it.

Sidebar Tab stops expose individual colors; Space/Enter selects the focused color.
Arrow keys return to the palette's existing active-brush navigation group. Both
the group and its focused descendants use the palette key context, so navigation
and tool bindings cannot leak into the viewport.

## Review procedure

The Impeccable native critique inspected all 90 baseline screens. The baseline
scored 30/40: the primary issues were palette keyboard focus and readable
interaction/waiting feedback. The polish pass addresses these in shared controls;
the tool's web detector is inapplicable to GPUI. The three new fixtures expand the
PR matrix to 96 screens and the headless matrix from 26 to 32:

- `dialogs-commands-empty-*`: empty results, the real reset action and field focus.
- `overlays-loading-*`: the production scene shimmer at a fixed 400 ms.
- `overlays-controls-*`: primary, secondary, ghost and danger buttons in rest,
  hover, pressed, focused, disabled and loading states, plus a long list row.

Run `cargo xtask screenshots --screen '*'` and inspect both themes. Exercise Tab /
Shift-Tab and Space/Enter on buttons and palette colors; arrow through the palette
manager grid/library, inspect recent/previous-color focus, hover and hold actions,
release outside, and check that tooltips and menu dismissal still work. Check
empty command search resets its query and restores field focus. Use both themes
and inspect long names and loading surfaces at their normal window sizes.

The documented `cargo xtask ui-snapshots --rebaseline` flow records intentional
reference changes, followed by an ordinary comparison. The six new references cover
the three fixtures in both themes. All 26 existing references remain byte-identical.
The ordinary comparison passes all 32 references.

## Owner verification

Linux evidence covers GPUI composition, CPU viewport images and pointer/keyboard
interaction. The owner should verify macOS Reduce Motion (enable it in System
Settings, return to each Quantum window, then hover/press), native menu focus
restoration, VoiceOver names, IME composition and field-local undo in both themes
on a physical Mac. Also inspect native titlebars, save panels and the Metal surface
handoff. CI macOS compilation/tests do not replace these visual/input checks.
No signing, publishing or new credentials are required for the UI change.

Project-board updates require the token's missing `project` scope, tracked in
#148. The owner can move #135 to In review; the PR remains unmerged for review.
