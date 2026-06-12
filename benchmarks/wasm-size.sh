#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

rustc_path="${RUSTC:-$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin/rustc}"

build_and_measure() {
  local name="$1" features="$2"
  RUSTC="$rustc_path" CARGO_TARGET_DIR="target/wasm-$name" \
    rustup run stable cargo build -p spoor-wasm --release \
      --target wasm32-unknown-unknown --no-default-features --features "$features"
  local wasm="target/wasm-$name/wasm32-unknown-unknown/release/spoor_wasm.wasm"
  printf '%s raw bytes: ' "$name"
  wc -c < "$wasm"
  printf '%s gzip bytes: ' "$name"
  gzip -9 -c "$wasm" | wc -c
}

build_and_measure core core-formats
build_and_measure full full
