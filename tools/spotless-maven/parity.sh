#!/usr/bin/env bash
# Stock spotless-maven-plugin 3.10.3 `<ktfmt>` / `<ktlint>` 1.8.0 vs the same configuration swapped to ktrs with
# `implementation="io.github.hexay.ktrs.spotless.maven.KtrsKtfmt|KtrsKtlint"` (research/30): per scenario and side,
# spotless:check, spotless:apply, spotless:check on a fresh project; compares exit codes, Spotless's messages and the
# applied sources byte for byte.
# ktfmt: ktrs formats as ktfmt 0.65, which this plugin can't run (it resolves `com.facebook:ktfmt:<version>` and calls
# the `com.facebook.ktfmt` API; 0.65 is `org.jetbrains.kotlinx`). So the stock side runs KTFMT_STOCK (default 0.64) for
# the exit codes and messages, and the applied sources are compared with the ktfmt 0.65 jar's (tools/sync-ktfmt.sh)
# on the same sources and options; the scenario line also counts the files where stock 0.64 differs (the upgrade diff).
#   tools/spotless-maven/parity.sh [scenario ...]   (default: all; needs `cargo build --bins`, network, a JDK 17+)
# Env: CORPUS (default <repo>/corpus; the real-* scenarios are skipped without it), M2 (local Maven repository,
# default target/spotless-maven/m2), KNOWN (accepted differences, default tools/parity/known-diffs/spotless-maven.tsv).
# Output: target/spotless-maven/<scenario>/{stock,ktrs}/ and <scenario>.diff. Exit 1 when a scenario differs beyond KNOWN.
set -uo pipefail
# Windows-form paths under Git Bash: they end up in Maven's JVM.
root="$(cd "$(dirname "$0")/../.." && (pwd -W 2> /dev/null || pwd))"
here="$root/tools/spotless-maven"
out="$root/target/spotless-maven"
exe="$root/target/debug/ktrs$([[ $OSTYPE == msys* || $OSTYPE == cygwin* ]] && echo .exe)"
corpus="${CORPUS:-$root/corpus}"
m2="${M2:-$out/m2}"
known="${KNOWN:-$root/tools/parity/known-diffs/spotless-maven.tsv}"
status=0
[[ -n ${JAVA_HOME:-} ]] || export JAVA_HOME="$(ls -d "$root"/tools/jdk/* 2>/dev/null | head -1)"
[[ -n $JAVA_HOME ]] || { echo "set JAVA_HOME (or run tools/ensure-jdk.sh)" >&2; exit 2; }
[[ -x $exe ]] || { echo "no $exe: run cargo build --bins" >&2; exit 2; }
mvn="$("$here/ensure-maven.sh")" || exit 2
ktrs_version=0.0.0-parity
ktfmt_stock=${KTFMT_STOCK:-0.64}
ktfmt_jar=$(ls "$root"/tools/ktfmt-oracle/lib/ktfmt-*-with-dependencies.jar 2>/dev/null | head -1)
export MAVEN_OPTS="-Dktrs.executable=$exe"

# ktfmt_reference <scenario> <dir>: the scenario's sources formatted by the ktfmt jar's CLI with the options of its
# <ktfmt> configuration (those without a flag through an .editorconfig).
ktfmt_reference() {
  local flags ec=""
  case $1 in
    ktfmt-meta|real-ktfmt) flags=--meta-style ;;
    ktfmt-google) flags=--google-style ;;
    ktfmt-kotlinlang) flags=--kotlinlang-style ec='max_line_length = 80' ;;
    ktfmt-options) flags="--meta-style --do-not-remove-unused-imports"
      ec=$'max_line_length = 120\nindent_size = 4\nij_continuation_indent_size = 4\nktfmt_trailing_comma_management_strategy = only_add' ;;
    ktfmt-commas) flags=--kotlinlang-style ec='ktfmt_trailing_comma_management_strategy = none' ;;
  esac
  if [[ -z $ktfmt_jar ]]; then
    "$root/tools/sync-ktfmt.sh" >&2 || exit 2
    ktfmt_jar=$(ls "$root"/tools/ktfmt-oracle/lib/ktfmt-*-with-dependencies.jar | head -1)
  fi
  rm -rf "$2"; mkdir -p "$2"
  sources "$1" "$2"
  if [[ -n $ec ]]; then
    printf 'root = true\n[*.kt]\n%s\n' "$ec" > "$2/.editorconfig"
    flags+=" --enable-editorconfig"
  fi
  # Until nothing changes: Spotless re-applies a step whose output isn't a fixed point (its PaddedCell), and ktfmt
  # has such inputs (okhttp's HttpUrl.kt: a KDoc table after a list item).
  local pass
  for pass in 1 2 3 4 5; do
    # shellcheck disable=SC2086
    (cd "$2" && "$JAVA_HOME/bin/java" -Xss64m -jar "$ktfmt_jar" $flags --quiet --set-exit-if-changed src) \
      >> "$2/../ktfmt.txt" 2>&1 && break
  done
  return 0
}

# The ktrs jar as built here (no bundled binary: -Dktrs.executable), installed without dependencies, as published.
install_ktrs() {
  "$root/java/gradlew" -p "$root/java" --console=plain -q jar || exit 2
  local jar; jar="$(ls "$root"/java/build/libs/ktrs-*.jar | grep -v -e -sources -e -javadoc | head -1)"
  "$mvn" -B -q -ntp -Dmaven.repo.local="$m2" install:install-file -Dfile="$jar" -DgroupId=io.github.hexay \
    -DartifactId=ktrs -Dversion="$ktrs_version" -Dpackaging=jar -DgeneratePom=true || exit 2
}

# kotlin <scenario>: the stock <kotlin> configuration; the ktrs side adds implementation= to its <ktfmt>/<ktlint>.
kotlin() {
  local inc="<includes><include>src/**/*.kt</include></includes>"
  case $1 in
    ktfmt-meta|real-ktfmt) echo "<kotlin>$inc<ktfmt><version>$ktfmt_stock</version></ktfmt></kotlin>" ;;
    ktfmt-google) echo "<kotlin>$inc<ktfmt><version>$ktfmt_stock</version><style>GOOGLE</style></ktfmt></kotlin>" ;;
    ktfmt-kotlinlang) echo "<kotlin>$inc<ktfmt><style>KOTLINLANG</style><maxWidth>80</maxWidth></ktfmt></kotlin>" ;;
    ktfmt-options) echo "<kotlin>$inc<ktfmt><style>META</style><maxWidth>120</maxWidth><blockIndent>4</blockIndent>
      <continuationIndent>4</continuationIndent><removeUnusedImports>false</removeUnusedImports>
      <trailingCommaManagementStrategy>ONLY_ADD</trailingCommaManagementStrategy></ktfmt></kotlin>" ;;
    ktfmt-commas) echo "<kotlin>$inc<ktfmt><style>KOTLINLANG</style>
      <trailingCommaManagementStrategy>NONE</trailingCommaManagementStrategy></ktfmt></kotlin>" ;;
    ktlint-default|real-ktlint) echo "<kotlin>$inc<ktlint><version>1.8.0</version></ktlint></kotlin>" ;;
    ktlint-override) echo "<kotlin>$inc<ktlint><editorConfigOverride><max_line_length>80</max_line_length>
      <ktlint_code_style>ktlint_official</ktlint_code_style></editorConfigOverride></ktlint></kotlin>" ;;
    ktlint-editorconfig) echo "<kotlin>$inc<ktlint><editorConfigPath>config/ktlint.editorconfig</editorConfigPath>
      </ktlint></kotlin>" ;;
    ktlint-compose|real-compose) echo "<kotlin>$inc<ktlint><customRuleSets>
      <value>io.nlopez.compose.rules:ktlint:0.6.7</value></customRuleSets></ktlint></kotlin>" ;;
    *) echo "unknown scenario $1" >&2; return 1 ;;
  esac
}

# sources <scenario> <dir>: the project's sources (and .editorconfig files).
sources() {
  case $1 in
    real-ktfmt|real-ktlint)
      mkdir -p "$2/src/main/kotlin"; cp -r "$corpus/okhttp/okhttp/src/commonJvmAndroid/kotlin/." "$2/src/main/kotlin/"
      cp "$corpus/okhttp/.editorconfig" "$2/" ;;
    real-compose)
      mkdir -p "$2/src/main/kotlin"; cp -r "$corpus/nowinandroid/core/designsystem/src/main/kotlin/." "$2/src/main/kotlin/"
      cp "$corpus/nowinandroid/.editorconfig" "$2/" ;;
    *)
      mkdir -p "$2/src/main/kotlin/demo"; cp "$here"/src/*.kt "$2/src/main/kotlin/demo/"
      cp "$root"/tools/ktlint-gradle/sample/src/main/kotlin/com/example/*.kt "$root/tools/ktlint-gradle/Ui.kt" \
        "$2/src/main/kotlin/demo/"
      [[ $1 == ktlint-editorconfig ]] && mkdir -p "$2/config" &&
        printf '[*.{kt,kts}]\nindent_size = 2\nktlint_standard_no-wildcard-imports = disabled\n' > "$2/config/ktlint.editorconfig"
      [[ $1 == ktlint-default ]] && printf 'root = true\n[*.kt]\nmax_line_length = 60\n' > "$2/.editorconfig" ;;
  esac
  return 0
}

py_() { if command -v py > /dev/null; then py -3 "$@"; else python3 "$@"; fi; }

# Spotless's messages, without paths to this side's project and Maven's timing.
messages() {
  grep -aE '^\[(ERROR|WARNING)\]|Spotless|BUILD' "$1" | grep -vE 'Total time|Finished at' | sed 's#\\#/#g' |
    sed "s#$2#<project>#g"
}

scenario() {
  local name=$1 side config dir pom
  kotlin "$name" > /dev/null || return
  pom="$(< "$here/pom.xml")"
  for side in stock ktrs; do
    config="$(kotlin "$name")"
    [[ $side == ktrs ]] && config="${config//<version>$ktfmt_stock<\/version><\/ktfmt>/</ktfmt>}" &&
      config="${config//<version>$ktfmt_stock<\/version><style>/<style>}" &&
      config="${config//<ktfmt>/<ktfmt implementation=\"io.github.hexay.ktrs.spotless.maven.KtrsKtfmt\">}" &&
      config="${config//<ktlint>/<ktlint implementation=\"io.github.hexay.ktrs.spotless.maven.KtrsKtlint\">}"
    dir="$out/$name/$side/project"
    rm -rf "$out/$name/$side"; mkdir -p "$dir"
    sources "$name" "$dir"
    pom="${pom//@KTRS_VERSION@/$ktrs_version}"
    echo "${pom//@KOTLIN@/$config}" > "$dir/pom.xml"
    for run in 1-check 2-apply 3-check; do
      # Spotless resolves the default ./.editorconfig against the working directory: run from the project, as users do.
      (cd "$dir" && "$mvn" -B -ntp -Dmaven.repo.local="$m2" "spotless:${run#*-}") > "$out/$name/$side/$run.txt" 2>&1
      { echo "== exit $? : $run"; messages "$out/$name/$side/$run.txt" "$dir"; } >> "$out/$name/$side/console.txt"
    done
  done
  local reference="$out/$name/stock/project/src" upgrade=""
  if [[ $name == *ktfmt* ]]; then
    ktfmt_reference "$name" "$out/$name/reference/project"
    reference="$out/$name/reference/project/src"
    upgrade=", $(diff -rq "$out/$name/stock/project/src" "$reference" | wc -l) files differ from stock ktfmt $ktfmt_stock"
  fi
  { py_ "$root/tools/parity/known_diffs.py" "$known" "$name" console.txt "$out/$name/stock/console.txt" \
      "$out/$name/ktrs/console.txt"
    diff -r "$reference" "$out/$name/ktrs/project/src"; } > "$out/$name.diff"
  local files changed
  files=$(find "$out/$name/stock/project/src" -name '*.kt' | wc -l)
  changed=$(grep -c '^diff ' "$out/$name.diff")
  [[ -s $out/$name.diff ]] && status=1
  echo "$name: $(grep '^== exit' "$out/$name/stock/console.txt" | cut -d' ' -f3 | tr '\n' ' ')(stock exits)," \
    "$files files, $changed differing applied files, $(wc -l < "$out/$name.diff") diff lines$upgrade"
}

all="ktfmt-meta ktfmt-google ktfmt-kotlinlang ktfmt-options ktfmt-commas ktlint-default ktlint-override
  ktlint-editorconfig ktlint-compose real-ktfmt real-ktlint real-compose"
mkdir -p "$out"
install_ktrs
for s in ${*:-$all}; do
  if [[ $s == real-* && ! -d $corpus/okhttp ]]; then echo "$s: skipped (no $corpus)"; continue; fi
  scenario "$s"
done
exit $status
