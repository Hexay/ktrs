# 20 — ktlint end to end: ktrs vs ktlint 2.0 / 1.8 vs ktlint-rs (2026-10-01)

Testbox: Xeon E-2136, 10 CPUs (`taskset -c 0-4,6-10`), Java 21.0.12; shared box, single runs swing ±15%.
ktrs master 33da41a (`--profile dist`, the `ktlint` drop-in); ktlint 2.0.0-ALPHA-4 and 1.8.0 release jars
(`-Xmx512m`); ktlint-rs v0.1.20 (`ed3777c`), default cache cleared per run, or cache disabled (`cache.json` made a
directory, research/13). Corpus: corpus/REVISIONS copied as `.kt`/`.kts` only under one root `.editorconfig`
(`ktlint_official`), 6,123 files, 30.8 MB. Median of 5 alternating runs after 1 warm-up (n=1: warm-up over 300 s
or timed out). Script: `tools/bench/lint-e2e.sh`; raw: testbox `~/work/lb-run.log`, `~/work/lb-run2.log`.

| Scenario | ktrs | ktlint 2.0 | ktlint 1.8 | ktlint-rs, no cache | ktlint-rs, default |
|---|--:|--:|--:|--:|--:|
| Corpus lint, wall | **17.1 s** | 50.5 s | 36.0 s | 20.2 s | >1,200 s (timeout) |
| Corpus lint, CPU | 166 s | 258 s | 197 s | 180 s | — |
| Corpus lint, RSS | 204 MB | 522 MB | 490 MB | 205 MB | 205 MB |
| okhttp lint (617 files) | **4.05 s** | 11.98 s | 8.76 s | 5.58 s | 45.7 s |
| okhttp lint, 1 core | **18.4 s**, 41 MB | 41.7 s, 343 MB | 30.9 s | 22.4 s | 63.1 s |
| One file | ~0 ms, 23 MB | 840 ms, 150 MB | 890 ms | ~0 ms | ~0 ms |
| 8 KB file on stdin | 30 ms | 1.11 s | 1.15 s | 20 ms | 20 ms |
| okhttp format | **13.3 s**, 87 CPU-s | 126 s, 309 CPU-s | 115 s | >60 s (cap) | >60 s (cap) |
| Corpus format | **51.5 s**, 509 CPU-s | 337 s (n=1), 1,003 CPU-s | 261 s, 724 CPU-s | not run | not run |

## Output parity (ktrs vs the 2.0 jar, same run)

- Lint: exit code, stdout and stderr identical in every scenario (corpus: 213,088 lines).
- Format: formatted trees byte-identical (okhttp, corpus). Console output differed on files where ktlint's
  `indent` throws "Stack should be empty" (Rust panic text on stderr, `Caused by:` without `java.lang.`) and by
  the missing `CodeFormatter` WARN "Format was not able to resolve all violations ... in 3 consecutive runs".
  cli-diff had no crash scenario; fixed and covered separately.

## Reading

- Startup and memory are the clear wins: ~0 ms and 23 MB per file vs ~0.85 s and 150 MB (editor, pre-commit).
- Throughput is not: 166 CPU-s for the corpus is ~27 ms per file, ~30x the CPU `ktrs fmt` needs for the same
  files, and only 1.55x below ktlint 2.0 (format: 2x). One core on okhttp: 2.3x. ktlint-rs (incorrect output,
  research/13) is within 1.2x of us. The per-file fixed cost (105 rule instances, per-rule EditorConfig
  filtering, suppression setup, seeding) is the first suspect; see the golden runner's ~170 ms per case.
- ktlint-rs `--format` didn't finish okhttp in 60 s with either cache setting.
