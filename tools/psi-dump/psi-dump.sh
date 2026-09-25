#!/usr/bin/env bash
# Build-on-demand wrapper for the reference PSI dumper. Usage: tools/psi-dump/psi-dump.sh one|tree|kinds ...
set -euo pipefail
KOTLIN_VERSION=2.4.20
COROUTINES_VERSION=1.8.0
here="$(cd "$(dirname "$0")" && pwd)"
lib="$here/lib"
classes="$here/build"
maven=https://repo1.maven.org/maven2

fetch() {
  local path=$1 jar="$lib/$(basename "$1").jar"
  [[ -f $jar ]] || curl -sfL -o "$jar" "$maven/$path.jar"
}

mkdir -p "$lib" "$classes"
fetch "org/jetbrains/kotlin/kotlin-compiler-embeddable/$KOTLIN_VERSION/kotlin-compiler-embeddable-$KOTLIN_VERSION"
fetch "org/jetbrains/kotlin/kotlin-stdlib/$KOTLIN_VERSION/kotlin-stdlib-$KOTLIN_VERSION"
fetch "org/jetbrains/kotlin/kotlin-script-runtime/$KOTLIN_VERSION/kotlin-script-runtime-$KOTLIN_VERSION"
fetch "org/jetbrains/kotlin/kotlin-daemon-embeddable/$KOTLIN_VERSION/kotlin-daemon-embeddable-$KOTLIN_VERSION"
fetch "org/jetbrains/kotlinx/kotlinx-coroutines-core-jvm/$COROUTINES_VERSION/kotlinx-coroutines-core-jvm-$COROUTINES_VERSION"

sep=:
cp=$(printf "%s$sep" "$lib"/*.jar)
# Windows JVMs can't read MSYS paths inside a classpath list.
if command -v cygpath >/dev/null; then
  sep=';'; cp=$(cygpath -mp "$cp"); classes=$(cygpath -m "$classes")
fi
if [[ ! -f $classes/PsiDump.class || $here/src/PsiDump.java -nt $classes/PsiDump.class ]]; then
  javac -nowarn -d "$classes" -cp "$cp" "$here/src/PsiDump.java"
fi
exec java -Xss64m -cp "$classes$sep$cp" PsiDump "$@"
