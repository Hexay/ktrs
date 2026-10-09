#!/usr/bin/env bash
# Prints a rules_multitool lockfile for locally built binaries, for this machine's platform only:
#   bazel/e2e/local-lock.sh target/debug > ktrs.lock.json
# Releases get theirs from tools/release/multitool-lock.sh.
set -euo pipefail
dir="$(cd "$1" && pwd)"
case $(uname -s) in Linux) os=linux ;; Darwin) os=macos ;; *) echo "unsupported OS $(uname -s)" >&2; exit 1 ;; esac
case $(uname -m) in x86_64) cpu=x86_64 ;; aarch64 | arm64) cpu=arm64 ;; *) echo "unsupported CPU $(uname -m)" >&2; exit 1 ;; esac
sha() { if command -v sha256sum > /dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi | cut -d' ' -f1; }

sep=""
echo "{"
for tool in ktrs ktfmt ktlint; do
  [[ -x $dir/$tool ]] || { echo "no $tool in $dir" >&2; exit 1; }
  printf '%s  "%s": {\n    "binaries": [\n' "$sep" "$tool"; sep=$',\n'
  printf '      { "kind": "file", "url": "file://%s/%s", "sha256": "%s", "os": "%s", "cpu": "%s" }\n' \
    "$dir" "$tool" "$(sha "$dir/$tool")" "$os" "$cpu"
  printf '    ]\n  }'
done
printf '\n}\n'
