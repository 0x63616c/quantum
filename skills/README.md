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

## Agent Skills CLI

Install all skills from this repository's `skills/` directory:

```sh
npx skills add 0x63616c/voxelite
```

Use `--skill <name>` to select one, `-g` for a global install, and
`-a claude-code`, `-a cursor` or `-a codex` to pick an agent. For example:

```sh
npx skills add 0x63616c/voxelite --skill voxelite-workflow -a cursor
```

## Manual installation

From a repository checkout or the extracted skills download, copy each complete
`skills/<name>/` folder into your host's skills directory:

| Host | Destination |
| --- | --- |
| Cursor | `.cursor/skills/` |
| Claude Code | `.claude/skills/` |
| Codex | `~/.codex/skills/` |

For example, from the repository root:

```sh
mkdir -p .cursor/skills
cp -R skills/voxelite-workflow .cursor/skills/
```

Repeat for the other chosen folders and restart your host to discover the skills.

## Verify, update, remove

Ask the host to use the workflow, list MCP tools, and call
`voxelite_capabilities_get`. Expect API `voxelite.v1`, an epoch and CPU render availability.
Try the included example request in a new 32³ project, inspect four real PNGs,
and save. Host UI loading is not tested on this headless machine; stdio execution
and the tutorial are independently verified by repository tests/scripts.

Before copying over a locally customized skill, compare it with the new source.
To uninstall, remove only the copied `voxelite-*` skill folders and any specific
rule/AGENTS.md line you added. Remove the MCP host entry separately if desired.

Skills require no credentials or personal filesystem paths. The static site
bundles these exact source files for download; edit this directory, then run
`scripts/build-site.sh --skip-web` to refresh the published copy.
