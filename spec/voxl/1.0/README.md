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

SHA-256 semantic hashes cover canonical bytes excluding `view`. Version hashes cover
the canonical nonrecursive core snapshot, including snapshot-level inert fields.
No timestamps, host identities, session revisions, or undo logs are generated.
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
