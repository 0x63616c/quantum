---
name: project-tracking
description: Use at the start of repository tasks, before opening a PR and before claiming completion. Track work in Tickets, the sole source of truth.
---

# Project tracking

[Tickets](https://tickets.worldwidewebb.co) is the sole source of truth for planned,
in-progress and completed work. GitHub issues and Projects are retired. Follow
[CONTRIBUTING.md](../../../CONTRIBUTING.md).

## 1. Read Tickets first

Read the assigned ticket, its **Done when** checklist, dependencies and current
project work before planning or coding. Quantum uses the `QUA` prefix. Search
Tickets before creating work to avoid duplicates. API documentation lives in
`docs/API.md` in `0x63616c/tickets`; use authenticated Tickets tools without
printing credentials. Existing PRs can still be read with `gh pr list`.

## 2. One ticket per ask

- Reuse the assigned ticket or create one in Tickets for each new request.
- State **Why** and a **Done when** checklist of observable outcomes.
- Keep priority, labels and status in Tickets. Move work to **In progress** when
  starting. Split work requiring separate focused PRs into separate tickets.
- Record dependencies as Tickets dependency edges. Check blockers before starting.
- Keep one canonical ticket for duplicates and consolidate acceptance criteria.

## 3. Model labels

Every open ticket carries exactly one of `model:astra` or `model:sol`. Pick the
model from the ticket label.

- `model:astra`: hard work: UI/design-heavy tasks, complex refactors, spikes and
  ports. Run `codex exec -m gpt-6-astra -c model_reasoning_effort=xhigh`.
- `model:sol`: CI config, lint setup, small fixes, straightforward refactors and
  docs. Run `codex exec -m gpt-6.1-sol`.

If the work changes complexity, relabel the ticket and explain why.

## 4. Worktrees and builds

Start agent work with:

```sh
P=$(job new quantum QUA-N [slug]); source "$P/.job.env"
```

Use one worktree per ticket and source its `.job.env` before building. Preserve
its shared build-cache configuration; cap concurrent builds at about 4.
Remove merged or abandoned worktrees only after checking `git status` and that
HEAD is on the remote. Never delete uncommitted or unpushed changes.

## 5. Disk hygiene

Keep the build machine's root disk under 85% (`df -h /`). Before starting jobs,
if usage is above that, trim stale shared cargo cache artifacts not read in over
24 hours and merged or abandoned worktrees. Never delete uncommitted or unpushed
work.

## 6. Keep conventions current

When labels, models or workflows change, update this skill and
`CONTRIBUTING.md` together. Do not recreate GitHub issue templates.

## 7. Link every PR

- Branch from `main` as `<type>/<KEY-N>-<slug>`.
- Use a Conventional Commit title ending in the ticket key, for example
  `feat(app): add thing [QUA-12]`. CI checks the suffix; bots are exempt.
- Start the body with a linked ticket, `Closes [QUA-12](https://tickets.worldwidewebb.co/t/QUA-12)`, or `Refs [QUA-12](https://tickets.worldwidewebb.co/t/QUA-12)`
  to link without closing. CI fails if the title key is not linked like this in the body,
  then the template's Summary, Changes and Testing sections.
- Include screenshots for UI changes and measured evidence for performance work.
- Tickets links the PR through a GitHub webhook, moves it to **In review** when
  the PR opens and to **Done** on merge. Do not mark unmerged work Done.

## 8. Audit before claiming done

Re-read every **Done when** item against the merged diff, CI results and attached
evidence. Check only items actually met; file remaining work as Tickets tickets
and link it. Confirm Tickets shows **Done** after merge. For an unmerged PR,
report that it is open and identify any outstanding validation.
