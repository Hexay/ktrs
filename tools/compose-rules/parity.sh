#!/usr/bin/env bash
# compose-rules parity on real code: ktlint + the compose-rules -R jar vs the ktrs `ktlint` drop-in with the same jar
# (which runs crates/ktrs-compose natively), through tools/parity/ktlint-compare.sh (lint rows and -F trees).
#
#   tools/compose-rules/parity.sh KTLINT_1_8 KTLINT_2_0 KTRS_KTLINT OUT TREE...
#
# KTLINT_*: the ktlint release launchers; KTRS_KTLINT: target/release/ktlint. Per tree and ktlint version: `compose-only`
# (ktlint_standard = disabled: the port alone) and `with-standard` (both rule sets, as users run them), style
# ktlint_official. JAR = the -R jar (default: tools/compose-rules/lib/ktlint-compose-<pin>-all.jar). Slow (JVM): background.
# Exit status: tools/parity/check-known.sh against KNOWN (default tools/parity/known-diffs/compose-rules.tsv).
set -uo pipefail
(($# >= 5)) || { sed -n 2,10p "$0"; exit 2; }
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
K18=$1 K20=$2 OURS=$3 OUT=$4; shift 4
pin=$(sed -n 's/^COMPOSE_RULES_TAG=v//p' "$repo/tools/sync-compose-rules.sh")
JAR=${JAR:-$here/lib/ktlint-compose-$pin-all.jar}
[[ -f $JAR ]] || { echo "no $JAR; run tools/sync-compose-rules.sh" >&2; exit 2; }
mkdir -p "$OUT"
for tree in "$@"; do
  name=$(basename "$tree")
  for v in 1.8 2.0; do
    jvm=$K20; [[ $v == 1.8 ]] && jvm=$K18
    for mode in compose-only with-standard; do
      extra=; [[ $mode == compose-only ]] && extra='ktlint_standard = disabled'
      EC_EXTRA=$extra "$repo/tools/parity/ktlint-compare.sh" "$jvm -R $JAR" "$OURS --ktlint-version=$v -R $JAR" \
        "$tree" "$OUT/$name/$v/$mode" ktlint_official > /dev/null
      echo "## $name ktlint $v $mode"; tail -n +4 "$OUT/$name/$v/$mode/summary.md"
    done
  done
done | tee "$OUT/summary.md"
"$repo/tools/parity/check-known.sh" "${KNOWN:-$repo/tools/parity/known-diffs/compose-rules.tsv}" "$OUT"
