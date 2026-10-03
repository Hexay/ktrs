#!/usr/bin/env bash
# Whether ktrs and the jar disagree on one file (the predicate for `kgen reduce`, research/24-fuzzing.md).
# Prints the differing aspects; exits 0 iff one of WANT differs.
#
#   tools/fuzz/differs.sh ktfmt  meta|google|kotlinlang                    FILE   aspects: file stderr exit
#   tools/fuzz/differs.sh ktlint ktlint_official|intellij_idea|android_studio FILE   aspects: lint lintexit format formatexit
#
# Env: WANT = space-separated aspects (default: any); KTRS_BIN, KTFMT_JAR, KTLINT2 as in tools/fuzz/diff.sh;
# EC_EXTRA as in tools/parity/ktlint-compare.sh.
set -uo pipefail
(($# == 3)) || { sed -n 2,10p "$0"; exit 2; }
repo="$(cd "$(dirname "$0")/../.." && pwd)"
tool=$1 style=$2 file=$3
bin=${KTRS_BIN:-$repo/target/release}
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
name=x.${file##*.}

run() { # side cmd args... -> $W/<side>.{out,err,exit}, cwd = $W/<side>
  local side=$1; shift
  (cd "$W/$side" && timeout 120 "$@" > "$W/$side.out" 2> "$W/$side.err"); echo $? > "$W/$side.exit"
}
same() { cmp -s "$W/a$1" "$W/b$1"; }
differing=()

for side in a b; do
  mkdir -p "$W/$side"; cp "$file" "$W/$side/$name"
  if [[ $tool == ktfmt ]]; then
    cmd=("$bin/ktfmt"); [[ $side == b ]] && cmd=(java -jar "${KTFMT_JAR:-$repo/tools/ktfmt-oracle/lib/ktfmt-0.64-with-dependencies.jar}")
    run "$side" "${cmd[@]}" "--$style-style" "$name"
  else
    cmd=("$bin/ktlint"); [[ $side == b ]] && cmd=("${KTLINT2:?set KTLINT2}")
    printf 'root = true\n\n[*.{kt,kts}]\nktlint_code_style = %s\n%b' "$style" "${EC_EXTRA:-}" > "$W/$side/.editorconfig"
    run "$side" "${cmd[@]}" --relative "$name"
    for x in out exit; do mv "$W/$side.$x" "$W/$side.lint.$x"; done
    run "$side" "${cmd[@]}" --relative -F "$name"
  fi
done
cmp -s "$W/a/$name" "$W/b/$name" || differing+=(file)
if [[ $tool == ktfmt ]]; then
  # ktrs prints a rethrown exception's first stack-trace line only (crates/ktrs-cli/src/ktfmt/main.rs).
  for side in a b; do
    grep -avE '^\s+at |^Exception in thread|^\s*\.\.\. [0-9]+ more|^Caused by|^$' "$W/$side.err" > "$W/$side.err.f"
    mv "$W/$side.err.f" "$W/$side.err"
  done
  same .err || differing+=(stderr); same .exit || differing+=(exit)
else
  # The CLI's logback lines start with a wall-clock time.
  for side in a b; do sed -Ei 's/^[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3} //' "$W/$side.lint.out"; done
  [[ " ${differing[*]} " == *" file "* ]] && differing=(format)
  same .lint.out || differing+=(lint); same .lint.exit || differing+=(lintexit); same .exit || differing+=(formatexit)
fi
echo "${differing[*]}"
for want in ${WANT:-${differing[*]}}; do [[ " ${differing[*]} " == *" $want "* ]] && exit 0; done
exit 1
