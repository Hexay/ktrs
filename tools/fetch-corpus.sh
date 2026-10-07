#!/usr/bin/env bash
# Fetches the real-world Kotlin repos pinned in tools/corpus/REVISIONS (`owner/repo sha` lines) into corpus/
# (gitignored) for the corpus gates, and records the commits actually checked out in corpus/REVISIONS.
# The pins also key the oracle caches of .github/workflows/parity.yml; bumping one means rebuilding every oracle.
#   tools/fetch-corpus.sh            existing checkouts are kept (a warning names any not at its pin)
#   tools/fetch-corpus.sh --latest   rewrites the pins to each repo's current default-branch HEAD (then delete corpus/)
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
corpus="$root/corpus"
pins="$root/tools/corpus/REVISIONS"
if [[ ${1:-} == --latest ]]; then
  while read -r repo _; do
    echo "$repo $(git ls-remote "https://github.com/$repo.git" HEAD | cut -f1)"
  done < "$pins" > "$pins.new"
  mv "$pins.new" "$pins"
  cat "$pins"
  exit
fi
mkdir -p "$corpus"
while read -r repo sha; do
  dir="$corpus/${repo#*/}"
  if [[ ! -d $dir/.git ]]; then
    git init -q "$dir"
    git -C "$dir" fetch -q --depth 1 "https://github.com/$repo.git" "$sha"
    git -C "$dir" -c advice.detachedHead=false checkout -q FETCH_HEAD
  fi
  at=$(git -C "$dir" rev-parse HEAD)
  [[ $at == "$sha" ]] || echo "warning: corpus/${repo#*/} is at $at, pinned $sha" >&2
  echo "$repo $at"
done < "$pins" > "$corpus/REVISIONS"
cat "$corpus/REVISIONS"
