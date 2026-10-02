#!/usr/bin/env bash
# Checks out the pinned mrmans0n/compose-rules (third_party/compose-rules, gitignored), the source of the native
# port in crates/ktrs-compose, and fetches its ktlint CLI jar (`-R` jar) into tools/compose-rules/lib (gitignored).
# A bump also needs the jar's fingerprint in crates/ktrs-compose (research/27-custom-rulesets-impl.md).
set -euo pipefail
COMPOSE_RULES_TAG=v0.6.7
root="$(cd "$(dirname "$0")/.." && pwd)"

dir="$root/third_party/compose-rules"
[[ -d $dir/.git ]] || git -c advice.detachedHead=false clone -q --depth 1 --branch "$COMPOSE_RULES_TAG" \
  https://github.com/mrmans0n/compose-rules.git "$dir"
at=$(git -C "$dir" describe --tags --exact-match 2>/dev/null || git -C "$dir" rev-parse --short HEAD)
[[ $at == "$COMPOSE_RULES_TAG" ]] || echo "warning: third_party/compose-rules is at $at, not $COMPOSE_RULES_TAG" >&2

version=${COMPOSE_RULES_TAG#v}
jar="$root/tools/compose-rules/lib/ktlint-compose-$version-all.jar"
mkdir -p "$(dirname "$jar")"
[[ -f $jar ]] || curl -sfL -o "$jar" \
  "https://github.com/mrmans0n/compose-rules/releases/download/$COMPOSE_RULES_TAG/ktlint-compose-$version-all.jar"
echo "compose-rules $COMPOSE_RULES_TAG ($at), jar $(du -h "$jar" | cut -f1)"
