#!/usr/bin/env bash
# Regenerates testdata/detekt/<RuleName>/<case>.{input.kt|kts,options.json,findings,error} from detekt's own rule
# tests (the spec files in specs.txt): compiles them against a recording `Rule.lint` (extract/) and runs them on the
# detekt fat jar, so every expectation is what the real rule returned. Slow (JVM): run in the background.
#   extract-goldens.sh [--no-copy]    # --no-copy leaves the cases in target/detekt-extract/cases
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
upstream="$root/third_party/detekt"
lib="$here/lib"
work="$root/target/detekt-extract"
maven=https://repo1.maven.org/maven2

tag=$(sed -n 's/^DETEKT_TAG=v//p' "$root/tools/sync-detekt.sh")
jar=${DETEKT_JAR:-$root/tools/detekt-oracle/lib/detekt-cli-$tag-all.jar}
[[ -f $jar && -d $upstream ]] || { echo "no detekt jar or checkout; run tools/sync-detekt.sh" >&2; exit 1; }
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
fetch org/jetbrains/annotations/23.0.0/annotations-23.0.0.jar
deps=("$jar" "$lib"/*.jar)

rm -rf "$work"; mkdir -p "$work/src/test" "$work/src/specs" "$work/classes" "$work/cases"
cp "$here"/extract/*.kt "$work/src/test/"
cp "$upstream/detekt-test/src/main/kotlin/dev/detekt/test/"{TestConfig,FakeLanguageVersionSettings,FindingExtensions}.kt "$work/src/test/"
cp "$upstream/detekt-test-utils/src/main/kotlin/dev/detekt/test/utils/KtTestCompiler.kt" "$work/src/test/"
cp "$upstream/detekt-test-assertj/src/main/kotlin/dev/detekt/test/assertj/FindingsAssertions.kt" "$work/src/test/"
cp -r "$here/extract/META-INF" "$work/classes/"
# CaseRecorder reads TestConfig's pairs; fail if upstream moved the anchor.
sed -i 's/^class TestConfig private constructor(override val parent: Config?, private val values:/class TestConfig private constructor(override val parent: Config?, val values:/' "$work/src/test/TestConfig.kt"
grep -q 'override val parent: Config?, val values:' "$work/src/test/TestConfig.kt" || { echo "TestConfig.kt anchor not found; update the sed patch" >&2; exit 1; }

modules=() packages=()
while read -r module package glob; do
  [[ -z $module || $module == \#* ]] && continue
  modules+=("$module"); packages+=("$package")
  mkdir -p "$work/src/specs/$module"
  eval "cp \"$upstream/$module/src/test/kotlin/${package//.//}\"/$glob \"$work/src/specs/$module/\""
done < "$here/specs.txt"

echo "compiling upstream tests..."
# The fat jar embeds the Kotlin compiler detekt is built with; friend paths expose the rules' `internal` members.
"$java" -Xmx4g -cp "$(to_host "$jar")" org.jetbrains.kotlin.cli.jvm.K2JVMCompiler -no-stdlib -no-reflect -nowarn \
  -jvm-target 17 -Xfriend-paths="$(to_host "$jar")" \
  -cp "$(cp_of "${deps[@]}")" -d "$(to_host "$work/classes")" "$(to_host "$work/src")"

echo "running upstream tests with the recorder..."
: > "$work/junit.log"
for i in "${!modules[@]}"; do
  # In the module's directory: specs open `src/test/resources/...` relative to it.
  # One class path, not `-jar ... --class-path`: JUnit's Kotlin assertAll must see kotlin-stdlib (in the fat jar).
  (cd "$upstream/${modules[$i]}" && "$java" -Xss64m -Xmx4g -Dgolden.out="$(to_host "$work/cases")" \
    -Dgolden.root="$(to_host "$upstream")" -Djunit.jupiter.extensions.autodetection.enabled=true \
    -Djunit.jupiter.testinstance.lifecycle.default=per_class \
    -cp "$(cp_of "$work/classes" "${deps[@]}")" org.junit.platform.console.ConsoleLauncher execute \
    --select-package "${packages[$i]}" --include-classname '.*' --details=summary --disable-banner >> "$work/junit.log" 2>&1) || true
done
grep -aE '^\[ +[0-9]+ (tests|containers) (successful|failed|aborted|skipped)' "$work/junit.log" || tail -20 "$work/junit.log"

cases=$(find "$work/cases" -name '*.options.json' | wc -l)
echo "recorded $cases cases in $(find "$work/cases" -mindepth 1 -maxdepth 1 -type d | wc -l) rule dirs"
[[ ${1:-} == --no-copy ]] && exit
mkdir -p "$root/testdata/detekt"
find "$root/testdata/detekt" -mindepth 1 -maxdepth 1 -type d -exec rm -rf {} +
cp -r "$work/cases/." "$root/testdata/detekt/"
