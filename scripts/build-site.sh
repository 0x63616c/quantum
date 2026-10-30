#!/usr/bin/env bash
# Generate the static website and documentation from the actual MCP catalog.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
if (( $# != 0 )); then
  printf 'Usage: scripts/build-site.sh\n' >&2
  exit 2
fi
# Also exports Markdown/LLM indexes and preserves site/appcast.xml if supplied.
cargo run --locked -p voxelite-site -- site --write-integrations
mkdir -p site/assets/skills
cp -R skills/. site/assets/skills/
mkdir -p site/assets/integrations
cp -R integrations/. site/assets/integrations/
# Stable paths, metadata and ordering make downloads reproducible and self-contained.
python3 - <<'PYZIP'
from pathlib import Path
from zipfile import ZipFile, ZipInfo, ZIP_DEFLATED
with ZipFile('site/assets/voxelite-skills.zip', 'w', compression=ZIP_DEFLATED) as archive:
    for root in ('skills', 'integrations'):
        for path in sorted(Path(root).rglob('*')):
            if path.is_file():
                info = ZipInfo(path.as_posix())
                info.compress_type = ZIP_DEFLATED
                info.external_attr = 0o100644 << 16
                archive.writestr(info, path.read_bytes())
PYZIP
printf 'Site ready: python3 -m http.server 8090 --directory site --bind 127.0.0.1\n'
