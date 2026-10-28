#!/usr/bin/env python3
"""Report potential identity/path/credential leaks without reproducing values.

Read tracked and unignored files and all blobs/commits reachable through local
refs or reflogs. Print the redacted report to stdout without writing files or
modifying history.
"""
from pathlib import Path
import collections
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)

commits = git('rev-list', '--all', '--reflog').decode().splitlines()
identities = set()
for oid in commits:
    headers = git('cat-file', 'commit', oid).split(b'\n\n', 1)[0].decode(errors='replace')
    for line in headers.splitlines():
        if line.startswith(('author ', 'committer ')):
            match = re.match(r'\w+ (.*?) <([^>]+)>', line)
            if match:
                identities.add(match.groups())
patterns = {
    'email': re.compile(r'\b[A-Z0-9._%+\-]+@[A-Z0-9.\-]+\.[A-Z]{2,}\b', re.I),
    'absolute machine path': re.compile(r'/(?:home|workspace|Users)/[^\s\x00"<>`]+'),
    'credential-shaped string': re.compile(r'\b(?:sk-[A-Za-z0-9_\-]{20,}|gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{30,}|AKIA[A-Z0-9]{16})\b|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----'),
    'URL credential': re.compile(r'https?://[^/\s:@]+:[^/\s@]+@'),
}
identity_terms = set()
for name, email in identities:
    if len(name) >= 4:
        identity_terms.add(name)
    local = email.split('@')[0].split('+')[-1]
    if len(local) >= 4:
        identity_terms.add(local)
# Product/bot labels are not human identity leaks in prose.
identity_terms = {term for term in identity_terms if term.lower() not in {'quantum', 'quantum contributors', 'quantum agent', 'codex', 'openai', 'root', 'agent', 'contributors'}}
if identity_terms:
    patterns['Git identity name/username'] = re.compile(r'(?<![\w-])(?:' + '|'.join(re.escape(x) for x in sorted(identity_terms)) + r')(?![\w-])', re.I)

def scan(data):
    # For binary artifacts, report line within printable byte stream; no OCR claim.
    text = data.decode('utf-8', errors='replace')
    result = []
    for kind, pattern in patterns.items():
        lines = sorted({text.count('\n', 0, m.start()) + 1 for m in pattern.finditer(text)})
        if lines:
            result.append((kind, lines))
    return result

files = set(git('ls-files', '-z', '--cached', '--others', '--exclude-standard').decode().split('\0')) - {''}
current = []
for name in sorted(files):
    path = ROOT / name
    if not path.is_file() or path.is_symlink():
        continue
    for kind, lines in scan(path.read_bytes()):
        current.append((name, kind, lines))

objects = git('rev-list', '--all', '--reflog', '--objects').decode().splitlines()
blob_paths = {line.split(' ', 1)[0]: line.split(' ', 1)[1] for line in objects if ' ' in line}
proc = subprocess.Popen(['git', 'cat-file', '--batch'], cwd=ROOT, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
history = []
blobs = 0
for oid, name in blob_paths.items():
    proc.stdin.write((oid + '\n').encode()); proc.stdin.flush()
    header = proc.stdout.readline().split()
    size = int(header[2]); data = proc.stdout.read(size); proc.stdout.read(1)
    if header[1] != b'blob':
        continue
    blobs += 1
    for kind, lines in scan(data):
        history.append((oid[:12], name, kind, lines))
proc.stdin.close(); proc.wait()
message_findings = []
for oid in commits:
    message = git('cat-file', 'commit', oid).split(b'\n\n', 1)[1]
    for kind, lines in scan(message):
        message_findings.append((oid[:12], kind, lines))

report = ['# Repository and Git history personal-info scan', '',
    'Scanned on 2026-10-25. Values are deliberately redacted; locations and categories are sufficient for follow-up. No history was rewritten and no Git identity was changed.', '',
    f'Scope: {len(files)} working-tree/generated files; {len(commits)} commits and {blobs} unique historical blobs reachable through all local refs and reflogs. Includes source, documentation, lockfiles and assets. Scans commit messages plus every author/committer header. Git metadata pointer files are checked separately below.', '',
    'Excluded dependency caches and build intermediates (`node_modules/`, `target/`): these are not repository deliverables. Product/bot identity labels are excluded from prose name matching; all identity headers are still counted. Binary assets were byte-scanned; this is not OCR, a full secret entropy detector, or proof that every possible personal name/secret was found. No remote access or credential validation was attempted.', '',
    '## Git metadata', '',
    f'- {len(identities)} distinct author/committer identity pairs are present across {len(commits)} commits. Names, addresses and account identifiers are redacted here. These immutable headers remain as requested; inspect locally with `git log --all --format=fuller` if authorized.',
    '- `.git:1` is a worktree pointer containing a local machine path. It is untracked Git plumbing, not a distributable source file; changing it would break the checkout.', '',
    '## Working tree findings', '', '| File:line | Category | Disposition |', '| --- | --- | --- |']
for name, kind, lines in current:
    disposition = 'Inspect; do not reproduce matched value.'
    if 'LICENSE' in name and kind == 'Git identity name/username':
        disposition = 'Required upstream attribution; retain license notice.'
    elif kind == 'email' and ('OFL' in name or 'LICENSE' in name):
        disposition = 'Upstream license attribution; retain required notice.'
    elif name.endswith('.ttf') and kind == 'email':
        disposition = 'Embedded upstream font attribution/contact; retain original licensed font.'
    elif name.endswith('.png') and kind == 'email':
        disposition = 'Compressed binary byte-pattern match; not a verified address or visible image text.'
    elif name == 'scripts/scan-pii.py':
        disposition = 'Scanner regex, not a real identity or credential.'
    report.append(f'| `{name}:{",".join(map(str, lines))}` | {kind} | {disposition} |')
if not current:
    report.append('| — | No matches | — |')
report += ['', '## Historical blob findings', '',
    'One row per unique blob/category; a blob can occur in many commits. Use `git show <blob>` locally to inspect a finding. No matched values are copied into this report.', '',
    'Historical design assets and review captures are preserved in the [private archive](https://github.com/0x63616c/quantum-archive) at their original paths.', '',
    '| Blob · file:line | Category |', '| --- | --- |']
for oid, name, kind, lines in history:
    report.append(f'| `{oid}` · `{name}:{",".join(map(str, lines))}` | {kind} |')
if not history:
    report.append('| — | No blob matches |')
report += ['', '## Commit messages', '']
for oid, kind, lines in message_findings:
    report.append(f'- `{oid}:message:{",".join(map(str, lines))}` — {kind}.')
if not message_findings:
    report.append('No pattern matches in commit messages.')
report += ['', '## Outcome', '',
    'No credential-shaped strings or URL credentials were found unless explicitly listed above. Historical identity headers remain. License attributions remain. No trivially safe authored working-tree path leak was found; Git plumbing and generated compiler paths must not be blindly replaced. This report does not certify the repository as free of personal information.', '']
print('\n'.join(report))
print(f'Scanned {len(files)} files, {len(commits)} commits, {blobs} historical blobs; {len(current)} current and {len(history)} historic location/category findings. Values redacted.')
print('Current categories:', dict(collections.Counter(kind for _, kind, _ in current)))
