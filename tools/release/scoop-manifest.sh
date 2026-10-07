#!/usr/bin/env bash
# Prints bucket/ktrs.json for a release, from its SHA256SUMS (the bucket is this repo:
# `scoop bucket add ktrs https://github.com/Hexay/ktrs`). Run by the release workflow:
#   tools/release/scoop-manifest.sh v0.5.0 SHA256SUMS > bucket/ktrs.json
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
tag=$1 sums=$2
# shellcheck source=release-assets.sh
source "$here/release-assets.sh"

arch() {
  local target=$1
  cat <<EOF
      "url": "$(asset_url "$target")",
      "hash": "$(asset_sha "$target")",
      "extract_dir": "ktrs-$tag-$target"
EOF
}

cat <<EOF
{
  "version": "${tag#v}",
  "description": "Fast Kotlin formatter and linter: drop-in ktfmt and ktlint binaries",
  "homepage": "https://github.com/Hexay/ktrs",
  "license": "MIT|Apache-2.0",
  "architecture": {
    "64bit": {
$(arch x86_64-pc-windows-msvc)
    },
    "arm64": {
$(arch aarch64-pc-windows-msvc)
    }
  },
  "bin": ["ktrs.exe", "ktfmt.exe", "ktlint.exe"]
}
EOF
