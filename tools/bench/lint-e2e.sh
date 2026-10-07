#!/usr/bin/env bash
# End to end: the ktrs `ktlint` drop-in vs the ktlint 2.0.0-ALPHA-4 and 1.8.0 CLIs and ktlint-rs, run the way
# users run them (same flags, same files, cwd = the project). Linux. Results: research/20-ktlint-bench.md.
#
#   tools/bench/lint-e2e.sh [--runs N] [--only TEXT] [--tools "ktrs kt2 ..."] [--timeout S] [--out DIR]
#
# Env (defaults = the testbox layout): CORPUS = a tree of .kt/.kts with exactly one .editorconfig, at its root
# (research/13 "Corpus and config"); KTRS = target/dist/ktlint; KTLINT2 / KTLINT18 = the self-executing release
# assets; KTLINT_RS = ktlint-rs binary; ONE_CPU = the CPU for the 1-core scenario.
# Each scenario: one untimed warm-up per tool, then N alternating rounds; medians of wall, user+sys CPU and max
# RSS. A tool whose warm-up exceeds SLOW seconds (or times out) is not repeated (n=1). After each scenario the
# last ktrs and kt2 runs are compared: exit code, sorted stdout/stderr and, for -F, the formatted tree.
set -uo pipefail

B=${B:-$HOME/work/ktlint-bench}
CORPUS=${CORPUS:-$B/corpus}
KTRS=${KTRS:-$HOME/work/ktrs-lbench/target/dist/ktlint}
KTLINT2=${KTLINT2:-$B/bin/ktlint-2.0.0-ALPHA-4}
KTLINT18=${KTLINT18:-$B/bin/ktlint-1.8.0}
KTLINT_RS=${KTLINT_RS:-$B/src/target/release/ktlint-rs}
ONE_CPU=${ONE_CPU:-0}
RUNS=5 ONLY= TMO=1200 SLOW=300 OUT=$B/out/e2e
TOOLS="ktrs kt2 kt18 rs-nc rs"
while (($#)); do
  case $1 in
    --runs) RUNS=$2; shift 2;; --only) ONLY=$2; shift 2;; --tools) TOOLS=$2; shift 2;;
    --timeout) TMO=$2; shift 2;; --out) OUT=$2; shift 2;; *) echo "unknown arg $1" >&2; exit 2;;
  esac
done
declare -A CMD=([ktrs]=$KTRS [kt2]=$KTLINT2 [kt18]=$KTLINT18 [rs]=$KTLINT_RS [rs-nc]=$KTLINT_RS)
declare -A NAME=([ktrs]="ktrs" [kt2]="ktlint 2.0.0-ALPHA-4" [kt18]="ktlint 1.8.0"
  [rs]="ktlint-rs (default cache, cold)" [rs-nc]="ktlint-rs (cache disabled)")
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
mkdir -p "$OUT"

# ktlint-rs keeps .cache/ktlint-rs/cache.json in the cwd; making it a directory makes every load/save fail (research/13).
prep_cache() { # tool cwd
  rm -rf "$2/.cache"
  [[ $1 == rs-nc ]] && mkdir -p "$2/.cache/ktlint-rs/cache.json"
  return 0
}

run_once() { # tool slug -> appends "wall cpu rss_mb exit" to $W/t.<tool>; keeps that run's out/err/exit/tree
  local tool=$1 slug=$2 ec
  eval "$SETUP"
  prep_cache "$tool" "$CWD"
  (cd "$CWD" && $PIN /usr/bin/time -f "%e %U %S %M" -o "$W/time" timeout "$TMO" ${CMD[$tool]} "${ARGS[@]}" \
    < "$STDIN" > "$W/out" 2> "$W/err"); ec=$?
  read -r wall u s rss < <(tail -1 "$W/time")
  echo "$wall $(awk -v u="$u" -v s="$s" 'BEGIN{print u+s}') $((rss / 1024)) $ec" >> "$W/t.$tool"
  local d=$OUT/$slug; mkdir -p "$d"
  mv "$W/out" "$d/$tool.out"; mv "$W/err" "$d/$tool.err"; echo "$ec" > "$d/$tool.exit"
  if [[ $FORMAT == 1 ]]; then rm -rf "$d/$tool.tree"; rm -rf "$CWD/.cache"; mv "$CWD" "$d/$tool.tree"; fi
}

median() { # file column
  local n; n=$(wc -l < "$1")
  cut -d' ' -f"$2" "$1" | sort -g | awk -v n="$n" '{a[NR]=$1} END {print (n%2) ? a[(n+1)/2] : (a[n/2]+a[n/2+1])/2}'
}

fmt_s() { awk -v s="$1" 'BEGIN { if (s < 1) printf "%.0f ms", s*1000; else printf "%.2f s", s }'; }

# ktlint's log lines start with a wall-clock timestamp.
norm() { sed -E 's/^[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3} //' "$1" | sort; }
same_sorted() { cmp -s <(norm "$1") <(norm "$2") && echo same || echo "DIFF($(diff <(norm "$1") <(norm "$2") | grep -c '^[<>]'))"; }

parity() { # slug -> one line, ktrs vs kt2
  local d=$OUT/$1 r
  [[ -f $d/ktrs.exit && -f $d/kt2.exit ]] || return 0
  r="exit $(cmp -s "$d/ktrs.exit" "$d/kt2.exit" && echo same || echo "DIFF($(cat "$d/ktrs.exit") vs $(cat "$d/kt2.exit"))")"
  r+=", stdout $(same_sorted "$d/ktrs.out" "$d/kt2.out"), stderr $(same_sorted "$d/ktrs.err" "$d/kt2.err")"
  [[ $FORMAT == 1 ]] && r+=", tree $(diff -rq "$d/ktrs.tree" "$d/kt2.tree" > "$d/tree.diff" && echo same || echo "DIFF($(wc -l < "$d/tree.diff") files)")"
  echo "parity ktrs vs ktlint 2.0: $r (stdout $(wc -l < "$d/kt2.out") lines)"
}

# scenario label slug cwd stdin format(0|1) setup pin args... ; setup is eval'd before every run (untimed)
scenario() {
  local label=$1 slug=$2; CWD=$3 STDIN=$4 FORMAT=$5 SETUP=$6 PIN=$7; shift 7; ARGS=("$@")
  [[ -n $ONLY && $label != *"$ONLY"* ]] && return
  rm -f "$W"/t.*
  local active=() t first
  for t in $TOOLS; do
    run_once "$t" "$slug"; first=$(cut -d' ' -f1 "$W/t.$t")
    if [[ $(tail -1 "$W/t.$t" | cut -d' ' -f4) == 124 ]] || awk -v w="$first" -v s="$SLOW" 'BEGIN{exit !(w > s)}'; then
      continue
    fi
    : > "$W/t.$t"; active+=("$t")
  done
  for ((i = 0; i < RUNS; i++)); do for t in "${active[@]}"; do run_once "$t" "$slug"; done; done
  for t in $TOOLS; do
    local f=$W/t.$t n wall; n=$(wc -l < "$f"); wall=$(median "$f" 1)
    [[ $(tail -1 "$f" | cut -d' ' -f4) == 124 ]] && wall=">$TMO"
    printf '| %s | %s | %s | %s | %s MB | %s | %s |\n' "$label" "${NAME[$t]}" \
      "$([[ $wall == '>'* ]] && echo "$wall s (timeout)" || fmt_s "$wall")" "$(fmt_s "$(median "$f" 2)")" "$(median "$f" 3)" "$n" \
      "$(cut -d' ' -f4 "$f" | sort -u | tr '\n' ' ')"
  done | tee -a "$OUT/results.md"
  parity "$slug" | tee -a "$OUT/results.md"
}

# Untimed: the json reporter of ktrs vs kt2 on the corpus, files sorted (reporting order follows thread scheduling).
verify_json() {
  [[ -n $ONLY && "Verify json" != *"$ONLY"* ]] && return
  local d=$OUT/verify-json t; mkdir -p "$d"
  for t in ktrs kt2; do (cd "$CORPUS" && ${CMD[$t]} --relative "--reporter=json,output=$d/$t.json" > /dev/null 2>&1); done
  for t in ktrs kt2; do
    python3 -c 'import json,sys; print(json.dumps(sorted(json.load(open(sys.argv[1])), key=lambda f: f["file"]), indent=1))' \
      "$d/$t.json" > "$d/$t.sorted.json"
  done
  echo "parity json reporter, corpus: $(cmp -s "$d/ktrs.sorted.json" "$d/kt2.sorted.json" && echo same || echo DIFF)" \
    "($(grep -c '"rule"' "$d/kt2.sorted.json") errors)" | tee -a "$OUT/results.md"
}

okhttp=$CORPUS/okhttp
one_dir=$okhttp/okhttp/src/commonJvmAndroid/kotlin/okhttp3
editor_file=$(find "$okhttp" -name '*.kt' -size +8k -size -16k | sort | head -1)
fresh() { echo "rm -rf $W/tree; cp -a $1 $W/tree; cp $CORPUS/.editorconfig $W/tree/"; }
files=$(find "$CORPUS" -name '*.kt' -o -name '*.kts' | wc -l)

{
  echo "$(uname -sm), $(nproc) CPUs usable, $(lscpu | sed -n 's/^Model name: *//p'); $(java -version 2>&1 | head -1)"
  echo "corpus $files files, okhttp $(find "$okhttp" -name '*.kt' -o -name '*.kts' | wc -l); editor file $editor_file"
  for t in ktrs kt2 kt18 rs; do echo "${NAME[$t]}: $(${CMD[$t]} --version 2>&1 | tail -1)"; done
  echo "median of $RUNS alternating runs after 1 warm-up; timeout $TMO s; no repeats past $SLOW s"
  echo; echo "| Scenario | Tool | Wall | User+sys CPU | Max RSS | n | Exit |"; echo "|---|---|--:|--:|--:|--:|---|"
} | tee -a "$OUT/results.md"

scenario "Corpus lint ($files files)" corpus-lint "$CORPUS" /dev/null 0 : "" --relative
scenario "okhttp lint" okhttp-lint "$okhttp" /dev/null 0 : "" --relative
scenario "okhttp lint, 1 core" okhttp-lint-1core "$okhttp" /dev/null 0 : "taskset -c $ONE_CPU" --relative
scenario "One file lint (Credentials.kt)" one-file "$one_dir" /dev/null 0 : "" --relative Credentials.kt
scenario "Editor: $(($(stat -c %s "$editor_file") / 1000)) KB file on stdin" stdin "$(dirname "$editor_file")" "$editor_file" 0 : "" --stdin
CWD_T=$W/tree
scenario "Corpus format (-F, fresh copy)" corpus-format "$CWD_T" /dev/null 1 "$(fresh "$CORPUS")" "" --relative -F
scenario "okhttp format (-F, fresh copy)" okhttp-format "$CWD_T" /dev/null 1 "$(fresh "$okhttp")" "" --relative -F
verify_json
