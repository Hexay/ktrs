#!/usr/bin/env bash
# Predicate for `kgen reduce` on a fuzz artifact (research/24-fuzzing.md): exits 0 iff the fuzz target still fails
# on FILE with PATTERN (fixed string) in its output, e.g. the panic location.
#
#   tools/fuzz/crashes.sh parser|ktfmt|ktlint PATTERN FILE
set -u
(($# == 3)) || { sed -n 2,5p "$0"; exit 2; }
repo="$(cd "$(dirname "$0")/../.." && pwd)"
bin=$repo/fuzz/target/$(rustc +nightly -vV | sed -n 's/^host: //p')/release/$1
out=$(timeout "${TMO:-60}" "$bin" -rss_limit_mb="${RSS_MB:-2048}" "$3" 2>&1)
grep -aqF -- "$2" <<< "$out"
