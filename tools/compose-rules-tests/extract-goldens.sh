#!/usr/bin/env bash
# Regenerates testdata/compose-rules/<rule-dir>/ (rule-dir = rule id with ':' -> '_', e.g. compose_modifier-missing-check), the
# parity goldens of crates/ktrs-compose, from compose-rules' own ktlint rule tests run on the real engines. Slow (two JVMs):
# run it in the background (on testbox: see the testbox memory). Needs tools/sync-compose-rules.sh and tools/sync-ktlint.sh.
#   extract-goldens.sh [--no-copy]    # --no-copy leaves the cases in target/compose-rules-extract/cases
#
# 1. ktlint 1.8.0 (what compose-rules v0.6.7 is built and tested against): the rule tests
#    (rules/ktlint/src/test/.../ktlint/*CheckTest.kt) are compiled against ktlint 1.8.0's ktlint-test sources (sparse clone in
#    third_party/ktlint-1.8.0) patched with a recording hook (extract/CaseRecorder.kt, the 1.8 port of
#    tools/ktlint-tests/extract/CaseRecorder.kt), and run with JUnit on one class path: ktlint-cli-1.8.0-all.jar first, the
#    compose-rules -R jar last (its bundled ktlint/Kotlin classes are shadowed, as under the CLI's parent-first -R loader).
#    Tests that build a KtLintRuleEngine themselves get RecordingKtLintRuleEngine instead. Not recorded: ComposeRuleSetProviderTest
#    (reflection/Konsist checks of the rule set) and KtlintComposeKtConfigTest (unit test of the config reader): neither lints code.
# 2. ktlint 2.0.0-ALPHA-4 (replay/ReplayOn20.kt, separate JVM on the ktlint-cli-2.0.0-ALPHA-4-all.jar + the compose jar): every
#    recorded case is re-run with the compose rules loaded as 2.0's CLI loads a 1.x -R jar (RuleSetProviderV3 service ->
#    RuleProvider.toRuleV2Provider()) and the same .editorconfig overrides.
#
# Case format (same as testdata/ktlint, see tools/ktlint-tests/extract-goldens.sh): <case>.input.kt|kts; <case>.options
# (ktlint=1.8, rules=<ids, subject first>, path=, ec.<property>=<value>, test=<JUnit display path>); <case>.lint and <case>.format
# (rows `line:col<TAB>ruleId<TAB>auto|manual<TAB>detail`, '\' and newline escaped); <case>.expected.kt|kts (format result,
# only when it changed the input); <case>.error (`lint|format<TAB>exception`). All of these are ktlint 1.8's expectations.
#
# ktlint 2.0 overrides, written only where 2.0 differs from 1.8 (a harness uses <case>.2_0.K when present, else <case>.K):
#   <case>.2_0.lint | .2_0.format | .2_0.error   2.0's content of that kind; an EMPTY file when 2.0 has none but 1.8 has some.
#   <case>.2_0.expected.kt|kts                   2.0's format result; the unchanged input text when 2.0 leaves the input as is
#                                                (or fails) but 1.8 changed it.
# No other marker files. target/compose-rules-extract/replay.log lists each differing case and the kinds that differ.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
compose="$root/third_party/compose-rules"
ktlint18="$root/third_party/ktlint-1.8.0"
lib="$root/tools/compose-rules/lib"
work="$root/target/compose-rules-extract"
maven=https://repo1.maven.org/maven2

compose_jar=$(ls "$lib"/ktlint-compose-*-all.jar 2>/dev/null | head -1 || true)
[[ -n $compose_jar && -d $compose ]] || { echo "no compose-rules jar or checkout; run tools/sync-compose-rules.sh" >&2; exit 1; }
jar20=$(ls "$root"/tools/ktlint-oracle/lib/ktlint-cli-2.0.0-ALPHA-4-all.jar 2>/dev/null || true)
[[ -n $jar20 ]] || { echo "no ktlint 2.0 jar; run tools/sync-ktlint.sh" >&2; exit 1; }
bin=$(ls -d "$root"/tools/jdk/*/bin 2>/dev/null | head -1 || true)
java=${bin:+$bin/}java

sep=: ; to_host() { echo "$1"; }
if command -v cygpath >/dev/null; then sep=';'; to_host() { cygpath -m "$1"; }; fi
cp_of() { local out="" f; for f in "$@"; do out+="$(to_host "$f")$sep"; done; echo "${out%$sep}"; }

[[ -d $ktlint18/ktlint-test ]] || {
  git -c advice.detachedHead=false clone -q --depth 1 --branch 1.8.0 --filter=blob:none --sparse \
    https://github.com/pinterest/ktlint.git "$ktlint18"
  git -C "$ktlint18" sparse-checkout set ktlint-test
}

mkdir -p "$lib"
fetch() { [[ -f $lib/$(basename "$1") ]] || curl -sfL -o "$lib/$(basename "$1")" "$maven/$1"; }
fetch com/pinterest/ktlint/ktlint-cli/1.8.0/ktlint-cli-1.8.0-all.jar
fetch org/junit/platform/junit-platform-console-standalone/6.1.3/junit-platform-console-standalone-6.1.3.jar
fetch org/assertj/assertj-core/3.27.7/assertj-core-3.27.7.jar
fetch net/bytebuddy/byte-buddy/1.18.3/byte-buddy-1.18.3.jar
fetch com/google/jimfs/jimfs/1.3.1/jimfs-1.3.1.jar
fetch com/google/guava/guava/33.4.8-jre/guava-33.4.8-jre.jar
fetch com/google/guava/failureaccess/1.0.3/failureaccess-1.0.3.jar
jar18="$lib/ktlint-cli-1.8.0-all.jar"
test_deps=("$lib"/junit-platform-console-standalone-*.jar "$lib"/assertj-core-*.jar "$lib"/byte-buddy-*.jar)
jimfs=("$lib"/jimfs-*.jar "$lib"/guava-*.jar "$lib"/failureaccess-*.jar)
deps18=("$jar18" "${test_deps[@]}" "${jimfs[@]}" "$compose_jar")

rm -rf "$work"; mkdir -p "$work/src/test" "$work/src/rules" "$work/classes" "$work/replay" "$work/cases"
testsrc="$ktlint18/ktlint-test/src/main/kotlin/com/pinterest/ktlint/test"
for f in "$testsrc"/*.kt; do [[ $(basename "$f") == RuleSetProviderTest.kt ]] || cp "$f" "$work/src/test/"; done
cp "$here/extract/CaseRecorder.kt" "$root/tools/ktlint-tests/extract/CaseNaming.kt" "$work/src/test/"
cp -r "$root/tools/ktlint-tests/extract/META-INF" "$work/classes/"
cp "$compose"/rules/ktlint/src/test/kotlin/io/nlopez/compose/rules/ktlint/*CheckTest.kt "$work/src/rules/"

# The recording hook, and @Poko (a compiler plugin) replaced by a data class; fail if upstream moved either anchor.
assert_that="$work/src/test/KtLintAssertThat.kt"
sed -i -e '/^import dev.drewhamilton.poko.Poko$/d' -e '/^@Poko$/d' -e 's/^public class LintViolation$/public data class LintViolation/' \
  -e '/^) : AbstractAssert<KtLintAssertThatAssertable, String>(code.content, KtLintAssertThatAssertable::class.java) {$/a\
    init {\
        CaseRecorder.record(\
            setOf(ruleProvider).plus(additionalRuleProviders),\
            code,\
            editorConfigOverride.enableExperimentalRules().extendWithRuleSetRuleExecutionsFor(setOf(ruleProvider).plus(additionalRuleProviders)),\
            KTLINT_TEST_FILE_SYSTEM.fileSystem,\
        )\
    }' "$assert_that"
grep -q 'CaseRecorder.record' "$assert_that" && grep -q '^public data class LintViolation' "$assert_that" \
  || { echo "KtLintAssertThat.kt anchors not found; update the sed patch" >&2; exit 1; }
direct=$(grep -l '^import com.pinterest.ktlint.rule.engine.api.KtLintRuleEngine$' "$work"/src/rules/*.kt || true)
for f in $direct; do sed -i 's/\bKtLintRuleEngine(/com.pinterest.ktlint.test.RecordingKtLintRuleEngine(/' "$f"; done

echo "compiling compose-rules tests (ktlint 1.8)..."
# The compose jar is built by Kotlin 2.4; ktlint 1.8 embeds 2.2.21, hence the metadata check skip.
"$java" -Xmx4g -cp "$(to_host "$jar18")" org.jetbrains.kotlin.cli.jvm.K2JVMCompiler -no-stdlib -no-reflect -nowarn \
  -jvm-target 17 -Xskip-metadata-version-check -Xfriend-paths="$(to_host "$compose_jar")" \
  -cp "$(cp_of "${deps18[@]}")" -d "$(to_host "$work/classes")" "$(to_host "$work/src")"

echo "running compose-rules tests with the recorder (ktlint 1.8)..."
# One class path, not `-jar ... --class-path`: JUnit's Kotlin assertAll must see kotlin-stdlib (in the fat jar).
"$java" -Xss64m -Xmx4g -Dgolden.out="$(to_host "$work/cases")" -Djunit.jupiter.extensions.autodetection.enabled=true \
  -Dlogback.configurationFile="$(to_host "$root/tools/ktlint-oracle/logback.xml")" \
  -cp "$(cp_of "$work/classes" "${deps18[@]}")" org.junit.platform.console.ConsoleLauncher execute \
  --select-package io.nlopez.compose.rules.ktlint \
  --details=summary --disable-banner > "$work/junit.log" 2>&1 || true
grep -aE '^\[ +[0-9]+ (tests|containers) (successful|failed|aborted|skipped)' "$work/junit.log" || tail -20 "$work/junit.log"
[[ -f $work/cases/upstream-failures.txt ]] && { echo "upstream test failures:"; cat "$work/cases/upstream-failures.txt"; }

echo "replaying the cases on ktlint 2.0..."
deps20=("$jar20" "${jimfs[@]}" "$compose_jar")
"$java" -Xmx4g -cp "$(to_host "$jar20")" org.jetbrains.kotlin.cli.jvm.K2JVMCompiler -no-stdlib -no-reflect -nowarn \
  -jvm-target 17 -cp "$(cp_of "${deps20[@]}")" -d "$(to_host "$work/replay")" "$(to_host "$here/replay")"
"$java" -Xss64m -Xmx4g -Dlogback.configurationFile="$(to_host "$root/tools/ktlint-oracle/logback.xml")" \
  -cp "$(cp_of "$work/replay" "${deps20[@]}")" ReplayOn20Kt "$(to_host "$work/cases")" > "$work/replay.log"
tail -1 "$work/replay.log"

cases=$(find "$work/cases" -name '*.options' | wc -l)
echo "recorded $cases cases in $(find "$work/cases" -mindepth 1 -maxdepth 1 -type d | wc -l) rule dirs"
[[ ${1:-} == --no-copy ]] && exit
rm -rf "$root/testdata/compose-rules"; mkdir -p "$root/testdata/compose-rules"
find "$work/cases" -mindepth 1 -maxdepth 1 -type d -exec cp -r {} "$root/testdata/compose-rules/" \;
