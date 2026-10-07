#!/usr/bin/env bash
# Checks out the pinned ktlint (third_party/ktlint, gitignored) and fetches its CLI fat jar into
# tools/ktlint-oracle/lib (gitignored). The fat jar carries everything the oracle needs: rule engine,
# standard rule set, the embedded Kotlin compiler (2.4.10 for 2.0.0-ALPHA-4) and kotlin-stdlib.
set -euo pipefail
KTLINT_TAG=2.0.0-ALPHA-4
root="$(cd "$(dirname "$0")/.." && pwd)"

dir="$root/third_party/ktlint"
[[ -d $dir/.git ]] || git -c advice.detachedHead=false clone -q --depth 1 --branch "$KTLINT_TAG" https://github.com/ktlint/ktlint.git "$dir"
at=$(git -C "$dir" describe --tags --exact-match 2>/dev/null || git -C "$dir" rev-parse --short HEAD)
[[ $at == "$KTLINT_TAG" ]] || echo "warning: third_party/ktlint is at $at, not $KTLINT_TAG" >&2

jar="$root/tools/ktlint-oracle/lib/ktlint-cli-$KTLINT_TAG-all.jar"
mkdir -p "$(dirname "$jar")"
if [[ ! -f $jar ]]; then
  curl -sfL -o "$jar.part" \
    "https://repo1.maven.org/maven2/io/github/ktlint/core/ktlint-cli/$KTLINT_TAG/ktlint-cli-$KTLINT_TAG-all.jar"
  mv "$jar.part" "$jar"
fi

"$root/tools/ensure-jdk.sh"
echo "ktlint $KTLINT_TAG ($at), oracle jar $(du -h "$jar" | cut -f1)"
