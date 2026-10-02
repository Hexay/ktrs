# 22 — ktlint 1.x vs 2.0 gap (2026-10-02)

ktrs ports ktlint 2.0.0-ALPHA-4; most users run 1.8.0. Both jars ran over the 6,123-file bench corpus on testbox with
`tools/parity/ktlint-compare.sh A=ktlint-1.8.0 B=ktlint-2.0.0-ALPHA-4` (cwd = tree, `--relative`, root `.editorconfig`
sets only `ktlint_code_style`; lint rows diffed, then `-F` on fresh copies and the trees diffed). Raw output:
`testbox:~/work/parity/out/kt18-vs-kt2/<style>/` (`only.a`/`only.b`, `rules-diff.txt`, `format-files.txt`, `format.diff`,
`format.{a,b}.out`), summary `../summary.md`. Upstream history: `testbox:~/work/ktlint-src` (clone of ktlint/ktlint),
`git log 1.8.0..2.0.0-ALPHA-4` = 183 commits, ~40 non-dependency/non-release. Kotlin compiler: 2.2.21 (1.8.0) vs 2.4.10.

## Rows differing per rule (only 1.8 / only 2.0)

| rule | ktlint_official | intellij_idea | android_studio | cause (below) |
|---|--:|--:|--:|---|
| argument-list-wrapping | 1001/0 | – | 8646/0 | R1 + R2 |
| function-literal | 232/0 | – | 1581/0 | R1 (+ R2) |
| blank-line-between-when-conditions | 58/24 | 58/24 | 58/24 | R4 |
| chain-method-continuation | 11/3 | – | – | R2 (11) + R1 off-by-one (3) |
| function-signature | 8/12 | – | 2/53 | R2 |
| binary-expression-wrapping | – | – | 13/0 | R2 |
| wrapping | – | – | 3/0 | R2 |
| class-signature | – | – | 0/3 | R2 |
| context-receiver-list-wrapping → context-parameter-list-wrapping | – | – | 4/0 + 0/4 | R5 (rename, same rows) |
| blank-line-before-declaration | – | – | 0/1367 | R3 |
| no-multi-spaces | 1/0 | 1/0 | 1/0 | R7 (compiler) |
| **format: files differing** | 338 | 96 | 1346 | see "Format diffs" |

Lint exit codes equal (1/1) in every style. Same-set rows also differ in *order* in 186 places (two rules at the same
line:col): ties follow rule execution order, which changed (E1).

## Rule-level causes

**R1 — "exceeds max line length" now measures only up to the node's own closing token.**
[#3252](https://github.com/ktlint/ktlint/pull/3252) (729a0fe2) and [#3253](https://github.com/ktlint/ktlint/pull/3253) (152b35d4).
- argument-list-wrapping 1.8: `leavesOnLine.dropTrailingEolComment().lineLength > max` (whole line). 2.0: leaves up to and
  including the list's `RPAR`. Lines that only overflow *after* `)` (trailing lambda, `.chain()`, outer call's `)`) no
  longer fire. Sample: `assertEqualLists(emptyList(), dest.selectAll().where { ... }.toList())` → 1.8 fires on the inner lists.
- function-literal 1.8: whole line (minus EOL comment); 2.0: up to the lambda's `RBRACE`. Sample:
  `testTable.selectAll().where { testTable.id eq cairoId }.single()[...]` → 1.8 wants `{`/`}` wrapped.
- chain-method-continuation: `takeWhile { it != stopAtLeaf }` → `takeWhile { it.prevLeaf != stopAtLeaf }` (now *includes*
  the stop leaf: off-by-one). The 3 only-2.0 rows are lines of exactly 141/142 chars with max 140.
- 1.x mode: **small switch** — three `exceeds_max_line_length` bodies get a 1.x branch (old expression is one line each).

**R2 — max-line-length suppressions now disable wrapping decisions.**
[#3255](https://github.com/ktlint/ktlint/pull/3255) (00dd1417): new `hasNoMaxLineLengthSuppression()` (node or any parent
carries `@Suppress("ktlint:standard:max-line-length")`, file-level included) ANDed into every "line too long" test in
argument-list-wrapping, binary-expression-wrapping, call-expression-wrapping, chain-method-continuation, class-signature,
context-receiver(-list)-wrapping, function-literal, function-signature, parameter-list-wrapping, parameter-wrapping,
property-wrapping, wrapping. In suppressed files 1.8 still wraps (only-1.8 rows) while 2.0 treats the line as fitting and
so the signature rules want to *join* lines (only-2.0 rows: `No whitespace expected between opening parenthesis and first
parameter name`, `Single whitespace expected before parameter`). All function-signature/class-signature/binary-expression/
wrapping rows sit in files with that suppression (ktor `@file:Suppress`, ktlint's own tests). Of the
argument-list-wrapping only-1.8 rows, 516/1001 (official) and 700/8646 (android) are in files containing the suppression
(upper bound; the rest are R1).
- 1.x mode: **small switch** — `Ast::has_no_max_line_length_suppression` returns `true` in 1.x mode (one site).

**R3 — blank-line-before-declaration enabled for android_studio.**
[#3318](https://github.com/ktlint/ktlint/pull/3318) (a4bd0b74): rule lost the `RuleV2.OfficialCodeStyle` marker (1.8:
ktlint_official only, unless explicitly enabled) and gained `beforeFirstNode { if intellij_idea -> stopTraversalOfAST() }`.
So 2.0 fires on android_studio by default and can no longer be enabled for intellij_idea at all.
- 1.x mode: **small switch** — `is_official_code_style() = true` and skip the intellij guard.

**R4 — blank-line-between-when-conditions and trailing EOL comments.** Two fixes:
- [#3177](https://github.com/ktlint/ktlint/pull/3177) (151cd8d0): `isPrecededByComment()` now counts only a comment on its
  own line; a trailing `// ...` on the previous entry no longer makes the `when` "multiline" (the extra only-1.8 rows).
- [#3261](https://github.com/ktlint/ktlint/pull/3261) (533bd9ae): the whitespace checked/fixed is the first newline
  whitespace after the previous entry's EOL comment, not the whitespace right after the previous code sibling. Same
  violation moves from `691:62` (before `// For SQLite`) to `692:1`; format keeps the comment on its line (1.8 moved
  the comment below the blank line — visible in format.diff for ColumnType.kt/References.kt).
- 1.x mode: **small switch** — two conditions (old `isPrecededByComment`, old `findWhitespaceAfterPreviousCodeSibling`).

**R5 — `context-receiver-list-wrapping` renamed `context-parameter-list-wrapping`.**
[#3367](https://github.com/ktlint/ktlint/pull/3367) (1d32900d), follows the compiler's `CONTEXT_RECEIVER_LIST` →
`CONTEXT_PARAMETER_LIST`. No alias: 1.x `ktlint_standard_context-receiver-list-wrapping = disabled` and
`@Suppress("ktlint:standard:context-receiver-list-wrapping")` stop working on 2.0. Rows identical otherwise.
- 1.x mode: **small switch** — rule id string per mode (and accept the old id in suppressions/editorconfig).

**R6 — rule set membership.** Removed: `condition-wrapping`, `discouraged-comment-location`
([#3237](https://github.com/ktlint/ktlint/pull/3237)) — both were empty deprecated stubs in 1.8 (no rows). Added, all
`EXPERIMENTAL` (only with `ktlint_experimental = enabled`): call-expression-wrapping (#3253), lambda-return (#3274),
blank-line-before-{file-annotation,imports,package} (#3266/#3319/#3350/#3355), no-blank-line-at-start-of-file (#3361).
1.8's only experimental rule is expression-operand-wrapping. No stable/experimental flips; no `.editorconfig` property
added, removed or re-defaulted (diffed every `ktlint_*`/`ij_kotlin_*` literal and `*CodeStyleDefaultValue`).
- 1.x mode: **small switch** — registration list per mode (drop the 7 experimental rules; optionally register the 2 stubs).

**R7 — no-multi-spaces (1 row).** KDoc line ` *           ` (trailing spaces, TickerChannels.kt:51). Rule unchanged
(only a rename `isWhiteSpace20`→`isWhiteSpace`, same body); most likely the KDoc lexer of Kotlin 2.2.21 vs 2.4.10 splits
the whitespace differently. Not worth a switch; would need a 2.2 KDoc-lexer quirk.

**Zero-row fixes on this corpus** (behaviour differs only on rare input): paren-spacing EOL comment (#3236), annotation
before lambda (#3268), comment-spacing NPE (#3314), class-signature supertype after EOL comment (#3312), indent tabs before
`where` (#3363), positional destructuring in spacing-around-square-brackets/trailing-comma-on-declaration-site (#3365),
lambda-return labels (#3358, experimental), no-unused-import kept opt-in (#3192, doc-only).

## Engine-level changes

**E1 — rule execution order: node-major instead of rule-major.** [#3252](https://github.com/ktlint/ktlint/pull/3252)
(729a0fe2, "possibly breaking"). 1.8: `RuleProviderSorter` orders rules (standard first, then id, then `VisitorModifier`s
`RunAfterRule(id, mode)` / `RunAsLateAsPossible`, enforced by `RunAfterRuleFilter`), and each rule traverses the whole AST
before the next one starts. 2.0: all rules run on a node before moving to the next node; `VisitorModifier`s are gone
(removed from argument-list-wrapping, annotation, block-comment-initial-star-alignment, chain-method-continuation,
class-signature, function-literal, function-signature, indent, max-line-length, modifier-list-spacing, no-semicolons,
no-single-line-block-comment, string-template-indent, trailing-comma-on-{call,declaration}-site, wrapping). Lint rows
are unaffected (nothing mutates), except tie order at equal line:col (186 rows). Format output is affected everywhere
rules interact (see below). Only `internal:ktlint-suppression` still runs over the whole AST first.

**E2 — indent crash in format (2.0 only).** 64 files per style: `standard:indent IllegalArgumentException: Stack should
be empty` in `afterLastNode` → "Internal Error", file left unformatted. Indent alone does not crash; indent +
no-consecutive-blank-lines does (bisected on CombineFlowsBenchmark.kt, ends in `}\n\n`): under E1 the other rule replaces
the whitespace the FILE indent context points at before indent pops it. 1.8: 0 crashes. Already reproduced by ktrs
(research/18, research/19). [#3371](https://github.com/ktlint/ktlint/pull/3371) only made `before/afterLastNode`
exceptions reportable.

**E3 — smaller engine/CLI-visible changes.** UTF BOM kept when not at file start ([#3264](https://github.com/ktlint/ktlint/pull/3264));
obsolete-property warnings for `disabled_rules`/`ktlint_disabled_rules` removed (531eccb0); exception text says the full
path instead of the file name ([#3355](https://github.com/ktlint/ktlint/pull/3355)).

## Format diffs

| style | files | 2.0 indent crash (E2) | also have lint-row diffs | rest |
|---|--:|--:|--:|--:|
| ktlint_official | 338 | 64 | 95 | 179 |
| intellij_idea | 96 | 64 | 22 | 10 |
| android_studio | 1346 | 64 | 1033 (437 with blank-line-before-declaration) | 249 |

- Lint-row files: explained by R1–R5 (2.0 doesn't wrap where 1.8 wrapped; android adds blank lines before declarations).
- ktlint_official "rest" is mostly **when-entry-bracing** (official-only rule, no lint-row diff): ~169 files where 1.8
  braces entries that 2.0 leaves bare. Cause: the [#3261](https://github.com/ktlint/ktlint/pull/3261) rewrite of
  `WhenEntryBracing.surroundWithBraces`. It rebuilds the entry from `prevCodeSibling(ARROW).text` = only the *last*
  condition, so **multi-condition entries lose conditions** (`is Int, is Long -> 1` → `is Long -> { 1 }`; repro'd on the
  jar; Exposed Op.kt/SchemaUtilityApi.kt: `is OracleDialect, is RedshiftDialect ->` → `is RedshiftDialect ->`). It also
  fails to converge on some `when`s ("not able to resolve all violations ... in 3 consecutive runs"; last `else -> 3`
  stays unbraced). ktrs ports this faithfully (`crates/ktrs-lint/src/rules/when_entry_bracing.rs:106-117`).
- Other "rest" hunks are E1 ordering effects with identical lint rows: 2.0 wraps argument lists 1.8 left alone
  (`listOf({ it -> ... })` → multiline), 2.0 wraps after `=` instead of keeping `= apply {` / `= "…${ … }"`, function-type
  parameters `emit: (offset: Int, …) -> X` collapse to `emit:\n(offset: Int, …)`, blank-line-before-declaration (android)
  firing on intermediate trees.
- No 1.8-only crash (0 "Internal Error" in any 1.8 run, lint or format).

## Drop-in (CLI) differences 1.8 → 2.0

| area | 1.8.0 | 2.0.0-ALPHA-4 |
|---|---|---|
| removed flags | `--code-style`, `--disabled_rules`, `--experimental` accepted, fail with a migration message | gone (clikt "no such option") |
| exit codes | 0 / 1 (all failures) / 123 (format output unparsable) | enum: 0 OK, 1 lint errors, 2 IO, 3 stdin parse error, 4 stdin exception, 5 file not found, 6 invalid ruleset jar, 7 invalid reporter config, 123 |
| `--stdin` parse failure | error row, exit 1, no stdout | `-F`: original code echoed to stdout, exit 3 (4 for other exceptions) ([#3359](https://github.com/ktlint/ktlint/pull/3359)) |
| `--stdin -F` output | `printf` → breaks on `%` ([#3281](https://github.com/ktlint/ktlint/pull/3281)) | `write`; kotlin-logging startup banner suppressed |
| `--stdin-path` relative w/o `./` | `.editorconfig` not found ([#3322](https://github.com/ktlint/ktlint/pull/3322)) | fixed |
| baseline | rule ids without `standard:` accepted (warns, prefixes) | must match exactly; old 0.x baselines silently stop matching |
| rule ids | `context-receiver-list-wrapping` | `context-parameter-list-wrapping` (R5) |
| obsolete `disabled_rules` property | warning | silent |
| `-R` rule sets | 1.x API | 2.x API + 1.3–1.8 jars via `ktlint-com-pinterest-backward-compatibility` (ktrs loads none) |
| reporters | plain, plain-summary, json, sarif, checkstyle, html, format, baseline | same set, same output (only package renames) |
| misc | Java 8+ | Java 17+ ([#3256](https://github.com/ktlint/ktlint/pull/3256)); native images; Maven `io.github.ktlint.core` |

## What a 1.x mode would take

Lint (rows): **small** — about 1–2 days plus a 1.8 oracle.
- One mode flag (e.g. `ktlint_compat = 1` / `--ktlint-version 1.8`) read by: `has_no_max_line_length_suppression` (R2,
  one site), three `exceeds_max_line_length` bodies (R1), blank-line-between-when-conditions ×2 (R4),
  blank-line-before-declaration marker/guard (R3), the rule id of context-parameter-list-wrapping (R5), the registration
  list (R6). Covers every row in the table except the 1 KDoc no-multi-spaces row (R7) and the 186 same-position tie
  orderings (need E1).
- Oracle: `tools/ktlint-oracle/ktlint-probe.sh` against the 1.8.0 jar, a `--oracle` dir per style for `cargo lint-diff`;
  golden tests would need ktlint 1.8.0's rule tests extracted separately (they differ for every rule touched above).

Format (bytes): **medium-large** — a second engine mode, roughly 1–2 weeks.
- E1: port 1.8's `RuleProviderSorter` + `RunAfterRuleFilter` and the per-rule `VisitorModifier`s (~16 rules, listed in
  E1) and a rule-major `RuleExecutionContext` (each rule visits the whole tree, then the next). ktrs today has only the
  2.0 sorter (`crates/ktrs-lint/src/engine/visitor_provider.rs`) and node-major traversal.
- Revert-in-mode: `WhenEntryBracing.surroundWithBraces` (#3261, old 20-line body) — otherwise 1.x users get the
  condition-dropping bug they never had; plus the R1–R5 switches above.
- E2 disappears by itself under rule-major order (1.8 has no crash), but must be verified on the 64 files.
- Rule internals written for the 2.0 order (#3253 tweaks to chain-method-continuation/function-literal assume E1) may
  still leave residual diffs; budget a cargo fmt-diff-style iteration loop against a 1.8 format oracle.
- CLI: optional — old exit codes (collapse 2–7 → 1), stdin parse failure behaviour, baseline id prefixing, accept the
  three removed flags with 1.8's error text. Each is a few lines in `crates/ktrs-cli/src/ktlint/`.

Alternative with near-zero cost: document that `ktlint` (ktrs) follows 2.0 and list R1–R5 as the expected deltas;
the lint-only switch set is cheap enough to ship first if 1.x users report noise.
