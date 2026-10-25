# Voxelite agent skills

Four portable workflows for the implemented `voxelite.v1` MCP server:

| Skill | Use |
| --- | --- |
| [voxelite-workflow](voxelite-workflow/SKILL.md) | Complete creation → inspection → save/export loop |
| [voxelite-palettes](voxelite-palettes/SKILL.md) | Material roles, exact presets and stable slot mapping |
| [voxelite-primitives](voxelite-primitives/SKILL.md) | Box blockouts, symmetry and computed curved shapes |
| [voxelite-render-review](voxelite-render-review/SKILL.md) | Real four-angle PNG critique and verified repairs |

Connect MCP separately using [the integration guide](../integrations/README.md).
Installing instructions does not install or start the server. Browser editing
currently has no live MCP attachment. These skills do not grant permissions or
change host settings. No host plugin package is shipped in this stage.

## Claude Code

Copy a skill folder to `~/.claude/skills/` (personal) or `.claude/skills/` (project).
For example, from this repository:

```sh
mkdir -p ~/.claude/skills
cp -R skills/voxelite-workflow ~/.claude/skills/
```

Repeat for the other chosen folders. Invoke `/voxelite-workflow` or ask Claude
to use it. A plugin can bundle these folders under `skills/`, but requires its
own manifest and installation; copying files here does not install a plugin.
[Official Claude Code skill documentation](https://code.claude.com/docs/en/skills).

## Cursor

Copy chosen folders into `.cursor/skills/` in the project, or `~/.cursor/skills/`
for local personal use. Confirm them in Customize → Skills. For installations
using rules instead, add a scoped `.cursor/rules/voxelite.mdc` that instructs the
agent to read the relevant `skills/.../SKILL.md` from this repository for voxel
work. A rule file and an automatically discovered skill are different mechanisms;
do not simply rename SKILL.md to a rule file.
[Official Cursor skills](https://cursor.com/docs/skills).

## Codex

For current Codex, use `.agents/skills/` in the repository or `~/.agents/skills/`
for personal skills, as specified by the current official documentation. Some
Codex environments (including this build environment) expose skills under
`~/.codex/skills/`; use that path only if your installed client discovers it.
Confirm the skill appears after restarting; don't install duplicate copies in
multiple search roots. [Official skill locations](https://learn.chatgpt.com/docs/build-skills).

Alternatively, keep `skills/` in the repository and add a scoped instruction to
the project's existing `AGENTS.md` without replacing its other instructions:

> For Voxelite art tasks, read skills/voxelite-workflow/SKILL.md. Read the palette,
> primitive, or render-review skill when that workflow is needed.

[Official AGENTS.md guidance](https://learn.chatgpt.com/docs/agent-configuration/agents-md).

## Verify, update, remove

Ask the host to use the workflow, list MCP tools, and call
`voxelite_capabilities_get`. Verify API, epoch and CPU render availability.
Try the included example request in a new 32³ project, inspect four real PNGs,
and save. Host UI loading is not tested on this headless machine; stdio execution
and the tutorial are independently verified by repository tests/scripts.

Before copying over a locally customized skill, compare it with the new source.
To uninstall, remove only the copied `voxelite-*` skill folders and any specific
rule/AGENTS.md line you added. Remove the MCP host entry separately if desired.

Skills require no credentials or personal filesystem paths. The static site
bundles these exact source files for download; edit this directory, then run
`scripts/build-site.sh --skip-web` to refresh the published copy.
