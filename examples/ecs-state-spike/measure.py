#!/usr/bin/env python3
"""Reproduce isolated cold builds and alternating runtime samples (no repo edits)."""
import csv
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import time

HERE = Path(__file__).resolve().parent
RESULTS = HERE / "results"
MANIFEST = HERE / "Cargo.toml"


def run(args, **kwargs):
    return subprocess.run(args, check=True, text=True, **kwargs)


def measure():
    RESULTS.mkdir(exist_ok=True)
    data = {
        "machine": platform.platform(),
        "rustc": run(["rustc", "--version"], capture_output=True).stdout.strip(),
        "jobs": 4,
        "cargo_home_config": (Path.home() / ".cargo/config.toml").read_text(),
        "source_sha256": {
            str(p.relative_to(HERE)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(HERE.glob("src/*.rs"))
        },
        "builds": [], "sizes": {}, "runtime": [],
    }

    def save():
        (RESULTS / "measurements.json").write_text(json.dumps(data, indent=2) + "\n")

    # Fresh targets are kept away from other jobs' shared-cache cleanup.
    parent = Path("/tmp/ecs-state-spike-measure")
    parent.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=parent) as workspace:
        workspace = Path(workspace)
        binaries = {}
        with (RESULTS / "build.log").open("w") as log:
            # Alternate order to reduce systematic warm-filesystem/order bias.
            for profile, trials in [("dev", 3), ("release", 1)]:
                for trial in range(trials):
                    order = ["direct", "ecs"] if trial % 2 == 0 else ["ecs", "direct"]
                    for backend in order:
                        target = workspace / f"{profile}-{trial}-{backend}"
                        env = dict(os.environ, CARGO_TARGET_DIR=str(target))
                        cmd = ["cargo", "build", "--manifest-path", str(MANIFEST),
                               "--locked", "--offline", "-j", "4"]
                        if backend == "ecs":
                            cmd += ["--features", "ecs"]
                        if profile == "release":
                            cmd += ["--release"]
                        before = os.getloadavg()
                        start = time.perf_counter()
                        run(cmd, env=env, stdout=log, stderr=log)
                        seconds = time.perf_counter() - start
                        data["builds"].append({
                            "profile": profile, "trial": trial, "backend": backend,
                            "kind": "cold-target-warm-source-cache", "seconds": seconds,
                            "load_before": before, "load_after": os.getloadavg(),
                        })
                        print(profile, trial, backend, round(seconds, 3), flush=True)
                        if profile == "release":
                            binary = target / "release" / "ecs-state-spike"
                            copy = workspace / backend
                            shutil.copy2(binary, copy)
                            binaries[backend] = copy
                            stripped = workspace / f"{backend}.stripped"
                            run(["strip", "-o", str(stripped), str(copy)])
                            sections = run(["size", str(copy)], capture_output=True).stdout
                            data["sizes"][backend] = {
                                "unstripped_bytes": copy.stat().st_size,
                                "stripped_bytes": stripped.stat().st_size,
                                "elf_size": sections.splitlines()[1].split()[:4],
                            }
                        save()
                        shutil.rmtree(target)
        for trial in range(5):
            for backend in (["direct", "ecs"] if trial % 2 == 0 else ["ecs", "direct"]):
                before = os.getloadavg()
                try:
                    output = run([str(binaries[backend])], capture_output=True).stdout
                except subprocess.CalledProcessError as error:
                    print(error.stdout, error.stderr, flush=True)
                    raise
                rows = list(csv.DictReader(io.StringIO(output)))
                for row in rows:
                    row["iterations"] = int(row["iterations"])
                    row["ns_per_operation"] = float(row["ns_per_operation"])
                data["runtime"].append({"backend": backend, "trial": trial,
                                        "load_before": before, "rows": rows})
                save()
                print("runtime", trial, backend, flush=True)


if __name__ == "__main__":
    measure()
