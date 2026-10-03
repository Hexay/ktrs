#!/usr/bin/env bash
# Fetches each repo in a REVISIONS file (`owner/repo sha` lines) at its pinned commit into DEST/src, then copies only
# the .kt/.kts files into DEST/tree (so no project .editorconfig overrides the style a comparison sets at the root).
# Default: the held-out corpus, repos never used while porting or fixing (research/21-holdout.md). Linux and macOS.
#
#   tools/holdout/fetch.sh [DEST] [REVISIONS]   DEST defaults to corpus-holdout/ (gitignored), REVISIONS to ./REVISIONS
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
dest="${1:-$here/../../corpus-holdout}"
revisions="${2:-$here/REVISIONS}"
mkdir -p "$dest/src"
while read -r repo sha; do
  dir="$dest/src/${repo#*/}"
  if [[ ! -d $dir/.git ]]; then
    git init -q "$dir"
    git -C "$dir" fetch -q --depth 1 "https://github.com/$repo.git" "$sha"
    git -C "$dir" -c advice.detachedHead=false checkout -q FETCH_HEAD
  fi
done < "$revisions"
rm -rf "$dest/tree"; mkdir -p "$dest/tree"
# tar, not GNU `cp --parents`: macOS has no such flag.
(cd "$dest/src" && find . \( -name .git -prune \) -o -type f \( -name '*.kt' -o -name '*.kts' \) -print \
  | tar -cf - -T - | tar -xf - -C "$dest/tree")
echo "$(find "$dest/tree" -type f | wc -l | tr -d ' ') files, $(du -sh "$dest/tree" | cut -f1) in $dest/tree"
