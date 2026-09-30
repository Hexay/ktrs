#!/usr/bin/env bash
# Usage: bundle-natives.sh <dir with the release archives> <out dir>
# Unpacks `ktrs` from each target's archive into <out>/<platform>/, the layout java/ bundles
# (platform names as in NativeBinary.platform()).
set -euo pipefail
dist=$1 out=$2
for pair in x86_64-unknown-linux-musl:linux-x86_64 aarch64-unknown-linux-musl:linux-aarch64 \
            x86_64-apple-darwin:macos-x86_64 aarch64-apple-darwin:macos-aarch64 \
            x86_64-pc-windows-msvc:windows-x86_64 aarch64-pc-windows-msvc:windows-aarch64; do
  target=${pair%%:*} dir=$out/${pair##*:}
  mkdir -p "$dir"
  case $target in
    *windows*) unzip -j -q -o "$dist"/ktrs-*-"$target".zip '*/ktrs.exe' -d "$dir" ;;
    *) tar -xzf "$dist"/ktrs-*-"$target".tar.gz -C "$dir" --strip-components=1 --wildcards '*/ktrs' ;;
  esac
done
find "$out" -type f
