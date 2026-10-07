#!/usr/bin/env bash
# Formats a copy of a Kotlin tree with the real ktfmt release, as the byte-exact reference.
#   ktfmt-oracle.sh <meta|google|kotlinlang> <in-dir> <out-dir>
# Files ktfmt rejects (parse errors) are left as copied; their paths go to <out-dir>/.failed.
set -euo pipefail
style=$1 in=$2 out=$3
here="$(cd "$(dirname "$0")" && pwd)"
jar=$(ls "$here"/lib/ktfmt-*-with-dependencies.jar 2>/dev/null | head -1 || true)
[[ -n $jar ]] || { echo "no oracle jar; run tools/sync-ktfmt.sh" >&2; exit 1; }
java=java
bundled=$(ls -d "$here"/../jdk/*/bin 2>/dev/null | head -1 || true)
if [[ -n $bundled ]]; then java="$bundled/java"; fi
case $style in
  meta) flag=--meta-style ;; google) flag=--google-style ;; kotlinlang) flag=--kotlinlang-style ;;
  *) echo "unknown style $style" >&2; exit 2 ;;
esac

rm -rf "$out"; mkdir -p "$out"; out=$(cd "$out" && pwd)
(cd "$in" && find . -type f \( -name '*.kt' -o -name '*.kts' \) -print0) | (cd "$in" && xargs -0 cp --parents -t "$out")
command -v cygpath >/dev/null && { jar=$(cygpath -m "$jar"); out_arg=$(cygpath -m "$out"); } || out_arg=$out
"$java" -Xss64m -jar "$jar" "$flag" --quiet "$out_arg" 2> "$out/.stderr" || true
sed -E -e 's#^(([A-Za-z]:)?[^:]*\.kts?)(:[0-9]+)*: .*#\1#' -e 't ok' -e d -e ':ok' -e 's#\\#/#g' "$out/.stderr" \
  | awk -v p="$out_arg/" 'index($0, p) == 1 { $0 = substr($0, length(p) + 1) } { print }' | sort -u > "$out/.failed" || true
echo "ktfmt $style: $(find "$out" -name '*.kt*' | wc -l) files, $(wc -l < "$out/.failed") rejected"
