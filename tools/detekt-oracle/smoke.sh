#!/usr/bin/env bash
# Writes the smoke files of smoke/cases.txt (`path|content`; content escapes `\n \r \t \\ \xHH`, `{xN}` = N times
# `x`) under <out-dir>: byte-exact inputs for location semantics (UTF-16 columns, CRLF, BOM, empty files),
# suppression ids, path filters and MaxLineLength's URL exemptions. The jar's rows on them are
# crates/ktrs-detekt/tests/data/smoke.jvm.tsv (tests/smoke.rs decodes the same file):
#   smoke.sh <dir> && detekt-probe.sh <dir> <out> --sequential && cp <out>/rows.tsv crates/ktrs-detekt/tests/data/smoke.jvm.tsv
set -euo pipefail
out=$1
here="$(cd "$(dirname "$0")" && pwd)"
rm -rf "$out"
while IFS= read -r line || [[ -n $line ]]; do
  path=${line%%|*} content=${line#*|}
  while [[ $content =~ \{x([0-9]+)\} ]]; do
    run=$(printf 'x%.0s' $(seq 1 "${BASH_REMATCH[1]}"))
    content=${content/"${BASH_REMATCH[0]}"/$run}
  done
  mkdir -p "$out/$(dirname "$path")"
  printf '%b' "$content" > "$out/$path"
done < "$here/smoke/cases.txt"
