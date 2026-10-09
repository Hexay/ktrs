#!/usr/bin/env bash
# Checks out the pinned detekt (third_party/detekt, gitignored), the source of the native port in
# crates/ktrs-detekt, and fetches its CLI fat jar into tools/detekt-oracle/lib (gitignored). The fat jar carries
# the engine, the nine core rule sets, the embedded Kotlin compiler (2.4.10 for 2.0.0-alpha.6) and kotlin-stdlib.
# A bump also needs `crates/ktrs-detekt/src/default-detekt-config.yml` recopied and the goldens regenerated.
set -euo pipefail
DETEKT_TAG=v2.0.0-alpha.6
DETEKT_JAR_SHA256=d46ca62e
root="$(cd "$(dirname "$0")/.." && pwd)"

dir="$root/third_party/detekt"
[[ -d $dir/.git ]] || git -c advice.detachedHead=false clone -q --depth 1 --branch "$DETEKT_TAG" \
  https://github.com/detekt/detekt.git "$dir"
at=$(git -C "$dir" describe --tags --exact-match 2>/dev/null || git -C "$dir" rev-parse --short HEAD)
[[ $at == "$DETEKT_TAG" ]] || echo "warning: third_party/detekt is at $at, not $DETEKT_TAG" >&2

version=${DETEKT_TAG#v}
jar="$root/tools/detekt-oracle/lib/detekt-cli-$version-all.jar"
mkdir -p "$(dirname "$jar")"
if [[ ! -f $jar ]]; then
  curl -sfL -o "$jar.part" "https://github.com/detekt/detekt/releases/download/$DETEKT_TAG/detekt-cli-$version-all.jar"
  mv "$jar.part" "$jar"
fi
sum=$(sha256sum "$jar" | cut -c1-${#DETEKT_JAR_SHA256})
[[ $sum == "$DETEKT_JAR_SHA256" ]] || { echo "error: $jar sha256 starts with $sum, expected $DETEKT_JAR_SHA256" >&2; exit 1; }

cmp -s "$dir/detekt-core/src/main/resources/default-detekt-config.yml" \
  "$root/crates/ktrs-detekt/src/default-detekt-config.yml" 2>/dev/null ||
  echo "warning: crates/ktrs-detekt/src/default-detekt-config.yml differs from the pinned one" >&2

"$root/tools/ensure-jdk.sh"
echo "detekt $DETEKT_TAG ($at), oracle jar $(du -h "$jar" | cut -f1)"
