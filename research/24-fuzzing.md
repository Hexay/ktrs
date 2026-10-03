# 24 — Fuzzing (2026-10-02)

The corpus and held-out runs (research/21) only see code that compiles. This round feeds the port inputs nobody
would write: coverage-guided fuzzing for crashes and invariants, plus a differential run of generated inputs through
the `ktfmt`/`ktlint` binaries and the upstream jars. ktrs at 53bb811 (`--profile dist`), ktfmt 0.64 jar, ktlint
2.0.0-ALPHA-4 release asset, on testbox. Raw output: testbox `~/work/fuzz/` (logs, `diff/<run>/out`, `min/`) and
`~/work/ktrs-chk/fuzz/artifacts/`.

## Method

**Coverage-guided** (`fuzz/`, cargo-fuzz, its own workspace root so the main workspace never builds it):

| Target | Checks |
|---|---|
| `parser` | no panic; as `.kt` and `.kts`: the tree's text and its tokens spell the input, every element's range lies in its parent's, `psi_dump` runs, and `parse_file_cached` (one `ChameleonCache` shared across inputs) gives the same tree and errors as `parse_file` |
| `ktfmt` | `ktrs_fmt::format` with meta, google and kotlinlang: no panic |
| `ktlint` | lint, then format (autocorrect everything), with every ported standard rule; one of ktlint_official, ktlint_official + experimental, intellij_idea, android_studio and `.kt`/`.kts` per input (by hash, so an input always takes the same path) |

A panic whose message is a Java exception name (`IllegalArgumentException: ...`, `AssertionError`) is ktrs mimicking
an upstream throw and is tolerated (`ktrs_fuzz::tolerate_java_panics`); any other panic is a finding, including one
ktrs-lint's engine catches at the rule boundary, since that one still turns into a `KtLintRuleException` the jar
would not raise. `KTRS_FUZZ_KNOWN=file.rs:line,...` tolerates already-reported panic sites so a session gets past them.

Mutation is token-level (`fuzz/src/splice.rs`): edits land on `ktrs_lexer` token boundaries — insert a fragment
(keywords, operators, comments, KDoc, string templates, small declarations), delete/duplicate/move/swap runs of up to
12 tokens, splice tokens from elsewhere in the input — plus a token-boundary crossover between corpus entries. One
mutation in nine falls back to libFuzzer's byte-level mutator. Seeds: every `.kt`/`.kts` under `testdata/` below 8 KB
(7,340 files: PSI fixtures, ktfmt and ktlint test inputs).

**Differential** (`tools/fuzz/diff.sh`): inputs that parse without a single error element (both upstream tools reject
the rest), from `kgen mutate` (1–4 splice/crossover edits of a random seed; ~6% of attempts parse cleanly) or
`kgen filter` (the clean part of the fuzzers' corpora). Each tool, each style, two passes through
`tools/parity/{ktfmt,ktlint}-compare.sh`: pass 1 on the inputs, pass 2 on the jar's pass-1 output (`KEEP=`), so a
format that isn't idempotent counts only where the jar's second pass differs from ours. Pass-2 rows for files that
already differed in pass 1 are dropped as consequences. Files ktrs's ktfmt rejects are compared one by one
(`differs.sh`): on a `FormattingError` the jar's CLI aborts the whole run (`Exception in thread "main"`) and leaves
the rest of the tree unformatted.

## How to run (Linux, nightly + cargo-fuzz; testbox etiquette: `flock ~/bench.lock taskset -c 0-4,6-10`, nohup)

```sh
tools/fuzz/fuzz.sh 1800                         # 30 min, parser:2 ktfmt:4 ktlint:4 fork jobs; seeds fuzz/corpus/ on first run
KTRS_FUZZ_KNOWN=input_output.rs:74 tools/fuzz/fuzz.sh 1800 ktfmt:6
(cd fuzz && cargo +nightly build --release --bin kgen)
fuzz/target/release/kgen mutate testdata /tmp/in 3000 1        # or: kgen filter fuzz/corpus/ktlint /tmp/in
KTRS_BIN=target/dist KTLINT2=<ktlint-2.0.0-ALPHA-4> tools/fuzz/diff.sh /tmp/in /tmp/out    # report.md, mismatches.tsv, repro/
EC_EXTRA='ktlint_experimental = enabled\n' tools/fuzz/diff.sh /tmp/in /tmp/out-exp ktlint
```

Triage: `tools/fuzz/crashes.sh <target> <panic-site> FILE` and `tools/fuzz/differs.sh <tool> <style> FILE` are
predicates for `kgen reduce FILE <predicate...>` (greedy token-run deletion; keeps the input error-free if it was);
`kgen dump FILE` prints the PSI dump in `psi-dump.sh one` format. `fuzz.sh` uses `-s none` (no ASan: safe Rust, half
the speed) and an explicit host `--target` (this cargo-fuzz build defaults to musl). Fork jobs can hang in libFuzzer's
timeout handler (it deadlocks on the allocator lock); `fuzz.sh` kills them after the session.

Inputs with `(` nested deeper than 10 are skipped (`KNOWN_SLOW_PAREN_DEPTH`, finding 1); without that every target
drowns in timeouts. Since the finding-1 fix memory stays bounded, but time is still exponential, as upstream.

## Run

Five 30-minute sessions (plus a 2-minute smoke), 10 cores, 2.5 h of wall clock under the lock:

| Target | Execs | Coverage (edges / features) | Corpus | Artifacts |
|---|--:|--:|--:|--:|
| parser | 1.84 M | 6,324 / 39,414 | 14,352 | 8 crashes (finding 4), 11 slow/timeouts (finding 1) |
| ktfmt | 3.47 M | 14,188 / 77,762 | 18,100 | 13 crashes (findings 2, 6), 32 slow/timeouts (findings 1, 5) |
| ktlint | 4.96 M | 20,144 / 103,248 | 19,043 | 2 crashes (not findings, see below), 36 slow/timeouts (finding 1) |

Corpus counts include the 7,340 seeds; coverage was still creeping up in the last session (ktlint +1.9% features).

Differential, 3 styles × 2 passes each:

| Run | Inputs | ktlint mismatches | ktfmt mismatches |
|---|--:|--:|--:|
| `m3000`: kgen on testdata | 3,000 | 0 | 3 files (findings 2, 3) |
| `corpus3000`: kgen on the 7 dev repos (≤ 6 KB) | 3,000 | 0 | 8 files (findings 3, 7) |
| `exp3000`: as corpus3000, `ktlint_experimental = enabled` | 3,000 | 0 | — |
| `fromcorpus`: fuzz corpora's clean entries + 500 kgen | 3,237 | not run yet | not run yet |

`fromcorpus` (2,737 error-free fuzzer-made entries: 5,519 of ktfmt's and 6,616 of ktlint's corpus parse cleanly,
seeds excluded) was still queued behind another session's held-out run when this was written; it runs by itself
under the lock and reports to testbox `~/work/fuzz/diff-fromcorpus.log` and `~/work/fuzz/diff/fromcorpus/out/report.md`.

ktlint: no lint row, `-F` output or exit code differed anywhere, idempotence pass included (19k–45k lint rows per
style per run). No uncaught panic in 4.96 M executions; both crash artifacts are
`anchorBefore == null || anchorBefore.getTreeParent() == parent` (`crates/ktrs-ast/src/composite_element.rs:56`),
IntelliJ's `LOG.assertTrue` text, and `differs.sh` finds no difference from the jar on either (all three styles).
On the minimized one (`class/** … */vararg/** … */:` + a backticked name; testbox `~/work/fuzz/min/l1.min.kt`) the jar
logs the same assertion and both report `Internal Error (rule 'standard:colon-spacing')`. The `ChameleonCache`
exactness claim held throughout.

## Status (2026-10-03): findings 1, 2, 6, 7 fixed; 3 kept by decision; 4, 5 open

- **1.** Marker ids are reused like IntelliJ's `MarkerPool` (separate free lists for start markers and error items, last
  freed first; message slots too). Peak memory is bounded by live markers: `val x = ` + `(a<`×14 3.96 GB → 9 MB, ×16
  9 MB (18 s); `a<` + `(`×22 4.97 GB → 9 MB. Trees unchanged (corpus-diff 6123/6123); parser throughput −1.9%.
  Regression test: `crates/ktrs-parser/tests/backtracking_memory.rs` (heap peak under 4 MB).
- **2.** A comment's `Doc.Tok` range is `[-1, 0)`, so `makeKToIJ` maps k = -1; the port's vector is now offset by one.
  Correction to the finding below: the jar keeps the leading space (`" // c"` → `" // c\n"`).
- **3.** Kept: ktrs formats Kotlin 2.4 syntax the 0.64 jar rejects (user decision; listed in the README limits).
- **6.** The unused import's `IMPORT_DIRECTIVE` swallows the trailing comment, so it is removed before its own `;`, and
  `StringBuilder.replace` throws `StringIndexOutOfBoundsException`, which the CLI doesn't catch. ktrs now raises the
  same exception as an error. `ktrs_syntax::caught_panic` (shared with ktrs-lint) silences panics `ktfmt` and
  `ktrs fmt` catch.
- **7.** Ported `visitElement`'s `catch (t: Throwable) { throw FormattingError(...) }`; output matches the jar apart from
  stack frames.
- Rerun: all 13 ktfmt crash artifacts pass; the m3000/corpus3000 repros match the jar except the 3 finding-3 files.
- **New (open):** ktlint `crates/ktrs-ast/src/text.rs:139` slices inside a multi-byte char (U+2029), testbox
  `~/work/ktrs-fz/fuzz/artifacts/ktlint/crash-5117c70b…`. Two ktlint artifacts hit IntelliJ assertion texts
  (`composite_element.rs:56`, `:103`), not yet checked against the jar.

## Findings (by severity)

### 1. High — parser memory is exponential where upstream's is bounded

`val x = a<` followed by `(` × n, or `val x = ` followed by `(a<` × n: the type-argument backtracking is exponential
in time upstream too, but ktrs's memory grows with it.

| Input | ktrs (`ktfmt` CLI, dist) | Kotlin 2.4.20 PSI (`psi-dump.sh one`) |
|---|---|---|
| `val x = a<` + `(`×22 (32 B) | 5.5 s, 4.3 GB | 11.9 s, 430 MB |
| `val x = a<` + `(`×24 | 28 s, 15.6 GB | — |
| `val x = ` + `(a<`×14 | 2.8 s, 4.0 GB | 8.0 s, 429 MB |
| `val x = ` + `(a<`×16 (56 B) | 38 s, 24.7 GB | 62 s, 430 MB |

Every tool parses first, so a 56-byte file (with or without parse errors) can OOM `ktfmt`, `ktlint` and `ktrs serve`.
Cause, by reading: `crates/ktrs-parser/src/builder/production.rs` never reuses marker ids ("a freed marker just gets
`lexeme = -1`"), so `markers` grows with every marker ever allocated, rolled-back attempts included; IntelliJ pools
them, bounding memory by live markers. Also hit by mutated real code: unterminated strings that leave many `(` open
(1.2 KB → 1.1 GB).

### 2. Medium — `ktfmt` panics on a comment-only file with stray whitespace

| Input (no trailing newline) | ktrs | jar |
|---|---|---|
| `" // c"`, `"/* c */ "`, `" /* c */"` | panic at `crates/ktrs-fmt/src/doc/input_output.rs:74` (`index out of bounds: the len is 0 but the index is 18446744073709551615`), exit 1, file untouched, Rust panic on stderr | `// c\n` / `/* c */\n`, exit 0 |

Also `"  // comment  // comment"` (diff run m3000) and other comment-plus-whitespace endings the fuzzer found
(testbox `~/work/fuzz/min/f-*.min.kt`). `// c`,
`/** d */ `, `/* c */\t`, and anything with a final newline are fine. `make_k_to_ij` indexes its `Vec` by token index
`k`, which is -1 here; upstream's `Map<Integer, Range>` takes -1 as a key. 12 of the 13 ktfmt crash artifacts.

### 3. Medium — ktfmt accepts Kotlin 2.4 syntax that ktfmt 0.64 rejects

ktfmt 0.64 bundles kotlin-compiler 2.3.20 (`META-INF/compiler.version` in the jar); ktrs-fmt parses with the 2.4.20
grammar.

| Input | ktrs | jar |
|---|---|---|
| `class A {\n    companion {\n        fun baz() {}\n    }\n}\n` | formats, exit 0 | `2:14: error: Expecting member declaration`, exit 1 |
| `fun f(b: B) {\n    val [key, value] = b\n}\n` | formats | `2:20: error: Expecting a name` |
| `fun f(l: L) {\n    l.forEach { [key, value] -> }\n}\n` | formats | `2:28: error: Expecting a name` |

`(val key, val value) = b` and `[val key, val value] = b` fail identically on both sides (FormattingError
`expected token: 'val'; generated key instead`). ktlint is unaffected (its jar bundles 2.4.10).

### 4. Low — the parser silently drops tokens where IntelliJ asserts

When a lazy `BLOCK` re-parse stops before its last token, IntelliJ's `PsiBuilderImpl.prepareLightTree` throws
`AssertionError: Tokens [RPAR] were not inserted into the tree`; ktrs builds a tree whose text is missing them.
Minimal: `{fun<)]<T:@( {})` (16 B; `kgen dump` shows `tree text == input: false`). All 8 parser crash artifacts are
this class, and in each the JVM asserts too (`.kt` or `.kts`). Only reachable with parse errors, so both CLIs reject
the file: ktfmt prints the same parse error on both sides (the jar also logs the assertion); the ktlint jar dies with
`ExecutionException: AssertionError` for the whole run while ktrs reports `Not a valid Kotlin file` for that file.

### 5. Low — exponential re-indent of adjacent block comments (upstream behaviour, mirrored)

`    /*\n      * x\n    */` followed by `/*\n      * x\n    */` × (n-1): each `*//*` shifts the next comment's lines by
the previous column, so the output grows 4× per 2 comments. Byte-identical to the jar up to n=14 (271 B → 229 KB).
The fuzzer's 1,282-byte case: ktrs writes 369 MB (2.6 s, 2.2 GB RSS, exit 0); the jar runs 4.8 s, 4.5 GB RSS and exits
1 with no output. Not a port bug; noted because it shows up as timeouts.

### 6. Low — `ktfmt` prints Rust panic messages where the jar fails silently

`import a; /* x */` (also `import a;// x\n`): both exit 1 and leave the file alone; the jar's stderr is empty
(ktfmt's CLI swallows exceptions other than parse/formatting errors), ktrs prints `thread '<unnamed>' panicked at
.../slice/index.rs ... range end index 9 out of range for slice of length 1` (from
`crates/ktrs-fmt/src/format/redundant_element_manager.rs:103`). Same gap research/21 closed for ktlint
(`engine::rule_panic`), on the ktfmt side; finding 2 prints the same way.

### 7. Low — multiple trailing lambdas in a top-level statement: different error text

`foo {} {}` as a top-level (script) statement, `.kt` or `.kts`; both exit 1:

- ktrs: `t.kts:1:8: error: Maximum one trailing lambda is allowed`
- jar: `t.kts:1:1: error: com.facebook.ktfmt.format.ParseError: 1:8: error: Maximum one trailing lambda is allowed`
  then `com.google.googlejavaformat.FormattingError: 1:1: ...`.

Inside a function body, both print `2:12: error: Maximum one trailing lambda is allowed`. 7 of corpus3000's 8 files.

### Not findings

- The ktfmt jar aborting the run on one `FormattingError` (other files left unformatted, which ones depends on
  thread timing) while ktrs formats the rest: ktrs keeps upstream's per-file messages and exit code; matching the
  abort would mean matching a race.
- Idempotence: no pass-2 mismatch on either tool beyond consequences of pass-1 ones.
