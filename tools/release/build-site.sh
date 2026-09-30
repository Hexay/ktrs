#!/usr/bin/env bash
# Builds the browser playground into <out-dir> (default target/site): site/ plus ktrs.wasm.
# Preview: tools/release/build-site.sh && python3 -m http.server -d target/site
set -euo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd)
out=${1:-$root/target/site}
cargo build -q --locked -p ktrs-wasm --profile wasm --target wasm32-unknown-unknown --manifest-path "$root/Cargo.toml"
mkdir -p "$out"
cp "$root"/site/* "$out/"
cp "$root/target/wasm32-unknown-unknown/wasm/ktrs_wasm.wasm" "$out/ktrs.wasm"
echo "site in $out ($(du -h "$out/ktrs.wasm" | cut -f1) wasm)"
