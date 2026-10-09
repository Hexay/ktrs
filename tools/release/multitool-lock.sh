#!/usr/bin/env bash
# Prints a rules_multitool lockfile (tools `ktrs`, `ktfmt`, `ktlint`) for a release, from its SHA256SUMS; attached to
# the release as ktrs-<tag>.multitool.lock.json (README "Bazel"). Run by the release workflow:
#   tools/release/multitool-lock.sh v0.5.1 SHA256SUMS > ktrs-v0.5.1.multitool.lock.json
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
tag=$1 sums=$2
# shellcheck source=release-assets.sh
source "$here/release-assets.sh"

# <target triple> <rules_multitool os> <rules_multitool cpu>
platforms=(
  "x86_64-unknown-linux-musl linux x86_64"
  "aarch64-unknown-linux-musl linux arm64"
  "x86_64-apple-darwin macos x86_64"
  "aarch64-apple-darwin macos arm64"
  "x86_64-pc-windows-msvc windows x86_64"
  "aarch64-pc-windows-msvc windows arm64"
)

# Here, in the main shell: asset_sha's `exit` inside the $(...) below would not stop the script.
for platform in "${platforms[@]}"; do asset_sha "${platform%% *}" > /dev/null; done

binaries() {
  local tool=$1 platform target os cpu exe sep=""
  for platform in "${platforms[@]}"; do
    read -r target os cpu <<< "$platform"
    exe=""; [[ $os == windows ]] && exe=".exe"
    printf '%s      {\n' "$sep"; sep=$',\n'
    printf '        "kind": "archive",\n'
    printf '        "url": "%s",\n' "$(asset_url "$target")"
    printf '        "sha256": "%s",\n' "$(asset_sha "$target")"
    printf '        "file": "ktrs-%s-%s/%s%s",\n' "$tag" "$target" "$tool" "$exe"
    printf '        "os": "%s",\n        "cpu": "%s"\n      }' "$os" "$cpu"
  done
}

tool() { printf '  "%s": {\n    "binaries": [\n%s\n    ]\n  }' "$1" "$(binaries "$1")"; }

cat <<EOF
{
  "\$schema": "https://raw.githubusercontent.com/theoremlp/rules_multitool/main/lockfile.schema.json",
$(tool ktrs),
$(tool ktfmt),
$(tool ktlint)
}
EOF
