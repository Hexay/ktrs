#!/usr/bin/env bash
# Usage: package-vscode.sh <dir with the release archives> <out dir>
# Builds one VSIX per VS Code target with that target's `ktrs` in bundled/, plus a universal one without a
# binary (it runs `ktrs` from PATH). Needs `npm ci` done in editors/vscode.
set -euo pipefail
dist=$(realpath "$1") out=$(realpath -m "$2")
ext=$(cd "$(dirname "$0")/../../editors/vscode" && pwd)
version=$(node -p "require('$ext/package.json').version")
mkdir -p "$out"
cp "$ext/../../LICENSE-MIT" "$ext/../../LICENSE-APACHE" "$ext/"
vsce() { (cd "$ext" && npx vsce package --skip-license "$@"); }

for pair in x86_64-unknown-linux-musl:linux-x64,alpine-x64 aarch64-unknown-linux-musl:linux-arm64,alpine-arm64 \
            x86_64-apple-darwin:darwin-x64 aarch64-apple-darwin:darwin-arm64 \
            x86_64-pc-windows-msvc:win32-x64 aarch64-pc-windows-msvc:win32-arm64; do
  target=${pair%%:*}
  rm -rf "$ext/bundled" && mkdir -p "$ext/bundled"
  case $target in
    *windows*) unzip -j -q -o "$dist"/ktrs-*-"$target".zip '*/ktrs.exe' -d "$ext/bundled" ;;
    *) tar -xzf "$dist"/ktrs-*-"$target".tar.gz -C "$ext/bundled" --strip-components=1 --wildcards '*/ktrs' ;;
  esac
  IFS=, read -ra vscode_targets <<< "${pair##*:}"
  for t in "${vscode_targets[@]}"; do
    vsce --target "$t" -o "$out/ktrs-vscode-$t-$version.vsix"
  done
done
rm -rf "$ext/bundled"
vsce -o "$out/ktrs-vscode-universal-$version.vsix"
ls -l "$out"
