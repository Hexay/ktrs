#!/usr/bin/env bash
# Regenerates testdata/ktfmt/<group>/<case>.{input.kt,options,expected.kt|error} from ktfmt's own
# file-based format cases (third_party/ktfmt/core/src/test/resources/cases/<group>): each case's
# options are its group's style plus the directive header of its .input, and every expectation is
# re-derived with the real ktfmt jar (KtfmtOracle.java), listing where it disagrees with upstream's
# .output. JVM: run in the background.
#   extract-goldens.sh            # import + verify + copy into testdata/ktfmt
#   extract-goldens.sh oracle DIR # only (re)write expectations for the cases under DIR
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
lib="$here/lib"
work="$root/target/ktfmt-extract"
cases="$root/third_party/ktfmt/core/src/test/resources/cases"
# <group>:<style>. new_codestyle's plain .output files are META (its .new.output: the unported experimental engine).
groups=(format:meta google:google kotlinlang:kotlinlang new_codestyle:meta)

jar=$(ls "$lib"/ktfmt-*-with-dependencies.jar 2>/dev/null | head -1 || true)
[[ -n $jar ]] || { echo "no oracle jar; run tools/sync-ktfmt.sh" >&2; exit 1; }
bin=$(ls -d "$root"/tools/jdk/*/bin 2>/dev/null | head -1 || true)
java=${bin:+$bin/}java javac=${bin:+$bin/}javac

sep=: ; to_host() { echo "$1"; }
if command -v cygpath >/dev/null; then sep=';'; to_host() { cygpath -m "$1"; }; fi

mkdir -p "$work/oracle-classes"
"$javac" -nowarn -d "$(to_host "$work/oracle-classes")" -cp "$(to_host "$jar")" "$(to_host "$here/KtfmtOracle.java")"
oracle() { "$java" -Xss64m -cp "$(to_host "$work/oracle-classes")$sep$(to_host "$jar")" KtfmtOracle "$@"; }

if [[ ${1:-} == oracle ]]; then oracle "$(to_host "$2")" | grep -E '^(DISAGREE|oracle:)'; exit; fi

[[ -d $cases ]] || { echo "no $cases; run tools/sync-ktfmt.sh" >&2; exit 1; }
rm -rf "$work/cases"; mkdir -p "$work/cases"
for entry in "${groups[@]}"; do
  group=${entry%%:*} style=${entry##*:}
  oracle import "$(to_host "$cases/$group")" "$style" "$(to_host "$work/cases/$group")"
done
oracle "$(to_host "$work/cases")" | grep -E '^(DISAGREE|oracle:)'

rm -rf "$root/testdata/ktfmt"; mkdir -p "$root/testdata/ktfmt"
for entry in "${groups[@]}"; do
  group=${entry%%:*}
  (cd "$work/cases" && find "$group" -type f ! -name '*.upstream.kt' -print0 | xargs -0 cp --parents -t "$root/testdata/ktfmt")
  echo "$group: $(find "$root/testdata/ktfmt/$group" -name '*.input.kt' | wc -l) cases"
done
