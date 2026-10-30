"""Offline checks for Voxelite's stable Sparkle feed (no private keys needed)."""

import base64
from pathlib import Path
import re
import xml.etree.ElementTree as ET

SPARKLE = "{http://www.andymatuschak.org/xml-namespaces/sparkle}"
REPOSITORY = "0x63616c/voxelite"
DOWNLOAD = f"https://github.com/{REPOSITORY}/releases/download/"
VERSION = r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)"


def version_tuple(version):
    """Require the stable version format used by release-macos.yml."""
    if not re.fullmatch(VERSION, version):
        raise ValueError("Expected a stable MAJOR.MINOR.PATCH version")
    return tuple(map(int, version.split(".")))


def signature_and_length(element):
    """Check signature encoding and size; Sparkle verifies the cryptography."""
    encoded = element.get(SPARKLE + "edSignature", "")
    try:
        signature = base64.b64decode(encoded, validate=True)
    except ValueError:
        raise ValueError("Invalid Ed25519 signature encoding") from None
    if len(signature) != 64 or base64.b64encode(signature).decode() != encoded:
        raise ValueError("Expected a base64 64-byte Ed25519 signature")
    attribute = SPARKLE + "length" if element.tag == SPARKLE + "releaseNotesLink" else "length"
    length = element.get(attribute, "")
    if not re.fullmatch(r"[1-9][0-9]*", length):
        raise ValueError("Expected a positive asset length")
    return int(length)


def validate(path, *, allow_empty=False, asset_sizes=None):
    """Validate every item, and exact lengths when an asset-size map is supplied.

    asset_sizes maps complete release URLs to lengths, from local files or the
    GitHub release API. If supplied, it must cover every enclosure and notes file.
    This is a structural check, not a substitute for Sparkle's signature check.
    """
    data = Path(path).read_bytes()
    if b"<!DOCTYPE" in data or b"<!ENTITY" in data:
        raise ValueError("Appcasts must not declare a DTD or entities")
    root = ET.fromstring(data)
    if root.tag != "rss" or root.get("version") != "2.0" or len(root.findall("channel")) != 1:
        raise ValueError("Expected one RSS 2.0 channel")
    items = root.findall("channel/item")
    if not items and not allow_empty:
        raise ValueError("Expected a signed update item")
    versions = []
    for item in items:
        version = item.findtext(SPARKLE + "version", "")
        versions.append(version_tuple(version))
        if item.findtext(SPARKLE + "shortVersionString") != version:
            raise ValueError("shortVersionString must match version")
        minimum = item.findtext(SPARKLE + "minimumSystemVersion", "")
        if not re.fullmatch(r"[0-9]+\.[0-9]+(?:\.[0-9]+)?", minimum):
            raise ValueError("Missing or invalid minimumSystemVersion")
        prefix = DOWNLOAD + "v" + version + "/"
        # Only full DMGs are shipped; reject extra enclosures/deltas too.
        if len(item.findall(".//enclosure")) != 1:
            raise ValueError("Expected exactly one full-update enclosure")
        enclosure = item.find("enclosure")
        notes = item.find(SPARKLE + "releaseNotesLink")
        if enclosure is None or enclosure.get("url") != prefix + "Voxelite.dmg":
            raise ValueError("Enclosure must reference this version's GitHub release DMG")
        if notes is None or notes.text != prefix + "Voxelite.html":
            raise ValueError("Missing or incorrect GitHub release-notes link")
        for element, url in ((enclosure, enclosure.get("url")), (notes, notes.text)):
            length = signature_and_length(element)
            if asset_sizes is not None and asset_sizes.get(url) != length:
                raise ValueError("Appcast asset length does not match the release asset")
    if versions != sorted(set(versions), reverse=True):
        raise ValueError("Appcast versions must be unique and ordered newest first")
    return items


def validate_release(path, info, dmg, notes, previous):
    """Check a newly generated item against the bundle and preserve all history."""
    items = validate(path)
    old_items = validate(previous, allow_empty=True)
    version = info["CFBundleVersion"]
    if items[0].findtext(SPARKLE + "version") != version:
        raise ValueError("Latest appcast version does not match the bundle")
    if items[0].findtext(SPARKLE + "shortVersionString") != info["CFBundleShortVersionString"]:
        raise ValueError("Appcast short version does not match the bundle")
    if items[0].findtext(SPARKLE + "minimumSystemVersion") != info["LSMinimumSystemVersion"]:
        raise ValueError("Appcast minimum system version does not match the bundle")
    if signature_and_length(items[0].find("enclosure")) != Path(dmg).stat().st_size:
        raise ValueError("Appcast DMG length does not match")
    if signature_and_length(items[0].find(SPARKLE + "releaseNotesLink")) != Path(notes).stat().st_size:
        raise ValueError("Appcast release-notes length does not match")

    def content(element):
        return (element.tag, element.attrib, (element.text or "").strip(),
                [content(child) for child in element])

    if [content(item) for item in items[1:]] != [content(item) for item in old_items]:
        raise ValueError("Appcast history was changed or discarded")
