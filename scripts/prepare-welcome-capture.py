#!/usr/bin/env python3
"""Seed five real recent projects for both sides of a Welcome screenshot diff."""

import json
import os
from pathlib import Path
import shutil
import sys
import time


def prepare(root: Path, fixtures: Path, data: Path) -> None:
    """Use identical paths and mtimes on both sides, without touching user data."""
    fixtures.mkdir(parents=True, exist_ok=True)
    clock = fixtures / "clock.txt"
    if not clock.exists():
        clock.write_text(str(int(time.time())), encoding="utf-8")
    now = int(clock.read_text(encoding="utf-8"))
    projects = []
    for filename, name, age in [
        ("autumn-workshop", "Autumn Workshop", 2 * 3600),
        ("snow-cabin", "Snow Cabin", 86400),
        ("cherry-blossom", "Cherry Blossom", 3 * 86400),
        ("lighthouse", "Lighthouse", 5 * 86400),
        ("autumn-workshop", "Workshop Study", 8 * 86400),
    ]:
        path = fixtures / f"{name}.voxl"
        shutil.copyfile(root / "examples" / f"{filename}.voxl", path)
        modified = now - age
        os.utime(path, (modified, modified))
        projects.append({
            "name": name,
            "path": str(path.resolve()),
            "modified": {"secs_since_epoch": modified, "nanos_since_epoch": 0},
        })
    (data / "recents.json").write_text(
        json.dumps({"version": 1, "projects": projects}), encoding="utf-8"
    )


if __name__ == "__main__":
    prepare(*(Path(arg) for arg in sys.argv[1:]))
