# 23 — Public benchmark: `tools/bench/public.sh` (2026-10-02)

The README's numbers came from `tools/bench/e2e.py` (ktfmt, a Windows laptop) and `tools/bench/lint-e2e.sh` (ktlint,
the private testbox). `tools/bench/public.sh` measures the same scenarios with public inputs only, so anyone with
Linux or macOS can rerun them, and `.github/workflows/bench.yml` runs it on a hosted runner.

## Run it

```
cargo build --profile dist --bins                  # or KTRS_BIN=<dir with ktfmt, ktlint, ktrs>
tools/bench/public.sh                              # everything: ~2 h on a 10-CPU box
tools/bench/public.sh --only quick --ktlint 2.0.0-ALPHA-4   # what CI runs by default: ~25 min on 4 CPUs
tools/bench/public.sh --list                       # scenario ids and groups (all, quick, fmt, lint)
```

Needs bash (3.2 is enough), git, curl, java (21 for the numbers below), hyperfine (1.19+ for max RSS), python3.
`--runs N` (default 5), `--warmup N` (1), `--out DIR` (`target/bench/public`). `ONE_CPU` picks the CPU for the 1-core
scenarios, which need `taskset` and are skipped on macOS.

Inputs, all pinned and checked:
- Corpus: the 7 repos at the commits in `tools/bench/REVISIONS` (the same SHAs as the local `corpus/REVISIONS`, which
  is gitignored), fetched by `tools/holdout/fetch.sh DEST REVISIONS` into `~/.cache/ktrs-bench/corpus` (`CACHE`
  moves it). Only `.kt`/`.kts` are kept (6,123 files), under one root `.editorconfig` with
  `ktlint_code_style = ktlint_official`, as in research/13. `CORPUS=<tree>` reuses a prepared tree.
- Jars, downloaded into the cache and rejected unless their sha256 matches the value in the script: ktfmt 0.64
  `-with-dependencies` (Maven Central), ktlint 2.0.0-ALPHA-4 and 1.8.0 (the self-executing `ktlint` release assets).
  Another ktlint version needs its sha256 added to `ktlint_sha256`.

Output: `OUT/<id>.json` and `OUT/<id>.md` (hyperfine exports, every run's time, exit code and max RSS),
`OUT/env.md` (machine, Java, versions, corpus), `OUT/summary.md` (median, ±stddev, mean user+sys, median max RSS,
jar median / ktrs median). The workflow uploads `OUT` as the `bench-<sha>` artifact and posts the summary and the
hyperfine tables as the job summary. A WARNING line flags any exit code other than 0 or 1 (a broken command).

## What it measures

Same scenarios, flags and cwd as e2e.py and lint-e2e.sh. Each scenario is one hyperfine call: ktrs first, then each
jar; 1 warm-up and 5 timed runs per command; median wall time. Commands run through `sh -c` (hyperfine subtracts the
shell's startup; below ~5 ms that correction is the main error, so treat 3-8 ms as "a few ms").

| id | Command (ktrs side; the jar side is the same flags) |
|---|---|
| fmt-stdin | `ktfmt - < F`, F = the first okhttp `.kt` of 8-16 KB (`AndroidNetworksTest.kt`, 8.8 KB) |
| fmt-precommit | `ktfmt --quiet` on okhttp's first 10 files (sorted), copied once |
| fmt-check-okhttp | `ktfmt -n --set-exit-if-changed okhttp` |
| fmt-okhttp, fmt-corpus | `ktfmt --quiet <copy>`; copied once per tool (`--setup`), so the timed runs re-format formatted files, as e2e.py |
| fmt-okhttp-1core | the same under `taskset -c $ONE_CPU` (the JVM sizes its pools for 1 CPU) |
| lint-stdin | `cd dir(F) && ktlint --stdin < F` |
| lint-file | `cd okhttp/.../okhttp3 && ktlint --relative Credentials.kt` |
| lint-okhttp, lint-corpus, lint-okhttp-1core | `cd <tree> && ktlint --relative` |
| format-okhttp, format-corpus | `ktlint --relative -F` on a fresh copy before every run (`--prepare`, untimed) |

Not covered (still in the old scripts): ktlint-rs (lint-e2e.sh), the output-parity checks after each scenario
(lint-e2e.sh; the parity gates in CLAUDE.md own that), and Spotless. hyperfine does not interleave the tools the way
the old scripts did, so a slow drift on a shared box lands on one tool.

## Results

testbox (Xeon E-2136, Linux, Java 21.0.12, hyperfine 1.20.0), ktrs 0.3.1 dist binaries (551e46e), the full run
under `flock ~/bench.lock taskset -c 0-4,6-10` (10 CPUs). Speedup = jar median / ktrs median. README column: the
current README value (ktfmt rows: Windows 11 laptop, Core Ultra 22 threads; ktlint rows: testbox, lint-e2e.sh).

| Scenario | ktrs | ktfmt 0.64 / ktlint 2.0 | ktlint 1.8 | Speedup (2.0) | README |
|---|--:|--:|--:|--:|--:|
| ktfmt, 8 KB file on stdin | 3.5 ms | 791 ms | | 226x | 16 ms / 1.49 s, 96x |
| ktfmt, pre-commit 10 files | 16 ms | 743 ms | | 47x | 26 ms / 2.57 s, 99x |
| ktfmt, CI check okhttp | 183 ms | 3.87 s | | 21x | 316 ms / 9.08 s, 29x |
| ktfmt, format okhttp | 176 ms | 3.30 s | | 19x | 465 ms / 9.58 s, 21x |
| ktfmt, format okhttp, 1 core | 505 ms | 12.46 s | | 25x | 1.48 s / 40.79 s, 28x |
| ktfmt, format corpus | 813 ms | 10.91 s | | 13x | 3.69 s / 24.55 s, 7x |
| ktlint, 8 KB file on stdin | 8.1 ms | 1.06 s | 1.09 s | 130x | <10 ms / 1.04 s |
| ktlint, one file | 6.3 ms | 845 ms | 883 ms | 135x | <10 ms / 830 ms |
| ktlint, lint okhttp | 344 ms | 11.56 s | 10.31 s | 34x | 390 ms / 11.33 s, 29x |
| ktlint, lint okhttp, 1 core | 1.37 s, 49 MB | 39.36 s, 366 MB | 28.82 s | 29x | 1.35 s / 38.15 s, 28x |
| ktlint, lint corpus | 1.72 s, 268 MB | 49.73 s, 553 MB | 34.10 s | 29x | 1.79 s / 48.24 s, 27x |
| ktlint, autocorrect okhttp | 640 ms | 123.0 s | 117.6 s | 192x | 620 ms / 118.94 s, 192x |
| ktlint, autocorrect corpus | 4.00 s | 338.9 s | 267.0 s | 85x | 3.76 s / 320.12 s, 85x |

Reading:
- **ktlint rows reproduce.** Same box, different harness (hyperfine vs `/usr/bin/time`, sequential vs interleaved,
  a day apart): every ktrs and ktlint 2.0 wall time is within 12% of research/20 (ktlint 1.8: within 18%), and the
  headline speedups come out at 34x / 29x / 192x / 85x against the README's 29x / 27x / 192x / 85x.
  RSS reads higher than lint-e2e.sh's (553 vs 515 MB for the 2.0 corpus lint) because hyperfine reports the max RSS
  of the process tree, including the `sh` wrapper of the self-executing jar.
- **ktfmt rows don't, by design.** The README's ktfmt table is from a Windows laptop, where every process start and
  first file open costs more (Defender), which inflates the jar's small-run times and ktrs's large-run times. On
  Linux the small-run ratios are bigger (226x vs 96x) and the whole-project ones similar (13-25x vs 7-28x). A README
  regenerated from public.sh would quote one Linux machine for both tables.
- ktlint 1.8 is faster than 2.0 on whole projects (34 s vs 50 s for the corpus lint), as research/20 found.

### 4 CPUs: the CI default as a hosted-runner proxy

`--only quick --ktlint 2.0.0-ALPHA-4` under `taskset -c 0-3` on the testbox (ubuntu-latest has 4 vCPUs): 1,471 s of
benchmarking (format-okhttp 801 s, lint-corpus 401 s, the rest under 100 s each), plus the corpus fetch and the dist
build, so the default workflow run should fit in ~45 min. Medians: ktfmt stdin 4.1 ms / 806 ms (197x), pre-commit
5.6 ms / 757 ms (136x), CI check 181 ms / 5.25 s (29x), okhttp 163 ms / 4.37 s (27x), corpus 982 ms / 13.71 s
(14x); ktlint stdin 7.9 ms / 1.00 s, one file 5.9 ms / 789 ms, okhttp 402 ms / 15.03 s (37x), corpus 2.46 s /
65.13 s (26x), autocorrect okhttp 719 ms / 133.6 s (186x). With fewer cores the jars lose more than ktrs, so the
multi-file speedups grow a little. (ktrs's pre-commit is faster on 4 CPUs than on 10: 5.6 vs 16 ms, the cost of
starting a thread pool sized for the machine on 10 small files.)

## Noise

- **testbox** (shared; other sessions' jobs run between, not during, ours under the lock, but unpinned load from
  other users still lands on the same cores): within one hyperfine call, stddev was under ~5% of the median for
  most commands. Exceptions: ktlint 1.8 lint okhttp (10.31 s ± 2.82 s, one run at 15.8 s), ktrs pre-commit
  (16 ± 7 ms), ktlint 1.8 one file (10%), ktrs autocorrect okhttp (7%). Run-to-run against research/20: 2-12% on
  wall time (18% for ktlint 1.8 lint okhttp). The ±15% guidance in the
  testbox memory still holds for a single run; the median of 5 is good to ~10%.
- **GitHub runner: not measured yet.** The workflow was written and the script verified on the testbox, but not
  run on Actions (that needs the branch pushed). Hosted runners are shared VMs whose CPU model varies run to run, so
  expect absolute times to move between runs more than the testbox's (compare speedups, not seconds) and check
  each run's `env.md`. When the first runs land, record their stddevs and the run-to-run spread here.
