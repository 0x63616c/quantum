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
gh issue list --repo 0x63616c/quantum --state open --limit 200
gh pr list --repo 0x63616c/quantum --state open
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

## 3. Model labels

Every open issue carries exactly one of `model:astra` or `model:sol`. Label new
issues when filing. Pick the model from the issue's label, not by habit.

- `model:astra`: hard work: UI/design-heavy tasks, complex refactors, spikes and
  ports of someone else's code. Run `codex exec -m gpt-6-astra -c model_reasoning_effort=xhigh`.
- `model:sol`: everything else: CI config, lint setup, small fixes, straightforward
  refactors and docs. Run `codex exec -m gpt-6.1-sol`.

If a ticket turns out harder or easier, relabel it and explain why in a comment.

## 4. Dependencies and duplicates

In the blocked issue's body, add a `## Dependencies` section with one
`Blocked by #N` line per blocker. Put the lines between `<!-- deps:start -->` and
`<!-- deps:end -->` markers so they can be rewritten idempotently. Add
`status:blocked` while any blocker is open. Remove it when the last blocker
closes. Before starting a ticket, check that its blockers are closed.

For duplicates, keep one issue and move the other's checklist into it. Close the
other as not planned with "Duplicate of #N" and the `duplicate` label.

## 5. Parallel work

Run unblocked tickets in parallel only when they do not touch the same files.
Use one git worktree or clone per ticket, e.g. `/workspace/vx-<issue>-<slug>`,
with a shared `CARGO_TARGET_DIR`. Cap concurrent builds at about 4.

Remove the worktree or clone once its PR merges or is abandoned. First check
`git status` and that HEAD is on the remote. Never delete uncommitted or unpushed
changes.

## 6. Disk hygiene

Keep the build machine's root disk under 85% (`df -h /`). Before starting jobs,
if usage is above that, trim the shared cargo cache: stale incremental dirs and
artifacts not read in over 24 h. Delete merged or abandoned clones and their
`target/` dirs. Never delete a work dir with uncommitted or unpushed changes.

## 7. Keep skills current

When a convention changes (labels, models, templates or workflow), update this
skill, `CONTRIBUTING.md` and the issue templates in the same PR or the same day.

## 8. Link every PR

- Branch `<type>/<issue#>-<slug>`.
- Conventional Commit PR title: `feat|fix|refactor|chore|docs|test|ci|perf(scope): summary`.
- PR body starts with `Closes #N` (or `Refs #N` for partial work), then summary,
  test evidence, and before/after screenshots for UI or numbers for performance.
- Move the card to **In progress** when you start and **In review** when the PR
  opens. Merging closes the issue and moves it to **Done**.

## 9. Audit before claiming done

Before telling anyone work is done:

1. Open the issue and re-read every **Done when** item.
2. Check each item against the merged diff, CI results and attached evidence, not
   against memory or a summary from another agent.
3. Tick only items that are actually met. File anything left over as a new issue
   and link it.
4. Confirm the issue is closed and the board shows **Done**.

If any item is unmet, say so plainly and say what is left.
