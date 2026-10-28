#!/usr/bin/env python3
"""Run fresh, sequential processes; retain samples and verify geometry digests."""
import argparse
import json
import platform
import statistics
import subprocess
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--process-runs", type=int, default=3)
    parser.add_argument("--runs", type=int, default=9)
    parser.add_argument("--size", type=int, default=256)
    args = parser.parse_args()
    results = []
    hardware = {
        "platform": platform.platform(),
        "cpu": subprocess.check_output(["lscpu"], text=True),
        "rust": subprocess.check_output(["rustc", "--version"], text=True).strip(),
        "revision": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
        "initial_load": Path("/proc/loadavg").read_text().strip(),
        "initial_memory": Path("/proc/meminfo").read_text(),
    }
    for repetition in range(args.process_runs):
        for workload in ("single", "boundary", "bulk", "full", "palette", "noop", "settings"):
            modes = ("current", "salsa") if repetition % 2 == 0 else ("salsa", "current")
            pair = []
            for mode in modes:
                output = subprocess.check_output(
                    [str(args.binary), mode, workload, str(args.runs), str(args.size)], text=True
                )
                result = json.loads(output)
                result["repetition"] = repetition
                results.append(result)
                pair.append(result)
                print(f"{repetition + 1} {mode:7} {workload:8} {result['median_us']:10.1f} us; "
                      f"RSS {result['final_memory']['rss_kib'] / 1024:.1f} MiB", flush=True)
                args.out.write_text(json.dumps({"hardware": hardware, "results": results}, indent=2) + "\n")
            for key in ("geometry_digest", "vertices", "exposed_faces", "voxels", "chunks"):
                if pair[0][key] != pair[1][key]:
                    raise RuntimeError(f"{workload}: differential failure for {key}")
    for workload in dict.fromkeys(r["workload"] for r in results):
        medians = [statistics.median(r["median_us"] for r in results
                                    if r["mode"] == mode and r["workload"] == workload)
                   for mode in ("current", "salsa")]
        print(f"{workload:8} {medians[0]:10.1f} -> {medians[1]:10.1f} us "
              f"({medians[1] / medians[0]:.2f}x)")


if __name__ == "__main__":
    main()
