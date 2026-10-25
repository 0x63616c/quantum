# Browser editor

The browser compiles the existing Rust GPUI editor view from `voxelite-app`.
There is no separate HTML editor. The generated JavaScript only starts wasm;
file upload, download, storage and UI behavior are implemented in Rust.

## Build and run locally

```sh
export PATH="$HOME/.cargo/bin:$PATH"
scripts/build-web.sh
python3 -m http.server 8080 --directory web/dist --bind 127.0.0.1
```

Open <http://127.0.0.1:8080>. The script installs the wasm target and the
`wasm-bindgen-cli` version matching `Cargo.lock`, builds the optimized `web`
profile, and writes `index.html`, bindings and wasm to ignored `web/dist/`.
No nightly compiler, workers, SharedArrayBuffer or COOP/COEP headers are needed.
Use localhost or HTTPS for browser graphics APIs.

Both targets pin the same GPUI 1.25.3 distribution: `gpui-unofficial` and
`gpui-web-gpui-unofficial`. The packaged source identifies upstream commit
`8dd255c3eafb5231056fe4e6f601cedd011f1c93`. This includes the web platform
introduced by [Zed PR #50228](https://github.com/zed-industries/zed/pull/50228).
WebGPU is preferred; GPUI retries with a fresh WebGL2 canvas if unavailable.
`?backend=webgl` and `?backend=webgpu` force a backend for verification.
`?theme=light` selects light startup; the shared appearance action toggles either
way during editing. Default startup loads the bundled Autumn Workshop.

Open accepts a `.voxl` upload, bounded to the format's 128 MiB limit. Save and
Save As download canonical `.voxl`; the browser manages the destination. The
status confirms that download started, not that a file was written to disk.
The File menu exports GLB and PNG. Palette import/export uses browser files;
custom libraries and local share preview metadata use origin-local storage.
Projects must be downloaded to preserve edits across reloads.

## Missing on web

- Native Unix sockets and the MCP bridge; companion pairing is future work.
  The session panel explains that agent sessions require the desktop app.
- Background worker rendering: the current single-thread baseline renders CPU
  frames on the browser task queue. Large models can delay input.
- Project autosave/recovery and direct overwrite of an uploaded file.
- Hosted sharing, collaboration and runtime plugins (also outside v1 scope).
- Release-wide Safari/Firefox, accessibility and hardware GPU qualification.

## Progress notes

### 2026-10-25 — web burst 1

- Committed the plugin core deferral and retained built-in tools as internal
  modules. Left `design/` untouched.
- Added a wasm library entry, shared GPUI views, matching GPUI web platform,
  bounded uploads, browser downloads and settings storage. Native MCP and
  filesystem persistence are gated out of the web dependency graph.
- Added reproducible build tooling and backend/theme query switches.
- Linux workspace tests passed (110 tests); wasm Clippy passed with warnings
  denied. Optimized bundle generation and browser captures are in progress.
