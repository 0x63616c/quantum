# Getting started

Install Quantum. Connect your agent. Make your first model.

- [Download for Mac](https://voxelite.worldwidewebb.co/download/)
- [Connect MCP](https://voxelite.worldwidewebb.co/docs/getting-started/#connect)

## Build from source

Install Rust with rustup, then run from a repository checkout. The pinned toolchain is Rust 1.99.1.

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --locked --release \
  -p voxelite-app -p voxelite-mcp
./target/release/voxelite
```

[Download the native Mac app](https://voxelite.worldwidewebb.co/download/) for macOS 13+ on Apple silicon, with installation instructions. The app is Developer ID signed and notarized by Apple, and checks for updates automatically from version 0.3.0. Homebrew packages are not available yet. On macOS, install the Xcode command line tools (`xcode-select --install`) before building. This native target is wired through GPUI, but has not been built on this Linux machine. Linux native compilation and capture are verified with X11/Xvfb and Mesa software rendering.

Linux build and desktop dependencies (Debian / Ubuntu)

```sh
sudo apt-get install build-essential pkg-config clang libclang-dev \
  libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libvulkan-dev \
  libx11-xcb-dev libxcb1-dev libfontconfig-dev libasound2-dev libssl-dev \
  mesa-vulkan-drivers dbus-x11 xdg-desktop-portal xdg-desktop-portal-gtk
```

A display server and a desktop portal are needed for native windows and file dialogs. Headless MCP builds need no display or GPU:

```sh
cargo build --locked --release -p voxelite-mcp
cargo test -p voxelite-app --no-default-features
```

## Build the website

```sh
scripts/build-site.sh
python3 -m http.server 8090 --directory site --bind 127.0.0.1
```

Open `http://127.0.0.1:8090`. The script generates the static website, documentation and downloadable agent instructions from repository sources.

## Make your first edit by hand

The desktop welcome window shows recent projects with previews. Use Tab to focus Recent, then Up/Down and Enter to open a project. Window → Welcome reopens it; File → Open Recent offers the same history. Clear Recent Projects removes the list without deleting your files.

1.  On desktop, choose New Project or Open… in the welcome window, or double-click a recent project. To explore Autumn Workshop, open `examples/autumn-workshop.voxl`. Right-drag to orbit; wheel to zoom; press F to frame.
2.  Select a visible, unlocked layer and a palette swatch. Press B to place on a face, E to erase, or P to paint.
3.  Use Ctrl/Cmd + Z to undo. Ctrl/Cmd + K opens command search.
4.  Choose File → Save As for your own `.voxl`. Export GLB or PNG through the File menu or command search.

## Connect your agent

The Mac app includes the MCP server. For source builds, build `voxelite-mcp` first. Each standalone process owns its own in-memory documents.

### Headless: create and save independently

[Install MCP in Cursor, Claude Code, Codex, VS Code or another host](https://voxelite.worldwidewebb.co/docs/mcp/#install-mcp). The guide includes one-click links, CLI commands and manual configuration. Use an existing project directory as the file root, restart your host, list tools and call `voxelite_capabilities_get`; expect API `voxelite.v1`. [Install agent skills](https://voxelite.worldwidewebb.co/docs/skills/#install-skills) to teach the creative workflow.

### Native: work in the same document

```sh
./target/release/voxelite examples/autumn-workshop.voxl --session-socket /tmp/voxelite-session.sock
```

Use the MCP executable from the install guide, replacing its arguments with `["--connect", "/tmp/voxelite-session.sock"]`. The socket path must not already exist. Or copy the configuration from Agent → Agent connection in the editor. Discover the existing model with capabilities; do not create a second document.

Watch committed calls in Session, pause new writes, and undo or edit by hand. The attached catalog excludes document create/open/save: the native editor owns file lifecycle. Save through the app.

## Your first model through MCP

In standalone mode, ask your connected agent:

> Build a 32 × 32 × 32 clay tower using one atomic box batch. Discover the epoch first. Inspect iso, front, right, and back PNGs, then save first-tower.voxl and export first-tower.glb. Use returned IDs and revisions for each write.

A minimal call sequence follows. These are tool argument objects, not full JSON-RPC messages. Replace placeholders with returned values, and generate a fresh UUID for each new mutation.

### 1\. Discover, then create

Call `voxelite_capabilities_get` with `{}`, then `voxelite_document_create`:

```json
{
  "name": "First tower",
  "bounds": {"min": [0, 0, 0], "max": [32, 32, 32]},
  "expected_epoch": "<returned epoch>",
  "operation_id": "<fresh UUID>"
}
```

### 2\. Batch the blockout

Call `voxelite_voxels_batch`. A new project has a Base layer and clay at slot 1.

```json
{
  "model_id": "<returned model_id>",
  "expected_epoch": "<returned epoch>",
  "expected_revision": "<current revision>",
  "operation_id": "<fresh UUID>",
  "label": "Block out tower",
  "operations": [
    {"kind": "box", "layer": "Base", "slot": 1,
     "region": {"min": [10, 0, 10], "max": [22, 2, 22]}},
    {"kind": "box", "layer": "Base", "slot": 1,
     "region": {"min": [13, 2, 13], "max": [19, 18, 19]}},
    {"kind": "box", "layer": "Base", "slot": 1,
     "region": {"min": [11, 18, 11], "max": [21, 20, 21]}}
  ]
}
```

### 3\. Inspect real pixels

Call `voxelite_render` and inspect the returned image block:

```json
{
  "model_id": "<returned model_id>",
  "views": ["iso", "front", "right", "back"],
  "width": 512, "height": 512,
  "image_mode": "contact_sheet",
  "revision": "<current revision>"
}
```

### 4\. Save the result

Call `voxelite_document_save` with the latest captured revision:

```json
{
  "model_id": "<returned model_id>",
  "expected_epoch": "<returned epoch>",
  "expected_revision": "<current revision>",
  "operation_id": "<fresh UUID>",
  "path": "first-tower.voxl"
}
```

Then call [voxelite\_export\_glb](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_export_glb). Exports use `revision`, without mutation guards or operation IDs:

```json
{
  "model_id": "<returned model_id>",
  "revision": "<current revision>",
  "path": "first-tower.glb"
}
```

Save before disconnecting: edits and undo history live in memory. Existing export paths reject; choose a fresh output name.

## When a call fails

Check `isError` and the structured envelope's `ok` flag. On a revision conflict, read current state and rebase. On a lost response, retry only the identical request with the same operation ID in the same session. Never guess whether an edit committed. [Read the error guide](https://voxelite.worldwidewebb.co/docs/mcp/#errors) before retrying.

## Crash reports and feedback privacy

Quantum has no analytics, telemetry or third-party crash service. A Rust panic writes a local report under `~/Library/Application Support/voxelite/crashes/` on macOS (on Linux, the app data directory under `$XDG_DATA_HOME/voxelite` or `~/.local/share/voxelite`). Reports contain the app version, operating-system name/version, architecture, panic message, source location, backtrace, UTC timestamp and up to 20 recent warning/error log lines. Home-directory prefixes become `~` and email-like text is redacted before storage and again before preview. Other paths, filenames or details in diagnostics may remain: review the complete preview. Scene contents and document metadata are never collected by this flow. Fatal signals, aborts, stack overflow and allocation failures may not produce a report.

On the next launch, including Welcome, you can review the full report, add context, keep it for later, copy or save the redacted preview, or delete the local report. Nothing is sent automatically. **Help → Send feedback…** uses the same preview and confirmation flow for bug reports or feature requests, with version and OS information. Editing your description or report kind invalidates the preview until you update it.

**Continue to GitHub…** opens a prefilled issue form in your browser at `github.com/0x63616c/voxelite`, using the Bug or Feature template and matching label. This sends the preview to GitHub as part of the URL (and may leave it in browser history). A GitHub account and a separate Submit action there are required to publish the issue. Issues are public. No relay endpoint is configured, so private email contact and scene attachments are unavailable; there is no app token. For a report too large for a browser link, copy or save the full preview and paste it into a new issue yourself. Opening the browser does not prove submission, so crash reports remain local until you explicitly delete them. Pending reports have no automatic expiry; delete them in the prompt or remove their local files.
