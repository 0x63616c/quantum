#!/usr/bin/env bash
# Build the shared Rust GPUI editor as a standalone static browser bundle.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
rustup target add wasm32-unknown-unknown
bindgen_version=$(sed -n '/^name = "wasm-bindgen"$/{n;s/version = "\(.*\)"/\1/p;}' Cargo.lock)
if ! command -v wasm-bindgen >/dev/null || [[ "$(wasm-bindgen --version)" != "wasm-bindgen $bindgen_version" ]]; then
  cargo install wasm-bindgen-cli --version "$bindgen_version" --locked
fi
cargo build --locked -p voxelite-app --lib --no-default-features --features web --target wasm32-unknown-unknown --profile web
mkdir -p web/dist
wasm-bindgen --target web --out-dir web/dist --out-name voxelite_app target/wasm32-unknown-unknown/web/voxelite_app.wasm
cp web/index.html web/dist/index.html
printf 'Web editor ready: python3 -m http.server 8080 --directory web/dist --bind 127.0.0.1\n'
