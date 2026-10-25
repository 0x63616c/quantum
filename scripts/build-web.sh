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
# Remap source locations in panic messages as well as debug metadata. Keep broad
# prefixes first: rustc applies the last matching remap. Encoded flags preserve
# paths containing spaces and any flags supplied by the caller.
repo_root=$(pwd -P)
if [[ -z "${CARGO_ENCODED_RUSTFLAGS:-}" ]]; then
  read -r -a caller_flags <<< "${RUSTFLAGS:-}"
  CARGO_ENCODED_RUSTFLAGS=$(IFS=$'\x1f'; printf '%s' "${caller_flags[*]}")
fi
web_flags=(
  "--remap-path-prefix=$HOME=~"
  "--remap-path-prefix=$repo_root=/voxelite"
  "--remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}/registry/src=/cargo"
  "--remap-path-prefix=${RUSTUP_HOME:-$HOME/.rustup}=/rustup"
  "-C" "debuginfo=0"
)
for flag in "${web_flags[@]}"; do
  CARGO_ENCODED_RUSTFLAGS+="${CARGO_ENCODED_RUSTFLAGS:+$'\x1f'}$flag"
done
export CARGO_ENCODED_RUSTFLAGS
cargo build --locked -p voxelite-app --lib --no-default-features --features web --target wasm32-unknown-unknown --profile web
mkdir -p web/dist
wasm-bindgen --target web --out-dir web/dist --out-name voxelite_app target/wasm32-unknown-unknown/web/voxelite_app.wasm
cp web/index.html web/dist/index.html
printf 'Web editor ready: python3 -m http.server 8080 --directory web/dist --bind 127.0.0.1\n'
