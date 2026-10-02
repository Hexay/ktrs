# 26 — ktlint 1.8 mode: lint rows, CLI (phase 1) and byte-exact format (phase 2) (2026-10-02)

ktrs ports ktlint 2.0.0-ALPHA-4; research/22 listed what separates it from 1.8.0 (R1–R7, E1–E3, the CLI table).
Phase 1 makes ktrs reproduce **1.8.0's lint rows and CLI behaviour** on request; phase 2 (section at the end) makes
`-F` byte-exact: 1.8's rule order from the rules' `VisitorModifier`s, rule-major traversal, the old
`when-entry-bracing` rewrite.

## Selecting the mode

- `.editorconfig`: `ktrs_ktlint_version = 1.8` (or `2.0`; ktrs-only, the jars ignore it). Invalid values count as 2.0.
- CLI: `--ktlint-version=<1.8|2.0>` on the `ktlint` drop-in (hidden: help stays byte-identical) and on `ktrs lint`
  (also `ktrs lint --list-rules --ktlint-version=1.8`). Invalid value: Clikt-style usage error, exit 1.
- The CLI decides **once per run**: the flag, else the property of the `.editorconfig` that applies to
  `<working dir>/.kt`, else 2.0 (`crates/ktrs-cli/src/ktlint/version.rs`). It then passes the result to the engine as an
  `.editorconfig` override, so every file follows the CLI's choice (a nested `.editorconfig` cannot mix modes in one run).
- Engine API users (`KtLintRuleEngine`) get per-file behaviour from the property, or set it in `EditorConfigOverride`.
- `ktrs serve` speaks the ktfmt protocol only (no lint request), so there is nothing to switch there.

## Plumbing (where phase 2 hangs off)

- `KtlintVersion` + `KTLINT_VERSION_PROPERTY` (`crates/ktrs-lint/src/editorconfig/ktlint_version.rs`).
  `KtlintVersion::of(&EditorConfig)` reads it without an `EditorConfigProperty` lookup, so rules need not declare it
  and `generateEditorConfig` never lists it.
- **Rules** read it in `before_first_node` into a `ktlint_version` field and branch where 1.8's code differs (one
  branch per upstream change, commented with the PR). The R2 sites go through one helper,
  `AstNodeQueries::has_no_max_line_length_suppression_in(node, version)` (always true in 1.8).
- **Rule set membership** is registration data: `RuleV2Provider::only_in(version)` in `rules/mod.rs`; the engine's
  `RuleSetupCache` filters providers by the file's version *before* the internal-rule and rule-execution filters, so
  the suppression rule's "unknown or not loaded" set is that release's. Rules whose identity differs per release are
  registered twice: `ContextParameterListWrappingRule::context_receiver_list_wrapping()` (1.8 id),
  `BlankLineBeforeDeclarationRule::ktlint_1_8()` (an `OfficialCodeStyle` rule in 1.8).
- **Engine** switches read the version from the file's loaded `.editorconfig`: BOM normalization and the exception's
  file name (`engine/rule_execution_context/create.rs`), suppression ids (`engine/suppression_ids.rs`), obsolete-property
  warnings (via `KtLintRuleEngine::with_engine_warnings`, which the CLI routes to its logger).
- Phase 2 hangs off `RuleSetup` and `RuleExecutionContext::execute_rules` (see "Phase 2" below).
- **CLI** (`crates/ktrs-cli/src/ktlint/version.rs`): release string, JVM package prefix (logger names are written as 2.0
  names and renamed in `Logger::log`), repository URL, exit-code mapping. `args.rs` picks the option table and help per
  version; `process.rs` holds the stdin differences; `java_printf.rs` emulates `PrintWriter.printf`.

## Switches (ported from `ktlint-src` tags 1.8.0 / 2.0.0-ALPHA-4)

| id | what 1.8 does | where |
|---|---|---|
| R1 | argument-list-wrapping / function-literal measure the whole line; chain-method-continuation stops *before* the stop leaf | `exceeds_max_line_length` ×3 |
| R2 | no `hasNoMaxLineLengthSuppression` (#3255) in 12 rules | `has_no_max_line_length_suppression_in` |
| R3 | blank-line-before-declaration is `OfficialCodeStyle`, no intellij_idea stop (#3318) | registration + rule |
| R4 | `isPrecededByComment` counts trailing EOL comments (#3177); whitespace right after the previous entry (#3261) | blank-line-between-when-conditions |
| R5 | id `context-receiver-list-wrapping` (#3367) | registration |
| R6 | no call-expression-wrapping, lambda-return, blank-line-before-{file-annotation,imports,package}, no-blank-line-at-start-of-file; empty `condition-wrapping` and `discouraged-comment-location` registered (#3237) | registration (101 vs 105 rules) |
| X1 | annotation wraps an annotated expression before a lambda (#3268) | annotation |
| X2 | class-signature joins a supertype after an EOL comment (#3312) | class-signature |
| X3 | indent aligns a wrapped `where` with its column even with tabs (#3363) | indent |
| X4 | paren-spacing does not stop at an EOL comment (#3236) | paren-spacing |
| E-a | `@Suppress("ktlint:foo")` suppresses `standard:foo` (prefix when the rule set is missing) | `suppression_ids.rs` |
| E-b | the first BOM is removed wherever it is (#3264) | `create.rs` |
| E-c | `disabled_rules` / `ktlint_disabled_rules` in `.editorconfig`: a WARN per file (`RuleExecutionContext`) | `create.rs` |
| E-d | rule exception text names the file, not its path (#3355); standard rules' `About` URLs are pinterest's | `create.rs`, `process.rs` |

Not switched, on purpose: the KDoc trailing-space branch of no-trailing-spaces (d3adc18f) compensates for the newer
KDoc lexer that ktrs shares with 2.0, so it stays on in 1.8 mode; positional destructuring (#3365) only parses with the
2.4 grammar; package-name's regex change is unobservable (underscores are reported first); format-only changes
(comment-spacing rewrite, when-entry-bracing #3261, `VisitorModifier`s) are phase 2.

### CLI (1.8.0's `KtlintCommandLine` and the jar, checked by `cli-diff.sh`)

- Exit codes 0 / 1 / 123: every 2.0 code but 0 and 123 maps to 1 (`version::exit_value`).
- `--code-style=<style>` is still declared (shown in help as deprecated): a valid value fails with "Parameter
  '--code-style' is no longer valid. …" (plain CliktError, no usage line), an invalid one with the enum usage error.
  research/22 was wrong about the other two: `--disabled_rules` and `--experimental` are declared with `=` instead of
  `by` in 1.8.0, so Clikt never registers them and the jar says "no such option" — reproduced.
- Help: `pinterest.github.io` URL and the `--code-style` entry; `--version` 1.8.0; SARIF tool version 1.8.0.
- `--stdin` parse failure (lint and `-F`): retry as script, then ERROR "Can not parse input from <stdin> as Kotlin, …",
  the parse error reported as a row, nothing on stdout, **exit 0** (research/22 said 1; the jar exits 0). Other stdin
  exceptions become a row (exit 1), as for files.
- `--stdin -F` output goes through `printf` (#3281): `%%`/`%n` rewritten, any other conversion throws
  (`MissingFormatArgumentException`, `UnknownFormatConversionException`, …): "Exception in thread "main" …", exit 1.
- Baseline: ids without rule set get `standard:` and one WARN with the count.
- Logger and exception class names `com.pinterest.ktlint.…`; issue URL and git-hook header `github.com/pinterest/ktlint`.

## Results (testbox, 2026-10-02)

Lint rows, `ktrs --ktlint-version=1.8` vs `ktlint-1.8.0`, `ktlint-compare.sh` (`--relative`, rows sorted, so the order of
rows at one position is not compared here):

| tree | style | rows (jar) | only jar | only ktrs | before (ktrs 2.0 vs 1.8 jar ≈ 2.0 jar vs 1.8 jar) |
|---|---|--:|--:|--:|--:|
| dev corpus (6,123 files) | ktlint_official | 214,281 | 1 | 0 | 1,311 / 39 |
| | intellij_idea | 156,474 | 1 | 0 | 59 / 24 |
| | android_studio | 231,144 | 1 | 0 | 10,308 / 1,451 |
| held-out (15,287 files) | ktlint_official | 828,404 | 3 | 0 | 6,687 / 80 |
| | intellij_idea | 617,781 | 3 | 0 | 187 / 51 |
| | android_studio | 795,268 | 3 | 0 | 22,335 / 6,786 |

Exit codes equal everywhere. The "before" column is the 1.8.0 vs 2.0.0-ALPHA-4 jar diff (dev: research/22; held-out:
`testbox:~/work/parity/out/kt18-vs-kt2-holdout/`); on the held-out tree it also shows what research/22's corpus did not
exercise (R5 25+25 rows, indent X3 1+1, KDoc trailing spaces 2).

- Order of rows at one position: 1.8 runs rule after rule, and so does 1.8 mode since phase 2 (phase 1 sorted ties by
  a fixed rank instead). Checked on the fixtures against the jar's unsorted output and by cli-diff, whose reporter
  outputs compare row order.
- CLI, `cli-diff.sh` (154 scenarios, 10 new: `--code-style`, the two never-registered options, `%` on stdin, obsolete
  properties): 1.8 mode vs `ktlint-1.8.0` **153/154**; 2.0 mode vs `ktlint-2.0.0-ALPHA-4` **154/154**.
- 2.0 unchanged: the branch's `ktlint` vs its base (53bb811) on the dev corpus, three styles: 0 lint-row diffs and
  0 files differing after `-F`; `cargo test -p ktrs-lint --release` and `cargo test -p ktrs-cli` green.

Found while porting, beyond research/22: 1.8 stdin parse failures exit 0 (not 1); `--disabled_rules`/`--experimental`
are unknown options in 1.8.0 too; `@Suppress("ktlint:<id>")` suppresses `standard:<id>` in 1.8; `generateEditorConfig`
lists only the rules' and the six default properties in 1.8 (its `filterBy` filters); SARIF names pinterest and prints
`"rules": [` / `]` on two lines; `-R`/reporter JAR errors name 1.8's `RuleSetProviderV3`/`com.pinterest` interfaces and
say "run in debug mode"; the obsolete-property warning; four small rule fixes (X1–X4); and the 2.0 jar's Clikt hint
"hint: generateEditorConfig has an option --code-style", which ktrs's 2.0 mode now prints too.

## Remaining differences

- **KDoc whitespace (R7 family), 1 row on the dev corpus, 3 on the held-out tree, in every style**: `no-multi-spaces` on
  a KDoc line ending in spaces (TickerChannels.kt:51; apollo DefaultHttpRequestComposer.kt:58) and `no-trailing-spaces`
  on ` * ` lines (kotlinx.serialization Json.kt:681/685). Kotlin 2.2.21's KDoc lexer (1.8.0) gives that whitespace its
  own token; ktrs shares 2.4's lexer with the 2.0 jar, which misses the same rows. Fixing it needs a 2.2 KDoc-lexer
  quirk. In `-F` the Json.kt rows are the **only formatted-file difference** left (1 file per style on the held-out
  tree: the jar trims the trailing space of two ` * ` lines inside a ```` ``` ```` KDoc code block; ktrs keeps it).
- **1.8's `RunAfterRuleFilter` failure** (e.g. `ktlint_standard_indent = disabled`, which leaves string-template-indent
  without its required predecessor): the jar dies with "Exception in thread "main" java.util.concurrent.ExecutionException:
  java.lang.IllegalStateException: Skipping rule(s) …" plus a JVM stack trace, exit 1. ktrs prints the same first lines
  and exits 1, without the stack trace.
- **`--stdin-path` without `./`** (#3322): 1.8 looks up `.editorconfig` from the relative path and so misses the working
  directory's; ktrs resolves it against the working directory as 2.0 does (not covered by a scenario that differs).
- Logging thread names: in file mode 1.8 logs per-file warnings (obsolete properties) from `pool-1-thread-N` in
  nondeterministic order; ktrs logs `[main]`.
- Not exercised: a mid-file BOM in format mode, rule crashes (the URL/class-name switches are unit-level only).

## How to run

- Tests: `cargo test -p ktrs-lint --release --test ktlint_1_8_mode` (one fixture per switch in
  `crates/ktrs-lint/tests/data/ktlint_1_8_mode/`, expected rows from both jars), `--test ktlint_1_8_format` (rule order,
  `RunAfterRuleFilter`, when-entry-bracing in both modes), `cargo test -p ktrs-cli --test ktlint_cli_1_8`,
  `--test ktlint_baseline`, and `java_printf`'s unit test.
- Lint rows and `-F` trees vs the jar (testbox, background, under the bench lock; `NO_FORMAT=1` for rows only):
  `tools/parity/ktlint-compare.sh ~/work/ktlint-bench/bin/ktlint-1.8.0 "target/release/ktlint --ktlint-version=1.8" <tree> <out>`
  on `~/work/ktlint-bench/corpus` and `~/work/holdout/tree`. Bisecting a format diff: `ktlint_standard_<id> = disabled`
  in the tree's `.editorconfig` on both sides, one rule at a time (mind the `RunAfterRuleFilter` failure above).
- CLI: `KTLINT_VERSION=1.8 JAR=~/work/ktlint-bench/bin/ktlint-1.8.0 tools/ktlint-oracle/cli-diff.sh target/release/ktlint`
  (locally the jar is fetched to `tools/ktlint-oracle/lib/`).
- Regenerating the fixture expectations: run `ktlint-1.8.0 --relative` and `ktlint-2.0.0-ALPHA-4 --relative` in the
  fixture directory, keep the `<file>.kt:` rows, `LC_ALL=C sort`.

## Phase 2: byte-exact `-F` (2026-10-02)

Ported from `ktlint-src` tag 1.8.0 (`RuleExecutionContext.kt`, `RuleProviderSorter.kt`, `rulefilter/RunAfterRuleFilter.kt`,
`CodeFormatter.kt`, the 16 rules' constructors, `WhenEntryBracing.kt`), diffed against 2.0.0-ALPHA-4.

### Design

- **`VisitorModifier`s are rule data again**: `RuleV2::visitor_modifiers()` (default none) returns 1.8.0's set for the
  16 rules that had one (research/22 E1 list); `RuleV2Provider` caches it like the rule id and exposes 1.8's
  `run_after_rules()` / `run_as_late_as_possible()`. 2.0 never reads them.
- **`engine/rule_provider_sorter_1_8.rs`**: `run_after_rule_filter` (drops nothing in practice, but a missing
  `ONLY_WHEN_RUN_AFTER_RULE_IS_LOADED_AND_ENABLED` predecessor is the jar's `IllegalStateException`) and
  `get_sorted_rule_providers_1_8` (suppression rule; then rules without modifiers, standard first, by id; then each
  remaining rule, in that order, once its run-after rules are placed). `RuleSetupCache` applies the filter after the
  rule-execution filter and picks the sorter per version (`VisitorProvider::for_version`); a filter failure is stored in
  the `RuleSetup` and raised by `create_rule_execution_context` as `KtLintException::IllegalState` (the CLI crashes with
  it, wrapped in `ExecutionException` for files).
- **Rule-major traversal** (`engine/rule_execution_context/rule_major.rs`): in 1.8 mode `execute_rules` runs each rule
  over the whole tree (`beforeFirstNode`, walk, `afterLastNode`) before the next. Two 1.8 details kept: no bail-out for
  nodes replaced by an earlier hook, and one suppression lookup per node, made before `beforeVisitChildNodes` and reused
  for `afterVisitChildNodes`. A rule exception is rethrown at every level up to the root, so lint reports it at 1:1.
  The 2.0 node-major code path is untouched.
- **Error order**: both modes now stable-sort by line/col only (upstream); in 1.8 mode the insertion order is the rule
  order, so phase 1's rank table (`KTLINT_1_8_LATE_RULES`) is gone.
- **when-entry-bracing**: `surround_with_braces_1_8`, the pre-#3261 body (replace everything after the arrow by
  `" "` + a block built from `true -> { … }`), chosen by the rule's `ktlint_version`.
- Rule internals: diffing every standard rule 1.8.0 vs 2.0.0-ALPHA-4 after normalising renames (`*20` accessors,
  package, `RuleV2`) leaves only the phase-1 switches, the `VisitorModifier` sets and when-entry-bracing; the rest are
  equivalent rewrites (function-literal `takeIf`/`if`, comment-spacing's leaf replacement, `Unit` vs comments) or
  doc changes. No further switch was needed: the corpora agree byte for byte.
- Cost: 1.8 mode traverses once per rule. On 1,500 held-out files, `lint` 0.49 s → 0.94 s and `-F` 2.24 s → 2.93 s
  (2.0 vs 1.8 mode, same binary).

### Results (testbox)

`ktlint-compare.sh` with `-F`, `ktrs --ktlint-version=1.8` vs `ktlint-1.8.0`:

| tree | style | lint rows only jar / only ktrs | `-F` files differing |
|---|---|--:|--:|
| dev corpus (6,123 files) | ktlint_official | 1 / 0 | 0 |
| | intellij_idea | 1 / 0 | 0 |
| | android_studio | 1 / 0 | 0 |
| held-out (15,287 files) | ktlint_official | 3 / 0 | 1 |
| | intellij_idea | 3 / 0 | 1 |
| | android_studio | 3 / 0 | 1 |

Before phase 2 the dev corpus differed in 338 / 96 / 1,346 files after `-F` (≈ research/22's 1.8-vs-2.0 jar table).
The remaining lint rows and the one held-out file (Json.kt) are the KDoc-lexer rows under "Remaining differences".
On the dev corpus the rows `-F` prints are also equal (sorted). The 64 files where 2.0 crashes in indent (E2) format
without error, as in 1.8.

- CLI, `cli-diff.sh`: 1.8 mode vs `ktlint-1.8.0` **154/154** (was 153: `rep_summary_format` now matches); 2.0 mode vs
  `ktlint-2.0.0-ALPHA-4` 154/154.
- 2.0 unchanged: branch vs master-built (53bb811) `ktlint` on the dev corpus, three styles: 0 lint-row diffs, 0 files
  differing after `-F`, stdout/stderr of both runs equal up to timestamps and temp paths; `cargo test -p ktrs-lint
  --release` and `cargo test -p ktrs-cli` green.
- New tests (`tests/ktlint_1_8_format.rs`): the sorter reproduces the jar's debug-logged order for ktlint_official;
  the filter's failure message; when-entry-bracing in both modes against each jar's `-F` output
  (`tests/data/ktlint_1_8_format/`).
