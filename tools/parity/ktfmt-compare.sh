#!/usr/bin/env bash
# Two ktfmt CLIs on fresh copies of the same tree (`ktfmt --<style>-style .`, cwd = the copy), once per style:
# exit codes, sorted stderr and the formatted trees compared per file. Used by tools/holdout/run.sh.
#
#   tools/parity/ktfmt-compare.sh A B TREE OUT [style...]     styles: meta google kotlinlang (default: all)
#
# A, B: commands (word-split, so "java -jar ktfmt.jar" works). Env: TMO = per-run timeout in seconds (default 3600);
# KEEP=DIR keeps B's formatted tree as DIR/<style> (tools/fuzz/diff.sh formats it again).
set -uo pipefail
(($# >= 4)) || { sed -n 2,9p "$0"; exit 2; }
A=$1 B=$2 TREE=$(cd "$3" && pwd) OUT=$4; shift 4
STYLES=("$@"); ((${#STYLES[@]})) || STYLES=(meta google kotlinlang)
TMO=${TMO:-3600}
mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT

{
  echo "A: $A ($($A --version 2>&1 | tail -1))"
  echo "B: $B ($($B --version 2>&1 | tail -1))"
  echo "tree: $TREE, $(find "$TREE" -type f \( -name '*.kt' -o -name '*.kts' \) | wc -l) files"
} | tee "$OUT/summary.md"

for style in "${STYLES[@]}"; do
  d=$OUT/$style; mkdir -p "$d"
  for s in a b; do
    cmd=$A; [[ $s == b ]] && cmd=$B
    rm -rf "$W/$s"; cp -a "$TREE" "$W/$s"
    (cd "$W/$s" && timeout "$TMO" $cmd "--$style-style" . > "$d/$s.out" 2> "$d/$s.err"); echo $? > "$d/$s.exit"
    sed "s|$W/$s/||g" "$d/$s.err" | LC_ALL=C sort > "$d/$s.err.sorted"
  done
  diff -r "$W/a" "$W/b" > "$d/format.diff"
  diff -rq "$W/a" "$W/b" | awk '{print $2}' | sed "s|^$W/a/||" > "$d/files.txt"
  [[ -n ${KEEP:-} ]] && { mkdir -p "$KEEP"; rm -rf "${KEEP:?}/$style"; cp -a "$W/b" "$KEEP/$style"; }
  echo "$style: exit A $(cat "$d/a.exit") B $(cat "$d/b.exit"); stderr $(cmp -s "$d/a.err.sorted" "$d/b.err.sorted" &&
    echo same || echo "DIFF ($(diff "$d/a.err.sorted" "$d/b.err.sorted" | grep -c '^[<>]') lines)");" \
    "files differing $(wc -l < "$d/files.txt")" | tee -a "$OUT/summary.md"
done
