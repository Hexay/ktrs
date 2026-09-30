#!/usr/bin/env bash
# Mutated-tree census and oracle on the real ktlint engine; output format in research/14-ktlint-probe.md.
#   ktlint-probe.sh <in-dir> <out-dir> [--rules a,b] [--dumps] [--no-lint] [--isolate] [--threads N]
# Stages the *.kt/*.kts of <in-dir> into <out-dir>/src under one root .editorconfig (ktlint_code_style =
# $KTLINT_CODE_STYLE, default ktlint_official), so the corpus repos' own .editorconfig files don't apply, then
# runs KtlintProbe on that copy.
set -euo pipefail
in=$1 out=$2
shift 2
here="$(cd "$(dirname "$0")" && pwd)"
jar=$(ls "$here"/lib/ktlint-cli-*-all.jar 2>/dev/null | head -1 || true)
[[ -n $jar ]] || { echo "no ktlint jar; run tools/sync-ktlint.sh" >&2; exit 1; }
java=java
bundled=$(ls -d "$here"/../jdk/*/bin 2>/dev/null | head -1 || true)
if [[ -n $bundled ]]; then java="$bundled/java"; fi
classes="$here/build"
sep=:
host() { if command -v cygpath >/dev/null; then cygpath -m "$1"; else echo "$1"; fi; }
command -v cygpath >/dev/null && sep=';'

stamp="$classes/KtlintProbeKt.class"
if [[ ! -f $stamp || -n $(find "$here/src" -newer "$stamp") ]]; then
  rm -rf "$classes"
  mkdir -p "$classes"
  srcs=()
  for f in "$here"/src/*.kt; do srcs+=("$(host "$f")"); done
  # The fat jar embeds the Kotlin compiler ktlint runs on, so the probe compiles against exactly that.
  "$java" -cp "$(host "$jar")" org.jetbrains.kotlin.cli.jvm.K2JVMCompiler -no-stdlib -no-reflect -nowarn \
    -jvm-target 17 -cp "$(host "$jar")" -d "$(host "$classes")" "${srcs[@]}"
fi

rm -rf "$out"
mkdir -p "$out/src"
out=$(cd "$out" && pwd)
(cd "$in" && find . -type f \( -name '*.kt' -o -name '*.kts' \) -print0) | (cd "$in" && xargs -0 cp --parents -t "$out/src")
printf 'root = true\n\n[*.{kt,kts}]\nktlint_code_style = %s\n' "${KTLINT_CODE_STYLE:-ktlint_official}" > "$out/src/.editorconfig"
# The fat jar's logback defaults to DEBUG (one line per file and rule order); keep only errors.
exec "$java" -Xss64m -Xmx8g -Dlogback.configurationFile="$(host "$here/logback.xml")" -cp "$(host "$classes")$sep$(host "$jar")" KtlintProbeKt "$(host "$out/src")" "$(host "$out")" "$@"
