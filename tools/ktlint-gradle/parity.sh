#!/usr/bin/env bash
# The io.github.hexay.ktrs.ktlint plugin vs org.jlleitschuh.gradle.ktlint 14.2.0 (ktlint 1.8.0) on the same project:
# console, task outcomes, report files and formatted sources (research/29-ktlint-gradle-dropin.md).
#   tools/ktlint-gradle/parity.sh [scenario ...]   (default: all; needs `cargo build --bins`, network, a JDK 17+)
# Scenarios: check-all format baseline options compose-maven compose-all realcode. Output: target/ktlint-gradle/<scenario>/
# {upstream,ktrs}/ and <scenario>.diff (byte comparison but for project paths and Gradle noise; SLASHES=1 also ignores
# path separators and ANSI colors). Accepted differences: KNOWN (default tools/parity/known-diffs/ktlint-gradle.tsv).
# Exit 1 when a scenario differs beyond them or Gradle did not run.
set -uo pipefail
# Windows-form paths under Git Bash: they end up in Gradle files and the JVM.
root="$(cd "$(dirname "$0")/../.." && (pwd -W 2> /dev/null || pwd))"
here="$root/tools/ktlint-gradle"
out="$root/target/ktlint-gradle"
exe="$root/target/debug/ktrs$([[ $OSTYPE == msys* || $OSTYPE == cygwin* ]] && echo .exe)"
gradle="${GRADLE:-$root/java/gradlew}"
[[ -n ${JAVA_HOME:-} ]] || export JAVA_HOME="$(ls -d "$root"/tools/jdk/* 2>/dev/null | head -1)"
[[ -n $JAVA_HOME ]] || { echo "set JAVA_HOME (or run tools/ensure-jdk.sh)" >&2; exit 2; }
compose_jar="$out/ktlint-compose-0.6.7-all.jar"
known="${KNOWN:-$root/tools/parity/known-diffs/ktlint-gradle.tsv}"
status=0

# project <scenario> <side> <block> [template]: a fresh sample project in $out/<scenario>/<side>/project.
project() {
  local dir="$out/$1/$2/project" plugin include=""
  rm -rf "$out/$1/$2"; mkdir -p "$dir"
  cp -r "${4:-$here/sample}/." "$dir/"
  if [[ $2 == upstream ]]; then
    plugin="id 'org.jlleitschuh.gradle.ktlint' version '14.2.0'"
  else
    plugin="id 'io.github.hexay.ktrs.ktlint'"
    include="includeBuild('$root/java')"
    echo "ktrs.executable=$exe" > "$dir/gradle.properties"
  fi
  printf "pluginManagement {\n    %s\n    repositories { gradlePluginPortal(); mavenCentral(); google() }\n}\nrootProject.name = 'sample'\n" \
    "$include" > "$dir/settings.gradle"
  printf "plugins {\n    id 'org.jetbrains.kotlin.jvm' version '2.4.10'\n    %s\n}\nrepositories { mavenCentral() }\n" "$plugin" \
    > "$dir/build.gradle"
  sed "s#@COMPOSE_ALL_JAR@#$compose_jar#" "$here/blocks/$3" >> "$dir/build.gradle"
}

# scenario <name> <block> <template> <gradle args>...: both sides, each run's output appended to console.txt.
scenario() {
  local name=$1 block=$2 template=$3; shift 3
  for side in upstream ktrs; do
    project "$name" "$side" "$block" "$template"
    for args in "$@"; do
      # shellcheck disable=SC2086
      "$gradle" -p "$out/$name/$side/project" --console=plain -Dorg.gradle.jvmargs=-Xmx768m -Dorg.gradle.welcome=never $args \
        > "$out/$name/$side/run.txt" 2>&1
      { echo "== exit $? : $args"; cat "$out/$name/$side/run.txt"; } >> "$out/$name/$side/console.txt"
      grep -q "^> Task :loadKtlintReporters" "$out/$name/$side/run.txt" || { echo "$name/$side: gradle did not run ($args)" >&2; status=1; }
    done
  done
  py_ "$here/compare.py" "$out/$name" --plugin-ids io.github.hexay.ktrs.ktlint org.jlleitschuh.gradle.ktlint ${SLASHES:+--slashes} \
    --known "$known" > "$out/$name.diff" || status=1
  echo "$name: $(tail -1 "$out/$name.diff") ($(wc -l < "$out/$name.diff") diff lines); $(py_ "$here/rows.py" "$out/$name" | head -1)"
}

py_() { if command -v py > /dev/null; then py -3 "$@"; else python3 "$@"; fi; }

realcode_template() {
  local t="$out/realcode-template"
  local corpus="${CORPUS:-$root/corpus}"
  rm -rf "$t"; mkdir -p "$t/src/main/kotlin" "$t/src/test/kotlin"
  cp -r "$corpus/okhttp/okhttp/src/commonJvmAndroid/kotlin/." "$t/src/main/kotlin/"
  cp -r "$corpus/ktlint/ktlint-rule-engine/src/main/kotlin/." "$t/src/test/kotlin/"
  cp "$corpus/okhttp/.editorconfig" "$t/"
  echo "$t"
}

compose_template() {
  local t="$out/compose-template"
  rm -rf "$t"; mkdir -p "$t"; cp -r "$here/sample/." "$t/"; cp "$here/Ui.kt" "$t/src/main/kotlin/com/example/"
  [[ -f $compose_jar ]] || curl -sfL -o "$compose_jar" \
    https://github.com/mrmans0n/compose-rules/releases/download/v0.6.7/ktlint-compose-0.6.7-all.jar
  echo "$t"
}

mkdir -p "$out"
for s in ${*:-check-all format baseline options compose-maven compose-all realcode}; do
  case $s in
    check-all) scenario check-all all-reporters.gradle "$here/sample" "ktlintCheck --continue" "ktlintCheck --continue" ;;
    format) scenario format default.gradle "$here/sample" "ktlintFormat --continue" "ktlintFormat --continue" "ktlintFormat --continue" ;;
    baseline) scenario baseline all-reporters.gradle "$here/sample" "ktlintGenerateBaseline" "ktlintCheck --continue" ;;
    options) scenario options options.gradle "$here/sample" "ktlintCheck --continue" "ktlintFormat --continue" ;;
    compose-maven) scenario compose-maven compose-maven.gradle "$(compose_template)" "ktlintCheck --continue" ;;
    compose-all) scenario compose-all compose-all.gradle "$(compose_template)" "ktlintCheck --continue" ;;
    realcode) scenario realcode all-reporters.gradle "$(realcode_template)" "ktlintCheck --continue" "ktlintFormat --continue" ;;
    *) echo "unknown scenario $s" >&2; exit 2 ;;
  esac
done
exit $status
