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
Projects must be downloaded to preserve edits across reloads. Browsers warn
before leaving a page with unsaved edits. PNG export uses the current camera
at 1024×768, within the renderer’s bounded image budget.

## Browser verification

```sh
npm ci
node scripts/capture-web.mjs
```

The capture script uses Playwright Chromium at 1440×900 and 1× scale. It tests
WebGPU, forced WebGL2, and automatic WebGL2 fallback with `navigator.gpu`
unavailable. Each run captures dark mode, clicks the real appearance control,
and captures light mode. WebGPU uses Vulkan/SwiftShader:
`--enable-unsafe-webgpu --use-angle=vulkan --use-vulkan=swiftshader
--enable-features=Vulkan --disable-vulkan-surface`. WebGL2 uses
`--use-angle=swiftshader --enable-unsafe-swiftshader`.

Using only `--use-angle=swiftshader` for WebGPU caused a Chromium software-device
loss in this environment; the Vulkan flags resolve it without application or
GPUI patches. This is a headless test configuration, not a requirement for users
with working browser graphics.

The script verifies project downloads, GLB/PNG signatures, upload/save equality,
and preservation of the open project after a malformed upload. It rejects
browser errors and blank captures. Results and browser version are recorded in
`screenshots/web-verification.json`; captures are `screenshots/web-{dark,light}.png`
plus `-webgl` and `-fallback` variants, all ignored by Git. Inspect captures as
well as test results. Historical evidence is in the
[private archive](https://github.com/0x63616c/quantum-archive).

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
  modules.
- Added a wasm library entry, shared GPUI views, matching GPUI web platform,
  bounded uploads, browser downloads and settings storage. Native MCP and
  filesystem persistence are gated out of the web dependency graph.
- Added reproducible build tooling and backend/theme query switches.
- Optimized wasm bundle builds on stable Rust 1.99.1 with wasm-bindgen 0.2.129;
  no upstream patches required. Native build and 110 workspace tests pass.
- Browser checks cover WebGPU, WebGL2 and automatic fallback, real theme toggling,
  save, GLB/PNG export, canonical upload/save round trips and invalid uploads.
- Fixed PNG dimensions after the browser smoke caught the renderer budget error;
  added unsaved-edit navigation protection and removed redundant empty history.
- Native and wasm Clippy run with warnings denied; rustfmt and diff checks pass.
  Dark/light screenshots were inspected for artwork, typography and layout.
