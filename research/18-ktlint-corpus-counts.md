# 18 — ktlint 2.0.0-ALPHA-4 on the corpus: violations per rule (phase-2 port order input) (2026-09-30)

Oracle: `KtlintProbe` (tools/ktlint-oracle) with all standard rules, `ktlint_code_style = ktlint_official`, on the 6,123
corpus files at `corpus/REVISIONS` (testbox). Regenerate:
```
KTLINT_CODE_STYLE=ktlint_official tools/ktlint-oracle/ktlint-probe.sh corpus target/ktlint-oracle/ktlint_official   # JVM, ~5 min on testbox
cargo lint-diff ktlint_official --counts
```

## Run facts

- 210,933 lint violations from 73 of the rules, in 5,145 files. Format changed 4,817 files.
- 427 files do not converge in 3 format runs. 97 files end with a mutated tree that differs from a fresh parse.
- **`standard:indent` crashes on 64 files** (`IllegalArgumentException: Stack should be empty`, mostly `.gradle.kts`). The
  engine then reports no result for the file. `lint-diff` skips these files (`oracle-crash`); a port has to reproduce the crash,
  not fix it.
- 2 parse failures: the Exposed `sourceFiles/*.kt` templates.

## Top 30 by lint violations

Lint mode reports each rule on the unformatted text, independently of the others. "Files" counts files with at least one
violation. The lint-only rules are `no-wildcard-imports`, `max-line-length`, `property-naming`, `filename`, `function-naming`
and `kdoc`; `if-else-wrapping` is partly autocorrectable. Every other row is fully autocorrectable.

| # | Rule | Violations | Files |
|---|---|---|---|
| 1 | indent | 119,949 | 858 |
| 2 | function-signature | 22,685 | 2,969 |
| 3 | multiline-expression-wrapping | 18,172 | 2,834 |
| 4 | no-wildcard-imports | 13,233 | 3,152 |
| 5 | trailing-comma-on-call-site | 5,432 | 1,454 |
| 6 | chain-method-continuation | 5,365 | 995 |
| 7 | class-signature | 4,222 | 1,054 |
| 8 | argument-list-wrapping | 4,146 | 232 |
| 9 | trailing-comma-on-declaration-site | 3,643 | 1,440 |
| 10 | no-empty-first-line-in-class-body | 1,594 | 1,393 |
| 11 | function-expression-body | 1,491 | 684 |
| 12 | blank-line-before-declaration | 1,345 | 457 |
| 13 | when-entry-bracing | 1,274 | 345 |
| 14 | max-line-length | 1,109 | 246 |
| 15 | blank-line-between-when-conditions | 1,058 | 215 |
| 16 | import-ordering | 801 | 799 |
| 17 | string-template-indent | 747 | 69 |
| 18 | wrapping | 594 | 176 |
| 19 | statement-wrapping | 516 | 94 |
| 20 | function-literal | 307 | 125 |
| 21 | parameter-list-wrapping | 297 | 71 |
| 22 | multiline-if-else (ported) | 234 | 96 |
| 23 | property-naming | 217 | 85 |
| 24 | filename | 199 | 199 |
| 25 | annotation | 190 | 95 |
| 26 | function-naming | 186 | 52 |
| 27 | colon-spacing | 172 | 117 |
| 28 | if-else-wrapping | 149 | 64 |
| 29 | kdoc | 129 | 44 |
| 30 | no-multi-spaces | 128 | 88 |

For comparison, the other two ported rules are `no-semi` (6 violations) and `comma-spacing` (5).

## Reading it for the port order

- The ko-only layout rules dominate: `function-signature`, `multiline-expression-wrapping`, `chain-method-continuation`,
  `class-signature` and the two trailing-comma rules each fire in 1,000–3,000 files. The corpus is mostly not in ktlint_official style.
- `indent` has 57% of all rows but fires in only 858 files, about 140 rows per file.
- File coverage is the metric for byte-identical format (research/12 §2). A file is matchable only when every rule that fires
  on it is ported. So `no-wildcard-imports` (3,152 files, lint-only, cheap) and `import-ordering` (799 files) unlock more files
  per LOC than their row counts suggest.
- The per-rule isolated format runs (`--isolate`) are in `target/ktlint-oracle/ktlint_official/isolate.tsv`.
