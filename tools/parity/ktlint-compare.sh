#!/usr/bin/env bash
# Two ktlint CLIs on the same tree, run as users run them (`--relative`, cwd = the tree), once per code style:
# lint rows diffed per rule, then `-F` on fresh copies with the formatted trees diffed per file.
# Used for the held-out corpus (tools/holdout/run.sh) and for ktlint 1.x vs 2.0 (research/21, research/22).
#
#   tools/parity/ktlint-compare.sh A B TREE OUT [style...]
#
# A, B: commands (word-split, so "java -jar x.jar" works). TREE: .kt/.kts only, no .editorconfig of its own
# below the root; its root .editorconfig is overwritten per style. Styles default to all three.
# Env: NO_FORMAT=1 skips -F; TMO = per-run timeout in seconds (default 3600); EC_EXTRA = more .editorconfig lines
# (`\n`-separated), e.g. 'ktlint_standard = disabled' to compare a custom rule set alone (with -R in A and B).
set -uo pipefail
(($# >= 4)) || { sed -n 2,12p "$0"; exit 2; }
A=$1 B=$2 TREE=$(cd "$3" && pwd) OUT=$4; shift 4
STYLES=("$@"); ((${#STYLES[@]})) || STYLES=(ktlint_official intellij_idea android_studio)
TMO=${TMO:-3600}
mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
W=$(mktemp -d)
[[ -f $TREE/.editorconfig ]] && cp "$TREE/.editorconfig" "$W/editorconfig.orig"
restore() { rm -f "$TREE/.editorconfig"; [[ -f $W/editorconfig.orig ]] && cp "$W/editorconfig.orig" "$TREE/.editorconfig"; rm -rf "$W"; }
trap restore EXIT

run() { # cmd cwd prefix args... -> prefix.out/.err/.exit
  local cmd=$1 cwd=$2 p=$3; shift 3
  (cd "$cwd" && timeout "$TMO" $cmd "$@" > "$p.out" 2> "$p.err"); echo $? > "$p.exit"
}
# A multi-line detail (compose-rules' "…\nSee https://…") continues on the next lines: joined with a literal \n.
rows() {
  awk 'BEGIN { bs = sprintf("%c", 92) }
       /^[^ ]+:[0-9]+:[0-9]+: / { if (r != "") print r; r = $0; next }
       r != "" && $0 != "" && !/^[0-9][0-9]:[0-9][0-9]:[0-9][0-9]\.[0-9]+ / { r = r bs "n" $0; next }
       { if (r != "") print r; r = "" }
       END { if (r != "") print r }' "$1" | LC_ALL=C sort
}
rule_counts() { sed -nE 's/.*\(([^()]+)\)$/\1/p' "$1" | sort | uniq -c | awk '{print $2, $1}'; }

{
  echo "A: $A ($($A --version 2>&1 | tail -1))"
  echo "B: $B ($($B --version 2>&1 | tail -1))"
  echo "tree: $TREE, $(find "$TREE" -type f \( -name '*.kt' -o -name '*.kts' \) | wc -l) files"
} | tee "$OUT/summary.md"

for style in "${STYLES[@]}"; do
  d=$OUT/$style; mkdir -p "$d"
  printf 'root = true\n\n[*.{kt,kts}]\nktlint_code_style = %s\n%b' "$style" "${EC_EXTRA:+$EC_EXTRA\n}" > "$TREE/.editorconfig"
  run "$A" "$TREE" "$d/lint.a" --relative; run "$B" "$TREE" "$d/lint.b" --relative
  rows "$d/lint.a.out" > "$d/rows.a"; rows "$d/lint.b.out" > "$d/rows.b"
  LC_ALL=C comm -23 "$d/rows.a" "$d/rows.b" > "$d/only.a"; LC_ALL=C comm -13 "$d/rows.a" "$d/rows.b" > "$d/only.b"
  join -a1 -a2 -e0 -o 0,1.2,2.2 <(rule_counts "$d/only.a") <(rule_counts "$d/only.b") |
    sort -k2,2nr -k3,3nr > "$d/rules-diff.txt"
  files=$(cat "$d/only.a" "$d/only.b" | cut -d: -f1 | sort -u | grep -c .)
  {
    echo; echo "## $style"
    echo "lint: exit A $(cat "$d/lint.a.exit") B $(cat "$d/lint.b.exit"); rows A $(wc -l < "$d/rows.a")," \
      "B $(wc -l < "$d/rows.b"); only A $(wc -l < "$d/only.a"), only B $(wc -l < "$d/only.b"); files differing $files"
    [[ -s $d/rules-diff.txt ]] && { echo "| rule | only A | only B |"; echo "|---|--:|--:|"; awk '{printf "| %s | %s | %s |\n", $1, $2, $3}' "$d/rules-diff.txt"; }
  } | tee -a "$OUT/summary.md"

  [[ -n ${NO_FORMAT:-} ]] && continue
  for s in a b; do rm -rf "$W/$s"; cp -a "$TREE" "$W/$s"; done
  run "$A" "$W/a" "$d/format.a" --relative -F; run "$B" "$W/b" "$d/format.b" --relative -F
  diff -r "$W/a" "$W/b" > "$d/format.diff"
  diff -rq "$W/a" "$W/b" | awk '{print $2}' | sed "s|^$W/a/||" > "$d/format-files.txt"
  echo "format: exit A $(cat "$d/format.a.exit") B $(cat "$d/format.b.exit"); files differing $(wc -l < "$d/format-files.txt")" |
    tee -a "$OUT/summary.md"
done
