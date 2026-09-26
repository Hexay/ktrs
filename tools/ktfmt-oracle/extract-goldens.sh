#!/usr/bin/env bash
# Regenerates testdata/ktfmt/<suite>/<case>.{input.kt,options,expected.kt|error} from ktfmt's own
# format tests: runs them on the JVM with a recording KtfmtTruth (extract/KtfmtTruth.kt), then
# re-derives every expectation with the real ktfmt jar (KtfmtOracle.java). Slow: run in background.
#   extract-goldens.sh            # extract + verify + copy into testdata/ktfmt
#   extract-goldens.sh oracle DIR # only (re)write expectations for the cases under DIR
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
lib="$here/lib"
work="$root/target/ktfmt-extract"
tests="$root/third_party/ktfmt/core/src/test/java/com/facebook/ktfmt/format"
suites=(FormatterTest GoogleStyleFormatterKtTest)
maven=https://repo1.maven.org/maven2

jar=$(ls "$lib"/ktfmt-*-with-dependencies.jar 2>/dev/null | head -1 || true)
[[ -n $jar ]] || { echo "no oracle jar; run tools/sync-ktfmt.sh" >&2; exit 1; }
bin=$(ls -d "$root"/tools/jdk/*/bin 2>/dev/null | head -1 || true)
java=${bin:+$bin/}java javac=${bin:+$bin/}javac

sep=: ; to_host() { echo "$1"; }
if command -v cygpath >/dev/null; then sep=';'; to_host() { cygpath -m "$1"; }; fi
cp_of() { local out="" f; for f in "$@"; do out+="$(to_host "$f")$sep"; done; echo "${out%$sep}"; }

run_oracle() {
  mkdir -p "$work/oracle-classes"
  "$javac" -nowarn -d "$(to_host "$work/oracle-classes")" -cp "$(cp_of "$jar")" "$(to_host "$here/KtfmtOracle.java")"
  "$java" -Xss64m -cp "$(cp_of "$work/oracle-classes" "$jar")" KtfmtOracle "$(to_host "$1")" \
    | grep -E '^(DISAGREE|oracle:)'
}

if [[ ${1:-} == oracle ]]; then run_oracle "$2"; exit; fi

fetch() { [[ -f $lib/$(basename "$1") ]] || curl -sfL -o "$lib/$(basename "$1")" "$maven/$1"; }
fetch junit/junit/4.13.1/junit-4.13.1.jar
fetch org/hamcrest/hamcrest-core/1.3/hamcrest-core-1.3.jar
fetch com/google/truth/truth/1.0/truth-1.0.jar
fetch org/jetbrains/annotations/13.0/annotations-13.0.jar
test_deps=("$jar" "$lib/junit-4.13.1.jar" "$lib/hamcrest-core-1.3.jar" "$lib/truth-1.0.jar")
kotlinc_cp=("$root"/tools/psi-dump/lib/*.jar "$lib/annotations-13.0.jar")
[[ -f ${kotlinc_cp[0]} ]] || { echo "no Kotlin compiler; run tools/psi-dump/psi-dump.sh kinds once" >&2; exit 1; }

rm -rf "$work"; mkdir -p "$work/src" "$work/classes" "$work/cases"
cp "$here/extract/KtfmtTruth.kt" "$work/src/"
for s in "${suites[@]}"; do
  sed 's/\bFormatter\.format(/com.facebook.ktfmt.testutil.recordedFormat(/g' "$tests/$s.kt" > "$work/src/$s.kt"
done

echo "compiling upstream tests..."
"$java" -cp "$(cp_of "${kotlinc_cp[@]}")" org.jetbrains.kotlin.cli.jvm.K2JVMCompiler \
  -no-stdlib -no-reflect -nowarn -jvm-target 17 -cp "$(cp_of "${test_deps[@]}")" \
  -d "$(to_host "$work/classes")" "$(to_host "$work/src")"

echo "running upstream tests with the recorder..."
"$java" -Xss64m -Dgolden.out="$(to_host "$work/cases")" -cp "$(cp_of "$work/classes" "${test_deps[@]}")" \
  org.junit.runner.JUnitCore "${suites[@]/#/com.facebook.ktfmt.format.}" > "$work/junit.log" 2>&1 || true
grep -E '^(OK|Tests run|[0-9]+\) )' "$work/junit.log" || true

run_oracle "$work/cases"

for s in "${suites[@]}"; do
  rm -rf "$root/testdata/ktfmt/$s"; mkdir -p "$root/testdata/ktfmt/$s"
  (cd "$work/cases/$s" && cp -- *.input.kt *.options "$root/testdata/ktfmt/$s/" \
    && { cp -- *.expected.kt "$root/testdata/ktfmt/$s/" 2>/dev/null || true; } \
    && { cp -- *.error "$root/testdata/ktfmt/$s/" 2>/dev/null || true; })
  echo "$s: $(ls "$root/testdata/ktfmt/$s" | grep -c '\.input\.kt$') cases"
done
