# CI runners

Quantum's PR and push-to-main macOS build gates run on GitHub-hosted `xcode-27`
runners. Hardware checks, full nightly runs and releases use two self-hosted runners
on one Mac mini (Apple M2, 8 cores, 8 GB RAM). Other jobs use GitHub-hosted Ubuntu.

## Runners

| Runner | Labels | Runs |
| --- | --- | --- |
| GitHub-hosted macOS | `xcode-27` | CI macOS / `macos`: PR and main-push Clippy, nextest, doctests; PR smoke DMGs when packaging inputs change |
| `homelab-quantum` | `self-hosted, macOS, ARM64, xcode, golden` | CI macOS / `macos-hardware`: Metal profiling, full nightly/manual checks; Release macOS; Rendering checks |
| `homelab-quantum-2` | `self-hosted, macOS, ARM64, xcode, runner-2` | CI macOS / `macos-hardware`; Release macOS; Rendering checks (visual) |
| GitHub-hosted Ubuntu | `ubuntu-24.04` | Linux workspace checks, packaging task tests, site, screenshots, snapshots, conventions, secret scanning and failure reporting |

The [hosted image](https://github.com/actions/runner-images/blob/main/images/macos/xcode-27-arm64-Readme.md)
is macOS 27 on arm64. The workflow explicitly selects
`/Applications/Xcode_27.0.app/Contents/Developer`, installs Rust and its components
from `rust-toolchain.toml` using `rustup show`, and checks the Metal compiler,
downloading the Xcode Metal toolchain if absent. The Mac mini retains
`/Applications/Xcode-27.0.app/Contents/Developer`. Standard hosted runners are
[free for public repositories](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
The hosted job has a read-only token, no repository secrets or variables, and uses
the committed Sparkle public key for ad-hoc packaging; fork PRs can run it too.

- Rendering visual checks run on either runner. Both are on the same machine, and
  runner 2 matched the committed goldens bit-exact with `--determinism` (#165).
  Benchmarks (`bench`, `bench-baseline`) need the `golden` label, so timing baselines
  always come from one runner. The `runner_label` dispatch input re-checks one runner.
- Each runner is its own install under `~/actions-runners/<owner>-quantum[-2]`, with its
  own `_work` checkout and its own launchd service (`actions.runner.<owner>-quantum.<name>`).
- Release signing is serialized by the `release-macos` concurrency group, so two runners
  never sign at once.
- Fork PRs never run on these machines: every self-hosted job checks the actor and the
  head repository.
- Two runners is the limit for this machine. It has 8 GB of RAM and also hosts runners
  for other repos, and two Rust builds at once already push it into swap.
- Every self-hosted Mac job (CI macOS / `macos-hardware`, Rendering checks, Release macOS) sets
  `CARGO_BUILD_JOBS: 4`, so two concurrent builds share the 8 cores instead of each
  starting 8 rustc processes and running out of memory. Add it to any new Mac job.

Add a runner with GitHub's runner package (Settings → Actions → Runners → New runner),
install it next to the others, use a neutral name (`homelab-quantum-N`, no personal
details), give it the labels above without `golden` plus a neutral `runner-N` label, and run
`./svc.sh install && ./svc.sh start`. Add the label to the Rendering checks `runner_label`
options and pass a `visual` dispatch on it before relying on it for visual checks.

## Build caches

The hosted `macos` job uses SHA-pinned Swatinem/rust-cache after selecting Rust.
Its `macos-xcode-27.0` cache key separates the Xcode configuration, while the action
also keys on the compiler and Cargo inputs. Main pushes populate caches that PRs
can restore. The first run or a dependency/toolchain change can still be cold.

CI macOS / `macos-hardware` keeps the existing two caches on the Mac mini:

1. **Warm target dir.** `_work/quantum/cargo-target-ci-macos`, one per runner, outside the
   checkout so `actions/checkout` does not delete it. It is cleared above 10 GiB or when
   the disk has under 30 GiB free. Warm target dirs under any other workspace in the same
   `_work` (for example the old workspace left by the repo rename) are deleted each run.
2. **Shared sccache.** One sccache server serves both runners on `127.0.0.1:4227`, with
   an 8 GiB cache in `~/Library/Caches/sccache-quantum`. Its base directories are both
   runners' `_work` dirs, so a crate built by one runner is a cache hit on the other.
   The workflow uses it only if the server is listening; otherwise it builds uncached.

The sccache server runs from the launchd agent
`~/Library/LaunchAgents/dev.quantum.sccache.plist`. sccache always daemonizes itself,
so the agent runs a small loop that restarts the server whenever port 4227 stops
answering (`AbandonProcessGroup` keeps launchd from killing the daemon). The settings
are the environment variables `SCCACHE_SERVER_PORT=4227`, `SCCACHE_DIR`,
`SCCACHE_CACHE_SIZE=8G` and `SCCACHE_BASEDIRS` (both `_work` dirs, colon-separated).
Only one server may use the cache dir: sccache's disk cache is not safe for two servers.
The Mac mini was set up before the repo rename, so its agent and cache dir still use the
old name (`launchctl list | grep sccache` shows the live label); the settings are the same.

**Disk budget.** On the 228 GiB Mac mini, Quantum CI should stay under about 30 GiB:
two warm target dirs (10 GiB each) plus the 8 GiB sccache. Release and render jobs
delete their in-checkout `target/` (and `dist/` for releases) when they finish, so
release-profile output does not sit on the disk between runs.

Useful commands on the Mac mini:

```sh
SCCACHE_SERVER_PORT=4227 sccache --show-stats      # hit rate and cache size
launchctl kickstart -k gui/$(id -u)/dev.quantum.sccache
tail ~/Library/Logs/quantum-sccache.log
```

## Where CI time goes

The [#235 timing breakdown](https://github.com/0x63616c/quantum/issues/235)
samples the last 15 successful workflow runs before the hosted migration. Its PR
jobs (13 macOS and 14 rendering samples) show queue time dominating feedback:

| PR job | Queue median / max | Run median / max | Total median |
| --- | --- | --- | --- |
| CI macOS / `macos` (previous Mac mini placement) | 27.7 / 89.3 min | 20.9 / 47.6 min | 58.2 min |
| Rendering checks / `render` | 14.3 / 119.3 min | 1.8 / 2.2 min | 16.1 min |

Mac step medians were 8.3 min for the conditional DMG build, 5.1 min for tests,
1.4 min for Clippy and 0.9 min for Metal profiling. Linux, UI snapshots, PR
screenshots and conventions took about 1–5 min without the Mac mini queue.
These are historical measurements, not hosted-runner results.

| Check | PR | Push to main | Nightly / manual |
| --- | --- | --- | --- |
| macOS-specific Clippy (including profiling), nextest and doctests | Hosted `macos`, job-scoped to Rust/build inputs | Hosted `macos`, always | Full workspace on Mac mini |
| Ad-hoc DMG | Hosted `macos`, `ci-dist`, only when packaging inputs change | Skipped as before | Mac mini, `release`, on main |
| Metal profiling test | Mac mini, job-scoped to render, render-plan, format, app, core, mesh and shared build/test inputs | Mac mini, always | Mac mini, always |
| Golden visuals and determinism | Mac mini, existing render paths checked at job level | Nightly coverage | Mac mini; dispatch also supports benchmark/baseline commands |
| Plugin rule and packaging task tests | Ubuntu, job-scoped with macOS code checks | Ubuntu, always | Ubuntu, always |
| Linux workspace and site checks | Ubuntu, unchanged | Ubuntu, unchanged | Existing dispatch support unchanged |

`macos` remains the stable check name. CI macOS and Rendering checks always start
on PRs; their `changes` jobs use paginated, rename-aware file lists to skip work
at job level. Docs-only PRs (docs/, design/, .agents/, PR/issue templates and
top-level Markdown) skip Mac builds; every other path, including site content and
specs that crate tests read, still runs the hosted `macos` job. An API failure
fails change detection instead of declaring the PR irrelevant. Site and portable
code still get Linux coverage, and every nightly Mac run checks the workspace.

The profiling test initializes a real Metal adapter and renders at production
quality. Hosted VM compatibility has not been verified, so the exact test command
stays on the Mac mini, with no new ignored tests or fallback success. Goldens and
benchmarks retain their hardware requirements. Forks need trusted verification
for these guarded self-hosted jobs; a skipped job is not hardware evidence.

Every PR workflow cancels superseded runs for the same PR. Main CI macOS pushes
can supersede earlier main pushes because each latest push still runs all of its
checks. Nightly and manual macOS/rendering runs have unique concurrency groups,
so PRs and other runs cannot cancel or replace their pending work. Release macOS
and its serialization are unchanged. The 10:00 UTC CI macOS nightly and 10:30 UTC
rendering nightly open or update `type:ci`, `area:ci`, `priority:p0` issues on
failures, including failed change detection or skipped hardware jobs.

The target is a median PR feedback time under 10 minutes. Removing routine Rust
builds from the two shared Mac slots should reduce both queues, but this requires
measurement after rollout. Cold hosted caches, conditional DMG builds, hosted
capacity and overlap with nightly/release work can still exceed the target.
Compare the next 15 successful PR runs using job creation/start/completion times;
measure feedback until the last applicable check completes, not just `macos`.
