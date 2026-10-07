#!/usr/bin/env bash
# io.github.hexay:ktrs-ktlint-maven-plugin vs com.github.gantsign.maven:ktlint-maven-plugin 3.7.1 (ktlint 1.8.0) on the
# same projects: exit codes, console, report files and formatted sources (research/31-ktlint-maven-dropin.md).
#   tools/ktlint-maven/parity.sh [scenario ...]   (default: all; needs `cargo build --bins`, network, a JDK 17+)
# Scenarios: clean violations format reporters options stdout unknown-reporter compose handoff parse-error report
# multimodule. Output: target/ktlint-maven/<scenario>/{upstream,ktrs}/ and <scenario>.diff. Our plugin is published to
# target/ktlint-maven/repo as $VERSION first (NO_PUBLISH=1 skips that); MVN=<mvn> skips the pinned Maven download.
set -uo pipefail
# Windows-form paths under Git Bash: they end up in POMs and the JVM.
root="$(cd "$(dirname "$0")/../.." && (pwd -W 2> /dev/null || pwd))"
here="$root/tools/ktlint-maven"
out="$root/target/ktlint-maven"
exe="$root/target/debug/ktrs$([[ $OSTYPE == msys* || $OSTYPE == cygwin* ]] && echo .exe)"
version="${VERSION:-0.0.0-parity}"
maven_version=3.9.16
maven_sha512=ed41650d42485cfc243fad22158caf9cbb5dc408ce7a09ddb94dd42a019de929ca43065bfa450612cf12bf78b5cafa3884b96c090de326ff590448c933454af3
[[ -n ${JAVA_HOME:-} ]] || export JAVA_HOME="$(ls -d "$root"/tools/jdk/* 2>/dev/null | head -1)"
[[ -n $JAVA_HOME ]] || { echo "set JAVA_HOME (or run tools/ensure-jdk.sh)" >&2; exit 2; }
mkdir -p "$out"

if [[ -z ${MVN:-} ]]; then
  MVN="$out/apache-maven-$maven_version/bin/mvn"
  if [[ ! -x $MVN ]]; then
    zip="$out/maven.zip"
    curl -sfL -o "$zip" "https://archive.apache.org/dist/maven/maven-3/$maven_version/binaries/apache-maven-$maven_version-bin.zip"
    [[ $(sha512sum "$zip" | cut -c1-128) == "$maven_sha512" ]] || { echo "maven.zip: checksum mismatch" >&2; exit 2; }
    unzip -q -o "$zip" -d "$out"; rm "$zip"
  fi
fi

if [[ -z ${NO_PUBLISH:-} ]]; then
  "$root/java/gradlew" -p "$root/java" --console=plain -q -PktrsVersion="$version" -PpagesRepo="$out/repo" \
    :publishAllPublicationsToGithubPagesRepository :ktrs-ktlint-maven-plugin:publishAllPublicationsToGithubPagesRepository \
    || exit 2
  # Maven never re-resolves a release version it has cached.
  rm -rf "$out/m2/io/github/hexay"
fi

repo_url="file:///${out#/}/repo"

# pom <dir> <side> <block> <packaging> <artifactId> [modules]: <dir>/pom.xml from pom.xml.in.
pom() {
  local coords block="$here/blocks/$3.xml"
  [[ $2 == upstream ]] \
    && coords="<groupId>com.github.gantsign.maven</groupId><artifactId>ktlint-maven-plugin</artifactId><version>3.7.1</version>" \
    || coords="<groupId>io.github.hexay</groupId><artifactId>ktrs-ktlint-maven-plugin</artifactId><version>$version</version>"
  [[ -f $block ]] || block=/dev/null
  sed -e "s#@PLUGIN@#        $coords#" -e "s#@PACKAGING@#$4#" -e "s#@ARTIFACT_ID@#$5#" -e "s#@REPO@#$repo_url#" \
    -e "s#@MODULES@#${6:-}#" -e "/@BLOCK@/r $block" -e "/@BLOCK@/d" "$here/pom.xml.in" > "$1/pom.xml"
  mkdir -p "$1/.mvn"
  # gantsign's documented workaround for ktlint on Java 17+.
  echo "--add-opens java.base/java.lang=ALL-UNNAMED" > "$1/.mvn/jvm.config"
}

# sources <template> <dir>: the scenario's Kotlin sources.
sources() {
  cp -r "$root/tools/ktlint-gradle/sample/." "$2/"
  case $1 in
    clean) rm "$2/script.kts" "$2/src/main/kotlin/com/example/Fail.kt" "$2/src/test/kotlin/com/example/FailTest.kt" ;;
    compose) cp "$root/tools/ktlint-gradle/Ui.kt" "$2/src/main/kotlin/com/example/" ;;
    parse-error) printf 'package com.example\n\nfun broken( {\n' > "$2/src/main/kotlin/com/example/Broken.kt" ;;
  esac
}

# project <scenario> <side> <block> <template>: a fresh project in $out/<scenario>/<side>/project.
project() {
  local dir="$out/$1/$2/project"
  rm -rf "$out/$1/$2"; mkdir -p "$dir"
  if [[ $4 == multimodule ]]; then
    pom "$dir" "$2" "$3" pom sample "<modules><module>a</module><module>b</module></modules>"
    for m in a b; do
      mkdir -p "$dir/$m"; sources sample "$dir/$m"
      printf '<project xmlns="http://maven.apache.org/POM/4.0.0"><modelVersion>4.0.0</modelVersion><parent><groupId>com.example</groupId><artifactId>sample</artifactId><version>1.0</version></parent><artifactId>%s</artifactId><packaging>pom</packaging></project>\n' \
        "$m" > "$dir/$m/pom.xml"
    done
    rm "$dir/b/src/main/kotlin/com/example/Fail.kt" "$dir/b/script.kts"
  else
    pom "$dir" "$2" "$3" "${PACKAGING:-jar}" sample; sources "$4" "$dir"
  fi
}

# scenario <name> <block> <template> <mvn args>...: both sides, each run's output appended to console.txt.
scenario() {
  local name=$1 block=$2 template=$3; shift 3
  for side in upstream ktrs; do
    project "$name" "$side" "$block" "$template"
    local extra=() dir="$out/$name/$side"
    [[ $side == ktrs ]] && extra=("-Dktrs.executable=$exe")
    for args in "$@"; do
      # shellcheck disable=SC2086
      (cd "$dir/project" && "$MVN" -B -ntp -Dmaven.repo.local="$out/m2" -Dmaven.plugin.validation=NONE "${extra[@]}" $args) \
        > "$dir/run.txt" 2>&1
      { echo "== exit $? : $args"; cat "$dir/run.txt"; } >> "$dir/console.txt"
    done
  done
  py_ "$here/compare.py" "$out/$name" > "$out/$name.diff"
  echo "$name: $(tail -1 "$out/$name.diff") ($(wc -l < "$out/$name.diff") diff lines)"
}

py_() { if command -v py > /dev/null; then py -3 "$@"; else python3 "$@"; fi; }

for s in ${*:-clean violations format reporters options stdout unknown-reporter compose handoff parse-error report multimodule}; do
  case $s in
    clean) scenario clean default clean "ktlint:check" "ktlint:format" ;;
    violations) scenario violations default sample "ktlint:check" "-Dktlint.verbose=true ktlint:check" \
      "-Dktlint.failOnViolation=false -Dktlint.android=true -Dktlint.experimental=true ktlint:check" \
      "-Dktlint.includeTestSources=false -Dktlint.includeScripts=false ktlint:check" "-Dktlint.skip=true ktlint:check" ;;
    format) scenario format default sample "ktlint:format" "ktlint:check" "ktlint:format" ;;
    reporters) scenario reporters reporters sample "ktlint:check" ;;
    options) scenario options options sample "ktlint:check" ;;
    stdout) scenario stdout stdout sample "ktlint:check" ;;
    unknown-reporter) scenario unknown-reporter unknown-reporter sample "ktlint:check" ;;
    compose) scenario compose compose compose "ktlint:check" ;;
    handoff) scenario handoff handoff compose "ktlint:check" "ktlint:format" ;;
    parse-error) scenario parse-error default parse-error "ktlint:check" "ktlint:format" ;;
    report) scenario report default sample "ktlint:ktlint" ;;
    multimodule) PACKAGING=pom scenario multimodule lifecycle multimodule "ktlint:check -fae" "verify" ;;
    *) echo "unknown scenario $s" >&2; exit 2 ;;
  esac
done
