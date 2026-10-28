# Agent skills

Teach the workflow once. Keep the creative loop moving.

These portable skills teach agents how to use the implemented Quantum MCP tools. They cover atomic edits, palette choices, primitive composition, real image inspection and recovery from conflicts.

## Install MCP

[Install Quantum for Mac](https://quantum.worldwidewebb.co/download/) in Applications first. The app includes the MCP server. Open your project folder in your host, then choose an install method below. For [source builds](https://quantum.worldwidewebb.co/docs/getting-started/#build) or a different app location, use the manual configuration with your executable's absolute path.

### Cursor

[Add to Cursor](cursor://anysphere.cursor-deeplink/mcp/install?name=quantum&config=eyJjb21tYW5kIjoiL0FwcGxpY2F0aW9ucy9RdWFudHVtLmFwcC9Db250ZW50cy9NYWNPUy9xdWFudHVtLW1jcCIsImFyZ3MiOlsiLS1yb290IiwiJHt3b3Jrc3BhY2VGb2xkZXJ9Il19)

Review and confirm the server in Cursor. The file root is `${workspaceFolder}`, your open project folder. [View mcp.json](https://quantum.worldwidewebb.co/assets/integrations/cursor/mcp.json) for manual setup.

### Claude Code

Run from your project directory; `"$PWD"` sets the file root to that directory.

```sh
claude mcp add quantum -- '/Applications/Quantum.app/Contents/MacOS/quantum-mcp' --root "$PWD"
```

To make the server available across projects, add `--scope user` after `claude mcp add`. Its file root still points to the directory where you ran the command.

### Codex

Run from your project directory; `"$PWD"` sets the file root to that directory.

```sh
codex mcp add quantum -- '/Applications/Quantum.app/Contents/MacOS/quantum-mcp' --root "$PWD"
```

Or merge this into `~/.codex/config.toml` (or your trusted project's `.codex/config.toml`). Replace `<project dir>` with an existing absolute path.

```toml
[mcp_servers.quantum]
command = "/Applications/Quantum.app/Contents/MacOS/quantum-mcp"
args = ["--root","<project dir>"]
```

### VS Code

[Install in VS Code](vscode:mcp/install?%7B%22name%22%3A%22quantum%22%2C%22command%22%3A%22%2FApplications%2FQuantum.app%2FContents%2FMacOS%2Fquantum-mcp%22%2C%22args%22%3A%5B%22--root%22%2C%22%24%7BworkspaceFolder%7D%22%5D%7D)

Review and confirm the server in VS Code, or run this CLI command. Both use `${workspaceFolder}` for the file root, resolved by VS Code in your open project.

```sh
code --add-mcp '{"name":"quantum","command":"/Applications/Quantum.app/Contents/MacOS/quantum-mcp","args":["--root","${workspaceFolder}"]}'
```

[View .vscode/mcp.json](https://quantum.worldwidewebb.co/assets/integrations/vscode/mcp.json) for manual setup.

### Other

For Claude Desktop, Windsurf and other MCP hosts, merge this entry into the host's MCP settings. Preserve existing servers and replace `<project dir>` with an existing absolute path.

```json
{
  "mcpServers": {
    "quantum": {
      "command": "/Applications/Quantum.app/Contents/MacOS/quantum-mcp",
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
npx skills add 0x63616c/quantum
```

Add `--skill <name>` for one skill, `-g` for a global install, or `-a claude-code`, `-a cursor` or `-a codex` to pick an agent. [Choose an individual skill](https://quantum.worldwidewebb.co/docs/skills/#available-skills).

For a manual install, [download the skill folders](https://quantum.worldwidewebb.co/assets/quantum-skills.zip) or use a repository checkout. Copy each `skills/<name>/` folder into `.cursor/skills/`, `.claude/skills/`, or `~/.codex/skills/`. [Install, update and removal notes](https://quantum.worldwidewebb.co/assets/skills/README.md).

## What you get

MCP gives your agent voxel editing tools, palettes, real PNG renders, and project save/export. Skills teach it to compose, inspect and refine a model. Each standalone server owns its own session; save `.voxl` files before disconnecting. To edit the live native canvas, [copy the connection configuration from the app](https://quantum.worldwidewebb.co/docs/getting-started/#connect).

## Verify it works

1.  Restart or reconnect your host and list its MCP tools.
2.  Call `quantum_capabilities_get` with `{}`. Expect API `voxelite.v1`, an epoch and available document IDs.
3.  Confirm the installed skills appear in your host, then ask it to use `quantum-workflow` for your first model.

## Choose a skill

### quantum-palettes

Choose, create or apply Quantum palettes with readable material roles and stable slot mappings, without unintended recoloring.

```sh
npx skills add 0x63616c/quantum --skill quantum-palettes
```

[Read skill instructions](https://quantum.worldwidewebb.co/assets/skills/quantum-palettes/SKILL.md)

### quantum-primitives

Build Quantum subjects from integer primitive masses, mirrored boxes and bounded explicit voxel batches for curved shapes.

```sh
npx skills add 0x63616c/quantum --skill quantum-primitives
```

[Read skill instructions](https://quantum.worldwidewebb.co/assets/skills/quantum-primitives/SKILL.md)

### quantum-render-review

Inspect real Quantum MCP renders from multiple angles, identify spatial flaws, and verify repairs at a consistent document revision.

```sh
npx skills add 0x63616c/quantum --skill quantum-render-review
```

[Read skill instructions](https://quantum.worldwidewebb.co/assets/skills/quantum-render-review/SKILL.md)

### quantum-workflow

Create or refine voxel art through Quantum MCP, with atomic edits, visual review, named versions and portable exports.

```sh
npx skills add 0x63616c/quantum --skill quantum-workflow
```

[Read skill instructions](https://quantum.worldwidewebb.co/assets/skills/quantum-workflow/SKILL.md)

## Give it a first brief

> Use the Quantum workflow to build a warm brass lantern within 32 × 32 × 32 cells. Save a blockout version, inspect iso/front/right/back renders, repair the handle silhouette, then save .voxl and GLB.

The render-review skill uses `quantum_render` for real PNG feedback. There is no screenshot-named MCP tool in this build. Primitive guidance also stays within today's catalog: boxes and explicit voxel batches, without invented cylinder or ellipsoid calls.

## Know when it worked

The agent should report the model revision, views it actually inspected, and saved output paths. A successful render call is not visual inspection, and exporting an image is not saving a project. These distinctions are part of every workflow.
