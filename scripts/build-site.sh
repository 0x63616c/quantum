#!/usr/bin/env bash
# Generate static documentation from the actual MCP catalog and bundle the editor.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
case "${1:-}" in
  --skip-web) ;; # Reuse an already verified web/dist bundle for content iteration.
  '') scripts/build-web.sh ;; # Includes source-path remapping for the published wasm.
  *) printf 'Usage: scripts/build-site.sh [--skip-web]\n' >&2; exit 2 ;;
esac
for asset in index.html voxelite_app.js voxelite_app_bg.wasm; do
  test -s "web/dist/$asset" || { printf 'Missing web/dist/%s; run scripts/build-web.sh first.\n' "$asset" >&2; exit 1; }
done
# Refresh the HTML source even when reusing the compiled editor bundle.
cp web/index.html web/dist/index.html
cargo run --locked -p voxelite-site -- site
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
mkdir -p site/app
cp web/dist/* site/app/
printf 'Site ready: python3 -m http.server 8090 --directory site --bind 127.0.0.1\n'
