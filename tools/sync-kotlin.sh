#!/usr/bin/env bash
# Sparse-checkout the pinned Kotlin sources we port from (third_party/kotlin, gitignored) and
# vendor the parser fixtures into testdata/kotlin/. Keep KOTLIN_TAG in sync with tools/psi-dump/psi-dump.sh.
set -euo pipefail
export MSYS_NO_PATHCONV=1
KOTLIN_TAG=v2.4.20
# pwd -W: native Windows path, since MSYS_NO_PATHCONV stops git.exe from translating /c/... paths.
root="$(cd "$(dirname "$0")/.." && (pwd -W 2>/dev/null || pwd))"
src="$root/third_party/kotlin"

if [[ ! -d $src/.git ]]; then
  git clone -q --filter=blob:none --no-checkout --depth 1 --branch "$KOTLIN_TAG" \
    https://github.com/JetBrains/kotlin.git "$src"
fi
git -C "$src" config core.longpaths true
git -C "$src" sparse-checkout set --no-cone \
  '/compiler/psi/parser/' \
  '/compiler/psi/psi-api/src/' \
  '/compiler/psi/psi-impl/src/' \
  '/compiler/psi/psi-impl/testData/psi/' \
  '/compiler/psi/psi-impl/testData/lexer/' \
  '/compiler/psi/psi-impl/testFixtures/'
git -C "$src" checkout -q "$KOTLIN_TAG"

# Only the PSI parse dumps (<name>.kt + <name>.txt); stub/decompiled dumps are out of scope.
dest="$root/testdata/kotlin"
rm -rf "$dest"
for kind in psi lexer; do
  from="$src/compiler/psi/psi-impl/testData/$kind"
  (cd "$from" && find . -type f \( -name '*.kt' -o -name '*.kts' -o -name '*.txt' \) \
      ! -name '*.stubs.txt' ! -name '*.decompiledText.txt' -print0) |
    while IFS= read -r -d '' f; do
      mkdir -p "$dest/$kind/$(dirname "$f")"
      cp "$from/$f" "$dest/$kind/$f"
    done
done
cp "$src/license/LICENSE.txt" "$dest/LICENSE.txt" 2>/dev/null || true
echo "synced $(find "$dest" -type f | wc -l) fixture files from Kotlin $KOTLIN_TAG"
