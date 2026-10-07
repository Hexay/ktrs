#!/usr/bin/env bash
# Gate over ktlint-compare.sh output: every difference it found must be listed in a reviewed known-diffs file, and
# every listed one must still occur. Exit 0 when they match exactly, 1 otherwise (both lists printed).
#
#   tools/parity/check-known.sh KNOWN OUT
#
# OUT: a ktlint-compare.sh out dir, or a tree of them (compose-rules/parity.sh). Each style dir below it yields
# `<dir>\texit\tlint <A> <B>` and `format <A> <B>` lines (so a run that did not happen fails), `<dir>\tonly-upstream\t<row>`
# (only.a), `<dir>\tonly-ktrs\t<row>` (only.b) and `<dir>\tformat\t<file>` (format-files.txt); <dir> is the style
# dir relative to OUT.
# KNOWN: those lines, `#` comments for the reasons. Only entries whose <dir> exists under OUT are checked.
set -uo pipefail
(($# == 2)) || { sed -n 2,11p "$0"; exit 2; }
known=$1 out=$(cd "$2" && pwd)
actual=$(mktemp) expected=$(mktemp); trap 'rm -f "$actual" "$expected"' EXIT
dirs=$(find "$out" -mindepth 2 -name lint.a.exit -printf '%h\n' | sed "s|^$out/||" | sort)
[[ -n $dirs ]] || { echo "no ktlint-compare.sh results under $out" >&2; exit 1; }
for d in $dirs; do
  p=$out/$d
  printf '%s\texit\tlint %s %s\n' "$d" "$(cat "$p/lint.a.exit")" "$(cat "$p/lint.b.exit")"
  [[ -f $p/format.a.exit ]] && printf '%s\texit\tformat %s %s\n' "$d" "$(cat "$p/format.a.exit")" "$(cat "$p/format.b.exit")"
  sed "s|^|$d\tonly-upstream\t|" "$p/only.a"
  sed "s|^|$d\tonly-ktrs\t|" "$p/only.b"
  [[ -f $p/format-files.txt ]] && sed "s|^|$d\tformat\t|" "$p/format-files.txt"
done | LC_ALL=C sort > "$actual"
grep -v -e '^#' -e '^$' "$known" | awk -F'\t' 'NR == FNR { d[$1] = 1; next } $1 in d' <(cut -f1 "$actual") - |
  LC_ALL=C sort > "$expected"
new=$(LC_ALL=C comm -13 "$expected" "$actual") gone=$(LC_ALL=C comm -23 "$expected" "$actual")
[[ -n $new ]] && { echo "Not in $known (regression, or review and add):"; echo "$new"; }
[[ -n $gone ]] && { echo "In $known but no longer seen (remove):"; echo "$gone"; }
[[ -z $new && -z $gone ]] && echo "check-known: $(wc -l < "$actual") lines match $known"
[[ -z $new && -z $gone ]]
