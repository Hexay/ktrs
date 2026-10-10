#!/usr/bin/env bash
# The io.github.hexay.ktrs.kotlinter plugin vs org.jmailen.kotlinter 5.7.0 (ktlint 1.8.0) on the same project: console,
# exit status, task outcomes and every project file (reports, formatted sources, the hook); research/34-kotlinter-dropin.md.
#   tools/kotlinter/parity.sh [scenario ...]   (default: the sample scenarios; needs `cargo build --bins`, network, a JDK 17+)
# Sample scenarios: lint-all lint-ignored format format-strict custom-tasks parse-error editorconfig hook graph
# compose-maven compose-all custom-rules kmp. Also: android kmp-android (need ANDROID_HOME; an empty directory will do),
# kmp-android-library (AGP 8: also GRADLE=<a Gradle 8.14 to 9.5 wrapper>), realcode (needs corpus/).
# Output: target/kotlinter/<scenario>/{upstream,ktrs}/ and <scenario>.diff (byte comparison but for project paths and
# Gradle noise). SIDES=upstream runs one side only (no comparison). Accepted differences: KNOWN (default
# tools/parity/known-diffs/kotlinter.tsv). Exit 1 when a scenario differs beyond them or Gradle did not run.
set -uo pipefail
# Windows-form paths under Git Bash: they end up in Gradle files and the JVM.
root="$(cd "$(dirname "$0")/../.." && (pwd -W 2> /dev/null || pwd))"
here="$root/tools/kotlinter"
shared="$root/tools/ktlint-gradle"
out="$root/target/kotlinter"
exe="$root/target/debug/ktrs$([[ $OSTYPE == msys* || $OSTYPE == cygwin* ]] && echo .exe)"
gradle="${GRADLE:-$root/java/gradlew}"
[[ -n ${JAVA_HOME:-} ]] || export JAVA_HOME="$(ls -d "$root"/tools/jdk/* 2>/dev/null | head -1)"
[[ -n $JAVA_HOME ]] || { echo "set JAVA_HOME (or run tools/ensure-jdk.sh)" >&2; exit 2; }
compose_jar="$out/ktlint-compose-0.6.7-all.jar"
known="${KNOWN:-$root/tools/parity/known-diffs/kotlinter.tsv}"
sides="${SIDES:-upstream ktrs}"
jvm="id 'org.jetbrains.kotlin.jvm' version '2.4.10'"
kmp="id 'org.jetbrains.kotlin.multiplatform' version '2.4.10'"
agp="${AGP:-9.2.1}"
status=0

# project <scenario> <side> <plugins> <block> <template>: a fresh project in $out/<scenario>/<side>/project. <plugins>:
# the `plugins { }` lines beside kotlinter's. A template's `settings.tail` is appended to settings.gradle; `@KOTLINTER@`
# in its build files is the side's plugin id.
project() {
  local dir="$out/$1/$2/project" id plugin include=""
  rm -rf "$out/$1/$2"; mkdir -p "$dir"
  cp -r "$5/." "$dir/"
  if [[ $2 == upstream ]]; then
    id="org.jmailen.kotlinter"; plugin="id '$id' version '5.7.0'"
  else
    id="io.github.hexay.ktrs.kotlinter"; plugin="id '$id'"
    include="includeBuild('$root/java')"
    echo "ktrs.executable=$exe" >> "$dir/gradle.properties"
  fi
  {
    printf "pluginManagement {\n    %s\n    repositories { gradlePluginPortal(); mavenCentral(); google() }\n}\n" "$include"
    printf "buildCache { local { directory = file('.gradle/build-cache') } }\nrootProject.name = 'sample'\n"
    cat "$dir/settings.tail" 2> /dev/null
  } > "$dir/settings.gradle"
  rm -f "$dir/settings.tail"
  printf "plugins {\n    %s\n    %s\n}\nrepositories { mavenCentral(); google() }\n" "$3" "$plugin" > "$dir/build.gradle"
  sed "s#@COMPOSE_ALL_JAR@#$compose_jar#" "$here/blocks/$4" >> "$dir/build.gradle"
  find "$dir" -name build.gradle -exec sed -i "s#@KOTLINTER@#$id#" {} +
}

# scenario <name> <plugins> <block> <template> <step>...: every side, each run's output appended to console.txt.
# A step is Gradle arguments, or `sh:<command>` run in the project directory.
scenario() {
  local name=$1 plugins=$2 block=$3 template=$4 side step; shift 4
  for side in $sides; do
    project "$name" "$side" "$plugins" "$block" "$template"
    for step in "$@"; do
      if [[ $step == sh:* ]]; then
        (cd "$out/$name/$side/project" && eval "${step#sh:}")
        continue
      fi
      # shellcheck disable=SC2086
      "$gradle" -p "$out/$name/$side/project" --console=plain -Dorg.gradle.jvmargs=-Xmx768m -Dorg.gradle.welcome=never $step \
        > "$out/$name/$side/run.txt" 2>&1
      { echo "== exit $? : $step"; cat "$out/$name/$side/run.txt"; } >> "$out/$name/$side/console.txt"
      # The last one: what kotlinter 5.7.0 does to a build with a Kotlin plugin and `com.android.base` (kmp-android-library).
      grep -Eq "^(> Task :|:[A-Za-z:]+ SKIPPED|Detailed task information|> Cannot add task 'lintKotlin')" "$out/$name/$side/run.txt" ||
        { echo "$name/$side: gradle did not run ($step)" >&2; status=1; }
    done
  done
  [[ $sides == "upstream ktrs" ]] || { echo "$name: ran $sides"; return; }
  py_ "$shared/compare.py" "$out/$name" --plugin-ids io.github.hexay.ktrs.kotlinter org.jmailen.kotlinter --known "$known" \
    > "$out/$name.diff" || status=1
  echo "$name: $(tail -1 "$out/$name.diff") ($(wc -l < "$out/$name.diff") diff lines); $(py_ "$shared/rows.py" "$out/$name" | head -1)"
}

py_() { if command -v py > /dev/null; then py -3 "$@"; else python3 "$@"; fi; }

# template <name> <dir>...: the directories merged into $out/<name>-template.
template() {
  local t="$out/$1-template" d; shift
  rm -rf "$t"; mkdir -p "$t"
  for d in "$@"; do cp -r "$d/." "$t/"; done
  echo "$t"
}

realcode_template() {
  local t corpus="${CORPUS:-$root/corpus}"
  t="$(template realcode)"; mkdir -p "$t/src/main/kotlin" "$t/src/test/kotlin"
  cp -r "$corpus/okhttp/okhttp/src/commonJvmAndroid/kotlin/." "$t/src/main/kotlin/"
  cp -r "$corpus/ktlint/ktlint-rule-engine/src/main/kotlin/." "$t/src/test/kotlin/"
  cp "$corpus/okhttp/.editorconfig" "$t/"
  echo "$t"
}

compose_template() {
  local t
  t="$(template compose "$shared/sample")"; cp "$shared/Ui.kt" "$t/src/main/kotlin/com/example/"
  [[ -f $compose_jar ]] || curl -sfL -o "$compose_jar" \
    https://github.com/mrmans0n/compose-rules/releases/download/v0.6.7/ktlint-compose-0.6.7-all.jar
  echo "$t"
}

need_android() {
  [[ -n ${ANDROID_HOME:-} ]] && return 0
  echo "$1: skipped, ANDROID_HOME is not set" >&2
  return 1
}

# The parent directory's .editorconfig (root = true) is part of both plugins' chain.
editorconfig_steps=(
  "sh:printf 'root = true\n[*.kt]\nktlint_standard_filename = disabled\n' > ../.editorconfig"
  "lintKotlin --continue" "lintKotlin --continue"
  "sh:printf '[*.{kt,kts}]\nktlint_standard = disabled\n' > .editorconfig"
  "lintKotlin --continue" "lintKotlin --continue"
  "sh:printf 'root = true\n[*.kt]\nmax_line_length = 40\n' > ../.editorconfig; printf '[*.kt]\nindent_size = 2\n' > .editorconfig"
  "lintKotlin --continue" "formatKotlin --continue" "lintKotlin --continue"
)
hook_steps=(
  "installKotlinterPrePushHook"
  "sh:mkdir .git" "installKotlinterPrePushHook" "installKotlinterPrePushHook"
  "sh:printf '#!/bin/sh\necho mine\n' > .git/hooks/pre-push" "installKotlinterPrePushHook"
  "sh:sed -i 's/KOTLINTER 5.7.0/KOTLINTER 5.0.0/; s/lintKotlin ;/lintKotlinOld ;/' .git/hooks/pre-push" "installKotlinterPrePushHook"
)
lint="lintKotlin --continue" format="formatKotlin --continue"

mkdir -p "$out"
sample="$(template sample "$shared/sample" "$here/sample")"
for s in ${*:-lint-all lint-ignored format format-strict custom-tasks parse-error editorconfig hook graph compose-maven compose-all custom-rules kmp}; do
  case $s in
    lint-all) scenario $s "$jvm" all-reporters.gradle "$sample" "$lint" "$lint" ;;
    lint-ignored) scenario $s "$jvm" lint-ignored.gradle "$sample" "$lint --build-cache" "$lint --build-cache" "clean $lint --build-cache" ;;
    format) scenario $s "$jvm" default.gradle "$sample" "$format" "$format" "$format" ;;
    format-strict) scenario $s "$jvm" format-strict.gradle "$sample" "$format" "$format" "$format" ;;
    custom-tasks)
      scenario $s "" custom-tasks.gradle "$here/custom-tasks" "customLint emptyLint noReportLint --continue" \
        "customFormat emptyFormat --continue" "customLint emptyLint noReportLint --continue" "tasks --all" ;;
    parse-error) scenario $s "$jvm" all-reporters.gradle "$here/parse-error" "$lint" "$format" "$lint" ;;
    editorconfig) scenario $s "$jvm" default.gradle "$sample" "${editorconfig_steps[@]}" ;;
    hook) scenario $s "$jvm" default.gradle "$sample" "${hook_steps[@]}" ;;
    graph)
      scenario $s "$jvm" default.gradle "$sample" "tasks --all" "lintKotlin formatKotlin check --dry-run" \
        "help --task lintKotlinMain" "help --task formatKotlinMain" "help --task installKotlinterPrePushHook" ;;
    compose-maven) scenario $s "$jvm" compose-maven.gradle "$(compose_template)" "$lint" "$format" ;;
    compose-all) scenario $s "$jvm" compose-all.gradle "$(compose_template)" "$lint" "$format" ;;
    custom-rules) scenario $s "$jvm" custom-rules.gradle "$here/custom-rules" "$lint" "$format" "$lint" ;;
    kmp) scenario $s "$kmp" kmp.gradle "$here/kmp" "$lint" "$format" "tasks --all" ;;
    android)
      need_android $s && scenario $s "id 'com.android.library' version '$agp'" android.gradle "$here/android" "$lint" "$format" "tasks --all" ;;
    kmp-android)
      need_android $s && scenario $s "$kmp
    id 'com.android.kotlin.multiplatform.library' version '$agp'" kmp-android.gradle "$here/kmp" "$lint" "$format" "tasks --all" ;;
    kmp-android-library)
      need_android $s && scenario $s "$kmp
    id 'com.android.library' version '${AGP_LEGACY:-8.13.2}'" kmp-android-library.gradle "$here/kmp" "$lint" "$format" "tasks --all" ;;
    realcode) scenario $s "$jvm" all-reporters.gradle "$(realcode_template)" "$lint" "$format" ;;
    *) echo "unknown scenario $s" >&2; exit 2 ;;
  esac
done
exit $status
