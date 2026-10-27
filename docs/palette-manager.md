# Portable palette manager

`voxelite-plugin-palette` owns manager behavior. Its only normal dependency is
`voxelite-plugin-api`; filesystem writes, settings admission, brush assignment and
core transactions remain with the host. API version 0.7.0 adds public fields and
exhaustive action variants. The app continues to persist an outcome before adopting
its state, so write failures preserve the previous library, selection and brush.

## Choices and selection

`PaletteView::choices` contains project colors, seven built-ins, then custom entries.
Each `PaletteChoice` has `id: PaletteId`, `kind: PaletteKind`, `read_only`, `name`
and the full palette for thumbnail rendering. `PaletteId::Source(0)` is the project;
source indices 1..=7 are the built-ins in their existing order. Custom IDs are UUIDs,
independent of library position. `PaletteView::selected`, `kind` and `read_only`
describe the current selection. `preset` retains the source choice while a custom
palette is selected. `ChoosePalette(id)` selects either kind; the existing
`ChoosePreset(index)` and `ChooseCustom(uuid)` actions still work.

Project and built-in palettes are source palettes in the manager. An
edit creates and selects a custom copy first. Its name is `<Name> copy`, then
`<Name> copy 2`, etc., using the first unused custom name. The source is truncated
at a UTF-8 boundary to reserve the suffix within the existing 128-byte name limit.
`PaletteOutcome::copied_from` reports the source name on the action that creates
the automatic copy. A rename then applies the requested name to that copy. Invalid
edits and no-op moves do not create copies. Explicit Duplicate still copies exact
slots, swatch names and display order; its legacy name field supplies the copy's
base name. Delete accepts only custom entries. The library limit remains 128.

`SelectSlot(position)` only selects a manager swatch, clamping to display order.
`SelectSwatch(position)` and `StepSwatch(delta)` choose the brush color and return
`color` plus `selected`; they leave the manager cursor alone. Library choices,
manager editing and recent-color actions never change the brush or document.

## Swatch actions

All swatch positions are zero-based **display positions**, not stable slot IDs.
The live slots retain their identities when colors or display order change.

| Action | Behavior |
| --- | --- |
| `SetColor { rgb: [u8; 3], commit }` | Sets exact opaque sRGB on the selected swatch and preserves its name. |
| `AddSwatch { rgb, at: Option<usize> }` | Inserts and selects a new swatch; None appends, positions beyond the end clamp to the end. |
| `RemoveSwatch(position)` | Removes that display position. Selection follows the existing selected identity; if removed, select the next swatch or previous final swatch. |
| `MoveSwatch { from, to }` | Removes from `from`, inserts at final position `to`; clamps `to` to the final position. Invalid `from` rejects. Same-position moves are no-ops. Selection follows its stable identity. |
| `DuplicateSwatch(position)` | Copies color and name immediately after that position and selects the new swatch. |
| `UseRecentColor { index, target }` | Commits an MRU color into `SelectedSwatch` or `NewSwatch { at }`. |

Insertion uses the lowest unused stable slot in 1..=256. Removing the last swatch
fails with `Cannot remove the last swatch from a palette`; adding beyond capacity
fails with `256 swatches maximum per palette`. Errors return no outcome. Legacy
`Edit { operation, name, color }` supports Create, Duplicate, Delete, Rename,
ApplyColor, AddSlot, RemoveSlot, MoveLeft and MoveRight. ApplyColor still parses
exactly one six-digit hex color; both hex and RGB edits now preserve swatch names.

## Preview lifecycle

On pointer movement send `SetColor { rgb, commit: false }`. The first preview
retains a rollback point in `PaletteState::preview`, creates a custom copy if
needed, and updates the working library. Subsequent movements reuse that copy.
Previews return no `library_bytes` and do not update recent colors. The view and
sidebar thumbnails reflect the working library.

On pointer release send `SetColor { rgb, commit: true }` or `CommitColor`. A commit
ends the preview and returns at most one library write, including the recent
color. Repeating `CommitColor` without an active preview is a no-op.
`CancelColor` restores the prior library and manager selection without a write.
Every other action cancels an active preview before it runs, including selection,
export and legacy Edit. Thus a caller must commit before switching selection or
exporting if it wants to retain the picker value. Invalid actions leave the
previous state intact, including its active preview. Hosts must adopt the returned
state even when there are no bytes; hosts must never serialize the working state
independently of an outcome. Only a successful persistence admits a committed edit.

`PalettePreview` contains the prior `library`, `custom` selection and `slot`.
It is transient and never serialized. HSV/HSB conversion, picker layout and pointer
capture remain UI responsibilities; an eyedropper supplies exact RGB to SetColor.

## Recent colors and persistence

`PaletteState::recent_colors` contains up to sixteen distinct `[u8; 3]` values in
MRU order. Committing a color, inserting/duplicating a swatch, choosing a recent
color, or choosing a brush swatch moves that RGB to the front, removes duplicates,
and drops the oldest overflow. Selection alone, previews, palette duplication,
rename, reorder, remove and import do not add colors. Repeating a committed color
already first in MRU produces no write when the palette is also unchanged.

The existing `palettes.json` object retains its `entries` array, UUIDs, names,
`slots` and `order`. An optional `recent_colors` array stores MRU values; absent
means empty, and an empty list is omitted on serialization. Old files and the
old byte-exact fixture remain valid. Readers reject malformed recent colors,
duplicates and more than sixteen values. The library file remains bounded at
4 MiB for both loading and saving. Large libraries use compact JSON if necessary;
an edit that still exceeds the bound is rejected before adoption. Loading never requests a write or serializes transient selection/preview.
No new setting or core document schema is introduced.

## Interchange

`PaletteView::formats` and `voxelite_plugin_palette::FORMATS` expose the same
`PaletteFormat { id, label, extension }` list. Every listed format supports import
and export: `hex` (Lospec), `gpl` (GIMP), and `pal` (JASC-PAL/Aseprite).
`ImportLibrary { name, format, text }` adds/selects a custom palette;
`ExportLibrary { format }` returns `PaletteOutcome::export`, a
`PaletteExport { format, text }`. The host owns choosing paths, reading and writing.
The plugin also exposes pure `import`, `export`, `from_hex`, `from_gpl`, `from_pal`
and `to_pal` functions.

All imports accept at most 64 KiB and 1..=256 colors, retaining duplicate RGBs and
source display order. GPL retains representable swatch names. HEX and JASC-PAL
omit names. No interchange format stores stable slot IDs. PAL requires the
`JASC-PAL` header, version `0100`, an exact bounded color count and three RGB bytes
per line. The optional fourth alpha column used by
[Aseprite](https://github.com/aseprite/aseprite/blob/main/src/doc/file/pal_file.cpp)
is accepted only at 255; extra columns and transparency reject. CRLF, surrounding
whitespace and blank color lines are accepted. The exporter emits three columns.

Existing HEX/GPL registry exporters remain owned by `voxelite-plugin-exporters`.
The palette plugin registers `dev.voxelite.palette.pal` as a stateless PAL exporter.
Disabling the palette plugin removes that export and manager formats. MCP's
`palette_import` and `palette_export` accept `pal` alongside `hex` and `gpl`.
Document imports still append into free slots and return source-to-document slot
mappings, without recoloring existing geometry or writing local library bytes.
