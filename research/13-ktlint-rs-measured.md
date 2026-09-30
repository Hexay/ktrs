# 13 — ktlint-rs measured against ktlint 1.8.0

Run 2026-09-30 on `testbox` (Linux, 12 cores, Java 21; every run under `flock ~/bench.lock taskset -c 0-4,6-10`, so
10 cores). The box is shared: expect roughly ±15% wall noise. Raw outputs are in `~/work/ktlint-bench/` (paths below).
Format mode was cut short: see "Format".

## Versions

| tool | version | source |
|---|---|---|
| ktlint-rs | v0.1.20, `ed3777c28b3debfc95605c64c512436ddff7b775` | `cargo build --release` of the tag (tree-sitter-kotlin based, rayon) |
| ktlint (JVM) | 1.8.0 (tag `c26a4ff`) | release asset `ktlint` (self-executing jar), Java 21 |
| ktlint native | 2.0.0-ALPHA-4 | release asset `ktlint_linux-x86-64`: **crashes at startup** (below) |

## Corpus and config

- The 7 repos at the SHAs in `corpus/REVISIONS` (okhttp, kotlinx.coroutines, nowinandroid, ktlint, ktfmt, Exposed, ktor):
  6123 `.kt`/`.kts` files, 30.8 MB. okhttp alone is 617 files.
- **Neutralized per-repo config.** The repos ship 11 `.editorconfig` files. Only the `.kt`/`.kts` files were copied (`cpio -pdm`)
  into a fresh tree, `~/work/ktlint-bench/corpus/`, so it has no `.git`, `.gitignore` or `.editorconfig` except one at the root:
  `root = true` / `[*.{kt,kts}]` / `ktlint_code_style = ktlint_official`. Both tools found all 6123 files
  (`ktlint-rs --print-files`).
- Most of the corpus is not in ktlint_official style, so both tools report many violations (214k from ktlint). That gives
  the agreement comparison plenty of data, but the counts say nothing about the repos themselves.

## Speed (lint, default parallelism, median of 3 alternating runs)

`/usr/bin/time -f "%e %U %M"`, plain reporter, stdout to a file. Wall and user are in seconds, RSS in MB.

| target | tool | wall | user CPU | max RSS | vs ktlint wall |
|---|---|--:|--:|--:|--:|
| corpus (6123 files) | ktlint 1.8.0 | 36.2 | 198.5 | 481 | 1.00x |
| corpus | ktlint-rs, cache disabled | 29.5 | 183.6 | 213 | 1.23x faster |
| corpus | ktlint-rs **default** (cold cache), 1 run | ~1041 (17m21s) | — | — | **29x slower** |
| corpus | ktlint-rs default, warm cache | killed at 1387 s | 1203 | 825 | >38x slower |
| okhttp (617 files) | ktlint 1.8.0 | 9.39 | 55.4 | 426 | 1.00x |
| okhttp | ktlint-rs, cache disabled | 5.40 | 33.3 | 134 | 1.74x faster |
| okhttp | ktlint-rs default (cold cache) | 42.9 | 46.5 | 151 | 4.6x slower |
| okhttp | ktlint-rs default (warm cache) | 45.7 | 50.4 | 157 | 4.9x slower |
| 1 file (`Credentials.kt`) | ktlint 1.8.0 | 0.96 | 1.79 | 150 | 1.00x |
| 1 file | ktlint-rs | <0.01 | <0.01 | 9 | >100x faster |

- **The incremental cache makes default runs quadratic.** In `src/cache.rs`, `save_cached` runs once per file
  (sequentially, after the parallel lint). Each call re-reads, re-parses and rewrites the whole
  `.cache/ktlint-rs/cache.json`, which reached 50 MB on this corpus, so the save phase is O(files × cache size).
  `get_cached` also reparses the whole file once per input file, which is why a warm run is no faster than a cold one.
  There is no flag to turn the cache off. The "cache disabled" rows turn it off by making `cache.json` a directory, so
  every read and write fails silently.
- Even without the cache, the lint engine barely beats the JVM on CPU: 183 s against 198 s user time for the corpus,
  about 170 KB/s per core. Its advantages are startup time (<10 ms against about 1 s) and memory (2–3x less RSS).
  The project's README claims 17–27x. That holds only for a single file or a tiny project, where JVM startup dominates.
- ktlint 2.0.0-ALPHA-4 native crashes on every input, including a 3-line file, while loading the rule providers
  (`--version` works): `ExceptionInInitializerError` ← `ReflectionUtil.<clinit>`: "Could not find 'theUnsafe' field in
  the Unsafe class". This looks like a GraalVM reflection-config gap. No native numbers.

## Lint agreement (ktlint 1.8.0 json vs ktlint-rs json)

Both outputs were normalized to a multiset of (file, line, col, rule id) by `~/work/ktlint-bench/compare.py`.
Precision (P) is the share of ktlint-rs findings that ktlint also reports. Recall (R) is the share of ktlint findings
that ktlint-rs also reports.

| slice | ktlint | ktlint-rs | exact match | P | R |
|---|--:|--:|--:|--:|--:|
| all rules | 214,281 | 173,635 | 160,291 | **0.923** | **0.748** |
| same (file, line, rule), any column | | | 160,939 | 0.927 | 0.751 |
| excluding `indent` | 94,060 | 67,002 | 54,395 | 0.812 | 0.578 |
| files with ≥1 violation | 5,224 | 5,255 | 5,036 both | | |

Per repo: okhttp 0.98/0.87, ktfmt 0.97/0.82, ktor 0.93/0.63, kotlinx.coroutines 0.87/0.67, Exposed 0.77/0.45,
nowinandroid 0.79/0.42, ktlint 0.06/0.24 (P/R). The ktlint repo is already ktlint-clean, so false positives dominate
there: ktlint-rs reports 4,043 violations against ktlint's 977. Of these, 1,231 are `no-consecutive-comments` on
ordinary consecutive `//` lines.

Top 20 rules by ktlint count (P and R are for ktlint-rs; all ids are `standard:`-prefixed):

| rule | ktlint | ktlint-rs | exact | P | R |
|---|--:|--:|--:|--:|--:|
| indent | 120,221 | 106,633 | 105,896 | 0.99 | 0.88 |
| function-signature | 23,046 | 7,803 | 7,673 | 0.98 | 0.33 |
| multiline-expression-wrapping | 18,391 | 16,464 | 16,395 | 1.00 | 0.89 |
| no-wildcard-imports | 13,403 | 13,464 | 13,403 | 1.00 | 1.00 |
| trailing-comma-on-call-site | 5,477 | 5,287 | 5,165 | 0.98 | 0.94 |
| chain-method-continuation | 5,449 | 0 | 0 | — | 0.00 |
| argument-list-wrapping | 5,210 | 742 | 668 | 0.90 | 0.13 |
| class-signature | 4,271 | 113 | 33 | 0.29 | 0.01 |
| trailing-comma-on-declaration-site | 3,688 | 2,098 | 2,089 | 1.00 | 0.57 |
| no-empty-first-line-in-class-body | 1,604 | 1,612 | 1,598 | 0.99 | 1.00 |
| function-expression-body | 1,500 | 1,501 | 1,423 | 0.95 | 0.95 |
| blank-line-before-declaration | 1,367 | 1,467 | 1,201 | 0.82 | 0.88 |
| when-entry-bracing | 1,341 | 1,900 | 1,321 | 0.70 | 0.99 |
| blank-line-between-when-conditions | 1,206 | 1,034 | 1,005 | 0.97 | 0.83 |
| max-line-length | 1,125 | 355 | 0 | 0.00 | 0.00 |
| import-ordering | 810 | 830 | 596 | 0.72 | 0.74 |
| string-template-indent | 747 | 66 | 0 | 0.00 | 0.00 |
| wrapping | 628 | 276 | 217 | 0.79 | 0.35 |
| function-literal | 575 | 1 | 0 | 0.00 | 0.00 |
| statement-wrapping | 558 | 410 | 388 | 0.95 | 0.70 |

Rule-id and config issues:
- Both tools use the same `standard:<id>` namespace, so no ids needed remapping.
- ktlint-rs emits ids that ktlint 1.8.0 never emitted on this corpus: `parameter-wrapping` 720, `no-unused-imports` 606,
  `no-blank-lines-in-chained-method-calls` 73, `block-comment-initial-star-blank-line` 61, `modifier-list-spacing` 55,
  `no-empty-class-body` 52, `unary-op-spacing` 50, and smaller counts of others.
- ktlint-rs never emits `chain-method-continuation` (5,449 in ktlint), `multiline-loop`, `chain-wrapping`,
  `no-empty-first-line-in-method-block`, `type-parameter-list-spacing`, `function-return-type-spacing` or
  `value-parameter-comment`.
- `max-line-length`: under ktlint_official, ktlint-rs uses a 120-column limit where ktlint uses 140, and it reports
  col 120 where ktlint reports col 141. That gives 0 exact matches; the 295 lines both tools flag differ only in column.
- ktlint reported 2 files (Exposed test resources) as "Not a valid Kotlin file", with an empty rule id. ktlint-rs does not
  report parse failures. ktlint-rs had no crashes or hangs in lint mode and skipped no files.

## Format

**Not measured.** `ktlint-rs --format` on the corpus with the cache disabled had not finished after about 8 minutes. The
run was stopped when the benchmark was cut short, and no format runs completed, so there is no byte-identical figure.
For comparison, ktlint 1.8.0 lints the whole corpus in 36 s. Probable cause (from reading the code, not profiled):
`--format` first lints, then re-lints each file *sequentially* with an indent probe (`src/main.rs`, the loop after
`if cli.format`). The comparison scripts are ready: `phase2.sh fmt`, then `fmtcmp.py <orig> <ktlint -F tree> <ktlint-rs tree>`.

## Reproduce (testbox, `B=~/work/ktlint-bench`)

```
git clone https://github.com/qdsfdhvh/ktlint-rs $B/src && git -C $B/src checkout v0.1.20
flock ~/bench.lock taskset -c 0-4,6-10 cargo build --release        # in $B/src
curl -L -o $B/bin/ktlint-1.8.0 https://github.com/pinterest/ktlint/releases/download/1.8.0/ktlint
# corpus: bash $B/klb-corpus.sh (cpio copy of *.kt/*.kts + root .editorconfig)
cd $B/corpus && $RS --reporter json --reporter-output $B/out/lint/rs.json .
cd $B/corpus && $B/bin/ktlint-1.8.0 --relative --reporter=json,output=$B/out/lint/kt18.json
python3 $B/compare.py out/lint/kt18.json out/lint/rs.json; python3 $B/compare_extra.py ...   # agreement
flock ~/bench.lock taskset -c 0-4,6-10 bash $B/phase2.sh lint-time   # timings -> out/time/lint-time.txt
```

Raw files: `out/lint/{kt18,rs}.json` (lint reports), `out/lint/rs-vs-kt18{,-extra}.txt` (agreement reports),
`out/time/lint-time.txt` (every timing run), `run2.log` and `run-agree.log` (run logs), `out/rs-agree-cache/`
(the 50 MB ktlint-rs cache).

## Verdict

ktlint-rs is **not yet a credible replacement** for ktlint. Its matches are mostly real: 92% of what it reports, ktlint
also reports. But it misses a quarter of ktlint's findings (58% recall once `indent` is excluded). It does not implement
`chain-method-continuation` at all, and it gets the ktlint_official line-length limit wrong. Its speed claim holds only for
startup. Across a large tree the engine is about 1.2x the JVM in wall time, and the default cache makes it 5–30x
*slower*. Format mode did not finish on the corpus. Of the three parity bars, the only one it clears is low memory.
