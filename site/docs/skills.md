# Agent skills

Teach the workflow once. Keep the creative loop moving.

These portable skills teach agents how to use the implemented Voxelite MCP tools. They cover atomic edits, palette choices, primitive composition, real image inspection and recovery from conflicts.

- [**Voxel art workflow**Discover, compose, inspect, refine and save.](https://voxelite.worldwidewebb.co/assets/skills/voxelite-workflow/SKILL.md)
- [**Palettes**Material roles, exact presets and stable slot mappings.](https://voxelite.worldwidewebb.co/assets/skills/voxelite-palettes/SKILL.md)
- [**Primitive shapes**Boxes, mirrored masses and computed curved forms.](https://voxelite.worldwidewebb.co/assets/skills/voxelite-primitives/SKILL.md)
- [**Render review**Four real views, concrete repairs and consistent evidence.](https://voxelite.worldwidewebb.co/assets/skills/voxelite-render-review/SKILL.md)

## Install in your host

[Download all four skills](https://voxelite.worldwidewebb.co/assets/voxelite-skills.zip), or copy their directories from the repository's `skills/` folder. [Full install, update and removal notes](https://voxelite.worldwidewebb.co/assets/skills/README.md).

| Host        | Location                                                                                                                          |
| ----------- | --------------------------------------------------------------------------------------------------------------------------------- |
| Claude Code | `~/.claude/skills/` or project `.claude/skills/`                                                                                  |
| Cursor      | `.cursor/skills/` or personal `~/.cursor/skills/`; scoped rules can point to source skills                                        |
| Codex       | Current docs: `.agents/skills/` or `~/.agents/skills/`. Some clients expose `~/.codex/skills/`; verify discovery for your client. |

Codex can also read these files through a scoped project `AGENTS.md` instruction. Installing a skill does not configure MCP: [connect the server separately](https://voxelite.worldwidewebb.co/docs/getting-started/#connect). No plugin marketplace package is included here.

Host guidance: [Claude Code](https://code.claude.com/docs/en/skills), [Cursor](https://cursor.com/docs/skills), [Codex skills](https://learn.chatgpt.com/docs/build-skills), and [AGENTS.md](https://learn.chatgpt.com/docs/agent-configuration/agents-md). Host application launches remain untested in this environment.

## Give it a first brief

> Use the Voxelite workflow to build a warm brass lantern within 32 × 32 × 32 cells. Save a blockout version, inspect iso/front/right/back renders, repair the handle silhouette, then save .voxl and GLB.

The render-review skill uses `voxelite_render` for real PNG feedback. There is no screenshot-named MCP tool in this build. Primitive guidance also stays within today's catalog: boxes and explicit voxel batches, without invented cylinder or ellipsoid calls.

## Know when it worked

The agent should report the model revision, views it actually inspected, and saved output paths. A successful render call is not visual inspection, and exporting an image is not saving a project. These distinctions are part of every workflow.
