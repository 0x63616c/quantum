#!/usr/bin/env bash
# Generate static documentation from the actual MCP catalog and bundle the editor.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
case "${1:-}" in
  --skip-web) ;; # Reuse an already verified web/dist bundle for content iteration.
  '') scripts/build-web.sh ;;
  *) printf 'Usage: scripts/build-site.sh [--skip-web]\n' >&2; exit 2 ;;
esac
for asset in index.html voxelite_app.js voxelite_app_bg.wasm; do
  test -s "web/dist/$asset" || { printf 'Missing web/dist/%s; run scripts/build-web.sh first.\n' "$asset" >&2; exit 1; }
done
cargo run --locked -p voxelite-site -- site
mkdir -p site/app
cp web/dist/* site/app/
printf 'Site ready: python3 -m http.server 8090 --directory site --bind 127.0.0.1\n'
