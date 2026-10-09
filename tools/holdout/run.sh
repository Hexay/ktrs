#!/usr/bin/env bash
# The held-out parity run (research/21-holdout.md): ktrs's `ktlint` vs the ktlint 2.0.0-ALPHA-4 jar (3 styles,
# lint + -F) and ktrs's `ktfmt` vs the ktfmt 0.65 jar (3 styles) on the corpus from fetch.sh. Linux.
#
#   tools/holdout/run.sh [DEST] [OUT]       defaults: corpus-holdout/, target/holdout/
#
# Env: KTRS_BIN = directory with the ktlint and ktfmt binaries (default target/release); KTLINT2 = the
# self-executing ktlint release asset; KTFMT_JAR = ktfmt-0.65-with-dependencies.jar (tools/sync-ktfmt.sh).
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
dest="${1:-$repo/corpus-holdout}" out="${2:-$repo/target/holdout}"
bin=${KTRS_BIN:-$repo/target/release}
ktlint2=${KTLINT2:?set KTLINT2 to the ktlint-2.0.0-ALPHA-4 release asset}
jar=${KTFMT_JAR:-$repo/tools/ktfmt-oracle/lib/ktfmt-0.65-with-dependencies.jar}
[[ -d $dest/tree ]] || "$repo/tools/holdout/fetch.sh" "$dest"
"$repo/tools/parity/ktlint-compare.sh" "$bin/ktlint" "$ktlint2" "$dest/tree" "$out/ktlint"
"$repo/tools/parity/ktfmt-compare.sh" "$bin/ktfmt" "java -jar $jar" "$dest/tree" "$out/ktfmt"
