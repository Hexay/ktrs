#!/bin/sh
# Installs the latest (or $KTRS_VERSION) `ktrs` and `ktfmt` release binaries (Linux, macOS, and
# Windows under Git Bash/MSYS2); also the body of the GitHub Action (action.yml):
#   curl -fsSL https://raw.githubusercontent.com/Hexay/ktrs/master/install.sh | sh
# Env: KTRS_VERSION (tag, e.g. v0.2.0), KTRS_INSTALL_DIR (default ~/.local/bin), KTRS_ARCH
# (x86_64 or aarch64; default: this machine's), KTRS_REPO.
set -eu
repo=${KTRS_REPO:-Hexay/ktrs}
dir=${KTRS_INSTALL_DIR:-$HOME/.local/bin}

exe="" ext=tar.gz
case "$(uname -s)" in
  Linux) os=unknown-linux-musl ;;
  Darwin) os=apple-darwin ;;
  MINGW* | MSYS* | CYGWIN*) os=pc-windows-msvc exe=.exe ext=zip ;;
  *) echo "unsupported OS $(uname -s); download a release from https://github.com/$repo/releases" >&2; exit 1 ;;
esac
case "${KTRS_ARCH:-$(uname -m)}" in
  x86_64 | amd64 | X64) arch=x86_64 ;;
  arm64 | aarch64 | ARM64) arch=aarch64 ;;
  *) echo "unsupported CPU ${KTRS_ARCH:-$(uname -m)}" >&2; exit 1 ;;
esac

# /releases/latest redirects to /releases/tag/<tag>; unlike the API it isn't rate limited.
version=${KTRS_VERSION:-$(curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/$repo/releases/latest" | sed -n 's|.*/tag/||p')}
[ -n "$version" ] || { echo "no release found for $repo" >&2; exit 1; }
name="ktrs-$version-$arch-$os"
base="https://github.com/$repo/releases/download/$version"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
curl -fsSL "$base/$name.$ext" -o "$tmp/$name.$ext"
curl -fsSL "$base/SHA256SUMS" -o "$tmp/SHA256SUMS"
expected=$(grep " $name.$ext\$" "$tmp/SHA256SUMS" | cut -d' ' -f1)
if command -v sha256sum > /dev/null; then actual=$(sha256sum "$tmp/$name.$ext" | cut -d' ' -f1)
else actual=$(shasum -a 256 "$tmp/$name.$ext" | cut -d' ' -f1); fi
[ -n "$expected" ] && [ "$expected" = "$actual" ] || { echo "checksum mismatch for $name.$ext" >&2; exit 1; }

if [ "$ext" = zip ]; then unzip -q "$tmp/$name.zip" -d "$tmp"; else tar xzf "$tmp/$name.tar.gz" -C "$tmp"; fi
mkdir -p "$dir"
cp "$tmp/$name/ktrs$exe" "$tmp/$name/ktfmt$exe" "$dir/"
echo "installed ktrs and ktfmt $version to $dir"
case ":$PATH:" in *":$dir:"*) ;; *) echo "note: $dir is not on your PATH" ;; esac
