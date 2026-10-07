#!/usr/bin/env bash
# Fetches the pinned Maven into tools/maven (gitignored) and prints its `mvn` path.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
version=3.9.16
sha512=831a8591fe20c8243b1dbe7d71e3244f31d1665b0804b2e825e38cbbe5ce0cafb8338851f90780735568773e0a6cd07bbec107cda0b896b008b861075358b6f6
home="$root/tools/maven/apache-maven-$version"
if [[ ! -x $home/bin/mvn ]]; then
  mkdir -p "$root/tools/maven"
  tmp="$root/tools/maven/maven.tar.gz"
  curl -sfL -o "$tmp" "https://archive.apache.org/dist/maven/maven-3/$version/binaries/apache-maven-$version-bin.tar.gz"
  echo "$sha512  $tmp" | sha512sum -c --quiet - >&2
  tar -xzf "$tmp" -C "$root/tools/maven"
  rm "$tmp"
fi
echo "$home/bin/mvn"
