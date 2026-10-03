#!/usr/bin/env bash
# One coverage-guided fuzzing session (research/24-fuzzing.md): the fuzz/ targets in libFuzzer fork mode, all at
# once, for SECONDS. The first run seeds fuzz/corpus/<target> from testdata/; crashes and timeouts land in
# fuzz/artifacts/<target>/ (fork mode keeps going after one). Linux, nightly + cargo-fuzz.
#
#   tools/fuzz/fuzz.sh SECONDS [target:jobs ...]       default: parser:2 ktfmt:4 ktlint:4
#
# Env: MAX_LEN = libFuzzer -max_len (default 8192); KTRS_FUZZ_KNOWN = panic locations to tolerate (see
# fuzz/src/lib.rs). Logs: fuzz/logs/<target>.log.
set -euo pipefail
(($# >= 1)) || { sed -n 2,8p "$0"; exit 2; }
repo="$(cd "$(dirname "$0")/../.." && pwd)"
secs=$1; shift
specs=("$@"); ((${#specs[@]})) || specs=(parser:2 ktfmt:4 ktlint:4)
fuzz=$repo/fuzz
mkdir -p "$fuzz/logs"
# No ASan: safe Rust gains little from it at a ~2x cost. An explicit target: a cargo-fuzz built for musl defaults to it.
opts=(-O -s none --target "$(rustc +nightly -vV | sed -n 's/^host: //p')")

seed() { # target
  local dir=$fuzz/corpus/$1
  [[ -d $dir ]] && return
  mkdir -p "$dir"
  find "$repo/testdata" -type f \( -name '*.kt' -o -name '*.kts' \) -size -8k -print0 |
    while IFS= read -r -d '' f; do cp "$f" "$dir/seed-$(echo "${f#"$repo"/testdata/}" | tr '/' '_')"; done
}

for spec in "${specs[@]}"; do
  target=${spec%%:*}
  seed "$target"
  (cd "$fuzz" && cargo +nightly fuzz build "${opts[@]}" "$target")
done
for spec in "${specs[@]}"; do
  target=${spec%%:*} jobs=${spec##*:}
  (cd "$fuzz" && cargo +nightly fuzz run "${opts[@]}" "$target" -- -fork="$jobs" -ignore_crashes=1 -max_total_time="$secs" \
    -max_len="${MAX_LEN:-8192}" -timeout=10 -rss_limit_mb=4096 -print_final_stats=1) > "$fuzz/logs/$target.log" 2>&1 &
done
# libFuzzer's timeout handler can deadlock on the allocator lock (a hung fork job keeps the session alive forever).
(sleep $((secs + 180)); pkill -9 -f -- "-artifact_prefix=$fuzz/artifacts/") & watchdog=$!
wait $(jobs -p | grep -vx "$watchdog")
kill "$watchdog" 2> /dev/null
for spec in "${specs[@]}"; do
  target=${spec%%:*}
  echo "$target: $(grep -a '^#[0-9]' "$fuzz/logs/$target.log" | tail -1 | cut -d' ' -f1-9)," \
    "corpus $(find "$fuzz/corpus/$target" -type f | wc -l), artifacts $(find "$fuzz/artifacts/$target" -type f 2>/dev/null | wc -l)"
done
