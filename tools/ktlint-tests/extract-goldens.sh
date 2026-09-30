#!/usr/bin/env bash
# Regenerates testdata/ktlint/<rule-id>/<case>.{input.kt|kts,options,lint,format,expected.kt|kts,error} from ktlint's
# own rule tests: compiles them against a recording KtLintAssertThat (extract/CaseRecorder.kt) and runs them on
# the ktlint fat jar, so every expectation is the real engine's output. Slow (JVM): run in the background.
#   extract-goldens.sh [--no-copy]    # --no-copy leaves the cases in target/ktlint-extract/cases
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
upstream="$root/third_party/ktlint"
lib="$here/lib"
work="$root/target/ktlint-extract"
maven=https://repo1.maven.org/maven2

jar=$(ls "$root"/tools/ktlint-oracle/lib/ktlint-cli-*-all.jar 2>/dev/null | head -1 || true)
[[ -n $jar && -d $upstream ]] || { echo "no ktlint jar or checkout; run tools/sync-ktlint.sh" >&2; exit 1; }
bin=$(ls -d "$root"/tools/jdk/*/bin 2>/dev/null | head -1 || true)
java=${bin:+$bin/}java

sep=: ; to_host() { echo "$1"; }
if command -v cygpath >/dev/null; then sep=';'; to_host() { cygpath -m "$1"; }; fi
cp_of() { local out="" f; for f in "$@"; do out+="$(to_host "$f")$sep"; done; echo "${out%$sep}"; }

mkdir -p "$lib"
fetch() { [[ -f $lib/$(basename "$1") ]] || curl -sfL -o "$lib/$(basename "$1")" "$maven/$1"; }
fetch org/junit/platform/junit-platform-console-standalone/6.1.3/junit-platform-console-standalone-6.1.3.jar
fetch org/assertj/assertj-core/3.27.7/assertj-core-3.27.7.jar
fetch net/bytebuddy/byte-buddy/1.18.3/byte-buddy-1.18.3.jar
fetch com/google/jimfs/jimfs/1.3.1/jimfs-1.3.1.jar
fetch com/google/guava/guava/33.4.8-jre/guava-33.4.8-jre.jar
fetch com/google/guava/failureaccess/1.0.3/failureaccess-1.0.3.jar
fetch org/jetbrains/kotlin/kotlin-reflect/2.4.10/kotlin-reflect-2.4.10.jar
deps=("$jar" "$lib"/*.jar)

rm -rf "$work"; mkdir -p "$work/src/test" "$work/src/rules" "$work/classes" "$work/cases"
testsrc="$upstream/ktlint-test/src/main/kotlin/io/github/ktlint/core/test"
for f in "$testsrc"/*.kt; do [[ $(basename "$f") == RuleSetProviderTest.kt ]] || cp "$f" "$work/src/test/"; done
cp "$here/extract/CaseRecorder.kt" "$here/extract/CaseNaming.kt" "$work/src/test/"
cp -r "$upstream/ktlint-ruleset-standard/src/test/kotlin/io/github/ktlint/core/ruleset/standard/rules/." "$work/src/rules/"
cp -r "$here/extract/META-INF" "$work/classes/"

# The recording hook, and @Poko (a compiler plugin) replaced by a data class; fail if upstream moved either anchor.
assert_that="$work/src/test/KtLintAssertThat.kt"
sed -i -e '/^import dev.drewhamilton.poko.Poko$/d' -e '/^@Poko$/d' -e 's/^public class LintViolation$/public data class LintViolation/' \
  -e '/^) : AbstractAssert<KtLintAssertThatAssertable, String>(code.content, KtLintAssertThatAssertable::class.java) {$/a\
    init {\
        CaseRecorder.record(\
            ruleProvider,\
            additionalRuleProviders,\
            code,\
            editorConfigOverride.enableExperimentalRules().extendWithRuleSetRuleExecutionsFor(setOf(ruleProvider).plus(additionalRuleProviders)),\
            KTLINT_TEST_FILE_SYSTEM.fileSystem,\
        )\
    }' "$assert_that"
grep -q 'CaseRecorder.record' "$assert_that" && grep -q '^public data class LintViolation' "$assert_that" \
  || { echo "KtLintAssertThat.kt anchors not found; update the sed patch" >&2; exit 1; }

echo "compiling upstream tests..."
# The fat jar embeds the Kotlin compiler ktlint is built with; friend paths expose the rules' `internal` members.
"$java" -Xmx4g -cp "$(to_host "$jar")" org.jetbrains.kotlin.cli.jvm.K2JVMCompiler -no-stdlib -no-reflect -nowarn \
  -jvm-target 17 -language-version 2.2 -api-version 2.2 -Xfriend-paths="$(to_host "$jar")" \
  -cp "$(cp_of "${deps[@]}")" -d "$(to_host "$work/classes")" "$(to_host "$work/src")"

echo "running upstream tests with the recorder..."
"$java" -Xss64m -Xmx4g -Dgolden.out="$(to_host "$work/cases")" -Djunit.jupiter.extensions.autodetection.enabled=true \
  -Dlogback.configurationFile="$(to_host "$root/tools/ktlint-oracle/logback.xml")" \
  -jar "$(to_host "$lib/junit-platform-console-standalone-6.1.3.jar")" execute \
  --class-path "$(cp_of "$work/classes" "${deps[@]}")" --select-package io.github.ktlint.core.ruleset.standard.rules \
  --details=summary --disable-banner > "$work/junit.log" 2>&1 || true
grep -aE '^\[ +[0-9]+ (tests|containers) (successful|failed|aborted|skipped)' "$work/junit.log" || tail -20 "$work/junit.log"

cases=$(find "$work/cases" -name '*.options' | wc -l)
echo "recorded $cases cases in $(find "$work/cases" -mindepth 1 -maxdepth 1 -type d | wc -l) rule dirs"
[[ ${1:-} == --no-copy ]] && exit
find "$root/testdata/ktlint" -mindepth 1 -maxdepth 1 -type d -exec rm -rf {} +
cp -r "$work/cases/." "$root/testdata/ktlint/"
