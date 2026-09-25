#!/usr/bin/env bash
# Shallow-clones real-world Kotlin repos into corpus/ (gitignored) for `cargo xtask corpus-diff`.
# Records the exact commits in corpus/REVISIONS so a diff run can be reproduced.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
corpus="$root/corpus"
repos=(
  square/okhttp
  Kotlin/kotlinx.coroutines
  android/nowinandroid
  pinterest/ktlint
  facebook/ktfmt
  JetBrains/Exposed
  ktorio/ktor
)
mkdir -p "$corpus"
for repo in "${repos[@]}"; do
  dir="$corpus/${repo#*/}"
  [[ -d $dir/.git ]] || git clone -q --depth 1 "https://github.com/$repo.git" "$dir"
done
for repo in "${repos[@]}"; do
  echo "$repo $(git -C "$corpus/${repo#*/}" rev-parse HEAD)"
done > "$corpus/REVISIONS"
cat "$corpus/REVISIONS"
