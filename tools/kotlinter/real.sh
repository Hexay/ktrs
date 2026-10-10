#!/usr/bin/env bash
# The kotlinter drop-in vs kotlinter 5.7.0 on real projects (tools/kotlinter/REVISIONS), research/34. The ktrs side is
# the project after `ktrs migrate --write`, with the plugin taken from a local Maven repository the script publishes
# to (as a release's users get it); the upstream side is the project with kotlinter forced to 5.7.0. Both run the
# project's own Gradle wrapper: `lintKotlin --continue`, `formatKotlin --continue`, `tasks --all`.
#   tools/kotlinter/real.sh [name ...]   (default: every project; needs `cargo build --bins`, network, the projects' JDKs;
#   Android builds need ANDROID_HOME)
# Output: target/kotlinter-real/<name>/{upstream,ktrs}/, <name>/migrate.txt (the migration's diff and notes) and
# <name>.diff (tools/ktlint-gradle/compare.py). Accepted differences: KNOWN (default
# tools/parity/known-diffs/kotlinter.tsv, scenario = the project's name). Exit 1 when a project differs beyond them.
set -uo pipefail
root="$(cd "$(dirname "$0")/../.." && (pwd -W 2> /dev/null || pwd))"
here="$root/tools/kotlinter"
shared="$root/tools/ktlint-gradle"
out="$root/target/kotlinter-real"
exe="$root/target/debug/ktrs$([[ $OSTYPE == msys* || $OSTYPE == cygwin* ]] && echo .exe)"
[[ -n ${JAVA_HOME:-} ]] || export JAVA_HOME="$(ls -d "$root"/tools/jdk/* 2>/dev/null | head -1)"
[[ -n $JAVA_HOME ]] || { echo "set JAVA_HOME (or run tools/ensure-jdk.sh)" >&2; exit 2; }
known="${KNOWN:-$root/tools/parity/known-diffs/kotlinter.tsv}"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -1)"
repo="$out/repo"
init="$out/init.gradle"
status=0

py_() { if command -v py > /dev/null; then py -3 "$@"; else python3 "$@"; fi; }

mkdir -p "$out"
"$root/java/gradlew" -p "$root/java" --console=plain -q -PktrsVersion="$version" -PpagesRepo="$repo" \
  :publishAllPublicationsToGithubPagesRepository :ktrs-gradle-plugin:publishAllPublicationsToGithubPagesRepository \
  > "$out/publish.txt" 2>&1 || { echo "publishing the plugin failed: $out/publish.txt" >&2; exit 2; }

# The plugins' versions for ids applied without one (builds that got kotlinter from an included build or a parent).
cat > "$init" << EOF
beforeSettings { settings ->
    settings.pluginManagement {
        repositories { maven { url = uri('$repo') }; gradlePluginPortal(); mavenCentral(); google() }
        resolutionStrategy.eachPlugin {
            if (requested.id.id == 'org.jmailen.kotlinter') useVersion('5.7.0')
            if (requested.id.id.startsWith('io.github.hexay.ktrs')) useVersion('$version')
        }
    }
}
EOF

# run <name> <repo> <commit> [build dir]
run() {
  local name=$1 dir side project gradle edited step
  for side in upstream ktrs; do
    dir="$out/$name/$side"
    rm -rf "$dir"; mkdir -p "$dir/checkout"
    git -C "$dir/checkout" init -q
    git -C "$dir/checkout" fetch -q --depth 1 "https://github.com/$2" "$3" && git -C "$dir/checkout" checkout -q FETCH_HEAD ||
      { echo "$name: fetching $2 failed" >&2; status=1; return; }
    # compare.py reads <side>/project.
    mv "$dir/checkout/${4:-.}" "$dir/project" 2> /dev/null || mv "$dir/checkout" "$dir/project"
    project="$dir/project"
    # kotlinter's own test projects take the plugin from the repository around them.
    sed -i '/includeBuild("..")/d' "$project"/settings.gradle* 2> /dev/null
    git -C "$project" init -q 2> /dev/null; git -C "$project" add -A > /dev/null 2>&1
    if [[ $side == ktrs ]]; then
      (cd "$project" && "$exe" migrate --write) > "$out/$name/migrate.txt" 2>&1 < /dev/null
      (cd "$project" && git diff) >> "$out/$name/migrate.txt"
    fi
    edited="$(cd "$project" && git diff --name-only)"
    gradle="$project/gradlew"; [[ -x $gradle ]] || gradle="$root/java/gradlew"
    for step in "lintKotlin --continue" "formatKotlin --continue" "tasks --all"; do
      # shellcheck disable=SC2086
      "$gradle" -p "$project" --console=plain --init-script "$init" -Pktrs.executable="$exe" -Dorg.gradle.welcome=never $step \
        > "$dir/run.txt" 2>&1 < /dev/null
      { echo "== exit $? : $step"; cat "$dir/run.txt"; } >> "$dir/console.txt"
      grep -Eq "^> Task :" "$dir/run.txt" || { echo "$name/$side: gradle did not run ($step)" >&2; status=1; }
    done
    # The migration's own edits are not a difference of the run.
    # shellcheck disable=SC2086
    [[ -z $edited ]] || (cd "$project" && git checkout -q -- $edited)
    rm -rf "$project/.git"
  done
  py_ "$shared/compare.py" "$out/$name" --plugin-ids io.github.hexay.ktrs.kotlinter org.jmailen.kotlinter --known "$known" \
    > "$out/$name.diff" || status=1
  echo "$name: $(tail -1 "$out/$name.diff") ($(wc -l < "$out/$name.diff") diff lines); $(py_ "$shared/rows.py" "$out/$name" | head -1)"
}

while read -r name repository commit build; do
  [[ -z $name || $name == \#* ]] && continue
  [[ $# -eq 0 || " $* " == *" $name "* ]] || continue
  run "$name" "$repository" "$commit" "$build"
done < "$here/REVISIONS"
exit $status
