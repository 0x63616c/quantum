# App polish round 1

Baseline: `739f767`, `build-v1`, 2026-10-25. Reviewed the actual 1440×900
native screenshots against `PLAN.md` §§8–9, `design/ART-CHECKLIST.md`, Concept A's
HTML, both concept themes and the palette reference. This is a deliberately
strict review by the implementing agent, not independent design approval.
Nothing under `design/` is edited.

## Before: dark 5.8/10, light 5.5/10

The artwork carries these screens. The shell is functional, but looks like an
engineering preview: almost uniform typography, arbitrary whitespace, too many
small outlines and no compelling activity hierarchy. A score of 9 would require
consistent states and convincing native finish; 10 is reserved for exceptional,
independently validated execution. These scores assess visual craft, not feature
completion or Apple accessibility certification.

| Area | Dark | Light | Concrete defects |
| --- | ---: | ---: | --- |
| Spacing and alignment | 6 | 6 | Sidebar gutters alternate between 16 and 20 px; headings, rows and swatches do not share an optical edge. History leaves a dead middle above an oversized disabled Undo. Navigation help floats 66 px above the bottom while camera controls sit on a different baseline. |
| Type and hierarchy | 5 | 5 | Project, tool, section titles and body labels all read at roughly 13 px and similar weight. The project is visually weaker than its toolbar deserves. `NATIVE / v0.1`, `r000`, `Human` and `CPU · 2× AA` give implementation metadata undue prominence. |
| Contrast and surfaces | 7 | 6 | Black/white bases are correct and secondary text is readable. Dark chrome lacks hierarchy beyond dividers; light outlines accumulate into a wireframe. The palette and layer panels draw adjacent horizontal separators. |
| Iconography | 5 | 5 | Four identical cube icons cannot communicate four camera directions. The active Frame button resembles a toggled mode, although it executes an action. Every unlocked layer has a tiny padlock competing with its name; icon-only controls need explicit tooltips. No orientation reference accompanies the model. |
| Borders and radii | 5 | 4 | Every swatch has an outer border, inset gap and inner rounded rectangle. This is particularly harsh on white. Radii of 3/5/6/7 px appear without a clear control hierarchy. The camera group adds another boxed perimeter. |
| States | 5 | 5 | Selected camera and layer rely predominantly on fill/color; selected swatches have a thin, weak blue outline. Disabled Undo has an active-looking pointer cursor. Hovering a selected action replaces its selection fill. |
| Palette previews | 5 | 4 | 36 double-framed squares dominate the lower sidebar. Color samples are needlessly smaller than their targets. The bracket hint is cryptic. The chooser compresses each palette to a 9 px strip and truncates palettes at 32 colors. |
| Empty history | 4 | 4 | “Local session / Human / History / Project ready / Your edits will appear here” repeats status across five labels without a useful next action. A large disabled Undo receives more emphasis than the empty state. |
| Viewport / artwork | 8 | 8 | The original autumn diorama has strong material separation and a readable silhouette. Large roof/foliage texture is slightly soft; the floor grid blankets the whole lower viewport. The giant upper void and low model placement weaken balance. |
| Theme-specific finish | 6 | 5 | Dark is appropriately black, but sparse hierarchy makes it feel unfinished. White exposes the repetitive swatch frames and weak type scale more severely. Neither yet meets the reference's calm density. |

## Fix scope

Rework the existing native shell: purposeful type hierarchy, shared gutters and
control metrics, calmer toolbar, honest local activity empty state, integrated
history actions, coherent selected/hover/disabled states, readable layer controls,
unframed color tiles with a strong selected marker, fuller preset previews and
clear camera orientation. Preserve real art, real document/history state and
pure black/white base surfaces. Re-capture and inspect both themes and the minimum
window size, then exercise changed interactions and run workspace checks.

The reference's live agent transcript, comments, captured-view inspector,
palette authoring and native material integration are broader implementation
gates already identified in the app README. Decorative fake activity, fake
presence or nonfunctional tabs would make this review worse, not close those
gates. No claim of full v1 acceptance follows from visual improvements.

## After

Pending implementation and native screenshot inspection.
