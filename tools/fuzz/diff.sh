#!/usr/bin/env bash
# Differential fuzzing (research/24-fuzzing.md): ktrs's `ktfmt`/`ktlint` binaries vs the upstream jars on generated
# inputs that parse without errors (`kgen` from the fuzz/ crate). Two passes per style: pass 1 on the inputs, pass 2
# on the jar's pass-1 output, so a non-idempotent format only counts where the jar's second pass differs from ours.
#
#   tools/fuzz/diff.sh INPUTS OUT [ktfmt|ktlint|all]
#
# INPUTS: a flat dir of .kt/.kts files. OUT/report.md is the summary, OUT/mismatches.tsv one row per
# (tool, style, pass, kind, file), OUT/repro/<file> the inputs involved. Linux.
# Env: KTRS_BIN = dir with the ktfmt and ktlint binaries (default target/release); KTFMT_JAR (default
# tools/ktfmt-oracle/lib/ktfmt-0.64-with-dependencies.jar); KTLINT2 = the ktlint-2.0.0-ALPHA-4 release asset
# (needed for ktlint); TMO as in tools/parity/*-compare.sh; EC_EXTRA as in tools/parity/ktlint-compare.sh.
set -euo pipefail
(($# >= 2)) || { sed -n 2,13p "$0"; exit 2; }
repo="$(cd "$(dirname "$0")/../.." && pwd)"
inputs=$(cd "$1" && pwd) out=$2 tools=${3:-all}
mkdir -p "$out"; out=$(cd "$out" && pwd)
bin=${KTRS_BIN:-$repo/target/release}
jar=${KTFMT_JAR:-$repo/tools/ktfmt-oracle/lib/ktfmt-0.64-with-dependencies.jar}
rm -rf "$out/tree" "$out/repro" "$out/mismatches.tsv" "$out/ktfmt" "$out/ktlint"; mkdir -p "$out/tree" "$out/repro"
find "$inputs" -maxdepth 1 -type f \( -name '*.kt' -o -name '*.kts' \) -exec cp -t "$out/tree" {} +
touch "$out/mismatches.tsv"

# tool style pass kind < file names
record() { while read -r f; do [[ -n $f ]] && printf '%s\t%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" "$f"; done >> "$out/mismatches.tsv"; }
files_in_stderr_diff() { diff "$1" "$2" | grep -aoE '[A-Za-z0-9_.-]+\.kts?' | sort -u || true; }

# compare-script tool tree styles...: pass 1 on TREE, pass 2 per style on the jar's pass-1 output. PREPARE (a function
# name or `:`) runs on each tree first, with the pass number.
run_tool() {
  local script=$1 tool=$2 a=$3 b=$4 tree=$5; shift 5
  $PREPARE "$tree" 1
  KEEP="$out/$tool/jar-pass1" "$repo/tools/parity/$script" "$a" "$b" "$tree" "$out/$tool/pass1" "$@" > /dev/null
  for style in "$@"; do
    $PREPARE "$out/$tool/jar-pass1/$style" 2
    "$repo/tools/parity/$script" "$a" "$b" "$out/$tool/jar-pass1/$style" "$out/$tool/pass2-$style" "$style" > /dev/null
  done
}

# A file ktrs's ktfmt rejects (parse or formatting error, any style) is compared on its own (tools/fuzz/differs.sh):
# on a FormattingError the jar's CLI aborts the whole run, leaving the rest of the tree unformatted.
quarantine() { # tree pass
  local q=$out/ktfmt/quarantine-$2-$(basename "$1") f style d
  mkdir -p "$q"
  for style in meta google kotlinlang; do
    (cd "$1" && "$bin/ktfmt" "--$style-style" --dry-run . 2>&1 > /dev/null) | grep -aoE '^\./[^:]+\.kts?:[0-9]+:[0-9]+: error' | cut -d: -f1 || true
  done | sort -u | while read -r f; do mv "$1/${f#./}" "$q/"; done
  for f in "$q"/*; do
    [[ -e $f ]] || continue
    for style in meta google kotlinlang; do
      if d=$("$repo/tools/fuzz/differs.sh" ktfmt "$style" "$f"); then basename "$f" | record ktfmt "$style" "$2" "alone: $d"; fi
    done
  done
}

collect_ktfmt() { # style pass dir
  record ktfmt "$1" "$2" format < "$3/files.txt"
  files_in_stderr_diff "$3/a.err.sorted" "$3/b.err.sorted" | record ktfmt "$1" "$2" stderr
  [[ $(cat "$3/a.exit") == "$(cat "$3/b.exit")" ]] || echo "(tree)" | record ktfmt "$1" "$2" "exit $(cat "$3/a.exit")/$(cat "$3/b.exit")"
}

collect_ktlint() { # style pass dir
  cat "$3/only.a" "$3/only.b" | cut -d: -f1 | sort -u | record ktlint "$1" "$2" lint
  record ktlint "$1" "$2" format < "$3/format-files.txt"
  [[ $(cat "$3/lint.a.exit") == "$(cat "$3/lint.b.exit")" ]] ||
    echo "(tree)" | record ktlint "$1" "$2" "lint exit $(cat "$3/lint.a.exit")/$(cat "$3/lint.b.exit")"
  [[ $(cat "$3/format.a.exit") == "$(cat "$3/format.b.exit")" ]] ||
    echo "(tree)" | record ktlint "$1" "$2" "format exit $(cat "$3/format.a.exit")/$(cat "$3/format.b.exit")"
}

if [[ $tools == all || $tools == ktfmt ]]; then
  styles=(meta google kotlinlang)
  rm -rf "$out/tree-ktfmt"; cp -a "$out/tree" "$out/tree-ktfmt"
  PREPARE=quarantine run_tool ktfmt-compare.sh ktfmt "$bin/ktfmt" "java -jar $jar" "$out/tree-ktfmt" "${styles[@]}"
  for s in "${styles[@]}"; do collect_ktfmt "$s" 1 "$out/ktfmt/pass1/$s"; collect_ktfmt "$s" 2 "$out/ktfmt/pass2-$s/$s"; done
fi
if [[ $tools == all || $tools == ktlint ]]; then
  styles=(ktlint_official intellij_idea android_studio)
  PREPARE=: run_tool ktlint-compare.sh ktlint "$bin/ktlint" "${KTLINT2:?set KTLINT2 to the ktlint-2.0.0-ALPHA-4 release asset}" \
    "$out/tree" "${styles[@]}"
  for s in "${styles[@]}"; do collect_ktlint "$s" 1 "$out/ktlint/pass1/$s"; collect_ktlint "$s" 2 "$out/ktlint/pass2-$s/$s"; done
fi

# Pass 2 rows for a file that already differed in pass 1 (same tool and style) are consequences, not findings.
awk -F'\t' '$3 == 1 { first[$1 FS $2 FS $5] = 1 } $3 == 1 || !first[$1 FS $2 FS $5]' "$out/mismatches.tsv" > "$out/m.tmp"
mv "$out/m.tmp" "$out/mismatches.tsv"
cut -f5 "$out/mismatches.tsv" | { grep -v '^(tree)$' || true; } | sort -u | while read -r f; do cp "$out/tree/$f" "$out/repro/" 2>/dev/null || true; done
{
  echo "# fuzz diff: $(find "$out/tree" -type f | wc -l) inputs from $inputs"
  echo; echo "| tool | style | pass | kind | files |"; echo "|---|---|--:|---|--:|"
  awk -F'\t' '{ n[$1 FS $2 FS $3 FS $4]++ } END { for (k in n) { split(k, p, FS); printf "| %s | %s | %s | %s | %d |\n", p[1], p[2], p[3], p[4], n[k] } }' \
    "$out/mismatches.tsv" | sort
  echo; echo "distinct files: $(cut -f5 "$out/mismatches.tsv" | { grep -v '^(tree)$' || true; } | sort -u | wc -l)"
} | tee "$out/report.md"
