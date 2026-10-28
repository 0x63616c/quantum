#!/usr/bin/env python3
"""Test packaging failure paths using fixture tools; no Apple services or keys."""

import base64
import hashlib
from html.parser import HTMLParser
import io
import json
import os
from pathlib import Path
import plistlib
import re
import shutil
import subprocess
import tarfile
import tempfile
import unittest
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
SECRETS = (
    "MACOS_CERTIFICATE_P12_BASE64", "MACOS_CERTIFICATE_PASSWORD", "MACOS_SIGNING_IDENTITY",
    "APPLE_API_KEY_P8_BASE64", "APPLE_API_KEY_ID", "APPLE_API_ISSUER_ID", "SPARKLE_ED_PRIVATE_KEY",
)
CERT = {
    "MACOS_CERTIFICATE_P12_BASE64": base64.b64encode(b"fixture certificate").decode(),
    "MACOS_CERTIFICATE_PASSWORD": "fixture password",
}
NOTARY = {
    "APPLE_API_KEY_P8_BASE64": base64.b64encode(b"fixture API key").decode(),
    "APPLE_API_KEY_ID": "fixture ID", "APPLE_API_ISSUER_ID": "fixture issuer",
}
PUBLIC = {"SPARKLE_PUBLIC_ED_KEY": (ROOT / "packaging/macos/sparkle_public_ed_key.txt").read_text().strip()}

MOCK = r'''#!/usr/bin/env python3
import json, os, pathlib, shutil, signal, sys
name = pathlib.Path(sys.argv[0]).name
args = sys.argv[1:]
secret_names = json.loads(os.environ['SECRET_NAMES'])
logged_args = args if name != 'security' or args[0] == 'list-keychains' else args[:1]
with open(os.environ['MOCK_LOG'], 'a') as log:
    log.write(json.dumps([name, logged_args, [key for key in secret_names if key in os.environ]]) + '\n')
operation = name + (':' + args[0] if args else '')
if os.environ.get('SIGNAL_AT') == operation:
    os.kill(os.getppid(), signal.SIGTERM)
    sys.exit(1)
if os.environ.get('FAIL_TOOL') in (name, operation):
    print('PRIVATE-DIAGNOSTIC-SENTINEL', file=sys.stderr)
    sys.exit(1)
if name == 'uname': print('Darwin')
elif name == 'cargo':
    pathlib.Path('target/release').mkdir(parents=True, exist_ok=True)
    binary = 'voxelite-mcp' if 'voxelite-mcp' in args else 'voxelite'
    pathlib.Path('target/release', binary).write_text('fixture ' + binary)
elif name == 'security':
    if args[0] == 'create-keychain': pathlib.Path(args[-1]).touch()
    elif args[0] == 'delete-keychain': pathlib.Path(args[-1]).unlink()
    elif args[0] == 'list-keychains':
        if '-s' not in args: print(os.environ.get('KEYCHAIN_LIST', '"/tmp/login.keychain-db"'))
        else: assert '' not in args, 'Empty keychain path'
    elif args[0] == 'find-identity': print('1) ' + 'A'*40 + ' "Developer ID Application: Calum"')
    elif args[0] == 'import':
        assert pathlib.Path(args[1]).stat().st_mode & 0o777 == 0o600
elif name == 'iconutil': pathlib.Path(args[-1]).write_text('fixture icon')
elif name == 'ditto':
    if '-c' in args: pathlib.Path(args[-1]).write_text('fixture zip')
    else: shutil.copytree(args[0], args[1], symlinks=True)
elif name == 'hdiutil': pathlib.Path(args[-1]).write_text('fixture dmg')
elif name == 'curl':
    output = args[args.index('--output')+1]
    shutil.copyfile(os.environ['FIXTURE_ARCHIVE'], output)
    if os.environ.get('CORRUPT_ARCHIVE'):
        with open(output, 'ab') as out: out.write(b'corrupt')
elif name == 'xcrun' and args[:2] == ['notarytool', 'submit']:
    assert pathlib.Path(args[args.index('--key')+1]).stat().st_mode & 0o777 == 0o600
    print(json.dumps({'id':'fixture-submission', 'status': os.environ.get('NOTARY_STATUS', 'Accepted')}))
'''

GENERATOR = r'''#!/usr/bin/env python3
import base64, os, pathlib, plistlib, sys, xml.etree.ElementTree as ET
assert 'SPARKLE_ED_PRIVATE_KEY' not in os.environ
assert sys.stdin.read().strip() == 'fixture private key'
assert 'fixture private key' not in sys.argv
args = sys.argv[1:]
assert args[args.index('--maximum-versions')+1] == '0'
assert args[args.index('--maximum-deltas')+1] == '0'
notes = (pathlib.Path(args[-1]) / 'Voxelite.html').read_text(encoding='utf-8')
ns = '{http://www.andymatuschak.org/xml-namespaces/sparkle}'
if os.environ.get('GENERATOR_FAILURE'):
    print('PRIVATE-DIAGNOSTIC-SENTINEL')
    print('fixture private key', file=sys.stderr)
    sys.exit(1)
root = ET.parse(args[args.index('-o')+1]).getroot()
channel = root.find('channel')
item = ET.Element('item')
channel.insert(0, item)
if not os.environ.get('MISSING_NOTES'):
    link = ET.SubElement(item, ns+'releaseNotesLink', {
        ns+'edSignature':base64.b64encode(bytes(64)).decode(),
        ns+'length':str(len(notes.encode()))})
    link.text = args[args.index('--release-notes-url-prefix')+1]+'Voxelite.html'
    if os.environ.get('WRONG_NOTES'): link.text = 'https://example.invalid/notes'
if os.environ.get('DROP_HISTORY'):
    for old in channel.findall('item')[1:]: channel.remove(old)
info = plistlib.load(open('dist/Voxelite.app/Contents/Info.plist', 'rb'))
ET.SubElement(item, ns+'version').text = info['CFBundleVersion']
ET.SubElement(item, ns+'shortVersionString').text = info['CFBundleShortVersionString']
ET.SubElement(item, ns+'minimumSystemVersion').text = info['LSMinimumSystemVersion']
attrs = {'url':args[args.index('--download-url-prefix')+1]+'Voxelite.dmg',
         'length':str(pathlib.Path('dist/Voxelite.dmg').stat().st_size)}
if not os.environ.get('UNSIGNED_FEED'): attrs[ns+'edSignature'] = base64.b64encode(bytes(64)).decode()
if os.environ.get('WRONG_URL'): attrs['url'] = 'https://example.invalid/Voxelite.dmg'
ET.SubElement(item, 'enclosure', attrs)
ET.ElementTree(root).write(args[args.index('-o')+1], encoding='utf-8', xml_declaration=True)
if info.get('SURequireSignedFeed') and not os.environ.get('UNSIGNED_METADATA'):
    with open(args[args.index('-o')+1], 'a') as output:
        output.write('<!-- fixture feed signature -->')
'''

VERIFIER = r'''#!/usr/bin/env python3
import os, pathlib, sys
assert 'SPARKLE_ED_PRIVATE_KEY' not in os.environ
assert sys.stdin.read().strip() == 'fixture private key'
assert sys.argv[1:4] == ['--ed-key-file', '-', '--verify']
if '<!-- fixture feed signature -->' not in pathlib.Path(sys.argv[4]).read_text():
    print('PRIVATE-DIAGNOSTIC-SENTINEL')
    sys.exit(1)
'''


class PackagingTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="voxelite-packaging-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for name in ["scripts/package-macos.sh", "scripts/generate-macos-appcast.sh",
                     "scripts/generate-macos-release-notes.py", "packaging/macos/RELEASE_NOTES.md",
                     "packaging/macos/RELEASE_NOTES.unnotarized.md",
                     "scripts/check-macos-appcast.py", "scripts/macos_appcast.py", "site/appcast.xml",
                     "packaging/macos/sparkle_public_ed_key.txt",
                     "packaging/macos/Info.plist", "packaging/macos/Voxelite.entitlements"]:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, path)
        for name in ["tools", "tmp"]:
            (self.root / name).mkdir()
        mock = self.root / "tools/mock"
        mock.write_text(MOCK)
        mock.chmod(0o755)
        for name in ["uname", "cargo", "security", "codesign", "sips", "iconutil",
                     "ditto", "hdiutil", "curl", "xcrun", "spctl"]:
            (self.root / "tools" / name).symlink_to(mock)
        self.archive = self.root / "Sparkle.tar.xz"
        # A tiny fixture replaces the dependency only in the disposable test
        # checkout. The real checksum verifier still runs, including rejection.
        with tarfile.open(self.archive, "w:xz") as archive:
            for name in ["./Sparkle.framework/Versions/B/Sparkle", "./bin/generate_appcast", "./LICENSE"]:
                data = b"fixture"
                entry = tarfile.TarInfo(name)
                entry.size = len(data)
                archive.addfile(entry, io.BytesIO(data))
        packager = self.root / "scripts/package-macos.sh"
        digest = hashlib.sha256(self.archive.read_bytes()).hexdigest()
        packager.write_text(re.sub(r"SPARKLE_SHA256=[0-9a-f]{64}", "SPARKLE_SHA256=" + digest, packager.read_text()))
        self.env = {k: v for k, v in os.environ.items()
                    if not k.startswith(("MACOS_", "APPLE_", "SPARKLE_")) and k != "CARGO_TARGET_DIR"}
        self.env.update(
            PATH=str(self.root / "tools") + os.pathsep + os.environ["PATH"],
            VERSION="1.2.3", TMPDIR=str(self.root / "tmp"),
            MOCK_LOG=str(self.root / "calls"), SECRET_NAMES=json.dumps(SECRETS),
            FIXTURE_ARCHIVE=str(self.archive), TAG="v1.2.3", GITHUB_REPOSITORY="0x63616c/voxelite",
        )

    def run_script(self, name="package-macos.sh", overrides=None, args=(), success=True, trace=False):
        result = subprocess.run(
            ["bash", *(["-x"] if trace else []), "scripts/" + name, *args], cwd=self.root,
            env=self.env | (overrides or {}), capture_output=True, text=True,
        )
        self.assertEqual(result.returncode == 0, success, result.stdout + result.stderr)
        self.assertEqual(list((self.root / "tmp").iterdir()), [], "Temporary credentials leaked")
        return result

    def events(self):
        return [json.loads(line) for line in (self.root / "calls").read_text().splitlines()]

    def generate_notes(self):
        # Use the real notes generator after packaging, as the release workflow does.
        if not (self.root / ".git").exists():
            for args in (["init", "--initial-branch=main"],
                         ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                          "-c", "commit.gpgsign=false", "commit", "--allow-empty", "-m",
                          'Fix <script>alert("x")</script> & café </pre> ]]>'],
                         ["-c", "tag.gpgsign=false", "tag", self.env["TAG"]]):
                subprocess.run(["git", *args], cwd=self.root, env=self.env,
                               check=True, capture_output=True)
        subprocess.run(["python3", "scripts/generate-macos-release-notes.py"],
                       cwd=self.root, env=self.env, check=True, capture_output=True)
        return (self.root / "dist/RELEASE_NOTES.md").read_text(encoding="utf-8")

    def test_release_notes_match_packaging_notarization(self):
        for credentials, notarized in [(CERT | NOTARY, True), (CERT, False),
                                       (CERT | NOTARY, True),
                                       (CERT | NOTARY | {"MACOS_SIGNING_MODE": "adhoc"}, False)]:
            with self.subTest(credentials=list(credentials), notarized=notarized):
                self.run_script(overrides=credentials)
                notes = self.generate_notes()
                self.assertEqual("Developer ID signed and notarized by Apple" in notes, notarized)
                self.assertEqual("right-click" in notes, not notarized)
                self.assertEqual("Open Anyway" in notes, not notarized)

    def test_no_secrets_reach_build_or_tools(self):
        result = self.run_script(overrides=CERT | NOTARY | {"SPARKLE_ED_PRIVATE_KEY": "unused fixture"}, trace=True)
        for value in (CERT | NOTARY | {"SPARKLE_ED_PRIVATE_KEY": "unused fixture"}).values():
            self.assertNotIn(value, result.stdout + result.stderr)
        for name, _, secrets in self.events():
            self.assertEqual(secrets, [], name)
        self.assertTrue((self.root / "dist/.notarized").exists())
        # Check from a download folder, independently of the repository layout.
        subprocess.run(["shasum", "-a", "256", "--check", "Voxelite.dmg.sha256"],
                       cwd=self.root / "dist", check=True, capture_output=True)

    def test_default_public_key_and_matching_override(self):
        raw = (ROOT / "packaging/macos/sparkle_public_ed_key.txt").read_text()
        # The macOS runner's python3 predates tomllib, so read the one value directly.
        feed_url = re.search(
            r'^sparkle_appcast_url\s*=\s*\{\s*current\s*=\s*"([^"]+)"',
            (ROOT / "brand.toml").read_text(), re.M,
        ).group(1)
        self.assertEqual(raw, raw.strip() + "\n")
        self.assertEqual(len(base64.b64decode(raw.strip(), validate=True)), 32)
        for overrides in ({}, PUBLIC):
            self.run_script(overrides=overrides)
            info = plistlib.loads((self.root / "dist/Voxelite.app/Contents/Info.plist").read_bytes())
            self.assertEqual(info['SUPublicEDKey'], raw.strip())
            self.assertEqual(info['SUFeedURL'], feed_url)
            self.assertIs(info['SUEnableAutomaticChecks'], True)
            self.assertIs(info['SUAutomaticallyUpdate'], False)
            signatures = [args for name, args, _ in self.events()
                          if name == 'codesign' and '--verify' not in args]
            self.assertTrue(signatures)
            self.assertTrue(all('--options' not in args for args in signatures))
            self.assertTrue(all('--deep' not in args for args in signatures))

    def test_mismatched_public_override_and_missing_default_fail(self):
        self.run_script(overrides={"SPARKLE_PUBLIC_ED_KEY": base64.b64encode(bytes(32)).decode()}, success=False)
        (self.root / "packaging/macos/sparkle_public_ed_key.txt").unlink()
        self.run_script(overrides=PUBLIC, success=False)

    def test_appcast_requires_credentials_and_notarization(self):
        self.run_script()
        self.run_script("generate-macos-appcast.sh", success=False)
        self.run_script("generate-macos-appcast.sh", {"SPARKLE_ED_PRIVATE_KEY": "fixture private key"}, success=False)

    def test_skip_build(self):
        binary = self.root / "target/release/voxelite"
        binary.parent.mkdir(parents=True)
        binary.write_text("fixture app")
        binary.with_name('voxelite-mcp').write_text('fixture server')
        self.run_script(overrides=CERT, args=["--skip-build"])
        self.assertNotIn("cargo", [e[0] for e in self.events()])

    def test_bundled_executables_signed_before_bundle(self):
        self.run_script(overrides=CERT)
        builds = [args for name, args, _ in self.events() if name == 'cargo']
        self.assertIn(['build', '--release', '--locked', '-p', 'voxelite-mcp'], builds)
        signatures = [args for name, args, _ in self.events()
                      if name == 'codesign' and '--verify' not in args]
        self.assertEqual([Path(args[-1]).name for args in signatures[5:8]],
                         ['voxelite-mcp', 'Voxelite', 'Voxelite.app'])
        for args in signatures[5:8]:
            self.assertEqual(args[args.index('--sign') + 1], 'A' * 40)
            self.assertEqual(args[args.index('--options') + 1], 'runtime')
        for binary in ('voxelite-mcp', 'Voxelite'):
            source = 'voxelite' if binary == 'Voxelite' else binary
            self.assertEqual((self.root / f'dist/Voxelite.app/Contents/MacOS/{binary}').read_bytes(),
                             (self.root / f'target/release/{source}').read_bytes())

    def test_skip_build_requires_mcp_executable(self):
        binary = self.root / 'target/release/voxelite'
        binary.parent.mkdir(parents=True)
        binary.write_text('fixture app')
        self.run_script(args=['--skip-build'], success=False)
        self.assertNotIn('codesign', [e[0] for e in self.events()])

    def test_adhoc_and_partial_credentials(self):
        for credentials in [{}, {"MACOS_CERTIFICATE_P12_BASE64": CERT["MACOS_CERTIFICATE_P12_BASE64"]},
                            CERT | NOTARY | {"MACOS_SIGNING_MODE": "adhoc"}]:
            self.run_script(overrides=credentials)
            self.assertFalse((self.root / "dist/.notarized").exists())
        self.assertNotIn("security", [e[0] for e in self.events()])

    def test_empty_keychain_list_and_lowercase_fingerprint(self):
        self.run_script(overrides=CERT | {"KEYCHAIN_LIST": "", "MACOS_SIGNING_IDENTITY": "a" * 40})
        lists = [args for name, args, _ in self.events() if name == "security" and args[0] == "list-keychains"]
        self.assertEqual(lists[-1], ["list-keychains", "-d", "user", "-s"])

    def test_malformed_keychain_list_stops_before_mutation(self):
        self.run_script(overrides=CERT | {"KEYCHAIN_LIST": '"unterminated'}, success=False)
        self.assertNotIn(["create-keychain"], [e[1] for e in self.events()])

    def test_private_failures_clean_keychain_and_hide_diagnostics(self):
        for operation in ["security:import", "codesign", "xcrun:notarytool", "spctl"]:
            with self.subTest(operation=operation):
                result = self.run_script(overrides=CERT | NOTARY | {"FAIL_TOOL": operation}, success=False)
                self.assertNotIn("PRIVATE-DIAGNOSTIC-SENTINEL", result.stdout + result.stderr)
                self.assertEqual(self.events()[-2][:2], ["security", ["delete-keychain"]])
                self.assertFalse((self.root / "dist/.notarized").exists())

    def test_term_cleans_keychain(self):
        result = self.run_script(overrides=CERT | {"SIGNAL_AT": "security:import"}, success=False)
        self.assertEqual(result.returncode, 143)
        self.assertIn(["delete-keychain"], [e[1] for e in self.events()])

    def test_failure_cannot_reuse_notarized_marker(self):
        self.run_script(overrides=CERT | NOTARY)
        self.run_script(overrides={"FAIL_TOOL": "cargo"}, success=False)
        self.assertFalse((self.root / "dist/.notarized").exists())

    def test_invalid_inputs_and_notarization_rejection(self):
        for overrides in [{"VERSION": "main"}, {"SPARKLE_PUBLIC_ED_KEY": "invalid"},
                          CERT | {"MACOS_CERTIFICATE_P12_BASE64": "invalid"},
                          CERT | NOTARY | {"NOTARY_STATUS": "Invalid"},
                          PUBLIC | {"CORRUPT_ARCHIVE": "1"}]:
            with self.subTest(keys=list(overrides)):
                self.run_script(overrides=overrides, success=False)
                self.assertFalse((self.root / "dist/.notarized").exists())

    def test_sparkle_signing_and_feed_validation(self):
        self.run_script(overrides=CERT | NOTARY | PUBLIC)
        with (self.root / 'dist/Voxelite.app/Contents/Info.plist').open('rb') as source:
            info = plistlib.load(source)
        self.assertIs(info.get('SURequireSignedFeed'), True)
        self.assertIs(info.get('SUVerifyUpdateBeforeExtraction'), True)
        signatures = [Path(args[-1]).name for name, args, _ in self.events()
                      if name == "codesign" and "--verify" not in args]
        self.assertEqual(signatures, ["Installer.xpc", "Downloader.xpc", "Autoupdate",
                                      "Updater.app", "Sparkle.framework", "voxelite-mcp", "Voxelite", "Voxelite.app", "Voxelite.dmg"])
        generator = self.root / "dist/sparkle/bin/generate_appcast"
        generator.write_text(GENERATOR)
        generator.chmod(0o755)
        verifier = self.root / "dist/sparkle/bin/sign_update"
        verifier.write_text(VERIFIER)
        verifier.chmod(0o755)
        shutil.copyfile(self.root / "site/appcast.xml", self.root / "dist/previous-appcast.xml")
        output = self.root / "outputs"
        credentials = {"SPARKLE_ED_PRIVATE_KEY": "fixture private key", "GITHUB_OUTPUT": str(output)}
        notes = self.root / "dist/RELEASE_NOTES.md"
        self.assertIn("Developer ID signed and notarized by Apple", self.generate_notes())
        for overrides, success in [({}, True), ({"TAG": "v9.9.9"}, False),
                                   ({"UNSIGNED_FEED": "1"}, False), ({"WRONG_URL": "1"}, False),
                                   ({"UNSIGNED_METADATA": "1"}, False), ({"GENERATOR_FAILURE": "1"}, False),
                                   ({"MISSING_NOTES": "1"}, False), ({"WRONG_NOTES": "1"}, False)]:
            with self.subTest(overrides=overrides):
                output.unlink(missing_ok=True)
                result = self.run_script("generate-macos-appcast.sh", credentials | overrides, success=success, trace=True)
                self.assertEqual("generated=true" in output.read_text(), success)
                self.assertNotIn("PRIVATE-DIAGNOSTIC-SENTINEL", result.stdout + result.stderr)
                self.assertNotIn("fixture private key", result.stdout + result.stderr)
                if success:
                    description = (self.root / "dist/Voxelite.html").read_text()
                    self.assertIn("&lt;script&gt;", description)
                    self.assertIn("&amp;", description)
                    parser = HTMLParser()
                    text = []
                    tags = []
                    parser.handle_data = text.append
                    parser.handle_starttag = lambda tag, attrs: tags.append(tag)
                    parser.feed(description)
                    self.assertEqual("".join(text).strip(), notes.read_text(encoding="utf-8").strip())
                    self.assertEqual(tags, ["html", "head", "meta", "body", "pre"])
        # Preserve the first signed update when generating a newer one.
        shutil.copyfile(self.root / "site/appcast.xml", self.root / "dist/previous-appcast.xml")
        plist = self.root / "dist/Voxelite.app/Contents/Info.plist"
        info = plistlib.loads(plist.read_bytes())
        info["CFBundleVersion"] = info["CFBundleShortVersionString"] = "1.2.4"
        plist.write_bytes(plistlib.dumps(info))
        credentials["TAG"] = "v1.2.4"
        self.run_script("generate-macos-appcast.sh", credentials | {"DROP_HISTORY": "1"}, success=False)
        self.run_script("generate-macos-appcast.sh", credentials)
        items = ET.parse(self.root / "site/appcast.xml").findall("channel/item")
        ns = "{http://www.andymatuschak.org/xml-namespaces/sparkle}"
        self.assertEqual([item.findtext(ns + "version") for item in items], ["1.2.4", "1.2.3"])
        self.run_script("generate-macos-appcast.sh", credentials | {"TAG": "v1.2.3"}, success=False)
        notes.unlink()
        self.run_script("generate-macos-appcast.sh", credentials, success=False)
        dmg = self.root / "dist/Voxelite.dmg"
        dmg.write_text("corrupt dmg")
        self.run_script("generate-macos-appcast.sh", credentials, success=False)


if __name__ == "__main__":
    unittest.main()
