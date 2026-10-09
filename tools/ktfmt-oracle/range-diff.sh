#!/usr/bin/env bash
# Differential test of partial formatting (`--lines`, `--offset`/`--length`) in the `ktfmt` drop-in
# against the real ktfmt CLI jar, on random ranges over a sample of a Kotlin tree.
#   range-diff.sh [dir] [files] [seed] [path/to/ktfmt-binary]
#   (defaults: corpus, 300 files, seed 1, target/release/ktfmt[.exe])
# Every sampled file gets several cases (line ranges, several ranges at once, character ranges, a
# cursor), each in one of the three styles. Both tools format their own copy of the file with the
# same flags; the file left behind and the exit code must be identical. Prints the first mismatches
# and a summary; exit 1 on any mismatch. JVM: run in the background.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
jar=$(ls "$here"/lib/ktfmt-*-with-dependencies.jar 2>/dev/null | head -1 || true)
[[ -n $jar ]] || { echo "no oracle jar; run tools/sync-ktfmt.sh" >&2; exit 1; }
bin=$(ls -d "$here"/../jdk/*/bin 2>/dev/null | head -1 || true)
java=${bin:+$bin/}java javac=${bin:+$bin/}javac
dir=${1:-$repo/corpus} files=${2:-300} seed=${3:-1}
ours=${4:-$(ls "$repo"/target/release/ktfmt.exe "$repo"/target/release/ktfmt 2>/dev/null | head -1)}
[[ -n $ours ]] || { echo "no ktfmt binary; cargo build --release --bin ktfmt" >&2; exit 1; }
ours="$(cd "$(dirname "$ours")" && pwd)/$(basename "$ours")"
dir="$(cd "$dir" && pwd)"
sep=: ; to_host() { echo "$1"; }
if command -v cygpath >/dev/null; then sep=';'; to_host() { cygpath -m "$1"; }; fi
scratch=$(mktemp -d)
# KEEP=1 leaves each case's files in $scratch/{jar,ours}/<id> for inspection.
if [[ -n ${KEEP:-} ]]; then echo "outputs in $scratch"; else trap 'rm -rf "$scratch"' EXIT; fi

# The plan: <id> <source> <file name> <flags>, from every step-th file's line and byte counts.
plan="$scratch/plan.tsv"
(cd "$dir" && find . -type f \( -name '*.kt' -o -name '*.kts' \) | LC_ALL=C sort) > "$scratch/all"
total=$(wc -l < "$scratch/all")
step=$((total / files)); ((step > 0)) || step=1
awk -v step="$step" 'NR % step == 1 || step == 1' "$scratch/all" | while IFS= read -r f; do
  printf '%s\t%s\t%s\n' "$f" "$(wc -l < "$dir/$f")" "$(wc -c < "$dir/$f")"
done | awk -F'\t' -v seed="$seed" '
  function pick(n) { return int(rand() * n) }
  function lines(   a, b) { a = 1 + pick(n); b = a + pick(n - a + 1 < 40 ? n - a + 1 : 40); return a == b && pick(2) ? a : a ":" b }
  function emit(flags) { printf "%d\t%s\tf.%s\t%s %s\n", ++id, $1, ext, style[id % 3], flags }
  BEGIN { srand(seed); style[0] = "--meta-style"; style[1] = "--google-style"; style[2] = "--kotlinlang-style" }
  {
    n = $2 < 1 ? 1 : $2; bytes = $3 < 1 ? 1 : $3; ext = $1 ~ /\.kts$/ ? "kts" : "kt"
    emit("--lines=" lines()); emit("--lines=" lines()); emit("--lines=" lines())
    emit("--lines=" (1 + pick(n)))
    emit("--lines=" lines() "," lines() " --lines " lines())
    emit("--offset=" pick(bytes) " --length=" pick(400))
    emit("--offset=" pick(bytes) " --length=" (1 + pick(30)) " --offset=" pick(bytes) " --length=" pick(2000))
    emit("--offset " pick(bytes) " --length 0")
    emit("--lines=" lines() " --offset=" pick(bytes) " --length=" pick(100))
  }' > "$plan"

while IFS=$'\t' read -r id source name _; do
  mkdir -p "$scratch/jar/$id" "$scratch/ours/$id"
  cp "$dir/$source" "$scratch/jar/$id/$name"
  cp "$dir/$source" "$scratch/ours/$id/$name"
done < "$plan"

mkdir -p "$scratch/classes"
"$javac" -nowarn -d "$(to_host "$scratch/classes")" -cp "$(to_host "$jar")" "$(to_host "$here/KtfmtRanges.java")"
"$java" -Xss64m -cp "$(to_host "$scratch/classes")$sep$(to_host "$jar")" KtfmtRanges \
  "$(to_host "$plan")" "$(to_host "$scratch/jar")"

while IFS=$'\t' read -r id _ name flags; do
  # shellcheck disable=SC2086
  "$ours" $flags "$scratch/ours/$id/$name" > /dev/null 2>&1
  echo $? > "$scratch/ours/$id/code"
done < "$plan"

pass=0 fail=0 changed=0
while IFS=$'\t' read -r id source name flags; do
  a="$scratch/jar/$id" b="$scratch/ours/$id" bad=()
  cmp -s "$a/code" "$b/code" || bad+=("exit $(cat "$a/code") vs $(cat "$b/code")")
  cmp -s "$a/$name" "$b/$name" || bad+=(output)
  cmp -s "$a/$name" "$dir/$source" || changed=$((changed + 1))
  if ((${#bad[@]})); then
    fail=$((fail + 1))
    ((fail <= 20)) && echo "MISMATCH $source [$flags]: ${bad[*]}"
    ((fail <= 3)) && diff "$a/$name" "$b/$name" | head -8
  else
    pass=$((pass + 1))
  fi
done < "$plan"

echo "range-diff: $pass passed, $fail mismatched ($changed of $((pass + fail)) cases changed their file)"
((fail == 0))
