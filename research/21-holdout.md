# 21 — Held-out corpus parity (2026-10-02)

Every corpus diff before this ran on the 7 dev repos (corpus/REVISIONS) that drove the fixes, two of them ktlint's and
ktfmt's own sources. This run uses 20 repos never looked at while porting, pinned in `tools/holdout/REVISIONS`:
arrow, koin, sqldelight, detekt, wire, coil, compose-samples, mockk, kotest, kotlinx.serialization, kotlinx-datetime,
leakcanary, mosaic, architecture-samples, mavericks, accompanist, thunderbird-android, apollo-kotlin, kotlinpoet,
circuit. 15,287 `.kt`/`.kts` files, 145 MB (2.5x the dev corpus by files, 4.7x by bytes).

Method: the CLIs as users run them, on the whole tree, cwd = the tree, one root `.editorconfig` per style.
`tools/holdout/fetch.sh` (clone at the pins, copy only `.kt`/`.kts`), `tools/holdout/run.sh` (calls
`tools/parity/ktlint-compare.sh` and `tools/parity/ktfmt-compare.sh`). ktrs at 0.3.1 (`--profile dist`) vs the
ktlint 2.0.0-ALPHA-4 release asset and the ktfmt 0.64 jar. Raw output: testbox `~/work/parity/out/holdout/`.

## Results (first run, before any fix)

| Tool | Style | Compared | Differing |
|---|---|--:|--:|
| ktlint lint | ktlint_official | 821,797 rows | 0 |
| ktlint lint | intellij_idea | 617,645 rows | 0 |
| ktlint lint | android_studio | 779,719 rows | 0 |
| ktlint `-F` | ktlint_official | 15,287 files | 3 |
| ktlint `-F` | intellij_idea | 15,287 files | 0 |
| ktlint `-F` | android_studio | 15,287 files | 0 |
| ktfmt | meta, google, kotlinlang | 15,287 files each | 0 |

Exit codes equal everywhere. ktfmt: stderr equal (sorted); both reject the same file
(kotest-intellij-plugin `behaviorspec.kt`, a test resource with a syntax error) with the same message.

## Found and fixed

- **`when-entry-bracing` margin stripping (the 3 files).** Upstream rebuilds the braced entry from
  `"""|when {|$whenEntry|}""".trimMargin()`; trimming after interpolation also strips `|` margins inside the entry's own
  raw strings (wire `Roots.kt` ×2, kotest `CommutativeEquality.kt`). The port built the text without `trim_margin`.
  Found by disabling one rule at a time on the jar. Regression test: `crates/ktrs-lint/tests/when_entry_bracing_margin.rs`.
  The other upstream interpolate-then-trim sites (`KtlintSuppression.createAnnotatedExpression`,
  `StringTemplateRule.toShortStringTemplateNode`) were already right or interpolate only identifiers.
- **Rust panic messages on stderr.** On the 412 `-F` files where ktlint's `indent` throws "Stack should be empty",
  the default panic hook printed `thread '<unnamed>' panicked at ...` to stderr; the jar's stderr is empty. The engine
  now catches rule panics through `engine::rule_panic`, and both lint entry points install a hook that stays quiet
  for caught ones. stdout still differs on those files as before (the jar's Java stack frames, research/20).

Both verified on testbox: the patched binary matches the jar's trees on the 3 files, and stderr is empty on a crash file.

## Reading

Lint rows have no miss on 2.2M held-out rows; format misses were 3 files in 15,287, from one rule. The dev-corpus
results were not overfitted to a measurable degree. Rerun after any engine or rule change that the dev corpus might
not exercise, and pick fresh repos (or new pins) now and then, since this set is no longer unseen once fixes come from it.
