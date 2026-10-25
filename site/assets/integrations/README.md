# Connect an agent

Build once from the repository root:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release -p voxelite-mcp
```

Use an absolute executable path so GUI-launched hosts do not depend on a shell's
PATH. `--root` must name an existing project directory. File tools accept paths
relative to that directory. Each launched process owns its own editing session;
save `.voxl` files before disconnecting.

| Host | Example | Installation |
| --- | --- | --- |
| Cursor | [cursor/mcp.json](cursor/mcp.json) | Merge `mcpServers.voxelite` into `.cursor/mcp.json` in this repository. The example uses Cursor's `${workspaceFolder}` interpolation. For a different project, replace the executable path with the built binary's absolute path. |
| Claude Code | [claude/mcp.json](claude/mcp.json) | Replace both `/absolute/path/to/...` values, then merge the entry into the project's `.mcp.json`. |
| Codex | [codex/config.toml](codex/config.toml) | Replace both absolute paths, then merge the table into `~/.codex/config.toml` or a trusted project's `.codex/config.toml`. |

Keep existing host settings when merging. Restart/reconnect the host, list tools,
and call `voxelite_capabilities_get`. It should report `voxelite.v1`, a CPU
renderer, an epoch, and an empty document list. Then ask the agent:

> Create a 32³ voxel scene using a small atomic batch. Render it from four angles,
> inspect the silhouettes, add an anchored comment, save a named version, and save
> the project as `first-scene.voxl`. Use returned epoch/revisions for edits.

To remove the integration, remove only the `voxelite` entry/table and reconnect.
These are configuration examples; no host plugin, marketplace installation, or
runtime editor plugin is installed by this stage. The Rust process tests exercise
both protocol profiles; the host applications themselves were not available on
the Linux build machine.

Configuration fields were checked against the official [Cursor MCP docs](https://cursor.com/docs/mcp),
[Claude Code MCP docs](https://code.claude.com/docs/en/mcp), and
[Codex MCP docs](https://developers.openai.com/codex/mcp) on 2026-10-25.
