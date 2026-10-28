# Contributing to Quantum

Quantum (repository `voxelite`) is tracked on the
[Quantum project board](https://github.com/users/0x63616c/projects). Every change
starts as an issue and lands as one focused pull request that closes it. These
rules apply to people and coding agents alike; [AGENTS.md](AGENTS.md) covers the
technical conventions.

## 1. Issue first

- Open an issue before starting work, using a template: **Feature**, **Bug**,
  **Refactor** or **Spike**. Search open issues first and reuse one if it exists.
- Every issue states **Why** and a **Done when** checklist of observable outcomes
  (behavior, tests, docs, screenshots). Vague checklists get rewritten before work
  starts.
- Labels: exactly one `type:*`, one or more `area:*`, and one `priority:*`
  (`p0` now, `p1` next, `p2` eventually). Add `status:blocked` with a comment
  naming the blocker. Parked ideas get `later` and the **Later** milestone.
- Put the issue in a milestone and on the board. Board status follows the work:
  Backlog → Next → In progress → In review → Done (or Later).

Every open issue carries exactly one model label:

- `model:sol`: everyday work: CI config, lint setup, small fixes, straightforward
  refactors and docs. Run `codex exec -m gpt-6.1-sol`.
- `model:astra`: hard work: UI/design-heavy tasks, complex refactors, spikes and
  ports of someone else's code. Run `codex exec -m gpt-6-astra -c model_reasoning_effort=xhigh`.

In the issue body's `## Dependencies` section, put one `Blocked by #N` line per
blocker between `<!-- deps:start -->` and `<!-- deps:end -->` markers. Keep
`status:blocked` while any blocker is open; remove it when the last one closes.

## 2. Branches

Name branches `<type>/<issue#>-<slug>`, for example `refactor/84-split-ui-rs` or
`fix/91-atomic-export-writes`. Branch from `main`; never push to `main`.

## 3. Pull request titles

Titles use [Conventional Commits](https://www.conventionalcommits.org/) and are
checked in CI:

```text
<type>(<optional scope>): <imperative summary>
```

Allowed types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`, `ci`, `perf`.
Use an `area:*` name as the scope where it helps, e.g. `fix(mcp): …`. The squash
commit keeps the title, so it must read well in public release notes.

## 4. Pull request bodies

The [PR template](.github/pull_request_template.md) asks for:

- `Closes #N` (or `Fixes #N`; `Refs #N` for partial work). CI rejects PRs that
  link no issue.
- **Summary**: the problem and the resulting behavior, in plain English.
- **Changes**: bullets, most important first.
- **Testing**: commands run and their result, plus untested platforms;
  before/after numbers for performance changes.
- **Checklist**, including the UI items for UI changes.

Keep the sections and their order so every PR reads the same way.

### Screenshots for UI changes

Any PR that changes what a user sees gets a **Screenshots** section with a
captioned Before | After table of the key screens (Welcome window, editor in dark
and light themes). It is generated, never pasted by hand:

```sh
cargo xtask screenshots              # local preview in target/pr-screenshots/
cargo xtask screenshots --publish 123  # fill the Screenshots section of PR #123
cargo xtask screenshots --screen 'settings-*' # local preview of just Settings
```

The default captures only `welcome`, `editor-dark` and `editor-light`. Migration
PRs add the surfaces they change with one plain, column-zero line in the PR body,
outside the generated Screenshots section: `Screens: panels-* settings-general-*`.
Exact names and trailing `*` prefixes work; unmatched selectors fail. Both CI and
local `--publish` add those matches to the key set. Explicit `--screen` options
select only their matches; quote patterns in shell commands.

The **PR screenshots** CI job runs the publish step for PRs touching UI paths.
After editing the `Screens:` line, push or use `workflow_dispatch`; body edits
alone do not trigger it. Dispatch accepts an optional `screens` field to replace
the body's extra selectors for that run (blank uses the body). Unchanged screens
appear as a collapsed text list; their PNGs stay in local/CI artifacts and are
never pushed to `pr-assets`. Non-UI PRs get no Screenshots section.
Details, adding screens and troubleshooting:
[ui-pr-screenshots skill](.agents/skills/ui-pr-screenshots/SKILL.md).

## 5. One focus per PR

One issue, one concern, one PR. Split drive-by fixes into their own issue. Do not
mix refactors with behavior changes, or formatting churn with features.

## 6. Definition of Done

An issue is done only when all of these hold:

- [ ] Every **Done when** item is checked, with evidence linked in the PR.
- [ ] The PR is merged into `main` with green required checks.
- [ ] Docs, schemas, fixtures and `AGENTS.md` are updated where behavior changed.
- [ ] UI changes have before/after screenshots; perf changes have numbers.
- [ ] Follow-up work is filed as new issues, not left in comments.
- [ ] The issue is closed by the merge and its board card shows **Done**.

Never call work done from memory. Re-read the issue's checklist against the merged
diff first.

## Snapshot tests

insta detects CI (the `CI` environment variable GitHub Actions sets) and then
behaves as `INSTA_UPDATE=no`: it never writes snapshots, so unreviewed output changes
fail tests. Locally, run the relevant tests with `INSTA_UPDATE=new`, then use
`cargo insta review` to inspect and accept intentional changes. Commit reviewed
`.snap` files alongside the tests; never commit `.snap.new` or `.pending-snap`
files. Use `INSTA_UPDATE=no` to verify the accepted snapshots without updating them.

## Workflow security and secret scanning

Every PR and push to main runs gitleaks 8.30.1 on complete reachable Git history
in a GitHub-hosted Linux job. The release archive is verified against its pinned
SHA-256 before execution; no gitleaks license or write token is required. To
reproduce with that version installed:

```sh
gitleaks git --redact --log-opts=--all .
cargo xtask check workflows
```

If a scan finds a credential, revoke or rotate it first; deleting the current
file does not remove it from history. Keep secret values out of issues and logs.
For a confirmed false positive, add a narrowly scoped, documented exception in
`.gitleaks.toml` that extends the default rules, then rerun the full-history scan.
Prefer an exact fingerprint or specific path/rule combination over disabling
a rule globally. Review exceptions like code; do not baseline real credentials.

Pin every remote workflow `uses:` reference to a full 40-character commit SHA,
with an exact `# vX.Y.Z` comment. Resolve annotated tags to their commit before
pinning. Dependabot checks the GitHub Actions ecosystem weekly; review both the
SHA and its version comment when updating. Keep explicit workflow-level
`permissions: {}` or `contents: read`, and grant additional permissions only
to the job that needs them. Site publishing writes the site-dist branch through
the contents permission; it uses Vercel credentials rather than Pages or OIDC grants.
