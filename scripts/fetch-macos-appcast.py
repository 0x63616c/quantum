#!/usr/bin/env python3
"""Fetch the highest stable release's signed feed, preserving its exact bytes."""

import argparse
import json
from pathlib import Path
import subprocess
import tempfile

from macos_appcast import REPOSITORY, SPARKLE, validate, version_tuple


def fetch(destination):
    """Fail on API/download/validation errors; only absence permits an empty feed."""
    pages = json.loads(subprocess.check_output([
        "gh", "api", "--paginate", "--slurp",
        f"repos/{REPOSITORY}/releases?per_page=100",
    ], text=True))
    releases = []
    sizes = {}
    for release in (release for page in pages for release in page):
        if release["draft"] or release["prerelease"]:
            continue
        tag = release["tag_name"]
        try:
            if not tag.startswith("v"):
                continue
            version = version_tuple(tag[1:])
        except ValueError:
            continue
        for asset in release["assets"]:
            sizes[asset["browser_download_url"]] = asset["size"]
        if any(asset["name"] == "appcast.xml" for asset in release["assets"]):
            releases.append((version, tag))
    if releases:
        _, tag = max(releases)
        with tempfile.TemporaryDirectory(prefix="voxelite-feed-") as directory:
            subprocess.run([
                "gh", "release", "download", tag, "--repo", REPOSITORY,
                "--pattern", "appcast.xml", "--dir", directory,
            ], check=True)
            source = Path(directory) / "appcast.xml"
            items = validate(source, asset_sizes=sizes)
            if items[0].findtext(SPARKLE + "version") != tag[1:]:
                raise ValueError("Release appcast does not begin with its own version")
            data = source.read_bytes()
    else:
        source = Path(__file__).resolve().parents[1] / "site/appcast.xml"
        if validate(source, allow_empty=True):
            raise ValueError("No released appcast exists; refusing to reuse a local nonempty feed")
        data = source.read_bytes()
        print("No stable release appcast exists; using the empty feed.")
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(data)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    fetch(args.destination)
