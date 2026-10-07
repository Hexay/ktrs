# Sourced by the package-manifest generators; needs $tag and $sums (the release's SHA256SUMS).
base="https://github.com/Hexay/ktrs/releases/download/$tag"

asset_name() { [[ $1 == *windows* ]] && echo "ktrs-$tag-$1.zip" || echo "ktrs-$tag-$1.tar.gz"; }

asset_url() { echo "$base/$(asset_name "$1")"; }

asset_sha() {
  local name sum
  name=$(asset_name "$1")
  sum=$(awk -v n="$name" '$2 == n { print $1 }' "$sums")
  [[ -n $sum ]] || { echo "no checksum for $name in $sums" >&2; exit 1; }
  echo "$sum"
}
