# 20 — ktlint end to end: ktrs vs ktlint 2.0 / 1.8 vs ktlint-rs (2026-10-01)

Testbox: Xeon E-2136, 10 CPUs (`taskset -c 0-4,6-10`), Java 21.0.12; shared box, single runs swing ±15%.
ktrs master 9ae68e2 (0.3.1, `--profile dist`, the `ktlint` drop-in); ktlint 2.0.0-ALPHA-4 and 1.8.0 release jars
(`-Xmx512m`); ktlint-rs v0.1.20 (`ed3777c`), default cache cleared per run, or cache disabled (`cache.json` made a
directory, research/13). Corpus: corpus/REVISIONS copied as `.kt`/`.kts` only under one root `.editorconfig`
(`ktlint_official`), 6,123 files, 30.8 MB. Median of 5 alternating runs after 1 warm-up (n=1: warm-up over 300 s
or timed out). Script: `tools/bench/lint-e2e.sh`; raw: testbox `~/work/lb-run4.log` (ktrs, ktlint 2.0, same run),
`~/work/lb-run.log`, `~/work/lb-run2.log` (ktlint 1.8, ktlint-rs: measured against ktrs 33da41a, same box).
`/usr/bin/time` resolves 10 ms, so "<10 ms" is a 0 reading.

| Scenario | ktrs | ktlint 2.0 | ktlint 1.8 | ktlint-rs, no cache | ktlint-rs, default |
|---|--:|--:|--:|--:|--:|
| Corpus lint, wall | **1.79 s** | 48.24 s | 36.0 s | 20.2 s | >1,200 s (timeout) |
| Corpus lint, CPU | 14.7 s | 246 s | 197 s | 180 s | — |
| Corpus lint, RSS | 249 MB | 515 MB | 490 MB | 205 MB | 205 MB |
| okhttp lint (617 files) | **390 ms**, 2.4 CPU-s | 11.33 s, 63.5 CPU-s | 8.76 s | 5.58 s | 45.7 s |
| okhttp lint, 1 core | **1.35 s**, 47 MB | 38.15 s, 344 MB | 30.9 s | 22.4 s | 63.1 s |
| One file | <10 ms, 23 MB | 830 ms, 148 MB | 890 ms | ~0 ms | ~0 ms |
| 8 KB file on stdin | <10 ms, 23 MB | 1.04 s, 168 MB | 1.15 s | 20 ms | 20 ms |
| okhttp format | **620 ms**, 4.4 CPU-s | 119 s, 292 CPU-s | 115 s | >60 s (cap) | >60 s (cap) |
| Corpus format | **3.76 s**, 35 CPU-s | 320 s (n=1), 945 CPU-s | 261 s, 724 CPU-s | not run | not run |

## Output parity (ktrs vs the 2.0 jar, same run)

- Lint: exit code, stdout and stderr identical in every scenario (corpus: 213,088 lines).
- Format: formatted trees byte-identical (okhttp, corpus); exit codes identical. Console output differs only on
  files where ktlint's `indent` throws "Stack should be empty": the jar prints Java stack frames and
  `java.lang.`-qualified `Caused by:`, lays out the `IndentContext` dump differently, and logs the `CodeFormatter`
  WARN "Format was not able to resolve all violations ... in 3 consecutive runs", which ktrs does not.

## Reading

- 14.7 CPU-s for the corpus is ~2.4 ms per file, 17x less CPU than ktlint 2.0 and 12x less than ktlint-rs (which
  also gives incorrect output, research/13). History of the same run: 166 CPU-s at 33da41a; 27.4 CPU-s after the
  engine overhead fixes (3815aa1) and the visited-types filter (0a23cc8); 14.7 CPU-s after the per-type dispatch
  table, the shared per-`.editorconfig` setup, the chain/suppression shortcuts and the per-node newline count
  (540aa0c..0128c7f).
- Startup and memory: <10 ms and 23 MB per file vs ~0.8-1.0 s and 150-170 MB (editor, pre-commit).
- Format is the widest gap (85-192x wall, 27-66x CPU).
- ktlint-rs `--format` didn't finish okhttp in 60 s with either cache setting.
