# 04 — Parity scope: ktfmt + ktlint in Rust

Measured 2026-09-25 from shallow clones (ktfmt `c223b51` 2026-09-18, ktlint `3820242` 2026-09-24,
google-java-format HEAD, compose-rules HEAD, kotlin HEAD). LOC = raw `wc -l` (includes comments/license headers).

## TL;DR

- Byte-for-byte ktfmt parity ≈ **30k Rust LOC** (~half is a PSI-shaped Kotlin parser + AST layer); full
  ktlint standard-ruleset parity adds **≈ 33–37k**. Core total incl. test harness ≈ **70k**; ≈ 80–85k with
  compose-rules port and integrations.
- Neither tool needs type resolution — both are PSI-only (ktfmt unused-import removal is a name-based heuristic).
  This is what makes a Rust port possible at all.
- The hardest part is not the formatter logic; it is reproducing **IntelliJ PSI tree shape** (node types, error
  recovery, whitespace/comment attachment), because every ktfmt visitor and every ktlint rule branches on it.
- **Moving target**: ktfmt ships ~6 releases/yr and nearly every release changes output; ktlint 2.0 (alpha)
  changes rule execution order. Kotlin 2.x keeps adding syntax (context params, guards, explicit backing fields,
  name-based destructuring).
- **Value prop is eroding**: both projects now ship GraalVM native binaries (ktfmt PR #584 merged 2026-06-30,
  "up to 100x faster" on small inputs, still marked Unreleased after 0.64; ktlint 2.0.0-ALPHA-4, 2026-08-21, ships
  linux/macOS/Windows native binaries — **no custom ruleset support in native mode**).
- ktlint has ~5–8x the usage of ktfmt. Custom JVM rulesets (compose-rules) are the main lock-in.

## 1. ktfmt

Repo moved to **github.com/Kotlin/ktfmt** (JetBrains stewardship); package renamed `com.facebook.ktfmt` →
`org.jetbrains.ktfmt` (Unreleased). Maven coordinates still `com.facebook:ktfmt`.

| Area | Files | LOC |
|---|---|---|
| Visitor (default engine, split from old `KotlinInputAstVisitor.kt`) `format/visitor/*` + `KotlinInputAstVisitor.kt` | Call/Declaration/Expression/ControlFlow/List/Type/Annotation formatters, helpers | ~4.6k |
| Experimental kotlinlang engine `format/visitor/kotlinlang/*`, `KotlinLangInputAstVisitor.kt` | dev-only (`--experimental`) | ~0.7k |
| Pipeline: Formatter, KotlinInput, Tokenizer, TrailingCommas, RedundantElementManager, RedundantImportDetector, RedundantSemicolonDetector, MultilineStringFormatter, WhitespaceTombstones, Options | | ~2.5k |
| **Total core `format/`** | | **7,851** |
| KDoc formatter `kdoc/*` (markdown-ish paragraph reflow, tables, lists) | | 3,489 |
| CLI + EditorConfigResolver (ec4j) | | 683 |
| IntelliJ plugin | | ~0.6k |

**gjf dependency** (the Doc/Level algorithm): ktfmt imports `Doc`, `Doc.Level`, `DocBuilder`, `OpsBuilder`,
`Op`/`OpenOp`/`CloseOp`, `Indent`, `Input`/`Input.Tok`, `Output`/`BreakTag`, `Newlines`, `CommentsHelper`,
`FormattingError`, `java.JavaOutput`, `java.FormatterException`. Those gjf files total **2,836 LOC** (Doc 750,
OpsBuilder 657, JavaOutput 391, Newlines 200, Input 147, DocBuilder 114, InputOutput 119, …). Self-contained, easy to port.

**Pipeline** (`Formatter.format`): parse → sort/dedupe imports → drop redundant elements (unused imports,
semicolons, trailing commas) → add redundant elements (trailing commas) → pretty-print (visitor → Ops → Doc →
`computeBreaks` → JavaOutput) **looped until trailing-comma insertion is a fixed point** → multiline-string
`trimIndent/trimMargin` reindent. Parses up to ~4x per file (issue #552, partly fixed 0.64).

**Styles**: `META_FORMAT` (2/4, trailing commas ONLY_ADD, default), `GOOGLE_FORMAT` (2/2), `KOTLINLANG_FORMAT` (4/4).
Options: maxWidth (100), blockIndent, continuationIndent, trailingCommaManagementStrategy (NONE/ONLY_ADD/COMPLETE),
removeUnusedImports, preserveLambdaBreaks (default true since 0.63), experimental engine, `--lines/--offset`
range formatting (Unreleased), `.editorconfig` (`max_line_length`, `indent_size`, `ij_kotlin_*`).

**Semantics**: PSI only (`KotlinCoreEnvironment` for parsing, no `BindingContext`/analysis). Unused-import
detection = collect simple names of `KtReferenceExpression`s + KDoc links + hard-coded operator names
(`getValue`, `setValue`, `componentN`, `plus`, …) and drop imports whose last segment is unseen. Portable
without a type checker; must replicate the operator list and KDoc handling exactly.

**Tests**: 540 file-based golden cases in `core/src/test/resources/cases/**` (`.input`, 129 with `.output`; the
rest are idempotency cases), directive headers (`// MAX_WIDTH 80`, `// TRAILING_COMMA_STRATEGY NONE`…),
~6.6k lines. Plus 251 JUnit `@Test`s (125 in KDocFormatterTest, 5.5k LOC). **The golden corpus is directly
reusable** by a Rust harness — biggest asset for parity.

**Release cadence / output churn** (from CHANGELOG + GitHub releases): 0.55 (2025-05) → 0.64 (2026-06) = 9
releases in 13 months. Output-changing entries almost every release: 0.57 trailing commas on by default for all
styles + where-clauses + multiline string reformatting; 0.62 ~10 formatting fixes (scoping funcs, comments,
KDoc lists, `when` lambdas); 0.63 preserveLambdaBreaks default flip; Unreleased: single-param trailing comma,
two non-idempotency fixes. Parity is a **version-pinned** target (users pin e.g. `ktfmt("0.61")` in Spotless),
so a Rust port must advertise "matches ktfmt X.Y" and chase releases.

## 2. ktlint

Repo moved to **github.com/ktlint/ktlint**; 2.0 changes Maven coordinates `com.pinterest.ktlint` → `io.github.ktlint`
("no longer maintained by Pinterest"). Latest stable 1.8.0 (2025-11); 2.0.0-ALPHA-4 (2026-08). ~3–4 releases/yr.

| Module | LOC |
|---|---|
| `ktlint-ruleset-standard` rules (104 rule files + `internal/importordering`) | 19,415 (module 19,664) |
| `ktlint-rule-engine` | 3,419 |
| `ktlint-rule-engine-core` (Rule API, ASTNode ext fns, editorconfig property types) | 2,425 |
| `ktlint-cli` | 1,878 |
| Reporters (plain, plain-summary, json, checkstyle, sarif, html, format, baseline, core) | 1,283 |
| Standard ruleset tests: 43.5k LOC, **1,962 `@Test`/`@ParameterizedTest`** (code snippets inline in Kotlin strings) | |

- **104 rules**; **87 autocorrect**, **17 lint-only** (naming rules, filename, max-line-length, kdoc,
  no-wildcard-imports, *-comment position rules, mixed-condition-operators, no-empty-file, no-consecutive-comments).
  8 experimental. Largest: `IndentationRule` 1,601 LOC. `no-unused-imports` (351 LOC) is opt-in in 2.0.
- **Rule API** = IntelliJ `ASTNode` (CST) with in-place mutation: `upsertWhitespaceBeforeMe` (127 call sites),
  `replaceTextWith` (78), `upsertWhitespaceAfterMe` (75), `addChild` (24), `rawInsert*`, `replaceChild`, `rawRemove`.
  A Rust port needs a **mutable lossless CST** (rowan-style green/red tree with edits) and must reproduce
  ktlint's `beforeVisitChildNodes`/`afterVisitChildNodes` traversal, rule ordering (`VisitorModifier.RunAfterRule`),
  `ktlint-suppress`/`@Suppress` handling, and the repeat-format loop (`maxFormatRunsPerFile`). 2.0 changed traversal
  to "all rules on a node before next node" — a breaking semantic change a port must pick a side on.
- **.editorconfig is the config surface**: `ktlint_code_style` (ktlint_official / intellij_idea / android_studio),
  `ktlint_<ruleset>_<rule> = disabled`, `ktlint_experimental`, ~15 rule-specific `ktlint_*` props, plus `ij_kotlin_*`
  (imports layout, trailing comma, formatter tags), `indent_size/style`, `max_line_length`, `end_of_line`,
  `insert_final_newline`. Uses ec4j; glob semantics must match.
- **Custom rulesets**: loaded via `ServiceLoader` (`RuleSetProviderV3`) from JARs. A Rust binary cannot load them.
  - compose-rules (mrmans0n, Apache-2.0): 734 stars, 76 checks, 9.5k LOC (ktlint + detekt flavors). ktlint flavor
    is PSI-only (portable); detekt flavor now uses the Kotlin Analysis API (`analysis.api.symbols`).
  - GitHub code search: `io.nlopez.compose.rules` in 830 `libs.versions.toml` vs `ktlint` in 5,952 → roughly
    1 in 7 ktlint-catalog projects also pull compose-rules (overcount: includes detekt users). Mitigation: bundle a
    native port of compose-rules' ktlint checks. Note: ktlint's **own** native binary has the same limitation.

## 3. Popularity (ktlint vs ktfmt)

| Signal | ktlint | ktfmt |
|---|---|---|
| GitHub stars | 6,746 | 1,326 |
| Code search `libs.versions.toml` | 5,952 | 801 |
| Code search `build.gradle.kts` | 17,984 | 2,200 |
| JetBrains plugin downloads | 456,946 (Ktlint, #15057) | 113,022 (ktfmt, #14912) |
| `.pre-commit-config.yaml` | 176 | 148 |

Maven Central download stats not publicly accessible (mvnrepository returned 403). ktlint is ~5–8x more used;
ktfmt skews to large orgs (Meta, Block/Square, Google-adjacent). Detekt (7,069 stars) wraps ktlint's rules as
its `formatting` ruleset — another indirect ktlint consumer.

## 4. Integration surface

| Integration | How it runs today | Native-binary fit |
|---|---|---|
| **Spotless** (5.6k stars) | ktfmt/ktlint in-process on JVM, pinned version, `customRuleSets(...)` for ktlint | Generic `nativeCmd(name, pathToExe, args)` step exists (`lib/.../generic/NativeCmdStep.java`): pipes stdin→stdout, **one process per file**. First-class step with auto-download (like `biome()`, `clangFormat().pathToExe`) needs an upstream PR. |
| kotlinter-gradle (709★) | ktlint engine in-process | Needs a fork/new plugin |
| ktlint-gradle (1,719★) | ktlint jars in worker | Needs new plugin |
| ktfmt-gradle (cortinico, 223★) | ktfmt in-process | Needs new plugin |
| IntelliJ/Android Studio | Ktlint plugin (bundles jars), ktfmt plugin | Plugin that shells out or talks LSP; big UX surface |
| pre-commit | language-formatters-pre-commit-hooks downloads jars + needs JVM | Easy win (binary hook, no JVM) |
| Bazel | aspect `rules_lint` drives ktfmt/ktlint binaries; rules_kotlin has ktlint rules | Easy (hermetic binary) |
| LSP | none official for ktlint/ktfmt | Opportunity (format-on-save, diagnostics) |

## 5. Licenses

- ktfmt: Apache-2.0 (confirmed LICENSE). gjf: Apache-2.0 for all code except `google-java-format-diff.py`.
  Kotlin compiler (parser, PSI, lexer .flex): Apache-2.0. compose-rules: Apache-2.0. ktlint: **MIT**
  (© Pinterest 2019–2026, Stanley Shyiko 2016–2019, Ktlint 2026).
- Porting = derivative work: permitted; keep NOTICE/copyright headers, state changes (Apache §4), include MIT notice
  for ktlint-derived code. Apache-2.0 and MIT are both compatible with an Apache-2.0 or MIT/Apache dual-licensed
  Rust project. Golden test data (ktfmt `.input` cases, ktlint test snippets) may be copied under the same terms.
- Trademarks: avoid naming the product "ktfmt"/"ktlint"; "ktfmt-compatible" is fine.

## 6. Demand signals

- ktfmt #552 (Stainless, large generated SDKs): parse called 4x per file — perf complaint; partial fix in 0.64
  (#620 −6–7% allocations, #622 reuse parse).
- ktfmt #584 native-image PR: strongly received (18 👍, 17 🚀, 12 🎉); "up to 100x faster" for small inputs; JVM
  catches up at thousands of files.
- ktlint #3284 native binaries merged 2026-08-01 for editor plugins / CI containers / JVM-less tooling.
- Block engineering blog: 3,500 files — ktfmt 5.9 s vs ktlint 14.8 s (speed not the primary reason for switching).
- Reddit/HN: little explicit "rewrite it in Rust" demand found; HN threads discuss ktfmt vs ktlint style, not speed.
  Demand is real but is being met by GraalVM native images. No existing Rust Kotlin formatter found (only
  tree-sitter-kotlin grammars, which are **not** PSI-shaped and unsuitable for byte parity).

## 7. Effort breakdown

| Component | Source (LOC) | Est. Rust LOC | Risk | Notes |
|---|---|---|---|---|
| Kotlin lexer (incl. string templates, nested comments, KDoc lexer) | Kotlin.flex 376 + KDoc.flex 405 + KtTokens 449 | 2,000 | Med | JFlex states must be replicated exactly |
| Kotlin parser with PSI-identical tree + error recovery | KotlinParsing 2,842 + KotlinExpressionParsing 1,884 + Abstract/PsiBuilder ~780 + KtNodeTypes 175 | 8,000–9,000 | **High** | Tree shape parity is the foundation of everything; must track Kotlin 2.x syntax |
| Typed AST layer (KtXxx accessors, psiUtil ext) | subset of compiler psi-api (ktfmt imports ~400 PSI symbols) | 4,000–6,000 | Med | Largely codegen-able |
| gjf Doc/OpsBuilder/JavaOutput | 2,836 | 2,500–3,000 | Med | Self-contained; exact break/fill semantics + comment reformatting |
| ktfmt visitor (meta/google/kotlinlang) + pipeline | 7,851 | 9,000–10,000 | **High** | Byte parity; fixed-point trailing-comma loop; per-version drift |
| ktfmt KDoc formatter | 3,489 | 3,500–4,000 | Med-High | Heuristic markdown reflow; 125 dedicated tests |
| ktfmt CLI + editorconfig | 683 | 1,000 | Low | `editorconfig` crate; match ec4j globbing |
| **ktfmt subtotal (incl. parser)** | ~21k (+parser deps) | **~30k** | | 540 golden cases reusable |
| ktlint rule engine + core API (mutable CST, suppression, traversal, editorconfig props) | 5,844 | 6,000–8,000 | **High** | Needs rowan-like mutable tree; 2.0 traversal semantics |
| ktlint 104 standard rules | 19,415 | 22,000–26,000 | **High** | IndentationRule 1.6k alone; 3 code styles |
| ktlint CLI + 8 reporters + baseline | 3,161 | 3,000 | Low | SARIF/checkstyle/json formats are specs |
| **ktlint subtotal (shares parser)** | ~28k | **~33–37k** | | 1,962 tests need snippet extraction tooling |
| Test harness + corpus diffing (goldens, ktlint test extraction, differential fuzz vs JVM) | ktfmt 9.4k + ktlint 43.5k test LOC | 5,000 | Med | Differential testing vs JVM jar on real repos is mandatory |
| compose-rules ktlint checks (optional, adoption) | ~5k of 9.5k | 5,000 | Med | PSI-only on ktlint side |
| Integrations (Spotless step PR, Gradle plugin, IntelliJ plugin/LSP, pre-commit, Bazel) | — | 3,000–6,000 (mixed Kotlin/Rust) | Med | Spotless `nativeCmd` works day 1 |
| **Grand total** | | **≈ 70–85k** | | |

## Key risks

1. PSI-shape fidelity (parser + error recovery) — any divergence breaks both tools' logic.
2. Moving targets: ktfmt output changes most releases; ktlint 2.0 semantics shift; Kotlin syntax additions.
3. GraalVM native images now cover the "fast startup, no JVM" story for both upstreams.
4. Custom JVM rulesets (compose-rules, in-house rules) cannot load — same gap as ktlint-native, so parity with
   the native binary, not the JAR, is the realistic bar.
5. Distribution through JVM-centric build plugins (Spotless/Gradle) needs per-platform binary resolution.

## Sources

- https://github.com/Kotlin/ktfmt (CHANGELOG.md, releases), https://github.com/facebook/ktfmt/pull/584, https://github.com/facebook/ktfmt/issues/552
- https://github.com/ktlint/ktlint (CHANGELOG.md, releases), https://github.com/ktlint/ktlint/pull/3284
- https://github.com/diffplug/spotless/blob/main/plugin-gradle/README.md, `lib/src/main/java/com/diffplug/spotless/generic/NativeCmdStep.java`
- https://github.com/mrmans0n/compose-rules
- https://engineering.block.xyz/blog/adopting-ktfmt-and-detekt
- https://plugins.jetbrains.com/api/plugins/14912 , /15057
- GitHub REST API (stars, releases, code search counts), 2026-09-25
