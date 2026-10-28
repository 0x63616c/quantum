#!/usr/bin/env python3
"""Seed five real recent projects for both sides of a Welcome screenshot diff."""

import json
import os
from pathlib import Path
import shutil
import sys
import time


def prepare(root: Path, capture_home: Path, data: Path) -> None:
    """Seed only the explicit disposable home, shared by both capture revisions."""
    capture_home.mkdir(parents=True, exist_ok=True)
    clock = capture_home / "clock.txt"
    if not clock.exists():
        clock.write_text(str(int(time.time())), encoding="utf-8")
    now = int(clock.read_text(encoding="utf-8"))
    projects = []
    for filename, name, folder, age in [
        ("autumn-workshop", "Autumn Workshop", "Quantum", 2 * 3600),
        ("snow-cabin", "Snow Cabin", "Quantum/client", 86400),
        ("cherry-blossom", "Cherry Blossom", "Desktop", 3 * 86400),
        ("lighthouse", "Lighthouse", "Quantum", 5 * 86400),
        ("autumn-workshop", "Workshop Study", "Quantum/samples", 8 * 86400),
    ]:
        path = capture_home / folder / f"{name}.voxl"
        path.parent.mkdir(parents=True, exist_ok=True)
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
