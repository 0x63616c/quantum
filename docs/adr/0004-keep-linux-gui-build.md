# 0004: Keep the Linux GUI build

Date: 2026-10-27

Status: Accepted.

## Context

[Issue #113](https://github.com/0x63616c/voxelite/issues/113) asks whether Linux
GUI builds justify their CI cost. The owner approved the recommendation to keep
them. Quantum targets a native macOS product; retaining a Linux development build
does not establish Linux release support.

[CI Linux](../../.github/workflows/ci-linux.yml) runs on hosted Ubuntu 24.04 for
main pushes, pull requests and manual dispatch. It installs GPUI system
dependencies, runs workspace Clippy with all targets and workspace tests with
default features, including the app's default `native` feature. This provides GUI
compilation and test coverage without occupying the self-hosted macOS runner;
it does not launch the editor or capture screenshots.

The existing manual [native capture script](../../scripts/capture-native.sh)
builds the app and MCP, opens real GPUI windows under Xvfb and captures both
themes. Its [Linux setup](../../crates/voxelite-app/README.md#linux-dependencies-and-captures)
documents the required dependencies. Automated Xvfb screenshot capture in CI
is planned in issue #119 / PR #130, not part of the current CI Linux workflow.

## Decision

Keep the Linux GUI build and its native GPUI dependencies for workspace CI and
UI capture work. Retain macOS as the product platform and require native macOS
verification for platform-specific behavior. This clarifies the macOS-only scope
in ADRs 0002 and 0003 without changing release commitments.

## Consequences

- Accept the CI time, dependency installation, cache storage and maintenance cost
  of compiling and testing the GUI on Linux. The current Linux job has a
  60-minute timeout; this is a limit, not a measured runtime or cost estimate.
- Preserve hosted CI coverage independent of the Mac runner and the existing
  manual Xvfb harness for UI review. CI screenshot automation remains follow-up
  work in #119 / PR #130.
- Linux checks and captures cannot verify AppKit menus, window behavior, native
  macOS input or Metal rendering/presentation. Keep native macOS checks and the
  pinned Metal visual/determinism checks authoritative for those paths.
- This decision changes documentation only; it adds no workflow, runtime or
  Linux packaging behavior.
