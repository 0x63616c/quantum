#!/usr/bin/env python3
"""Combine macOS installation notes with commit subjects for the release tag."""

import os
from pathlib import Path
import re
import subprocess


def git(*args):
    return subprocess.check_output(["git", *args], encoding="utf-8").strip()


def main():
    os.chdir(Path(__file__).resolve().parents[1])
    stable = r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)"
    tag = os.environ["TAG"]
    if not re.fullmatch(stable, tag):
        raise SystemExit("TAG must be a stable release version")
    # Fully qualified refs prevent a same-named branch from supplying history.
    release = f"refs/tags/{tag}"
    git("rev-parse", "--verify", f"{release}^{{commit}}")
    # Strict ancestors only: exclude every tag on the release commit itself.
    tags = git("tag", "--merged", release, "--no-contains", release).splitlines()
    matches = [f"--match={name}" for name in tags if re.fullmatch(stable, name)]
    previous = git("describe", "--tags", "--abbrev=0", *matches, release) if matches else None
    revision_range = f"refs/tags/{previous}..{release}" if previous else release
    changes = git("log", "--no-merges", "--no-show-signature", "--no-color",
                  "--pretty=- %s (%h)", revision_range, "--")
    # Packaging writes this marker only after notarization and validation succeed.
    template = "RELEASE_NOTES.md" if Path("dist/.notarized").is_file() else "RELEASE_NOTES.unnotarized.md"
    base = (Path("packaging/macos") / template).read_text(encoding="utf-8").rstrip()
    heading = f"## Changes since {previous or 'the beginning'}"
    Path("dist").mkdir(exist_ok=True)
    Path("dist/RELEASE_NOTES.md").write_text(
        f"{base}\n\n{heading}\n\n{changes}\n", encoding="utf-8",
    )


if __name__ == "__main__":
    main()
