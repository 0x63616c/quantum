# CI runners

Quantum's macOS jobs run on self-hosted runners on one Mac mini (Apple M2, 8 cores,
8 GB RAM). Everything else runs on GitHub-hosted Ubuntu runners.

## Runners

| Runner | Labels | Runs |
| --- | --- | --- |
| `homelab-voxelite` | `self-hosted, macOS, ARM64, xcode, golden` | CI macOS, Release macOS, Rendering checks |
| `homelab-voxelite-2` | `self-hosted, macOS, ARM64, xcode, runner-2` | CI macOS, Release macOS, Rendering checks (visual) |

- Rendering visual checks run on either runner. Both are on the same machine, and
  runner 2 matched the committed goldens bit-exact with `--determinism` (#165).
  Benchmarks (`bench`, `bench-baseline`) need the `golden` label, so timing baselines
  always come from one runner. The `runner_label` dispatch input re-checks one runner.
- Each runner is its own install under `~/actions-runners/<owner>-voxelite[-2]`, with its
  own `_work` checkout and its own launchd service (`actions.runner.<owner>-voxelite.<name>`).
- Release signing is serialized by the `release-macos` concurrency group, so two runners
  never sign at once.
- Fork PRs never run on these machines: every self-hosted job checks the actor and the
  head repository.
- Two runners is the limit for this machine. It has 8 GB of RAM and also hosts runners
  for other repos, and two Rust builds at once already push it into swap.

Add a runner with GitHub's runner package (Settings → Actions → Runners → New runner),
install it next to the others, use a neutral name (`homelab-voxelite-N`, no personal
details), give it the labels above without `golden` plus a neutral `runner-N` label, and run
`./svc.sh install && ./svc.sh start`. Add the label to the Rendering checks `runner_label`
options and pass a `visual` dispatch on it before relying on it for visual checks.

## Build caches

CI macOS keeps two caches on the Mac mini:

1. **Warm target dir.** `_work/cargo-target-ci-macos`, one per runner, outside the
   checkout so `actions/checkout` does not delete it. It is cleared above 15 GiB or when
   the disk has under 15 GiB free.
2. **Shared sccache.** One sccache server serves both runners on `127.0.0.1:4227`, with
   an 8 GiB cache in `~/Library/Caches/sccache-voxelite`. Its base directories are both
   runners' `_work` dirs, so a crate built by one runner is a cache hit on the other.
   The workflow uses it only if the server is listening; otherwise it builds uncached.

The sccache server runs from the launchd agent
`~/Library/LaunchAgents/dev.voxelite.sccache.plist`. sccache always daemonizes itself,
so the agent runs a small loop that restarts the server whenever port 4227 stops
answering (`AbandonProcessGroup` keeps launchd from killing the daemon). The settings
are the environment variables `SCCACHE_SERVER_PORT=4227`, `SCCACHE_DIR`,
`SCCACHE_CACHE_SIZE=8G` and `SCCACHE_BASEDIRS` (both `_work` dirs, colon-separated).
Only one server may use the cache dir: sccache's disk cache is not safe for two servers.

Useful commands on the Mac mini:

```sh
SCCACHE_SERVER_PORT=4227 sccache --show-stats      # hit rate and cache size
launchctl kickstart -k gui/$(id -u)/dev.voxelite.sccache
tail ~/Library/Logs/voxelite-sccache.log
```

## Where CI time goes

- CI Linux (fmt, Clippy, tests, tidy, cargo-deny, machete) and the Python script tests
  run on GitHub-hosted runners and do not queue.
- CI macOS on the Mac mini runs Clippy and the workspace tests. PRs build the
  release-profile DMG only when packaging inputs change; pushes to `main` skip it. A nightly
  run on `main` (03:00 PT) builds the DMG and opens a `priority:p0` issue if anything fails.
- Docs-only PRs skip the macOS build entirely.
