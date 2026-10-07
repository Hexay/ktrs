#!/usr/bin/env bash
# Fetches a portable JDK 21 into tools/jdk (gitignored) unless the system java is 17+.
# The oracles need a full JDK (javac, jdk.compiler), not a JRE. Run by the tools/sync-*.sh scripts.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
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
