#!/usr/bin/env python3
"""Validate an appcast offline, optionally against a packaged release."""

import argparse
from pathlib import Path
import plistlib
import xml.etree.ElementTree as ET

from macos_appcast import validate, validate_release

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("feed", type=Path)
parser.add_argument("--allow-empty", action="store_true")
parser.add_argument("--release-dir", type=Path, help="Check the newest item against this dist directory")
parser.add_argument("--previous", type=Path, help="Previous feed, required with --release-dir")
args = parser.parse_args()
if bool(args.release_dir) != bool(args.previous):
    parser.error("--release-dir and --previous must be supplied together")
try:
    if args.release_dir:
        with (args.release_dir / "Voxelite.app/Contents/Info.plist").open("rb") as source:
            info = plistlib.load(source)
        validate_release(args.feed, info, args.release_dir / "Voxelite.dmg",
                         args.release_dir / "Voxelite.html", args.previous)
    else:
        validate(args.feed, allow_empty=args.allow_empty)
except (ValueError, OSError, ET.ParseError) as error:
    raise SystemExit(f"Invalid appcast: {error}") from None
print("Appcast validation passed.")
