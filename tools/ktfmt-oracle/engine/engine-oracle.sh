#!/usr/bin/env bash
# JVM oracles for ktrs_fmt's gjf engine port (crates/ktrs_fmt/src/doc) and input layer
# (crates/ktrs_fmt/src/format/input), run against the real ktfmt jar.
#   engine-oracle.sh cases                  regenerate crates/ktrs_fmt/src/doc/tests/cases.expected
#   engine-oracle.sh tokens <src> <out>     dump KotlinInput tokens of every .kt under <src> into <out>
#                                           (then: KTRS_INPUT_ORACLE=<out> KTRS_INPUT_CORPUS=<src>
#                                            cargo test -p ktrs_fmt input_oracle -- --ignored)
#   engine-oracle.sh idtables <out.rs>      regenerate doc/java_identifier_tables.rs (header re-added by hand)
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$here/../../.."
jar=$(ls "$here"/../lib/ktfmt-*-with-dependencies.jar 2>/dev/null | head -1 || true)
[[ -n $jar ]] || { echo "no oracle jar; run tools/sync-ktfmt.sh" >&2; exit 1; }
bin=$(ls -d "$here"/../../jdk/*/bin 2>/dev/null | head -1 || true)
java=java javac=javac
if [[ -n $bin ]]; then java="$bin/java"; javac="$bin/javac"; fi
classes="$root/target/engine-oracle-classes"
sep=:
if command -v cygpath >/dev/null; then
  jar=$(cygpath -m "$jar"); classes=$(cygpath -m "$classes"); root=$(cygpath -m "$root"); here=$(cygpath -m "$here"); sep=';'
fi
mkdir -p "$classes"
"$javac" -nowarn -cp "$jar" -d "$classes" "$here"/*.java 2>&1 | { grep -v '^Note:' || true; }
run() { "$java" -Xss64m -cp "$jar$sep$classes" "$@" 2>&1 | { grep -v '^WARN' || true; }; }

case ${1:-} in
  cases) d="$root/crates/ktrs_fmt/src/doc/tests"; run DocScript "$d/cases.txt" "$d/cases.expected" ;;
  tokens) run InputDump "$2" "$3" ;;
  idtables) "$java" -cp "$classes" IdTables > "$2" ;;
  *) echo "usage: $0 cases | tokens <src> <out> | idtables <out.rs>" >&2; exit 2 ;;
esac
