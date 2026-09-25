#!/usr/bin/env bash
# Checks out the pinned ktfmt and the google-java-format it builds on (third_party/, gitignored),
# and fetches the ktfmt release jar used as the formatting oracle. Keep in sync with tools/ktfmt-oracle.sh.
set -euo pipefail
KTFMT_TAG=v0.64
GJF_TAG=v1.23.0
root="$(cd "$(dirname "$0")/.." && pwd)"

clone() {
  local repo=$1 tag=$2 dir=$3
  [[ -d $dir/.git ]] || git clone -q --depth 1 --branch "$tag" "https://github.com/$repo.git" "$dir"
}
clone facebook/ktfmt "$KTFMT_TAG" "$root/third_party/ktfmt"
clone google/google-java-format "$GJF_TAG" "$root/third_party/google-java-format"

jar="$root/tools/ktfmt-oracle/lib/ktfmt-${KTFMT_TAG#v}-with-dependencies.jar"
mkdir -p "$(dirname "$jar")"
[[ -f $jar ]] || curl -sfL -o "$jar" \
  "https://repo1.maven.org/maven2/com/facebook/ktfmt/${KTFMT_TAG#v}/ktfmt-${KTFMT_TAG#v}-with-dependencies.jar"

# ktfmt 0.64 needs Java 17+ and the jdk.compiler module (a JRE is not enough); fetch a portable
# JDK into tools/jdk (gitignored) if the system one is older.
major=$(java -version 2>&1 | grep -oE 'version "[0-9]+' | grep -oE '[0-9]+$' || echo 0)
if (( major < 17 )) && [[ ! -d $root/tools/jdk ]]; then
  case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) os=windows ext=zip ;; Darwin) os=mac ext=tar.gz ;; *) os=linux ext=tar.gz ;;
  esac
  arch=$(uname -m); [[ $arch == arm64 || $arch == aarch64 ]] && arch=aarch64 || arch=x64
  tmp="$root/tools/jdk.$ext"
  curl -sfL -o "$tmp" "https://api.adoptium.net/v3/binary/latest/21/ga/$os/$arch/jdk/hotspot/normal/eclipse"
  mkdir -p "$root/tools/jdk"
  if [[ $ext == zip ]]; then unzip -q "$tmp" -d "$root/tools/jdk"; else tar -xzf "$tmp" -C "$root/tools/jdk"; fi
  rm "$tmp"
fi
echo "ktfmt $KTFMT_TAG, google-java-format $GJF_TAG, oracle jar $(du -h "$jar" | cut -f1)"
