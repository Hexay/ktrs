#!/usr/bin/env bash
# Checks out the pinned ktfmt and the google-java-format it builds on (third_party/, gitignored),
# and fetches the ktfmt release jar used as the formatting oracle. Keep in sync with tools/ktfmt-oracle.sh.
set -euo pipefail
KTFMT_TAG=v0.65
GJF_TAG=v1.23.0
root="$(cd "$(dirname "$0")/.." && pwd)"

clone() {
  local repo=$1 tag=$2 dir=$3
  [[ -d $dir/.git ]] || git clone -q --depth 1 --branch "$tag" "https://github.com/$repo.git" "$dir"
}
clone Kotlin/ktfmt "$KTFMT_TAG" "$root/third_party/ktfmt"
clone google/google-java-format "$GJF_TAG" "$root/third_party/google-java-format"

jar="$root/tools/ktfmt-oracle/lib/ktfmt-${KTFMT_TAG#v}-with-dependencies.jar"
mkdir -p "$(dirname "$jar")"
[[ -f $jar ]] || curl -sfL -o "$jar" \
  "https://repo1.maven.org/maven2/org/jetbrains/kotlinx/ktfmt/${KTFMT_TAG#v}/ktfmt-${KTFMT_TAG#v}-with-dependencies.jar"

"$root/tools/ensure-jdk.sh"
echo "ktfmt $KTFMT_TAG, google-java-format $GJF_TAG, oracle jar $(du -h "$jar" | cut -f1)"
