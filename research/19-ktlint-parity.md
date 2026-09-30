# 19 — ktlint 2.0.0-ALPHA-4 parity on the corpus, all rules, three code styles (2026-09-30)

ktrs-lint (all 105 standard rules = the jar's whole `StandardRuleSetProvider`, checked by the probe's
"standard rules not in --rules: []") vs the real engine, on the 6,123 corpus files at `corpus/REVISIONS`.
Compared per file: lint rows (multiset), format callback rows (emit order), formatted bytes, `internal:ktlint-suppression`
rows, and, where ktlint's format throws, the exception (rule id + message).

## Result

| Run | Files | Lint identical | Format identical | Format throws the same | Both reject (parse) | Diffs |
|---|---|---|---|---|---|---|
| ktlint_official | 6,123 | 6,121 | 6,057 | 64 / 64 | 2 / 2 | 0 |
| intellij_idea | 6,123 | 6,121 | 6,057 | 64 / 64 | 2 / 2 | 0 |
| android_studio | 6,123 | 6,121 | 6,057 | 64 / 64 | 2 / 2 | 0 |
| ktlint_official + `ktlint_experimental = enabled` | 6,123 | 6,121 | 6,057 | 64 / 64 | 2 / 2 | 0 |

- "Lint identical" includes the 64 crash files: ktlint's lint does not throw on them (probe `lint-failed.tsv`: all
  `none`), and their lint rows match too. The 2 parse rejects are the Exposed `sourceFiles/*.kt` templates.
- Every file is accounted for: 6,057 byte-identical + 64 identical crashes + 2 identical rejects = 6,123 per run.
- No KDoc-line suspects and no other diff attributable to the 2.4.10 (jar) vs 2.4.20 (our parser) pin in any run.

## Oracle facts

| Run | Lint rows | Files with lint | Rules firing | Format rows | Files changed | Non-convergent (3 passes) |
|---|---|---|---|---|---|---|
| ktlint_official | 213,007 | 5,209 | 74 | 408,484 | 4,817 | 427 |
| intellij_idea | 156,437 | 4,914 | 59 | 180,015 | 4,033 | 5 |
| android_studio | 222,285 | 5,402 | 65 | 232,221 | 4,723 | 132 |
| ktlint_official + experimental | 216,232 | 5,229 | 81 | 410,679 | 4,847 | 421 |

- Experimental adds 7 firing rules on the corpus: blank-line-before-{file-annotation,imports,package},
  call-expression-wrapping, expression-operand-wrapping, lambda-return, no-blank-line-at-start-of-file.
- The corpus has no `internal:ktlint-suppression` rows in any run (no stale/unknown suppressions), so that rule's
  parity rests on the goldens (`tests/golden-passing.txt`), not on this diff.
- 64 files throw `standard:indent IllegalArgumentException: Stack should be empty` in format in every style (same file
  set in all four runs; the messages, i.e. the left-over contexts and their indents, differ per file and style).

## Diffs fixed

| Diff | Files | Cause | Fix |
|---|---|---|---|
| `Stack should be empty` message printed Rust `Debug` (`IndentContext { from_ast_node: NodeId(0), ... }`) | 64 per style | rule (exception text) | `IndentContext` prints like the Kotlin data class; nodes via `ASTNode.toString()` = `ktrs_ast::psi::ast_node_to_string` (jar: `Element(T)` composites, bare `T` for the lazy BLOCK/LAMBDA_EXPRESSION/DOC_COMMENT, `PsiWhiteSpace`, `PsiComment(T)`, `PsiElement(T)`); test `crates/ktrs-lint/tests/indent_crash.rs` |

No lint-row, format-row or formatted-byte diff was found in any run (an earlier all-rules ktlint_official oracle from
research/18 also matched on lint).

## Remaining diffs by cause

None: rule 0, engine 0, editorconfig defaults 0, parser pin 0.

## Harness changes (this round)

- `ktlint-probe.sh`: `KTLINT_EXPERIMENTAL=<value>` adds `ktlint_experimental` to the staged `.editorconfig`.
- `KtlintProbe`: when format throws, lints the file anyway (rows to `lint.tsv`, outcome to `lint-failed.tsv`); prints
  standard rules missing from `--rules`.
- `cargo lint-diff`: `--experimental` (stages the same property; default oracle `target/ktlint-oracle/<style>-experimental`,
  report `target/lint-diff-<style>-experimental.txt`); oracle-crash files are no longer skipped — ours must throw the
  same exception in format, and lint rows are compared.

## Reproduce

Oracles (JVM, testbox, ~4 min each, background; runner `~/work/ktlint-probe/parity-oracles.sh`, log `parity.log`):
```
R=$(tr -d '\n' < ~/work/ktlint-probe/parity-rules.txt)   # the ported ids, as lint-diff prints them
KTLINT_CODE_STYLE=<style> flock ~/bench.lock taskset -c 0-4,6-10 tools/ktlint-oracle/ktlint-probe.sh corpus out/parity-<style> --rules $R --threads 10
KTLINT_CODE_STYLE=ktlint_official KTLINT_EXPERIMENTAL=enabled ... out/parity-ktlint_official-experimental ...
```
Copy each (`tar --anchored --exclude=<dir>/src`) to `target/ktlint-oracle/<style>[-experimental]/`, then:
```
cargo lint-diff ktlint_official | intellij_idea | android_studio
cargo lint-diff ktlint_official --experimental
cargo test -p ktrs-lint --release --test indent_crash --test golden
```
