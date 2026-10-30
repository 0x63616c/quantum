#!/usr/bin/env python3
"""Validate feed and site-fetch behavior offline, using public fixture data."""

import base64
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
from macos_appcast import DOWNLOAD, SPARKLE, validate, validate_release


def fixture_feed(versions=("1.2.3",)):
    root = ET.Element("rss", version="2.0")
    channel = ET.SubElement(root, "channel")
    for version in versions:
        item = ET.SubElement(channel, "item")
        for field, value in (("version", version), ("shortVersionString", version),
                             ("minimumSystemVersion", "13.0")):
            ET.SubElement(item, SPARKLE + field).text = value
        signature = base64.b64encode(bytes(64)).decode()
        prefix = DOWNLOAD + "v" + version + "/"
        ET.SubElement(item, "enclosure", {
            "url": prefix + "Voxelite.dmg", "length": "11",
            SPARKLE + "edSignature": signature,
        })
        notes = ET.SubElement(item, SPARKLE + "releaseNotesLink", {
            SPARKLE + "length": "13", SPARKLE + "edSignature": signature,
        })
        notes.text = prefix + "Voxelite.html"
    return ET.ElementTree(root)


def release(version, *, draft=False, prerelease=False, feed=True):
    names = ["Voxelite.dmg", "Voxelite.html"] + (["appcast.xml"] if feed else [])
    return {"tag_name": "v" + version, "draft": draft, "prerelease": prerelease,
            "assets": [{"name": name, "size": size,
                        "browser_download_url": DOWNLOAD + "v" + version + "/" + name}
                       for name, size in zip(names, [11, 13, 900])]}


class AppcastTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.feed = self.root / "appcast.xml"
        self.tree = fixture_feed()
        self.tree.write(self.feed)

    def test_empty_feed_is_explicit_and_has_no_installable_item(self):
        empty = ROOT / "site/appcast.xml"
        self.assertEqual(validate(empty, allow_empty=True), [])
        with self.assertRaises(ValueError):
            validate(empty)

    def test_cli_rejects_wrong_local_dmg_length(self):
        import plistlib

        dist = self.root / "dist"
        bundle = dist / "Voxelite.app/Contents"
        bundle.mkdir(parents=True)
        (bundle / "Info.plist").write_bytes(plistlib.dumps({
            "CFBundleVersion": "1.2.3", "CFBundleShortVersionString": "1.2.3",
            "LSMinimumSystemVersion": "13.0",
        }))
        (dist / "Voxelite.html").write_bytes(b"fixture notes")
        dmg = dist / "Voxelite.dmg"
        command = [sys.executable, "-B", str(ROOT / "scripts/check-macos-appcast.py"),
                   str(self.feed), "--release-dir", str(dist),
                   "--previous", str(ROOT / "site/appcast.xml")]
        for content, success in ((b"fixture dmg", True), (b"altered", False)):
            dmg.write_bytes(content)
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode == 0, success, result.stdout + result.stderr)

    def test_offline_validation_matches_actual_files_and_history(self):
        previous = self.root / "previous.xml"
        fixture_feed(("1.2.2",)).write(previous)
        fixture_feed(("1.2.3", "1.2.2")).write(self.feed)
        dmg, notes = self.root / "Voxelite.dmg", self.root / "Voxelite.html"
        dmg.write_bytes(b"fixture dmg")
        notes.write_bytes(b"fixture notes")
        info = {"CFBundleVersion": "1.2.3", "CFBundleShortVersionString": "1.2.3",
                "LSMinimumSystemVersion": "13.0"}
        validate_release(self.feed, info, dmg, notes, previous)
        for key in info:
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate_release(self.feed, info | {key: "9.9.9"}, dmg, notes, previous)
        for file in (dmg, notes):
            data = file.read_bytes()
            file.write_bytes(data + b"!")
            with self.assertRaises(ValueError):
                validate_release(self.feed, info, dmg, notes, previous)
            file.write_bytes(data)
        fixture_feed().write(self.feed)
        with self.assertRaises(ValueError):
            validate_release(self.feed, info, dmg, notes, previous)

    def test_url_signature_length_and_metadata_rejection(self):
        cases = [
            ("enclosure", "url", "https://github.com.evil.invalid/0x63616c/voxelite/releases/download/v1.2.3/Voxelite.dmg"),
            ("enclosure", "url", DOWNLOAD + "v1.2.2/Voxelite.dmg"),
            ("enclosure", "url", DOWNLOAD + "v1.2.3/../Voxelite.dmg"),
            ("enclosure", "url", DOWNLOAD + "v1.2.3/Voxelite.dmg?redirect=1"),
            ("enclosure", SPARKLE + "edSignature", ""),
            ("enclosure", SPARKLE + "edSignature", "not base64!"),
            ("enclosure", SPARKLE + "edSignature", base64.b64encode(bytes(32)).decode()),
            ("enclosure", "length", "0"), ("enclosure", "length", "-1"),
            ("enclosure", "length", "1.1"),
            (SPARKLE + "releaseNotesLink", SPARKLE + "edSignature", ""),
        ]
        for node, attribute, value in cases:
            with self.subTest(attribute=attribute, value=value):
                tree = fixture_feed()
                tree.find("channel/item/" + node).set(attribute, value)
                tree.write(self.feed)
                with self.assertRaises(ValueError):
                    validate(self.feed)
        for tag in ("enclosure", SPARKLE + "minimumSystemVersion", SPARKLE + "shortVersionString",
                    SPARKLE + "releaseNotesLink"):
            tree = fixture_feed()
            item = tree.find("channel/item")
            item.remove(item.find(tag))
            tree.write(self.feed)
            with self.assertRaises(ValueError):
                validate(self.feed)

    def test_malformed_xml_and_unstable_or_unordered_versions(self):
        for data in (b"<rss>", b"<html/>", b'<!DOCTYPE rss [<!ENTITY x "x">]><rss/>'):
            self.feed.write_bytes(data)
            with self.assertRaises((ValueError, ET.ParseError)):
                validate(self.feed)
        for versions in (("1.2.3-rc.1",), ("01.2.3",), ("1.2.3", "1.2.3"),
                         ("1.2.3", "1.10.0")):
            fixture_feed(versions).write(self.feed)
            with self.assertRaises(ValueError):
                validate(self.feed)

    def test_asset_metadata_must_cover_every_asset(self):
        sizes = {asset["browser_download_url"]: asset["size"] for asset in release("1.2.3")["assets"]}
        validate(self.feed, asset_sizes=sizes)
        for sizes in ({}, sizes | {DOWNLOAD + "v1.2.3/Voxelite.dmg": 12}):
            with self.assertRaises(ValueError):
                validate(self.feed, asset_sizes=sizes)


class FetchTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        for name in ("scripts/fetch-macos-appcast.py", "scripts/macos_appcast.py", "site/appcast.xml"):
            target = self.root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, target)
        self.feed = self.root / "fixture.xml"
        fixture_feed(("1.10.0", "1.2.3")).write(self.feed)
        self.gh = self.root / "gh"
        self.gh.write_text('''#!/usr/bin/env python3
import json, os, pathlib, shutil, sys
args = sys.argv[1:]
if os.environ.get('FAIL_AT') == args[0]: sys.exit(1)
if args[0] == 'api':
    assert '--paginate' in args and '--slurp' in args
    print(os.environ['RELEASES'])
else:
    assert args[:3] == ['release', 'download', 'v1.10.0']
    shutil.copyfile(os.environ['FIXTURE'], pathlib.Path(args[args.index('--dir')+1]) / 'appcast.xml')
''')
        self.gh.chmod(0o755)

    def run_fetch(self, releases, *, fail="", success=True):
        result = subprocess.run([
            sys.executable, "-B", "scripts/fetch-macos-appcast.py", "output/appcast.xml",
        ], cwd=self.root, env=os.environ | {
            "PATH": str(self.root) + os.pathsep + os.environ["PATH"],
            "RELEASES": json.dumps(releases), "FIXTURE": str(self.feed), "FAIL_AT": fail,
        }, capture_output=True, text=True)
        self.assertEqual(result.returncode == 0, success, result.stdout + result.stderr)
        return self.root / "output/appcast.xml"

    def test_paginated_releases_select_numeric_highest_feed_and_copy_verbatim(self):
        releases = [[release("1.2.3"), release("3.0.0", draft=True)],
                    [release("1.10.0"), release("4.0.0", prerelease=True), release("2.0.0", feed=False)]]
        output = self.run_fetch(releases)
        self.assertEqual(output.read_bytes(), self.feed.read_bytes())

    def test_no_releases_or_no_feed_uses_empty_feed(self):
        for releases in ([], [[]], [[release("1.0.0", feed=False)]]):
            self.assertEqual(validate(self.run_fetch(releases), allow_empty=True), [])

    def test_api_download_and_invalid_asset_errors_preserve_destination(self):
        output = self.root / "output/appcast.xml"
        output.parent.mkdir()
        output.write_bytes(b"previous deployment")
        releases = [[release("1.10.0"), release("1.2.3")]]
        for failure in ("api", "release"):
            self.run_fetch(releases, fail=failure, success=False)
            self.assertEqual(output.read_bytes(), b"previous deployment")
        releases[0][0]["assets"][0]["size"] = 99
        self.run_fetch(releases, success=False)
        self.assertEqual(output.read_bytes(), b"previous deployment")


if __name__ == "__main__":
    unittest.main()
