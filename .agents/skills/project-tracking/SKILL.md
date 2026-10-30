---
name: project-tracking
description: Use at the start of any task in this repository, whenever new work is requested, before opening a PR, and before saying anything is done. Keeps work tracked on GitHub issues and the Quantum project board instead of in memory.
---

# Project tracking

GitHub is the source of truth for what is planned, in progress and done. Memory
and chat history are not. Follow [CONTRIBUTING.md](../../../CONTRIBUTING.md).

## 1. Read the board first

Before planning or coding, list open issues and board state:

```sh
gh issue list --repo 0x63616c/voxelite --state open --limit 200
gh pr list --repo 0x63616c/voxelite --state open
gh project item-list <project-number> --owner 0x63616c --limit 200   # needs the `project` scope
```

Note what is In progress or In review so you do not duplicate it, and pick work
from **Next** before **Backlog**. Leave **Later** items alone unless asked.

## 2. One issue per ask

- Every request, idea, bug or promise becomes an issue in the same turn, using a
  template (Feature, Bug, Refactor, Spike) with **Why** and a **Done when**
  checklist of observable outcomes.
- Search first (`gh issue list --search "<words>" --state all`); update an existing
  issue instead of filing a duplicate.
- Label it: one `type:*`, one or more `area:*`, one `priority:*`; `status:blocked`
  plus a comment when blocked; `later` for parked work. Set a milestone and add it
  to the board.
- Split anything that would need more than one focused PR into several issues.

## 3. Link every PR

- Branch `<type>/<issue#>-<slug>`.
- Conventional Commit PR title: `feat|fix|refactor|chore|docs|test|ci|perf(scope): summary`.
- PR body starts with `Closes #N` (or `Refs #N` for partial work), then summary,
  test evidence, and before/after screenshots for UI or numbers for performance.
- Move the card to **In progress** when you start and **In review** when the PR
  opens. Merging closes the issue and moves it to **Done**.

## 4. Audit before claiming done

Before telling anyone work is done:

1. Open the issue and re-read every **Done when** item.
2. Check each item against the merged diff, CI results and attached evidence, not
   against memory or a summary from another agent.
3. Tick only items that are actually met. File anything left over as a new issue
   and link it.
4. Confirm the issue is closed and the board shows **Done**.

If any item is unmet, say so plainly and say what is left.
