# `.voxl` 1.0

A project is one UTF-8 JSON object, MIME `application/vnd.voxelite.project+json`.
`schema.json` describes structure; the Rust validator additionally checks references,
coordinate uniqueness, order permutations, budgets and snapshot hashes.

Canonical output uses lexicographically sorted object keys, two-space indentation,
LF, one final newline, and no trailing whitespace. Nonempty objects and arrays of
objects/arrays expand one level per line. Scalar vectors are inline, separated by
comma-space; voxel tuples occupy one line each, sorted numerically x/y/z. Empty
arrays/objects are `[]`/`{}`. Slot records sort by index; comments and versions sort
by UUID. Layer, palette display, and message order are preserved.

Strings preserve Unicode without normalization. Quotes and backslashes are escaped;
U+0000–001F use lowercase `\u00xx`; other characters and slashes remain literal.
Integers use shortest decimal notation. Coordinates reject fractions, exponents,
overflow and negative zero. IDs are lowercase hyphenated UUIDs. Empty comments and
versions and absent optional fields are omitted. Unknown optional fields remain
inert and round-trip, including nested object metadata. Unsupported required
features and majors reject; compatible newer minors preserve their declaration.

SHA-256 semantic hashes cover canonical bytes excluding `view` and `build_history`.
Version hashes cover the canonical nonrecursive core snapshot, including
snapshot-level inert fields. No wall-clock timestamps, host identities, session
revisions, or raw undo logs are generated.
Portable view floats use serde_json's shortest round-trip representation, normalized
zero on input. Views are hints; hosts validate camera usability before applying them.

The parser streams voxel arrays directly into staged chunks, retaining at most 8 MiB
of metadata, 32 levels of JSON nesting and 16 levels of extension nesting. The file
cap is 128 MiB. There is an additional conservative admission limit of 4,194,304
aggregate decoded layer-voxels across the live model and versions. Each snapshot
also meets the model limits. Version canonical bytes are capped at 64 MiB. The
writer streams tuples and stages one sorted layer at a time. Writers may fail after
partial output: filesystem callers must use a temporary file and atomic replacement.
Durable filesystem save/recovery belongs to the session layer, outside this crate.

The format has no predecessor to migrate. Future major migrations must be explicit
pure transformations retaining original files; no lossy downgrade is attempted.
`fixtures/populated.input.voxl` is the compact specification input and
`fixtures/populated.canonical.voxl` freezes exact canonical bytes. Tests also cover
empty defaults, comments, versions, unknown fields, malformed bytes and randomized
round trips.

## Optional build history

`build_history` is a versioned command recipe for projects built from an empty
`Document::new(document.id, base_layer)` scene. Version 1 stores `base_layer` and
1–64 `steps`, each with a stable `id`, human-readable `label`, `origin`, logical
`elapsed_ms`, and a typed `voxelite-core::Command`. Commands use snake-case external
tags; `set_voxels.voxels` uses `[x, y, z, slot]` tuples (zero erases). Optional command
arguments are explicit nulls. Unknown fields inside recipes and noncanonical UUIDs
are rejected; recipe schema changes require a new recipe version.

Times are strictly increasing relative build offsets, not dates or claims about
actual authoring time. Loading executes every command against a private engine,
rejects no-ops, applies core work/history budgets and an aggregate 4,194,304
work-unit cap (4 × MAX_WORK), and compares the resulting **entire document** with
the file, including named versions. `Project::open_engine` replays the recipe into
the host's undo/scrub timeline and marks the result saved. Files without a recipe
retain the existing empty-history behavior. Recipe data is subject to the normal
metadata and file byte limits. The full recipe must fit the default 128 MiB history
budget; hosts with smaller budgets may evict its earliest entries normally.

This optional recipe describes the saved build, not subsequent interactive edits.
An unchanged save retains it. Saving changed content omits the stale recipe;
ordinary interactive undo/redo remains session-local. Recipe omission must not be
undone by merging retained optional metadata back into the output. Older readers
can still display the live scene without understanding its build recipe.
