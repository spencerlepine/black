#!/bin/sh
# Builds the playground's WASM module into web/. Serve web/ over HTTP (see web/README.md).
set -e
cd "$(dirname "$0")/.."
cargo build --release --target wasm32-unknown-unknown -p black-battle-wasm
cp target/wasm32-unknown-unknown/release/black_battle_wasm.wasm web/
