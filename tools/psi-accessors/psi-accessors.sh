#!/usr/bin/env bash
# JVM oracle for crates/ktrs_psi (see src/PsiAccessors.java). Reuses the compiler jars fetched by psi-dump.sh.
# Usage: tools/psi-accessors/psi-accessors.sh one|hashes|dump ... [--fixture] [--script]
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
lib="$here/../psi-dump/lib"
classes="$here/build"
[[ -f $lib/kotlin-compiler-embeddable-2.4.20.jar ]] || "$here/../psi-dump/psi-dump.sh" kinds >/dev/null

mkdir -p "$classes"
sep=:
cp=$(printf "%s$sep" "$lib"/*.jar)
if command -v cygpath >/dev/null; then
  sep=';'; cp=$(cygpath -mp "$cp"); classes=$(cygpath -m "$classes")
fi
stamp="$classes/PsiAccessors.class"
if [[ ! -f $stamp ]] || [[ -n $(find "$here/src" -name '*.java' -newer "$stamp") ]]; then
  javac -nowarn -encoding UTF-8 -d "$classes" -cp "$cp" "$here"/src/*.java
fi
exec java -Xss64m -cp "$classes$sep$cp" PsiAccessors "$@"
