# 20 — ktlint end to end: ktrs vs ktlint 2.0 / 1.8 vs ktlint-rs (2026-10-01)

Testbox: Xeon E-2136, 10 CPUs (`taskset -c 0-4,6-10`), Java 21.0.12; shared box, single runs swing ±15%.
ktrs master 569b2c7 (`--profile dist`, the `ktlint` drop-in); ktlint 2.0.0-ALPHA-4 and 1.8.0 release jars
(`-Xmx512m`); ktlint-rs v0.1.20 (`ed3777c`), default cache cleared per run, or cache disabled (`cache.json` made a
directory, research/13). Corpus: corpus/REVISIONS copied as `.kt`/`.kts` only under one root `.editorconfig`
(`ktlint_official`), 6,123 files, 30.8 MB. Median of 5 alternating runs after 1 warm-up (n=1: warm-up over 300 s
or timed out). Script: `tools/bench/lint-e2e.sh`; raw: testbox `~/work/lb-run3.log` (ktrs, ktlint 2.0),
`~/work/lb-run.log`, `~/work/lb-run2.log` (ktlint 1.8, ktlint-rs: measured against ktrs 33da41a, same box).

| Scenario | ktrs | ktlint 2.0 | ktlint 1.8 | ktlint-rs, no cache | ktlint-rs, default |
|---|--:|--:|--:|--:|--:|
| Corpus lint, wall | **3.03 s** | 49.26 s | 36.0 s | 20.2 s | >1,200 s (timeout) |
| Corpus lint, CPU | 27.4 s | 252 s | 197 s | 180 s | — |
| Corpus lint, RSS | 227 MB | 514 MB | 490 MB | 205 MB | 205 MB |
| okhttp lint (617 files) | **590 ms**, 4.3 CPU-s | 11.77 s, 64.5 CPU-s | 8.76 s | 5.58 s | 45.7 s |
| okhttp lint, 1 core | **2.42 s**, 40 MB | 39.59 s, 341 MB | 30.9 s | 22.4 s | 63.1 s |
| One file | 10 ms, 23 MB | 870 ms, 150 MB | 890 ms | ~0 ms | ~0 ms |
| 8 KB file on stdin | 10 ms | 1.09 s | 1.15 s | 20 ms | 20 ms |
| okhttp format | **1.00 s**, 7.8 CPU-s | 121 s, 299 CPU-s | 115 s | >60 s (cap) | >60 s (cap) |
| Corpus format | **6.92 s**, 65 CPU-s | 337 s (n=1), 997 CPU-s | 261 s, 724 CPU-s | not run | not run |

## Output parity (ktrs vs the 2.0 jar, same run)

- Lint: exit code, stdout and stderr identical in every scenario (corpus: 213,088 lines).
- Format: formatted trees byte-identical (okhttp, corpus); exit codes identical. Console output differs only on
  files where ktlint's `indent` throws "Stack should be empty": the jar prints Java stack frames and
  `java.lang.`-qualified `Caused by:`, lays out the `IndentContext` dump differently, and logs the `CodeFormatter`
  WARN "Format was not able to resolve all violations ... in 3 consecutive runs", which ktrs does not.

## Reading

- 27 CPU-s for the corpus is ~4.5 ms per file, 9x less CPU than ktlint 2.0 and 6.6x less than ktlint-rs (which
  also gives incorrect output, research/13). Before the engine overhead fixes (3815aa1) and the visited-types
  dispatch filter (0a23cc8) the same run took 166 CPU-s (17.1 s wall).
- Startup and memory: 10 ms and 23 MB per file vs ~0.9-1.1 s and 150 MB (editor, pre-commit).
- Format is the widest gap (49-121x wall, 15-38x CPU).
- ktlint-rs `--format` didn't finish okhttp in 60 s with either cache setting.
