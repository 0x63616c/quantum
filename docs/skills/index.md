# Agent skills

Teach the workflow once. Keep the creative loop moving.

These portable skills teach agents how to use the implemented Quantum MCP tools. They cover atomic edits, palette choices, primitive composition, real image inspection and recovery from conflicts.

## Install MCP

[Install Quantum for Mac](https://voxelite.worldwidewebb.co/download/) in Applications first. The app includes the MCP server. Open your project folder in your host, then choose an install method below. For [source builds](https://voxelite.worldwidewebb.co/docs/getting-started/#build) or a different app location, use the manual configuration with your executable's absolute path.

### Cursor

[Add to Cursor](cursor://anysphere.cursor-deeplink/mcp/install?name=voxelite&config=eyJjb21tYW5kIjoiL0FwcGxpY2F0aW9ucy9Wb3hlbGl0ZS5hcHAvQ29udGVudHMvTWFjT1Mvdm94ZWxpdGUtbWNwIiwiYXJncyI6WyItLXJvb3QiLCIke3dvcmtzcGFjZUZvbGRlcn0iXX0=)

Review and confirm the server in Cursor. The file root is `${workspaceFolder}`, your open project folder. [View mcp.json](https://voxelite.worldwidewebb.co/assets/integrations/cursor/mcp.json) for manual setup.

### Claude Code

Run from your project directory; `"$PWD"` sets the file root to that directory.

```sh
claude mcp add voxelite -- '/Applications/Voxelite.app/Contents/MacOS/voxelite-mcp' --root "$PWD"
```

To make the server available across projects, add `--scope user` after `claude mcp add`. Its file root still points to the directory where you ran the command.

### Codex

Run from your project directory; `"$PWD"` sets the file root to that directory.

```sh
codex mcp add voxelite -- '/Applications/Voxelite.app/Contents/MacOS/voxelite-mcp' --root "$PWD"
```

Or merge this into `~/.codex/config.toml` (or your trusted project's `.codex/config.toml`). Replace `<project dir>` with an existing absolute path.

```toml
[mcp_servers.voxelite]
command = "/Applications/Voxelite.app/Contents/MacOS/voxelite-mcp"
args = ["--root","<project dir>"]
```

### VS Code

[Install in VS Code](vscode:mcp/install?%7B%22name%22%3A%22voxelite%22%2C%22command%22%3A%22%2FApplications%2FVoxelite.app%2FContents%2FMacOS%2Fvoxelite-mcp%22%2C%22args%22%3A%5B%22--root%22%2C%22%24%7BworkspaceFolder%7D%22%5D%7D)

Review and confirm the server in VS Code, or run this CLI command. Both use `${workspaceFolder}` for the file root, resolved by VS Code in your open project.

```sh
code --add-mcp '{"name":"voxelite","command":"/Applications/Voxelite.app/Contents/MacOS/voxelite-mcp","args":["--root","${workspaceFolder}"]}'
```

[View .vscode/mcp.json](https://voxelite.worldwidewebb.co/assets/integrations/vscode/mcp.json) for manual setup.

### Other

For Claude Desktop, Windsurf and other MCP hosts, merge this entry into the host's MCP settings. Preserve existing servers and replace `<project dir>` with an existing absolute path.

```json
{
  "mcpServers": {
    "voxelite": {
      "command": "/Applications/Voxelite.app/Contents/MacOS/voxelite-mcp",
      "args": [
        "--root",
        "<project dir>"
      ]
    }
  }
}
```

## Install skills

Install all skills from the repository's `skills/` directory with the open Agent Skills CLI. This installs workflow instructions; configure MCP above to give them tools.

```sh
npx skills add 0x63616c/voxelite
```

Add `--skill <name>` for one skill, `-g` for a global install, or `-a claude-code`, `-a cursor` or `-a codex` to pick an agent. [Choose an individual skill](https://voxelite.worldwidewebb.co/docs/skills/#available-skills).

For a manual install, [download the skill folders](https://voxelite.worldwidewebb.co/assets/voxelite-skills.zip) or use a repository checkout. Copy each `skills/<name>/` folder into `.cursor/skills/`, `.claude/skills/`, or `~/.codex/skills/`. [Install, update and removal notes](https://voxelite.worldwidewebb.co/assets/skills/README.md).

## What you get

MCP gives your agent voxel editing tools, palettes, real PNG renders, and project save/export. Skills teach it to compose, inspect and refine a model. Each standalone server owns its own session; save `.voxl` files before disconnecting. To edit the live native canvas, [copy the connection configuration from the app](https://voxelite.worldwidewebb.co/docs/getting-started/#connect).

## Verify it works

1.  Restart or reconnect your host and list its MCP tools.
2.  Call `voxelite_capabilities_get` with `{}`. Expect API `voxelite.v1`, an epoch and available document IDs.
3.  Confirm the installed skills appear in your host, then ask it to use `voxelite-workflow` for your first model.

## Choose a skill

### voxelite-palettes

Choose, create or apply Voxelite palettes with readable material roles and stable slot mappings, without unintended recoloring.

```sh
npx skills add 0x63616c/voxelite --skill voxelite-palettes
```

[Read skill instructions](https://voxelite.worldwidewebb.co/assets/skills/voxelite-palettes/SKILL.md)

### voxelite-primitives

Build Voxelite subjects from integer primitive masses, mirrored boxes and bounded explicit voxel batches for curved shapes.

```sh
npx skills add 0x63616c/voxelite --skill voxelite-primitives
```

[Read skill instructions](https://voxelite.worldwidewebb.co/assets/skills/voxelite-primitives/SKILL.md)

### voxelite-render-review

Inspect real Voxelite MCP renders from multiple angles, identify spatial flaws, and verify repairs at a consistent document revision.

```sh
npx skills add 0x63616c/voxelite --skill voxelite-render-review
```

[Read skill instructions](https://voxelite.worldwidewebb.co/assets/skills/voxelite-render-review/SKILL.md)

### voxelite-workflow

Create or refine voxel art through Voxelite MCP, with atomic edits, visual review, named versions and portable exports.

```sh
npx skills add 0x63616c/voxelite --skill voxelite-workflow
```

[Read skill instructions](https://voxelite.worldwidewebb.co/assets/skills/voxelite-workflow/SKILL.md)

## Give it a first brief

> Use the Quantum workflow to build a warm brass lantern within 32 × 32 × 32 cells. Save a blockout version, inspect iso/front/right/back renders, repair the handle silhouette, then save .voxl and GLB.

The render-review skill uses `voxelite_render` for real PNG feedback. There is no screenshot-named MCP tool in this build. Primitive guidance also stays within today's catalog: boxes and explicit voxel batches, without invented cylinder or ellipsoid calls.

## Know when it worked

The agent should report the model revision, views it actually inspected, and saved output paths. A successful render call is not visual inspection, and exporting an image is not saving a project. These distinctions are part of every workflow.
