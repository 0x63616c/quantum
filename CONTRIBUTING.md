# Contributing to Quantum

[Tickets](https://tickets.worldwidewebb.co) is the sole source of truth for planned,
in-progress and completed work. GitHub issues and Projects are retired. Quantum
uses the `QUA` prefix. Every change starts as a ticket and lands as one focused
pull request. These rules apply to people and coding agents alike;
[AGENTS.md](AGENTS.md) covers technical conventions.

## 1. Ticket first

- Search Tickets before starting work and reuse an existing ticket when possible.
- Every ticket states **Why** and a **Done when** checklist of observable outcomes
  (behavior, tests, docs, screenshots).
- Track priority, labels and status in Tickets. Check dependencies before starting
  work and record blockers as Tickets dependency edges.

Every open ticket carries exactly one model label:

- `model:sol`: everyday work: CI config, lint setup, small fixes, straightforward
  refactors and docs. Run `codex exec -m gpt-6.1-sol`.
- `model:astra`: hard work: UI/design-heavy tasks, complex refactors, spikes and
  ports of someone else's code. Run `codex exec -m gpt-6-astra -c model_reasoning_effort=xhigh`.

Use Tickets dependency edges for blockers; do not encode dependencies in body text.

## 2. Branches

Name branches `<type>/<KEY-N>-<slug>`, for example `ci/QUA-87-tickets-conventions`. Branch from `main`; never push to `main`.

## 3. Pull request titles

Titles use [Conventional Commits](https://www.conventionalcommits.org/) and are
checked in CI:

```text
<type>(<optional scope>): <imperative summary> [QUA-N]
```

Allowed types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`, `ci`, `perf`.
Use an `area:*` name as the scope where it helps, e.g. `fix(mcp): …`. The squash
commit keeps the title, so it must read well in public release notes.

## 4. Pull request bodies

The [PR template](.github/pull_request_template.md) asks for:

- `Closes [QUA-N](https://tickets.worldwidewebb.co/t/QUA-N)` (or `Refs [QUA-N](https://tickets.worldwidewebb.co/t/QUA-N)` to link without closing).
  CI requires a Tickets key at the end of the title, e.g. `feat(app): add thing [QUA-12]`,
  and the same key linked like this in the body; on failure it prints the exact line to add.
- **Summary**: the problem and the resulting behavior, in plain English.
- **Changes**: bullets, most important first.
- **Testing**: commands run and their result, plus untested platforms;
  before/after numbers for performance changes.
- **Checklist**, including the UI items for UI changes.

Keep the sections and their order so every PR reads the same way.

Bot PRs (including Dependabot and Renovate) are exempt from the Ticket key check.
Tickets links PRs through a GitHub webhook, moves tickets to **In review** when
the PR opens and to **Done** on merge. GitHub Actions updates
use the accepted `ci:` commit-message prefix and one weekly group so related
actions, including upload/download-artifact, are reviewed together. Keep action
references SHA-pinned with exact version comments when reviewing that group.

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

### Design review (impeccable)

Agents and Codex jobs run `/impeccable critique` then `/impeccable polish` before
opening site, GPUI or future SwiftUI UI PRs. Use the locally served site per
[site/README.md](site/README.md), or native PR screenshots from
`cargo xtask screenshots`; reviewers critique the Before | After images.
[DESIGN.md](DESIGN.md) and [ART-CHECKLIST.md](design/ART-CHECKLIST.md) override
skill defaults. Follow [AGENTS.md](AGENTS.md#design-review-impeccable) for the
project-only impeccable **4.8.0** install/update command, telemetry opt-outs and
fallback cache. Keep upstream files verbatim. The repo git hooks in `.githooks/` are approved
(see [Git hooks](#git-hooks)); any new hook still needs Calum's approval.

## 5. One focus per PR

One ticket, one concern, one PR. Split drive-by fixes into their own ticket. Do not
mix refactors with behavior changes, or formatting churn with features.

## 6. Definition of Done

A ticket is done only when all of these hold:

- [ ] Every **Done when** item is checked, with evidence linked in the PR.
- [ ] The PR is merged into `main` with green required checks.
- [ ] Docs, schemas, fixtures and `AGENTS.md` are updated where behavior changed.
- [ ] UI changes have before/after screenshots; perf changes have numbers.
- [ ] Follow-up work is filed as new tickets, not left in comments.
- [ ] Tickets shows **Done** after the merge.

Never call work done from memory. Re-read the ticket's checklist against the merged
diff first.

Before coding, run `cargo xtask graphify` to refresh the local code graph; see
[AGENTS.md](AGENTS.md#local-code-graph) for the navigation workflow.

## Main branch protection

The reviewed ruleset payload is [protect-main.json](.github/rulesets/protect-main.json).
It requires a pull request and an up-to-date branch, blocks force-push and deletion,
and grants no bypasses. Required approvals are zero, so automation can merge a
PR once the required checks pass. GitHub Actions (integration ID `15368`) must
report these exact check contexts; workflow names are not part of the context:

| Workflow | Required check context |
| --- | --- |
| CI Linux | `linux` |
| CI macOS | `macos` |
| PR conventions | `Conventional Commit title` |
| PR conventions | `Ticket key` |

Linux and PR-conventions workflows run on every PR without path filters.
`Ticket key` preserves the bot exemption described above. The
`macos` check always evaluates change detection, packaging-task tests and the
`native-macos` build. It accepts skipped native work only after successful
docs-only detection; failed detection, cancelled jobs and unexpected skips fail
the required check. The PR native build runs on a hosted macOS runner. Owner-only,
same-repository guards remain mandatory for the separate self-hosted hardware checks; arrange
trusted verification before merging when those checks apply.
Rendering checks remain a separate applicable merge requirement; do not add
`render` to the ruleset until its path-filtered workflow reliably reports a
check on every PR (see #85).

### Apply and verify the ruleset

Ticket QUA-40 (originally issue #122) tracks rollout. Do not apply the payload
until the required workflows have been green and stable on `main` for several days. Inspect recent runs and
actual PR check contexts with `gh run list --branch main` and `gh pr checks <PR>`.
The existing Protect main ruleset already blocks deletion and force-pushes; the
payload preserves those protections while adding PR and CI requirements.

A repository administrator can inspect and update the existing ruleset:

```sh
gh api repos/0x63616c/quantum/rulesets/25883879
gh api --method PUT repos/0x63616c/quantum/rulesets/25883879 \
  --input .github/rulesets/protect-main.json
```

Read back the ruleset and verify required contexts, strict up-to-date checking,
zero approvals, no bypass actors, and both deletion/force-push rules. On disposable
same-repository test PRs, verify that a missing or mismatched ticket link fails
`Ticket key` and blocks merging; verify that a deliberately failing CI check also blocks
merging. Include failed change detection and a docs-only PR to verify that the
`macos` result rejects the former and accepts the latter. Record the PR URLs and
GitHub merge-state evidence in QUA-40. Finally fix
the failures, update the branch, and verify an authorized bot can merge the green
PR without human approval. Do not weaken the self-hosted runner guards or merge
failing test changes to perform this verification. Until these live checks are
recorded, the rollout is incomplete; a committed payload alone is not protection.

## Running Rust tests

CI runs `cargo nextest run --workspace --locked --profile ci`, followed by
`cargo test --doc --workspace --locked` because nextest excludes doctests.
Install the CI version locally with `cargo install cargo-nextest --version 0.9.150 --locked`.
`cargo xtask validate` uses nextest when installed and always runs doctests;
otherwise it falls back to `cargo test --workspace --locked` (including doctests).
The `ci` profile disables fail-fast and retries. Set `CI=true` locally to match
CI snapshot verification; insta inherits that variable under nextest too.

PR and push-to-main macOS checks run on GitHub-hosted `xcode-27` with the pinned
Rust toolchain and Xcode 27.0. The `macos` check keeps Clippy, nextest and doctests
for macOS-specific crates, plus `ci-dist` smoke DMGs on packaging-input PRs.
Filters apply at job level, so docs-only changes can skip the Mac build
without losing the required check context. Fork PRs can run this secret-free job.
Metal profiling and golden visuals stay on the guarded Mac mini runners; skipped
fork hardware jobs still need trusted verification before merge. Full workspace
checks and release-profile DMGs run nightly on the Mac mini, with failure reporting (see the legacy automation note in [CI runners](docs/ci-runners.md)).
Every PR workflow cancels superseded runs; nightly and release coverage is preserved.
See [CI runners](docs/ci-runners.md) for check placement, caches and timing evidence.

## Git hooks

Fast local hooks live in `.githooks/` (plain POSIX shell, no extra dependency) and
catch the cheap CI failures before a round-trip. Enable them once per clone or
worktree:

```sh
git config core.hooksPath .githooks
```

- **pre-commit** (about 0.5-2 s): `cargo fmt --all -- --check` when Rust files are
  staged, then `gitleaks git --pre-commit --staged --redact` on the staged diff.
  Without gitleaks installed (`brew install gitleaks`) the secret scan is skipped
  with a notice; CI still scans full history.
- **pre-push**: `cargo clippy --all-targets --locked -- -D warnings` on only the
  workspace crates the push changes (`-p <crate>` each, plus
  `--features quantum-app/test-support` when `quantum-app` is included). Changes to
  the root `Cargo.toml`, `Cargo.lock`, `clippy.toml`, `rust-toolchain.toml`,
  `.cargo/` or `vendor/` lint the whole workspace. Pushes with no Rust changes skip
  clippy. Tests, `cargo xtask lint` and the full CI matrix stay in CI.

Skip a hook for one command with `git commit --no-verify` or `git push --no-verify`.
Disable them with `git config --unset core.hooksPath`.

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

Run `cargo xtask check brand typos` before submitting changes (install the pinned
spelling checker with `cargo install typos-cli --version 1.51.1 --locked`). The
brand check scans tracked text and paths, including binary filenames. Add legacy
exceptions only in `brand.toml` under `compatibility`: list specific files, the
smallest required literals, and a compatibility reason. Every listed file and literal
must still match or the check fails as stale; remove them when retiring
compatibility. Fix spelling errors; reserve `_typos.toml`
word entries for justified domain terms and exclusions for upstream/generated assets.
