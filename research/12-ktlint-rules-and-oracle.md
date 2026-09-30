# 12 — ktlint standard rules: inventory, port order, test extraction, corpus oracle

Measured 2026-09-30 from ktlint **1.8.0** (`c26a4ff`, 2025-11-14, scratch clone) and **master** `3820242`
(2026-09-24, `corpus/ktlint`, 2.0 line). Rule counts, LOC and API surface are in 04 §2. CLI, config and MVP are in 08 §4.
Per-rule table: `research/12-ktlint-rules.tsv` (one row per rule id, both versions). Regenerate it with:
```
py -3 <scratchpad>/ktlint_inventory.py <ktlint-1.8.0> corpus/ktlint research/12-ktlint-rules.tsv
py -3 <scratchpad>/param_count.py <ktlint-1.8.0>/ktlint-ruleset-standard/src/test   # parameterized-test expansion
```

The scripts are regex scans of rule headers: `StandardRule(id, visitorModifiers, usesEditorConfigProperties)`,
`Rule.Experimental/OfficialCodeStyle/OnlyWhenEnabledInEditorconfig`, `ifAutocorrectAllowed`, and `@Test` counts per
`<Rule>Test.kt`. Master is diffed after mapping the `io.github.ktlint.core` package back to `com.pinterest.ktlint`, stripping
the `…20` helper suffixes, and dropping imports.

## TL;DR

- **1.8.0: 101 rule ids**. 82 autocorrect and 19 lint-only. 2 are deprecated no-ops (`condition-wrapping`, `discouraged-comment-location`).
  1 is experimental (`expression-operand-wrapping`), 12 run only in `ktlint_official`, and 1 is opt-in (`no-unused-imports`). The rule files
  total 18.4k LOC plus 208 in `internal/`, with **1,854 rule tests**. 04's "104 / 87 / 8 experimental" figures describe master.
- **Master: 106 ids.** 8 are new: 7 experimental plus `context-parameter-list-wrapping`, which renames `context-receiver-list-wrapping`.
  3 are removed. 54 rules are identical modulo the rename and 44 changed. The largest churn is in `indent` (+74/−67), `wrapping` (+80/−10),
  `when-entry-bracing` and `trailing-comma-on-declaration-site`.
- **Surprise 1: 2.0 deletes `VisitorModifier`.** `RunAfterRule` and `RunAsLateAsPossible` are gone. Rules run in alphabetical id order,
  and all rules run per node (`RuleProviderSorter`). In 1.8.0 the order is a topological sort of 35 edges, with LATE rules last.
  The same rule code therefore gives different output. **Pick one version per build**: the engine differs, the rules mostly do not.
- **Surprise 2: embedded Kotlin.** 1.8.0 embeds **Kotlin 2.2.21**, while master embeds **2.4.20**, which is our pin. Our PSI tree
  matches 2.0 by construction. For 1.8.0, every 2.2→2.4 PSI shape change becomes a potential ktlint diff, and files with 2.3+ syntax
  are parse errors in 1.8.0.
- **Surprise 3: rules are `ASTNode`-centric, not PSI-centric.** No rule has more than 13 lines touching `.psi`/`Kt*`, and only 9 rules have 5 or more.
  So the port needs `ASTNodeExtension.kt` (1,138 LOC) plus a mutable tree, and barely needs `ktrs_psi`.
- **Surprise 4: hard gates between rules.** `trailing-comma-on-call-site`/`-declaration-site` are **not run at all** unless `wrapping` is
  loaded and enabled, and the same holds for `string-template-indent` and `indent` (`ONLY_WHEN_RUN_AFTER_RULE_IS_LOADED_AND_ENABLED`).
  The port order must respect this.
- **Surprise 5: the CLI JSON reporter drops `FORMAT_IS_AUTOCORRECTED` errors**, so `-F --reporter=json` cannot attribute edits
  to rules. The corpus oracle has to call the engine API rather than only the CLI (§4).
- **Tests:** everything except 6 small files goes through `KtLintAssertThat`. Replacing that one file with a recording copy, plus a JUnit 5
  extension, should yield **≈2.3k cases** (§3).

## 1. Inventory (summary; full table in the TSV)

TSV columns: `rule_id file loc autocorrect experimental default_styles run_after editorconfig code_style_branch psi_lines
heavy_psi tests param_tests since master master_loc master_tests`.
- `default_styles`: `ko,ij,as`, `ko` (OfficialCodeStyle), `experimental`, or `opt-in`.
- `run_after`: `!` marks the ONLY_WHEN mode. `LATE` means RunAsLateAsPossible.
- `code_style_branch=y`: the rule reads `ktlint_code_style` itself.
- Rows marked `new-in-master` carry master's values.

| Class (1.8.0) | Rules | LOC | Tests | Lint-only |
|---|---|---|---|---|
| All three code styles | 87 | 15,291 | 1,551 | 18 |
| `ktlint_official` only | 12 | 2,556 | 245 | 1 |
| Experimental | 1 | 164 | 14 | 0 |
| Opt-in (`no-unused-imports`) | 1 | 341 | 44 | 0 |
| ≤100 LOC ("trivia") | 42 | 2,573 | 271 | |
| >400 LOC | 9 | 6,293 (34%) | 670 (36%) | |

The ko-only rules are `blank-line-before-declaration` (ko,as in master), `chain-method-continuation`, `if-else-bracing`, `if-else-wrapping`,
`multiline-expression-wrapping`, `no-blank-line-in-list`, `no-consecutive-comments`, `no-empty-first-line-in-class-body`,
`no-single-line-block-comment`, `string-template-indent`, `try-catch-finally-spacing` and `when-entry-bracing`. Code style also changes property
*defaults*, for example `max_line_length` and `ktlint_function_signature_rule_force_multiline…` (=2 for ko), so an `ij` run is not simply a subset of a `ko` run.

The lint-only rules are the naming rules (`backing-property-naming`, `class-naming`, `enum-entry-name-case`, `filename`, `function-naming`,
`package-name`, `property-naming`), plus `kdoc`, `max-line-length`, `mixed-condition-operators`, `no-consecutive-comments`, `no-empty-file`, `no-wildcard-imports`
and the four `*-comment` position rules.

**RunAfterRule graph (1.8.0; the only ordering constraints):**

| Rule | Runs after |
|---|---|
| annotation | enum-wrapping |
| wrapping | annotation |
| modifier-list-spacing | annotation, modifier-order |
| no-semi | wrapping |
| trailing-comma-on-call-site / -declaration-site | **wrapping!**, LATE |
| argument-list-wrapping | value-argument-comment, wrapping, class-signature, function-signature |
| class-signature | type-parameter-comment, value-parameter-comment, LATE |
| function-signature | type-parameter-comment, type-argument-comment, value-parameter-comment, context-receiver-wrapping, LATE |
| chain-method-continuation → function-literal | argument-list-wrapping → chain-method-continuation |
| indent | class-signature, function-signature, both trailing-comma rules, LATE |
| string-template-indent | **indent!** |
| block-comment-initial-star-alignment | indent |
| no-single-line-block-comment | comment-wrapping |
| max-line-length | both trailing-comma rules, LATE |

**Editorconfig reads:** `indent_size`/`indent_style` 39 rules, `max_line_length` 19, `ktlint_code_style` 8.
The rule-specific properties are:
- `ij_kotlin_allow_trailing_comma[_on_call_site]`, `ij_kotlin_imports_layout`, `ij_kotlin_packages_to_use_import_on_demand`
- `ij_kotlin_line_break_after_multiline_when_entry`, `ij_kotlin_indent_before_arrow_on_new_line`, `insert_final_newline`
- 10 `ktlint_*` properties: see the TSV `editorconfig` column

**Heavy PSI (≥5 lines):** `wrapping` 13, `trailing-comma-on-declaration-site` 11, `no-unused-imports` 9, `function-naming` 8,
`import-ordering` 7, `chain-wrapping` 6, and `multiline-if-else`/`multiline-loop`/`if-else-bracing` 5. Mostly these are `psi.getChildOfType`,
`KtPsiUtil`-style helpers and `KtNamedFunction` checks, all of which `ktrs_psi` already covers.

## 2. Port order

Measured 2.0.0-ALPHA-4 violation counts per rule on the corpus: research/17-ktlint-corpus-counts.md.

**Measure first (one testbox run per style, from the §4 API oracle):**
1. **Firing sets.** For each file, record `F(file)` = the set of rule ids that emitted anything during *format* (all runs). Use format
   rather than lint because format also catches cascades, where a rule fires only after another rule's fix.
2. **Share.** For each rule, count violations, files touched, and files where it is the *only* firing rule. Lint mode gives the
   violation counts (lint is rule-independent in 1.8.0), and the per-rule isolated format run gives changed lines (§4 `--isolate`).
3. **Coverage curve.** A file is byte-matchable with ported set P iff `F(file) ⊆ P`. Clean files need every rule to stay silent,
   so gate them through the P-restricted oracle (§4). Choose the order greedily: add the rule that maximises newly
   covered files per LOC, then check that the result respects the graph above. Report
   `coverage(P) = |{f : F(f) ⊆ P}| / |corpus|` per style.

The corpus is biased: ktlint and nowinandroid are already ktlint-clean, and ktfmt is ktfmt-formatted. So also report
coverage over the 4 other repos.

**Proposed order.** This is a hypothesis until the measurement runs. It is dependency-respecting and puts cheap items first.

| # | Step | Rules | ≈LOC | Why here |
|---|---|---|---|---|
| 0 | Engine (08 §5 C1) | 1.8 `RuleProviderSorter` topo order, `RuleExecutionRuleFilter` (style/experimental/opt-in), `@Suppress` + engine `ktlint-suppression` rule, `CodeFormatter` 3-run loop + `lintAfterFormat`, per-style property defaults, ASTNodeExtension | 3–4k | everything; the §3 harness runs on it with zero rules |
| 1 | Whitespace trivia | no-trailing-spaces, final-newline, no-consecutive-blank-lines, no-multi-spaces, no-blank-line-before-rbrace, no-empty-first-line-in-method-block, no-blank-lines-in-chained-method-calls, no-line-break-after-else | 0.5k | token-local, no deps, fire on unformatted code |
| 2 | Spacing batch | comma/dot/range/unary-op/nullable-type/double-colon/colon/curly/paren/op/keyword/square-brackets/angle-brackets/fun-keyword/function-*-spacing, then-spacing, annotation-spacing | 1.9k | leaf-adjacent rules; `curly-spacing` needs code-style handling |
| 3 | Lint-only batch | naming ×7, no-wildcard-imports, no-empty-file, kdoc, mixed-condition-operators, 4 `*-comment` rules | 1.3k | no edits, so parity is (line, col, message); the comment rules gate step 7 |
| 4 | Imports | import-ordering (+internal/importordering), no-unused-imports (opt-in) | 0.8k | high file coverage expected; layout parser has its own unit tests |
| 5 | Small structural | no-semi, modifier-order, no-unit-return, string-template, no-empty-class-body, statement-wrapping, spacing-between-declarations-*, blank-line-between-when-conditions, … | 1.5k | |
| 6 | Wrapping core | enum-wrapping → annotation → wrapping → modifier-list-spacing; comment-wrapping → no-single-line-block-comment | 1.7k | `wrapping` gates both trailing-comma rules |
| 7 | Trailing commas | trailing-comma-on-call-site, -declaration-site | 0.8k | default-on, widely violated (hypothesis) |
| 8 | Signatures | context-receiver-wrapping → function-signature, class-signature; parameter-list/argument-list/parameter/property wrapping | 3.0k | prerequisites of indent |
| 9 | Indent | indent → string-template-indent, block-comment-initial-star-alignment; then max-line-length (LATE) | 2.2k | largest rule, last in the graph |
| 10 | ko structure | chain-method-continuation → function-literal, multiline-expression-wrapping, if-else-*, when-entry-bracing, blank-line-before-declaration, function-expression-body, binary-expression-wrapping, chain-wrapping, multiline-if-else/loop, … | 3.0k | ko-only or behaviour-rich; many interplay with indent |

**Hard rules and why:**

| Rule | LOC / tests | Difficulty |
|---|---|---|
| indent | 1,597 / 257 (+140 tests used as an additional rule) | A stack of indent contexts over the whole file. Special cases: string templates, raw strings, KDoc, `when`/elvis/binary chains, where-clauses, tabs, and code-style branches. Runs LATE, after signatures and trailing commas. Changed most in 2.0. |
| wrapping | 719 / 90 | Many node-type heuristics and the most PSI use. `no-semi`, `argument-list-wrapping` and both trailing-comma rules are ordered after it or gated on it. |
| function-signature / class-signature | 834 / 65, 731 / 69 | Recompute single-line length from indent config and `max_line_length`, then re-wrap. The force-multiline defaults depend on code style. Comment-position rules must run first. |
| trailing-comma-on-declaration-site | 485 / 35 | Decides whether a list is multiline, and handles `when` entries and destructuring. PSI-heavy; runs LATE. |
| chain-method-continuation, function-literal | 530 / 46, 453 / 29 | ko-only chain layout, coupled with argument-list-wrapping. Both were rewritten in 2.0 (#3253). |
| annotation | 527 / 58 | Wrapping and joining of annotation arrays, file annotations and modifier interplay. |
| string-template-indent | 417 / 21 | Requires indent. Re-indents raw strings. |
| import-ordering, no-unused-imports | 283+208 / 29, 341 / 44 | IDEA layout semantics with ASCII versus IDEA sorting and a diacritics regex; name heuristics like ktfmt's. |
| max-line-length | 176 / 12 | Lint-only, but it judges the *final* text after every LATE rule, so its parity lags everything else. |

## 3. Test extraction (mirror `tools/ktfmt-oracle/extract-goldens.sh`)

Built for 2.0.0-ALPHA-4 as `tools/ktlint-tests/extract-goldens.sh`: 2,324 cases from 2,362 passing upstream tests. It
differs from the plan below in three ways. `.options` carries the engine's full override, including the forced properties.
`.lint`/`.format` use `line:col\trule\tauto|manual\tdetail`. There is no `.upstream` file: the upstream assertions stay live
instead. Runner: `crates/ktrs-lint/tests/golden/`.

**How tests are written (1.8.0 counts):**
- `val xAssertThat = assertThatRule { XRule() }`, or `assertThatRuleBuilder { … }.addAdditionalRuleProvider { … }.assertThat()`.
- Per snippet: `xAssertThat(code)[.withEditorConfigOverride(P to v)(473)][.setMaxLineLength()(177)][.asKotlinScript()(7)]`
  `[.asFileWithPath(p)(19)][.addAdditionalRuleProvider { R() }(264: indent 140, max-line-length 27, no-semi 21, wrapping 13, …)]`
  `[.addRequiredRuleProviderDependenciesFrom(StandardRuleSetProvider())(14)]`, then one of:
  - `.hasNoLintViolations()` (767)
  - `.hasLintViolation(l, c, msg)` (402) / `.hasLintViolations(LintViolation(...)…)` (497)
  - `…WithoutAutoCorrect` (117)
  - `…ForAdditionalRule(s)` (16)
  - `.hasNoLintViolationsExceptInAdditionalRules()` (27)
  - `.hasNoLintViolationsForRuleId(id)` (4)
  - chained `.isFormattedAs(expected)` (942)
- Harness semantics that must be reproduced:
  - The enabled set is exactly the rule plus the additional rules, with `ktlint_experimental=enabled` and that rule set `enabled` forced.
  - Code style defaults to `ktlint_official`. No `.editorconfig` is read (Jimfs).
  - `Code.fromSnippet` means there is no path unless `asFileWithPath` is used.
  - Lint uses max 1 run. Format uses 3 runs.
  - `hasNoLintViolations` also asserts that format is a no-op.
- The only other test files (6) are `ImportLayoutParserTest` and `ImportOrderingRuleTest` (a property-writer check). Port both as
  Rust unit tests. The rest are `RemoveDiacriticsFromLettersTest`, `ConditionWrappingRuleTest` (deprecated) and the provider/SinceKtlint
  meta tests. 11 tests are `@Disabled`.

**Extractor (`tools/ktlint-oracle/extract-tests.sh`, JVM, background, testbox):**
1. `tools/sync-ktlint.sh` shallow-clones tag `1.8.0` into `third_party/ktlint` (gitignored). It also fetches into `tools/ktlint-oracle/lib/`:
   - the release `ktlint` fat jar (an sh stub plus a zip, usable with `java -cp`)
   - the test deps: junit-jupiter 5.14.1, junit-platform-console-standalone 1.14.1, assertj, jimfs 1.3.1, kotlin-reflect 2.2.21
2. `extract/KtLintAssertThat.kt` is a copy of upstream's file with the same package and public API, with `@Poko` swapped for a
   data class. It gets two hooks:
   - `KtLintAssertThatAssertable.init` calls `CaseRecorder.record(ruleProvider, additionalRuleProviders, editorConfigOverride, code)`. This captures
     the code, script flag, path, rule ids, and the EC map serialized via `property.name` + `propertyWriter(value)`.
   - Each assertion method appends its upstream expectation to `<case>.upstream`, and the upstream assertion stays live.
     A failing assertion flags a broken test, like ktfmt's DISAGREE.
3. The recorder computes the truth **in the same JVM** with the same `KtLintRuleEngine` configuration:
   - `lint` over *all* enabled rules, without filtering to the rule under test
   - `format` with a callback that records `(LintError, corrected)`, plus the output text and any thrown exception
4. `extract/CaseNaming.kt` is a JUnit 5 extension, auto-registered via `META-INF/services` and
   `-Djunit.jupiter.extensions.autodetection.enabled=true`. `BeforeEach` stores the nested-class chain, display name and invocation
   index in a ThreadLocal. Tests run with parallel execution disabled. Case names are sanitized like `KtfmtTruth.sanitize`, get
   `-2`, `-3`… on collision, and are capped at 120 characters.
5. Compile the upstream `ktlint-ruleset-standard/src/test` (and `ktlint-test` minus the swapped file) with `K2JVMCompiler`, as
   extract-goldens.sh does. Put the fat jar on the classpath and pass `-Xfriend-paths=<fat jar>` so tests can see `internal` symbols.
   Run with `ConsoleLauncher --select-package com.pinterest.ktlint.ruleset.standard`. Fallback if kotlinc cannot build the tests:
   a Gradle init script on the tag checkout that swaps the file and forwards `-Dgolden.out` (`ignoreFailures=true`).
6. Copy the result into `testdata/ktlint/<rule-id>/`. Add a Rust `golden` test in the future `ktrs_lint` crate with a ratchet
   `tests/golden-passing.txt`, the same as `ktrs_fmt`.

**On-disk format (`testdata/ktlint/<rule-id>/<case>.*`):**

| File | Content |
|---|---|
| `.input.kt` / `.input.kts` | snippet exactly as passed (after `trimIndent`); the extension carries the script flag |
| `.options` | `rules=standard:indent,standard:wrapping` (first = under test, then additional in insertion order), optional `path=/project/src/Foo.kt`, then `ec.<name>=<value>` lines (including the forced `ec.ktlint_experimental=enabled`) |
| `.lint` | `line:col\t<rule-id>\t<auto\|manual>\t<detail>`, engine order (sorted by line, col; ties in rule execution order) |
| `.expected.kt[s]` | formatted text, only when ≠ input (absent = idempotent) |
| `.format` | format-callback errors, `line:col\t<rule-id>\t<corrected y\|n>\t<detail>` (positions may refer to run-2/3 text; see §4) |
| `.error` | exception class + message when lint/format throws (parse errors, rule crashes) |

**Yield:**
- 1,854 rule tests, minus 113 parameterized templates, plus 268 literal-source values, plus ≈53 method/enum sources × ~4,
  gives ≈2.2k invocations.
- Some tests assert several snippets, which brings the total to **≈2.3k cases**. That is ~97% of rule tests.
- The losses are the 11 `@Disabled` tests and the 6 non-AssertThat files.
- Master: the same recorder in package `io.github.ktlint.core.test` gives ≈2.45k cases (1,981 tests). Keep the two versions in separate trees
  (`testdata/ktlint-1.8/`, `testdata/ktlint-2.0/`) because rule order differs.

## 4. Corpus oracle (`tools/ktlint-oracle/ktlint-oracle.sh <ko|ij|as> <in> <out> [--rules a,b] [--isolate]`)

- **Staging:**
  - Copy only `*.kt`/`*.kts`, preserving the tree, as `ktfmt-oracle.sh` does. This drops the corpus repos' own
    `.editorconfig` files: 6 repos have one, and ktlint would otherwise honour them. `--editorconfig` only supplies *defaults*.
  - Write `<out>/src/.editorconfig`: `root = true`, then `[*.{kt,kts}]` with `ktlint_code_style = ktlint_official|intellij_idea|android_studio`.
  - With `--rules`, also write `ktlint_standard = disabled` plus `ktlint_standard_<id> = enabled` for each listed rule. The script
    auto-adds ONLY_WHEN deps such as wrapping for trailing-comma.
- **API oracle (`KtlintOracle.kt`, the primary gate).** It runs on the fat jar with `KtLintRuleEngine(StandardRuleSetProvider().getRuleProviders())`
  and `Code.fromFile`, so the path-dependent `filename` and `package-name` rules behave as they do on disk. Output per file:
  - `lint/<rel>.lint`, `fmt/<rel>` and `fmt/<rel>.format`, in the §3 formats
  - `.failed` for parse exceptions, and `.nonconvergent` for files that hit 3 runs
  - `--isolate` repeats the format pass once per autocorrect rule (plus its `!` deps) for the §2 edit attribution
    (~85 × 6.1k files; overnight on testbox)
- **CLI cross-check (secondary gate for `ktrs` CLI parity):**
  - Lint: `cd <out>/src && java -jar ktlint --relative --reporter=json,output=../cli-lint.json '**/*.kt' '**/*.kts'`
  - Format: the same on a copy, with `-F`
  - This must agree with the API oracle modulo reporter filtering. Later it is also the oracle for the reporters and exit codes.
- **Diff (`cargo lint-diff <style>`, xtask like `fmt-diff`):**
  - (a) **Violation multiset** per file, keyed on `(line, col, rule, detail, auto)` and unordered. This is the rule-parity metric.
  - (b) **Exact order** as a separate count. This is CLI parity.
  - (c) **Formatted bytes** versus `fmt/`.
  - (d) The `.format` remaining-error set.
  - Summary line: identical, differ, panic, rejected-mismatch, both-rejected, no-oracle. With a restricted rule set P, (a) can be computed from the full lint
    oracle by filtering to P, because lint is per-rule independent. (c) needs the oracle rerun with `--rules P`.

**Known non-determinism and ordering hazards:**

| Issue | Handling |
|---|---|
| Ties at the same (line, col): order = rule execution order (1.8 topo+LATE, 2.0 alphabetical) | compare as multiset for (a); exact for (b) with the version's sorter |
| Format errors are a `Set<Pair<LintError,Boolean>>` collected over up to 3 runs, then sorted; run 2/3 positions refer to *intermediate* text; `LintError` equality ignores `canBeAutoCorrected` | reproduce the loop exactly (3 runs, stop when unchanged, `lintAfterFormat` on the 3rd); never "until fixed point" |
| Non-converging files (3 runs exhausted) and "Format was not able to resolve…" warnings go to the logger, interleaved across CLI threads | take them from the API oracle's `.nonconvergent`, never from stderr |
| CLI processes files in parallel; JSON reporter sorts files by path string; plain reporter does not | use json; run on Linux (testbox) so paths use `/` and no autocrlf |
| Line endings: output follows `end_of_line` or the detected input separator | stage LF-only (git on Linux); add a CRLF subset later |
| BOM: 1.8.0 strips a UTF-8 BOM, 2.0 keeps it (#3264) | version-specific branch |
| 1.8.0 parses with Kotlin 2.2.21 (we pin 2.4.20): new syntax becomes `KOTLIN_PARSE_EXCEPTION`, and PSI shape may differ | `.failed` list; treat both-rejected as a pass; count "we parse, 1.8 rejects" separately |
| Engine `ktlint-suppression` rule rewrites legacy `ktlint-disable` comments during format | part of step 0; no corpus file outside ktlint's own engine contains `ktlint-disable` |
| Property defaults per code style (max_line_length, force-multiline…) | style is only set via `ktlint_code_style`; never pin the derived props |

**Version decision** (08 §4 recommends 1.8.x): the rule code is largely shared, the engine is not. Build the engine with a
`RuleOrder::{Topo18, Alpha20}` and traversal switch and gate both: 1.8 primary (Spotless pins it); 2.0 gets PSI parity free (same Kotlin pin) plus 44 changed + 8 new rules.
