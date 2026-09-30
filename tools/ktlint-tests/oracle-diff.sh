#!/usr/bin/env bash
# Go/no-go (a) of research/15-ktlint-spike.md: ktrs-lint vs the JVM ktlint oracle (tools/ktlint-oracle) on corpus/.
#   tools/ktlint-tests/oracle-diff.sh [jvm-out-dir]
# jvm-out-dir defaults to target/ktlint-oracle/three, the oracle's three-rule run; on the testbox it is
# ~/work/ktlint-probe/out/three (tarball: out/three-rule-oracle.tgz, unpack into target/ktlint-oracle/).
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
jvm="${1:-$root/target/ktlint-oracle/three}"
[[ -f $jvm/format.tsv ]] || { echo "no oracle output in $jvm (scp testbox:work/ktlint-probe/out/three-rule-oracle.tgz)" >&2; exit 1; }
rules=standard:no-semi,standard:comma-spacing,standard:multiline-if-else
cd "$root"
cargo ktlint-probe corpus target/ktlint-probe/three --rules "$rules" --dumps --threads "${THREADS:-4}"
cargo ktlint-probe compare "$jvm" target/ktlint-probe/three corpus
