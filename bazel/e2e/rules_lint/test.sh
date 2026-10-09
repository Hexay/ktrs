#!/usr/bin/env bash
# Runs rules_lint's formatter and ktlint aspect in this workspace with the `ktfmt` and `ktlint` of a rules_multitool
# lockfile, and checks the outcomes on src/Clean.kt and src/Violation.kt:
#   test.sh <lockfile>     a release's ktrs-<tag>.multitool.lock.json, or the output of ../local-lock.sh
# BAZEL=<bazel or bazelisk> (default bazel). Works on a copy in E2E_WORK (default $TMPDIR/ktrs-bazel-e2e): the
# formatter rewrites sources and finds them with `git ls-files`. Observed behaviour: research/35-bazel-reviewdog.md.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
lock="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
bazel=${BAZEL:-bazel}
work=${E2E_WORK:-${TMPDIR:-/tmp}/ktrs-bazel-e2e}

rm -rf "$work"
mkdir -p "$work"
cp -R "$here/." "$work"
cp "$lock" "$work/ktrs.lock.json"
cd "$work" || exit 1
rm -rf bazel-*
git init -q . && git add -A

failed=0
check() {
  local what=$1; shift
  if "$@"; then echo "ok    $what"; else echo "FAIL  $what"; failed=1; fi
}
has() { grep -qF -- "$2" "$1"; }
lacks() { ! grep -qF -- "$2" "$1"; }
exits() { [[ $(cat "$2") == "$1" ]]; }
empty() { [[ ! -s $1 ]]; }
# run <log name> <expected exit code> <bazel args...>
run() {
  local log=$work/$1.log want=$2 code; shift 2
  "$bazel" "$@" > "$log" 2>&1; code=$?
  check "bazel $* exits $want" test "$code" == "$want"
  [[ $code == "$want" ]] || tail -n 40 "$log"
}
aspect=(--aspects=//tools/lint:linters.bzl%ktlint --output_groups=rules_lint_human,rules_lint_machine)
out=bazel-bin/src/violation.AspectRulesLintKTLint
clean=bazel-bin/src/clean.AspectRulesLintKTLint

run format-check 1 run //tools/format:format.check
check "format.check lists src/Violation.kt" has "$work/format-check.log" "src/Violation.kt"
check "format.check does not list src/Clean.kt" lacks "$work/format-check.log" "src/Clean.kt"

run lint 0 build //src/... "${aspect[@]}"
check "violation: exit code 1" exits 1 "$out.out.exit_code"
# The human report is coloured; the SARIF one is rebuilt by rules_lint from ktlint's plain rows.
check "violation: reports op-spacing" has "$out.out" "(standard:op-spacing)"
check "violation: SARIF report has the message" has "$out.report" "No whitespace expected in empty parameter list (standard:function-signature)"
check "violation: SARIF report has the path" has "$out.report" '"uri": "src/Violation.kt"'
check "violation: the declared .editorconfig disables no-wildcard-imports" lacks "$out.out" "no-wildcard-imports"
check "violation: only its own srcs are linted" lacks "$out.out" "Clean.kt"
check "clean: exit code 0" exits 0 "$clean.out.exit_code"
check "clean: empty report" empty "$clean.out"

run lint-test 0 test //src/...
run lint-fail 1 build //src:violation "${aspect[@]}" --@aspect_rules_lint//lint:fail_on_violation

run format 0 run //tools/format:format
check "format rewrites src/Violation.kt" has src/Violation.kt "fun violation(): List<Int> {"
check "format leaves src/Clean.kt alone" cmp -s src/Clean.kt "$here/src/Clean.kt"
run format-recheck 0 run //tools/format:format.check

if ((failed)); then echo "bazel e2e: FAILED (logs in $work)"; exit 1; fi
echo "bazel e2e: passed"
