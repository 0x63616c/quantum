# Viewport debug overlay

Toggle **Settings → Viewport / Rendering → Debug overlay**, or use **Toggle
debug overlay** in the command palette or View menu. The shortcut is
**Cmd+Shift+D** on macOS, **Ctrl+Shift+D** on Linux/Windows. The keyboard guide
lists it under Navigate. `show_debug_overlay` is persisted, defaults to false,
and is independent of the status bar's Render performance setting. Older settings
files remain valid.

The small monospace panel sits below the view label at the top left of the
viewport (below the read-only banner during preview). It uses theme colors with
a translucent background and has no hitbox or event handlers: orbit, pan, zoom
and editing still reach the viewport beneath it.

| Field | Meaning |
| --- | --- |
| FPS / ms | Existing FrameMeter's roughly one-second average of newly painted viewport intervals. Unchanged UI repaints do not count; the last average is held while idle. |
| CPU wall | Snapshot synchronization through GPU completion, excluding timestamp readback and device initialization. |
| CPU sync / upload | Separate occupancy and mesh synchronization and upload times. Camera-only frames have zero sync times. |
| CPU encode / submit+complete | Command setup and encoding (including IOSurface acquisition and NV12 encoding on Mac), followed by queue submission through completion. Completion includes worker scheduling/poll latency. |
| GPU main / resolve / shadow / total | Timestamped scene and resolve passes, optional fallback shadow redraw, and the span from first pass start to resolve end. Total is not the sum of pass times and excludes NV12. `n/a` means unsupported/unavailable timestamps, or no shadow-map redraw this frame; exact shadows run within main. |
| Chunks / bricks | Resident chunk vertex buffers and buffers updated/deleted this frame; occupied and allocated 8³ bricks in the CPU occupancy cache (also available during shadow fallback). |
| Draws / vertices / triangles | Main scene work, one fullscreen resolve draw, and extra fallback shadow draws. Mac's two NV12 conversion draws are excluded. |
| Memory | Resident vertex buffer capacity (including spare capacity), GPU occupancy bytes (zero on fallback), estimated render targets, and optional Metal device allocated bytes. Binary KiB/MiB units. |
| Uploaded | Occupancy, vertex and uniform bytes written this frame. |
| Target / shadow | Actual supersampled render dimensions and the selected `exact` or `fallback` shadow path. |

Target memory uses 80 bytes per surface pixel plus the fallback shadow texture.
It excludes NV12, diagnostic readback, driver alignment and CPU memory. The Metal
device allocation counter has a broader scope; do not add it to these estimates.
CPU viewport fallback keeps FPS/ms and shows renderer diagnostics as `n/a`.

The renderer exposes UI-independent `Worker::set_diagnostics(bool)` and
`last_stats()`; Headless offers the same controls for adapter tests. Changes take
effect on the next serialized render request. The app requests a frame on toggle,
discards stale diagnostic results after preference/document changes, and adds no
continuous render or polling loop. A request already in flight can finish with
its original setting. Disabling releases query resources on the worker and clears
the last stats. Disabled frames take no diagnostic clock samples, scan no resource
counts, allocate no diagnostic buffers, and perform no timestamp queries or
readbacks. Enabled diagnostics add measurement/readback overhead; they do not
change shaders, render targets, resolution, AA, AO or shadows.

Pure formatting and preference tests run with
`cargo test -p quantum-app --no-default-features --lib --locked`.
The opt-in `runtime_diagnostics_release_resources_and_preserve_pixels` renderer
test checks repeated toggles, resource release, cache reuse and exact RGBA equality;
on Linux run it with `--features headless-vulkan -- --ignored`.
The bench adapter test `diagnostics_preserve_autumn_pixels` checks the same pixel
contract on the autumn fixture. Golden comparisons must use `--delta 0
--max-delta 0 --max-pct 0`; see [the harness guide](rendering-harness.md).
Metal timestamps, NV12 presentation and the existing Apple M2 goldens still need
the macOS runner; Vulkan captures cannot establish Metal pixel equality.
