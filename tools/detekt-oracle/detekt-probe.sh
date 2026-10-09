#!/usr/bin/env bash
# The detekt jar's rows on a directory, for `cargo detekt-diff` (format: src/DetektProbe.kt). JVM, 1 to 3 minutes
# on the corpus: run in the background.
#   detekt-probe.sh <in-dir> <out-dir> [--all-rules] [--sequential] [--build-upon-default-config] [--config <yml>]...
# The jar reads the files as they are on disk, so <in-dir> must hold LF files (on Windows `cargo detekt-diff`
# stages an LF copy for the Rust side; run the oracle on Linux or on that copy).
set -euo pipefail
in=$1 out=$2
shift 2
here="$(cd "$(dirname "$0")" && pwd)"
tag=$(sed -n 's/^DETEKT_TAG=v//p' "$here/../sync-detekt.sh")
jar=${DETEKT_JAR:-$here/lib/detekt-cli-$tag-all.jar}
[[ -f $jar ]] || { echo "no $jar; run tools/sync-detekt.sh" >&2; exit 1; }
java=java
bundled=$(ls -d "$here"/../jdk/*/bin 2>/dev/null | head -1 || true)
if [[ -n $bundled ]]; then java="$bundled/java"; fi
classes="$here/build"
sep=:
host() { if command -v cygpath >/dev/null; then cygpath -m "$1"; else echo "$1"; fi; }
command -v cygpath >/dev/null && sep=';'

stamp="$classes/DetektProbeKt.class"
if [[ ! -f $stamp || -n $(find "$here/src" -newer "$stamp") ]]; then
  rm -rf "$classes"
  mkdir -p "$classes"
  srcs=()
  for f in "$here"/src/*.kt; do srcs+=("$(host "$f")"); done
  # The fat jar embeds the Kotlin compiler detekt runs on, so the probe compiles against exactly that.
  "$java" -cp "$(host "$jar")" org.jetbrains.kotlin.cli.jvm.K2JVMCompiler -no-stdlib -no-reflect -nowarn \
    -jvm-target 17 -cp "$(host "$jar")" -d "$(host "$classes")" "${srcs[@]}"
fi

mkdir -p "$out"
out=$(cd "$out" && pwd)
exec "$java" -Xss64m -Xmx12g -cp "$(host "$classes")$sep$(host "$jar")" DetektProbeKt "$(host "$in")" "$(host "$out")" "$@"
