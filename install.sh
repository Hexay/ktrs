#!/bin/sh
# Installs the latest (or $KTRS_VERSION) `ktrs` and `ktfmt` release binaries:
#   curl -fsSL https://raw.githubusercontent.com/Hexay/ktrs/master/install.sh | sh
# Env: KTRS_VERSION (tag, e.g. v0.1.0), KTRS_INSTALL_DIR (default ~/.local/bin), KTRS_REPO.
set -eu
repo=${KTRS_REPO:-Hexay/ktrs}
dir=${KTRS_INSTALL_DIR:-$HOME/.local/bin}

case "$(uname -s)" in
  Linux) os=unknown-linux-musl ;;
  Darwin) os=apple-darwin ;;
  *) echo "unsupported OS $(uname -s); on Windows download the zip from https://github.com/$repo/releases" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64 | amd64) arch=x86_64 ;;
  arm64 | aarch64) arch=aarch64 ;;
  *) echo "unsupported CPU $(uname -m)" >&2; exit 1 ;;
esac

version=${KTRS_VERSION:-$(curl -fsSL "https://api.github.com/repos/$repo/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p')}
[ -n "$version" ] || { echo "no release found for $repo" >&2; exit 1; }
name="ktrs-$version-$arch-$os"
base="https://github.com/$repo/releases/download/$version"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
curl -fsSL "$base/$name.tar.gz" -o "$tmp/$name.tar.gz"
curl -fsSL "$base/SHA256SUMS" -o "$tmp/SHA256SUMS"
expected=$(grep " $name.tar.gz\$" "$tmp/SHA256SUMS" | cut -d' ' -f1)
if command -v sha256sum > /dev/null; then actual=$(sha256sum "$tmp/$name.tar.gz" | cut -d' ' -f1)
else actual=$(shasum -a 256 "$tmp/$name.tar.gz" | cut -d' ' -f1); fi
[ -n "$expected" ] && [ "$expected" = "$actual" ] || { echo "checksum mismatch for $name.tar.gz" >&2; exit 1; }

tar xzf "$tmp/$name.tar.gz" -C "$tmp"
mkdir -p "$dir"
cp "$tmp/$name/ktrs" "$tmp/$name/ktfmt" "$dir/"
echo "installed ktrs and ktfmt $version to $dir"
case ":$PATH:" in *":$dir:"*) ;; *) echo "note: $dir is not on your PATH" ;; esac
