#!/usr/bin/env bash
# Usage: docker-context.sh <dir with the release archives> <out dir>
# Unpacks the Linux binaries into <out>/linux-<amd64|arm64>/, the build context of docker/Dockerfile.
set -euo pipefail
dist=$1 out=$2
for pair in x86_64-unknown-linux-musl:amd64 aarch64-unknown-linux-musl:arm64; do
  target=${pair%%:*} dir=$out/linux-${pair##*:}
  archives=("$dist"/ktrs-*-"$target".tar.gz)
  [[ -e ${archives[0]} ]] || { echo "skipping linux/${pair##*:}: no $target archive" >&2; continue; }
  mkdir -p "$dir"
  tar -xzf "${archives[0]}" -C "$dir" --strip-components=1 --wildcards '*/ktrs' '*/ktfmt' '*/ktlint'
done
find "$out" -type f
