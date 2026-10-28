# Connect an agent

Install @@product@@ for Mac in `/Applications`. The app bundles its MCP executable at
`{command}`. For a source build, run `cargo build --release --locked -p quantum-mcp`
and replace the command path with your built binary's absolute path.

[Add to Cursor]({cursor_link}) · [Install in VS Code]({vscode_link})

Open your project folder before installing. Cursor and VS Code resolve
`${{workspaceFolder}}` to that folder. Review and confirm the server in your host.

For Claude Code and Codex, run from your project directory. `"$PWD"` becomes the
absolute file root when the shell runs the command:

```sh
{claude_command}
{codex_command}
```

Claude Code also accepts `--scope user` after `claude mcp add` to make the server
available across projects; its file root still points to the original directory.

VS Code CLI (the quoted workspace variable is resolved by VS Code):

```sh
{vscode_command}
```

| Host | Manual configuration |
| --- | --- |
| Cursor | Merge [cursor/mcp.json](cursor/mcp.json) into `.cursor/mcp.json`. |
| VS Code | Merge [vscode/mcp.json](vscode/mcp.json) into `.vscode/mcp.json`. |
| Claude Code | Replace `<project dir>` in [claude/mcp.json](claude/mcp.json), then merge into `.mcp.json`. |
| Codex | Replace `<project dir>` in [codex/config.toml](codex/config.toml), then merge into `~/.codex/config.toml` or a trusted project's `.codex/config.toml`. |
| Claude Desktop / Windsurf / Other | Replace `<project dir>` in [other/mcp.json](other/mcp.json), then merge into your host's MCP settings. |

Preserve existing host settings. The file root must be an existing absolute
directory. File tools accept relative paths inside it. Each standalone process
owns its own session; save `.voxl` files before disconnecting. For the live native
canvas, use **Copy MCP configuration** in the app's Agent connection panel instead.

Restart/reconnect the host, list tools and call `quantum_capabilities_get` with
`{{}}`. Expect API `quantum.v1`, an epoch and available document IDs. Then ask:

> Create a 32³ scene with an atomic batch. Inspect four real renders, refine it,
> and save first-scene.voxl. Use returned epoch/revisions for edits.

To remove the integration, remove only the `quantum` entry/table and reconnect.
Install workflow instructions separately using [the skills guide](../skills/README.md).
Host application launches and macOS signing require verification on those hosts.

These examples are generated from `crates/quantum-site/src/install.rs` and this
README's template in `crates/quantum-site/templates/integrations.md`.
Run `scripts/build-site.sh` to refresh them.
