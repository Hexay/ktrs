#!/usr/bin/env bash
# Held-out corpus: repos never used while porting or fixing (research/21-holdout.md). Fetches each at the commit
# pinned in REVISIONS into DEST/src, then copies only the .kt/.kts files into DEST/tree (so no project
# .editorconfig overrides the style a comparison sets at the root).
#
#   tools/holdout/fetch.sh [DEST]      DEST defaults to corpus-holdout/ (gitignored)
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
dest="${1:-$here/../../corpus-holdout}"
mkdir -p "$dest/src"
while read -r repo sha; do
  dir="$dest/src/${repo#*/}"
  if [[ ! -d $dir/.git ]]; then
    git init -q "$dir"
    git -C "$dir" fetch -q --depth 1 "https://github.com/$repo.git" "$sha"
    git -C "$dir" -c advice.detachedHead=false checkout -q FETCH_HEAD
  fi
done < "$here/REVISIONS"
rm -rf "$dest/tree"; mkdir -p "$dest/tree"
(cd "$dest/src" && find . \( -name .git -prune \) -o -type f \( -name '*.kt' -o -name '*.kts' \) \
  -exec cp --parents -t "$dest/tree" {} +)
echo "$(find "$dest/tree" -type f | wc -l) files, $(du -sh "$dest/tree" | cut -f1) in $dest/tree"
