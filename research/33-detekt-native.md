# 33 — detekt natively in ktrs: scope, parity and first deliverable (2026-10-09)

Question: what would it take to run detekt's rules natively, and what should be built first? Research and design
only; nothing is implemented. Sources: shallow clones of detekt at `v2.0.0-alpha.6` (401c64b) and `v1.23.8`
(0462637), the detekt jars run on the testbox, and the ktrs tree at ed15609.

Upstream paths below are relative to `https://github.com/detekt/detekt/blob/v2.0.0-alpha.6/` unless marked 1.x.
`C/` = `detekt-core/src/main/kotlin/dev/detekt/core/`.

## Verdict

| Item | Answer |
|---|---|
| Portable part | detekt's own `--analysis-mode light` (the CLI default and the Gradle `detekt` task): 134 of the 227 core rules, 76 of the 119 default-on ones. In that mode detekt itself skips the other 93, so a native light mode can be identical, not a subset. |
| Not portable | The 93 core rules implementing `RequiresAnalysisApi` (K2 Analysis API: types, symbols, resolved calls). Hand the run to the jar when `--analysis-mode full` is asked. |
| What ktrs has | The parser, `ktrs-psi` (typed PSI with the full `KtVisitorVoid` dispatch on the immutable tree), ktlint 1.8 natively (what detekt 2.0's `ktlint` rule set wraps), the checkstyle/SARIF/HTML/baseline writers, the jar hand-off, the golden and corpus-diff harnesses. |
| What is new | A detekt engine (YAML config, rule descriptors, suppression, signatures, baseline, reports), about 95 missing PSI accessors/helpers, the rules, and a flag-exact `detekt` CLI. Roughly 15k lines of Rust for core light mode; the ktlint wrapper set is extra. |
| First deliverable | A spike with go/no-go criteria, as research/15 was for ktlint: engine core + oracle probe + golden extractor + 27 rules (the `empty-blocks` set and the 12 rules that produce 93.6% of default findings on the corpus). No CLI yet. Section 6. |

## 0. Releases and pin

| Release | Date | Kotlin | Status |
|---|---|---|---|
| 1.23.8 | 2025-02-21 | 2.0.21 | Latest stable. No release in 20 months. |
| 2.0.0-alpha.0 … alpha.6 | 2025-09-04 … 2026-08-04 | 2.4.10 (alpha.6) | Pre-release; seven alphas. Milestone 2.0.0: 38 open, 1,251 closed. No beta or RC. |

2.0 is not released. Recommended pin: **`v2.0.0-alpha.6`**, with a version switch in the engine from the start.

| For alpha.6 | For 1.23.8 |
|---|---|
| Embeds Kotlin 2.4.10, the same compiler as the ktlint 2.0.0-ALPHA-4 jar that `ktrs_ast::psi` is already checked against; ktrs-psi is on 2.4.20. 1.23.8 parses with 2.0.21. | It is what people run: `libs.versions.toml` hits `io.gitlab.arturbosch.detekt` 5,696 vs `dev.detekt` 1,030 (GitHub code search, same caveats as research/25). |
| Its `ktlint` rule set wraps ktlint **1.8.0**, which ktrs runs natively (research/26). 1.23.8's `formatting` set wraps ktlint 0.50.0, which ktrs does not have. | More syntax-only rules: 146 core, 85 default-on. Twelve rules that were syntactic in 1.x need the Analysis API in 2.0. |
| `light`/`full` is an explicit, documented mode. In 1.x the split is implicit (`BindingContext.EMPTY`) and 13 syntax-only rules still read the binding context when present. | Stable: ids, flags and messages don't move. Each alpha renames things (16 rule names exist only in 1.23.8, 32 only in alpha.6). |
| The branch that gets fixes; 1.x is frozen. | |

Rules that became analysis-only in 2.0: CastToNullableType, ImplicitDefaultLocale, InstanceOfCheckForException,
LongParameterList, MapGetWithNotNullAssertionOperator, MemberNameEqualsClassName, OptionalUnit, SpreadOperator,
UnusedPrivateProperty, UseCheckOrError, UseDataClass, UseRequire. The direction upstream is toward the Analysis
API, so the portable share shrinks slowly with each release.

## 1. Rule inventory (v2.0.0-alpha.6)

Method: rules are the `::Rule` entries of each `*Provider.kt`; "default" is `active:` in
`detekt-core/src/main/resources/default-detekt-config.yml` (plugin sets: their `config/config.yml`), which agrees
with `@ActiveByDefault` on every rule; "needs analysis" is the class implementing `RequiresAnalysisApi` (95 files,
95 rules). No rule uses the Analysis API without the marker; the helper files that do are only called from marked
rules. Scripts were throwaway; rerun by repeating the method.

| Rule set | Rules | Default-on | Syntax-only | Syntax-only and default-on | Needs analysis | of those default-on | Syntax-only Kotlin LOC |
|---|--:|--:|--:|--:|--:|--:|--:|
| comments | 10 | 0 | 10 | 0 | 0 | 0 | 496 |
| complexity | 15 | 7 | 11 | 6 | 4 | 1 | 638 |
| coroutines | 9 | 4 | 1 | 0 | 8 | 4 | 15 |
| empty-blocks | 15 | 15 | 15 | 15 | 0 | 0 | 192 |
| exceptions | 15 | 11 | 11 | 9 | 4 | 2 | 329 |
| naming | 21 | 13 | 17 | 11 | 4 | 2 | 536 |
| performance | 8 | 4 | 5 | 2 | 3 | 2 | 168 |
| potential-bugs | 39 | 21 | 11 | 8 | 28 | 13 | 345 |
| style | 95 | 44 | 53 | 25 | 42 | 19 | 2,641 |
| **core, 9 sets (in detekt-cli)** | **227** | **119** | **134** | **76** | **93** | **43** | **5,360** |
| ktlint (plugin jar; wrappers of ktlint 1.8.0) | 100 | 85 | 100 | 85 | 0 | 0 | 913 |
| libraries (plugin jar) | 3 | 3 | 2 | 2 | 1 | 1 | 54 |
| ruleauthors (plugin jar) | 2 | 1 | 1 | 1 | 1 | 0 | 38 |
| **all 12 sets** | **332** | **208** | **237** | **164** | **95** | **44** | **6,365** |

- No core rule autocorrects. 84 of the 100 ktlint wrappers do.
- Shared code the syntax-only rules call: `detekt-psi-utils` 650 lines, `detekt-metrics` 553 lines.
- 1.23.8 for comparison: 216 rules in the same 11 non-formatting sets, 121 default-on, 149 syntax-only, 88
  syntax-only and default-on; the `formatting` set adds 78 wrappers (57 on).

### What fires on the corpus (jar, light mode, testbox)

6,123 files at `tools/corpus/REVISIONS` (5,730 `.kt`, 393 `.kts`). Counts are checkstyle `<error>` rows.

| Run | Exit | Findings | Rules firing | Wall |
|---|--:|--:|--:|--:|
| default config, sequential | 2 | 25,420 | 54 of 76 | 176 s |
| default config, `--parallel` (10 cores) | 2 | 25,420 | 54 of 76 | 64 s |
| `--all-rules --parallel` | 0 (`--fail-on-severity never`) | 75,773 | 102 of 134 | 48 s |
| default + the three plugin jars | 1 | – | – | crash, see below |
| same, `--excludes '**/sourceFiles/**'` | 0 | 211,775 (ktlint 176,650; libraries 9,705) | 118 | 167 s |
| `--all-rules` + plugins, same exclude | 0 | 292,719 (ktlint 207,241) | 181 | 274 s |
| 1.23.8, default config, `--parallel` | 2 | 28,469 | 62 | 47 s |

- Twelve rules give 93.6% of the default findings: WildcardImport 13,049, MaxLineLength 5,368, MagicNumber 1,405,
  InvalidPackageDeclaration 1,076, ReturnCount 580, PackageNaming 547, TooGenericExceptionCaught 413,
  TooManyFunctions 312, LongMethod 287, EmptyFunctionBlock 266, SwallowedException 257, CyclomaticComplexMethod 230.
- 22 default-on syntax-only rules never fire on the corpus; their parity rests on goldens.
- With `--debug` the default run logs 43 lines "The rule 'X' requires type resolution but it was run without it."
  (119 − 76). Without `--debug` nothing is said.
- The ktlint wrapper throws on the two Exposed `sourceFiles/*.kt` templates (`IllegalArgumentException: Required
  value was null`); a rule exception aborts the whole run with exit 1. The core rules don't throw on them.
- Sequential and parallel checkstyle files have the same rows in a different order: `LongMethod` reports from a
  `HashMap<KtNamedFunction, Int>` (identity hash), so its order inside a file changes between runs.
- In the `--debug` run, "Phase CreateSettings took 1m 28s" of 137 s: most of the jar's time is setting up the
  standalone Analysis API session, even in light mode.
- Jars: `detekt-cli-2.0.0-alpha.6-all.jar` sha256 `d46ca62e…c5fd` (85 MB), ktlint wrapper `48a8e4ae…d956`.
  Outputs: testbox `~/work/detekt-probe/out`.

### Full table

Columns: "Config keys" excludes the engine keys (`active`, `excludes`, `includes`, `aliases`, `autoCorrect`); LOC is
the rule file's code lines; corpus columns are findings in the runs above (core sets: default and `--all-rules`
runs; plugin sets: the two plugin runs). "–" = the rule does not run in light mode, or is off in that run.

### comments (10)

| Rule | Default | Needs analysis | Config keys | LOC | Corpus, default | Corpus, `--all-rules` |
|---|---|---|--:|--:|--:|--:|
| AbsentOrWrongFileLicense | off | no | 2 | 23 | – | 0 |
| DeprecatedBlockTag | off | no | 0 | 22 | – | 1 |
| DocumentationOverPrivateFunction | off | no | 0 | 19 | – | 530 |
| DocumentationOverPrivateProperty | off | no | 0 | 8 | – | 270 |
| EndOfSentenceFormat | off | no | 1 | 28 | – | 1,557 |
| KDocReferencesNonPublicProperty | off | no | 0 | 76 | – | 19 |
| OutdatedDocumentation | off | no | 4 | 185 | – | 676 |
| UndocumentedPublicClass | off | no | 6 | 62 | – | 1,721 |
| UndocumentedPublicFunction | off | no | 1 | 22 | – | 3,139 |
| UndocumentedPublicProperty | off | no | 2 | 51 | – | 4,290 |

### complexity (15)

| Rule | Default | Needs analysis | Config keys | LOC | Corpus, default | Corpus, `--all-rules` |
|---|---|---|--:|--:|--:|--:|
| CognitiveComplexMethod | off | no | 1 | 18 | – | 350 |
| ComplexCondition | on | no | 1 | 53 | 101 | 101 |
| ComplexInterface | off | no | 4 | 63 | – | 40 |
| CyclomaticComplexMethod | on | no | 6 | 61 | 230 | 230 |
| LabeledExpression | off | no | 1 | 34 | – | 962 |
| LargeClass | on | no | 1 | 39 | 67 | 67 |
| LongMethod | on | no | 1 | 53 | 287 | 287 |
| LongParameterList | on | yes | 5 | 78 | – | – |
| MethodOverloading | off | no | 1 | 54 | – | 19 |
| NamedArguments | off | yes | 3 | 64 | – | – |
| NestedBlockDepth | on | no | 1 | 79 | 26 | 26 |
| NestedScopeFunctions | off | yes | 2 | 73 | – | – |
| ReplaceSafeCallChainWithRun | off | yes | 0 | 24 | – | – |
| StringLiteralDuplication | off | no | 4 | 61 | – | 301 |
| TooManyFunctions | on | no | 10 | 123 | 312 | 312 |

### coroutines (9)

| Rule | Default | Needs analysis | Config keys | LOC | Corpus, default | Corpus, `--all-rules` |
|---|---|---|--:|--:|--:|--:|
| CoroutineLaunchedInTestWithoutRunTest | off | yes | 0 | 76 | – | – |
| GlobalCoroutineUsage | off | no | 0 | 15 | – | 167 |
| InjectDispatcher | on | yes | 1 | 54 | – | – |
| RedundantSuspendModifier | on | yes | 0 | 59 | – | – |
| SleepInsteadOfDelay | on | yes | 0 | 106 | – | – |
| SuspendFunInFinallySection | off | yes | 0 | 51 | – | – |
| SuspendFunSwallowedCancellation | off | yes | 0 | 204 | – | – |
| SuspendFunWithCoroutineScopeReceiver | off | yes | 0 | 26 | – | – |
| SuspendFunWithFlowReturnType | on | yes | 0 | 19 | – | – |

### empty-blocks (15)

| Rule | Default | Needs analysis | Config keys | LOC | Corpus, default | Corpus, `--all-rules` |
|---|---|---|--:|--:|--:|--:|
| EmptyCatchBlock | on | no | 1 | 21 | 7 | 7 |
| EmptyClassBlock | on | no | 0 | 18 | 0 | 0 |
| EmptyDefaultConstructor | on | no | 0 | 28 | 6 | 6 |
| EmptyDoWhileBlock | on | no | 0 | 7 | 0 | 0 |
| EmptyElseBlock | on | no | 0 | 14 | 0 | 0 |
| EmptyFinallyBlock | on | no | 0 | 7 | 1 | 1 |
| EmptyForBlock | on | no | 0 | 7 | 4 | 4 |
| EmptyFunctionBlock | on | no | 1 | 22 | 266 | 266 |
| EmptyIfBlock | on | no | 0 | 14 | 0 | 0 |
| EmptyInitBlock | on | no | 0 | 6 | 0 | 0 |
| EmptyKotlinFile | on | no | 0 | 19 | 1 | 1 |
| EmptySecondaryConstructor | on | no | 0 | 6 | 0 | 0 |
| EmptyTryBlock | on | no | 0 | 7 | 0 | 0 |
| EmptyWhenBlock | on | no | 0 | 9 | 0 | 0 |
| EmptyWhileBlock | on | no | 0 | 7 | 14 | 14 |

### exceptions (15)

| Rule | Default | Needs analysis | Config keys | LOC | Corpus, default | Corpus, `--all-rules` |
|---|---|---|--:|--:|--:|--:|
| ErrorUsageWithThrowable | off | yes | 0 | 32 | – | – |
| ExceptionRaisedInUnexpectedLocation | on | no | 1 | 26 | 1 | 1 |
| InstanceOfCheckForException | on | yes | 0 | 45 | – | – |
| NotImplementedDeclaration | off | no | 0 | 23 | – | 213 |
| ObjectExtendsThrowable | off | yes | 0 | 29 | – | – |
| PrintStackTrace | on | no | 0 | 25 | 15 | 15 |
| RethrowCaughtException | on | no | 0 | 22 | 9 | 9 |
| ReturnFromFinally | on | yes | 1 | 57 | – | – |
| SwallowedException | on | no | 2 | 93 | 257 | 257 |
| ThrowingExceptionFromFinally | on | no | 0 | 12 | 41 | 41 |
| ThrowingExceptionInMain | off | no | 0 | 9 | – | 9 |
| ThrowingExceptionsWithoutMessageOrCause | on | no | 1 | 30 | 12 | 12 |
| ThrowingNewInstanceOfSameException | on | no | 0 | 24 | 0 | 0 |
| TooGenericExceptionCaught | on | no | 2 | 35 | 413 | 413 |
| TooGenericExceptionThrown | on | no | 1 | 30 | 87 | 87 |

### naming (21)

| Rule | Default | Needs analysis | Config keys | LOC | Corpus, default | Corpus, `--all-rules` |
|---|---|---|--:|--:|--:|--:|
| BooleanPropertyNaming | off | yes | 1 | 42 | – | – |
| ClassNaming | on | no | 1 | 23 | 3 | 3 |
| ConstructorParameterNaming | on | no | 3 | 43 | 32 | 32 |
| EnumNaming | on | no | 1 | 20 | 4 | 4 |
| ForbiddenClassName | off | no | 1 | 15 | – | 0 |
| FunctionNameMaxLength | off | no | 1 | 23 | – | 7,992 |
| FunctionNameMinLength | off | no | 1 | 20 | – | 69 |
| FunctionNaming | on | no | 2 | 32 | 174 | 174 |
| FunctionParameterNaming | on | no | 2 | 30 | 192 | 192 |
| InvalidPackageDeclaration | on | no | 2 | 38 | 1,076 | 1,076 |
| LambdaParameterNaming | off | no | 1 | 24 | – | 3 |
| MatchingDeclarationName | on | no | 2 | 62 | 164 | 164 |
| MemberNameEqualsClassName | on | yes | 1 | 55 | – | – |
| NoNameShadowing | on | yes | 0 | 94 | – | – |
| NonBooleanPropertyPrefixedWithIs | off | yes | 1 | 53 | – | – |
| ObjectPropertyNaming | on | no | 3 | 45 | 0 | 0 |
| PackageNaming | on | no | 1 | 21 | 547 | 547 |
| TopLevelPropertyNaming | on | no | 3 | 44 | 25 | 25 |
| VariableMaxLength | off | no | 1 | 22 | – | 7 |
| VariableMinLength | off | no | 1 | 26 | – | 0 |
| VariableNaming | on | no | 3 | 48 | 107 | 107 |

### performance (8)

| Rule | Default | Needs analysis | Config keys | LOC | Corpus, default | Corpus, `--all-rules` |
|---|---|---|--:|--:|--:|--:|
| ArrayPrimitive | on | yes | 0 | 44 | – | – |
| CouldBeSequence | off | yes | 1 | 51 | – | – |
| ForEachOnRange | on | no | 0 | 50 | 0 | 0 |
| SpreadOperator | on | yes | 0 | 40 | – | – |
| UnnecessaryInitOnArray | off | no | 0 | 50 | – | 7 |
| UnnecessaryPartOfBinaryExpression | off | no | 0 | 30 | – | 0 |
| UnnecessaryTemporaryInstantiation | on | no | 0 | 16 | 0 | 0 |
| UnnecessaryTypeCasting | off | no | 0 | 22 | – | 0 |

### potential-bugs (39)

| Rule | Default | Needs analysis | Config keys | LOC | Corpus, default | Corpus, `--all-rules` |
|---|---|---|--:|--:|--:|--:|
| AvoidReferentialEquality | on | yes | 1 | 38 | – | – |
| CastNullableToNonNullableType | off | yes | 1 | 28 | – | – |
| CastToNullableType | off | yes | 0 | 23 | – | – |
| CharArrayToStringCall | off | yes | 0 | 51 | – | – |
| Deprecation | off | yes | 0 | 28 | – | – |
| DontDowncastCollectionTypes | off | yes | 0 | 50 | – | – |
| DoubleMutabilityForCollection | on | yes | 1 | 41 | – | – |
| ElseCaseInsteadOfExhaustiveWhen | off | yes | 1 | 41 | – | – |
| EqualsAlwaysReturnsTrueOrFalse | on | no | 0 | 42 | 1 | 1 |
| EqualsWithHashCodeExist | on | no | 0 | 39 | 4 | 4 |
| ExitOutsideMain | off | yes | 0 | 25 | – | – |
| ExplicitGarbageCollectionCall | on | no | 0 | 28 | 4 | 4 |
| HasPlatformType | on | yes | 0 | 32 | – | – |
| IgnoredReturnValue | on | yes | 5 | 139 | – | – |
| ImplicitDefaultLocale | on | yes | 0 | 33 | – | – |
| ImplicitUnitReturnType | off | yes | 2 | 40 | – | – |
| InvalidRange | on | no | 0 | 20 | 0 | 0 |
| IteratorHasNextCallsNextMethod | on | no | 0 | 26 | 0 | 0 |
| IteratorNotThrowingNoSuchElementException | on | no | 0 | 29 | 2 | 2 |
| LateinitUsage | off | no | 1 | 23 | – | 147 |
| MapGetWithNotNullAssertionOperator | on | yes | 0 | 52 | – | – |
| MissingPackageDeclaration | off | no | 0 | 7 | – | 145 |
| MissingSuperCall | off | yes | 1 | 36 | – | – |
| MissingUseCall | off | yes | 1 | 220 | – | – |
| NullCheckOnMutableProperty | off | yes | 0 | 89 | – | – |
| NullableToStringCall | off | yes | 0 | 36 | – | – |
| PropertyUsedBeforeDeclaration | off | yes | 0 | 112 | – | – |
| UnconditionalJumpStatementInLoop | off | no | 0 | 46 | – | 17 |
| UnnamedParameterUse | off | yes | 4 | 96 | – | – |
| UnnecessaryNotNullCheck | off | yes | 0 | 39 | – | – |
| UnnecessaryNotNullOperator | on | yes | 0 | 24 | – | – |
| UnnecessarySafeCall | on | yes | 0 | 26 | – | – |
| UnreachableCatchBlock | on | yes | 0 | 25 | – | – |
| UnreachableCode | on | yes | 0 | 25 | – | – |
| UnsafeCallOnNullableType | on | yes | 0 | 25 | – | – |
| UnsafeCast | on | yes | 0 | 28 | – | – |
| UnusedUnaryOperator | on | yes | 0 | 30 | – | – |
| UselessPostfixExpression | on | no | 0 | 60 | 0 | 0 |
| WrongEqualsTypeParameter | on | no | 0 | 25 | 0 | 0 |

### style (95)

| Rule | Default | Needs analysis | Config keys | LOC | Corpus, default | Corpus, `--all-rules` |
|---|---|---|--:|--:|--:|--:|
| AbstractClassCanBeConcreteClass | on | yes | 0 | 44 | – | – |
| AbstractClassCanBeInterface | on | yes | 1 | 94 | – | – |
| AlsoCouldBeApply | off | no | 0 | 14 | – | 75 |
| BracesOnIfStatements | off | no | 2 | 83 | – | 236 |
| BracesOnWhenStatements | off | no | 2 | 74 | – | 430 |
| CanBeNonNullable | off | yes | 0 | 412 | – | – |
| CascadingCallWrapping | off | no | 1 | 74 | – | 1,266 |
| ClassOrdering | off | no | 0 | 121 | – | 1,151 |
| CollapsibleIfStatements | off | no | 0 | 18 | – | 34 |
| DataClassContainsFunctions | off | no | 2 | 32 | – | 102 |
| DataClassShouldBeImmutable | off | no | 0 | 27 | – | 84 |
| DestructuringDeclarationWithTooManyEntries | on | no | 1 | 14 | 9 | 9 |
| DoubleNegativeExpression | off | yes | 0 | 52 | – | – |
| DoubleNegativeLambda | off | no | 2 | 76 | – | 12 |
| EqualsNullCall | on | no | 0 | 13 | 2 | 2 |
| EqualsOnSignatureLine | off | no | 0 | 15 | – | 0 |
| ExplicitCollectionElementAccessMethod | off | yes | 0 | 91 | – | – |
| ExplicitItLambdaMultipleParameters | on | no | 0 | 16 | 11 | 11 |
| ExplicitItLambdaParameter | on | no | 0 | 21 | 25 | 25 |
| ExpressionBodySyntax | off | no | 1 | 36 | – | 956 |
| ForbiddenAnnotation | off | yes | 1 | 54 | – | – |
| ForbiddenComment | on | no | 2 | 88 | 111 | 111 |
| ForbiddenImport | off | no | 2 | 36 | – | 0 |
| ForbiddenMethodCall | off | yes | 1 | 130 | – | – |
| ForbiddenNamedParam | off | yes | 1 | 52 | – | – |
| ForbiddenOptIn | off | yes | 1 | 44 | – | – |
| ForbiddenSuppress | off | no | 1 | 47 | – | 0 |
| ForbiddenVoid | on | yes | 2 | 38 | – | – |
| FunctionOnlyReturningConstant | on | no | 3 | 56 | 10 | 10 |
| LoopWithTooManyJumpStatements | on | no | 1 | 45 | 134 | 134 |
| MagicNumber | on | no | 11 | 146 | 1,405 | 1,405 |
| MandatoryBracesLoops | off | no | 0 | 43 | – | 27 |
| MaxChainedCallsOnSameLine | off | yes | 1 | 68 | – | – |
| MaxLineLength | on | no | 5 | 76 | 5,368 | 5,368 |
| MayBeConstant | on | no | 0 | 82 | 28 | 28 |
| ModifierOrder | on | no | 0 | 34 | 2 | 2 |
| MultilineLambdaItParameter | off | yes | 0 | 37 | – | – |
| MultilineRawStringIndentation | off | no | 2 | 119 | – | 14,276 |
| NestedClassesVisibility | on | no | 0 | 17 | 0 | 0 |
| NewLineAtEndOfFile | on | no | 0 | 26 | 31 | 31 |
| NoTabs | off | no | 0 | 10 | – | 0 |
| NullableBooleanCheck | off | yes | 0 | 33 | – | – |
| ObjectLiteralToLambda | on | yes | 0 | 52 | – | – |
| OptionalAbstractKeyword | on | no | 0 | 24 | 0 | 0 |
| OptionalUnit | off | yes | 0 | 91 | – | – |
| ProtectedMemberInFinalClass | on | no | 0 | 31 | 0 | 0 |
| RangeUntilInsteadOfRangeTo | off | no | 0 | 35 | – | 0 |
| RedundantConstructorKeyword | off | no | 0 | 22 | – | 3 |
| RedundantExplicitType | off | yes | 0 | 45 | – | – |
| RedundantHigherOrderMapUsage | on | yes | 0 | 73 | – | – |
| RedundantVisibilityModifier | off | no | 0 | 74 | – | 6,382 |
| ReturnCount | on | no | 5 | 50 | 580 | 580 |
| SafeCast | on | no | 0 | 33 | 0 | 0 |
| SerialVersionUIDInSerializableClass | on | no | 0 | 78 | 5 | 5 |
| SpacingAfterPackageAndImports | off | no | 0 | 45 | – | 26 |
| StringShouldBeRawString | off | no | 2 | 94 | – | 344 |
| ThrowsCount | on | no | 2 | 31 | 84 | 84 |
| TrailingWhitespace | off | no | 0 | 33 | – | 237 |
| TrimMultilineRawString | off | no | 1 | 48 | – | 119 |
| UnderscoresInNumericLiterals | off | no | 2 | 69 | – | 483 |
| UnnecessaryAny | off | yes | 0 | 151 | – | – |
| UnnecessaryApply | on | yes | 0 | 62 | – | – |
| UnnecessaryBackticks | off | no | 0 | 22 | – | 1 |
| UnnecessaryBracesAroundTrailingLambda | off | yes | 0 | 20 | – | – |
| UnnecessaryFilter | on | yes | 0 | 101 | – | – |
| UnnecessaryFullyQualifiedName | off | yes | 1 | 173 | – | – |
| UnnecessaryInheritance | on | no | 0 | 14 | 0 | 0 |
| UnnecessaryInnerClass | off | yes | 0 | 72 | – | – |
| UnnecessaryLet | off | yes | 0 | 78 | – | – |
| UnnecessaryParentheses | off | no | 1 | 101 | – | 789 |
| UnnecessaryReversed | off | yes | 0 | 56 | – | – |
| UnusedImport | off | yes | 1 | 171 | – | – |
| UnusedParameter | on | no | 1 | 71 | 66 | 66 |
| UnusedPrivateClass | on | no | 0 | 123 | 2 | 2 |
| UnusedPrivateFunction | on | yes | 1 | 138 | – | – |
| UnusedPrivateProperty | on | yes | 1 | 148 | – | – |
| UnusedVariable | on | yes | 1 | 84 | – | – |
| UseAnyOrNoneInsteadOfFind | on | yes | 0 | 36 | – | – |
| UseArrayLiteralsInAnnotations | on | no | 0 | 41 | 0 | 0 |
| UseCheckNotNull | on | yes | 0 | 18 | – | – |
| UseCheckOrError | on | yes | 0 | 20 | – | – |
| UseDataClass | off | yes | 1 | 110 | – | – |
| UseEmptyCounterpart | off | yes | 0 | 30 | – | – |
| UseIfEmptyOrIfBlank | off | yes | 0 | 77 | – | – |
| UseIfInsteadOfWhen | off | no | 1 | 23 | – | 641 |
| UseIsNullOrEmpty | on | yes | 0 | 127 | – | – |
| UseLet | off | no | 0 | 20 | – | 28 |
| UseOrEmpty | on | yes | 0 | 66 | – | – |
| UseRequire | on | yes | 0 | 19 | – | – |
| UseRequireNotNull | on | yes | 0 | 18 | – | – |
| UseSumOfInsteadOfFlatMapSize | off | yes | 0 | 32 | – | – |
| UselessCallOnNotNull | on | yes | 0 | 68 | – | – |
| UtilityClassWithPublicConstructor | on | no | 0 | 68 | 6 | 6 |
| VarCouldBeVal | on | yes | 1 | 156 | – | – |
| WildcardImport | on | no | 1 | 32 | 13,049 | 13,049 |

### ktlint (100)

| Rule | Default | Needs analysis | Config keys | LOC | Corpus, default | Corpus, `--all-rules` |
|---|---|---|--:|--:|--:|--:|
| AnnotationOnSeparateLine | on | no | 1 | 13 | 62 | 62 |
| AnnotationSpacing | on | no | 0 | 6 | 4 | 4 |
| ArgumentListWrapping | on | no | 2 | 17 | 7,447 | 7,447 |
| BackingPropertyNaming | on | no | 0 | 4 | 92 | 92 |
| BinaryExpressionWrapping | on | no | 1 | 18 | 587 | 587 |
| BlankLineBeforeDeclaration | off | no | 0 | 9 | – | 1,367 |
| BlankLineBetweenWhenConditions | on | no | 1 | 12 | 1,206 | 1,206 |
| BlockCommentInitialStarAlignment | on | no | 0 | 6 | 36 | 36 |
| ChainMethodContinuation | off | no | 2 | 18 | – | 6,232 |
| ChainWrapping | on | no | 1 | 11 | 23 | 23 |
| ClassName | off | no | 0 | 4 | – | 47 |
| ClassSignature | on | no | 2 | 19 | 3,613 | 3,613 |
| CommentSpacing | on | no | 0 | 5 | 32 | 32 |
| CommentWrapping | on | no | 1 | 11 | 38 | 38 |
| ConditionWrapping | on | no | 1 | 13 | 0 | 0 |
| ContextReceiverListWrapping | on | no | 1 | 15 | 0 | 0 |
| ContextReceiverMapping | on | no | 1 | 14 | 0 | 0 |
| EnumEntryNameCase | on | no | 1 | 13 | 5 | 5 |
| EnumWrapping | on | no | 1 | 12 | 33 | 33 |
| ExpressionOperandWrapping | off | no | 1 | 11 | – | 241 |
| Filename | on | no | 0 | 4 | 244 | 244 |
| FinalNewline | on | no | 1 | 9 | 31 | 31 |
| FunKeywordSpacing | on | no | 0 | 5 | 0 | 0 |
| FunctionExpressionBody | on | no | 1 | 15 | 1,500 | 1,500 |
| FunctionLiteral | on | no | 1 | 15 | 1,404 | 1,404 |
| FunctionName | off | no | 0 | 11 | – | 191 |
| FunctionReturnTypeSpacing | on | no | 0 | 12 | 11 | 11 |
| FunctionSignature | on | no | 3 | 22 | 9,787 | 9,787 |
| FunctionStartOfBodySpacing | on | no | 0 | 6 | 20 | 20 |
| FunctionTypeModifierSpacing | on | no | 0 | 6 | 0 | 0 |
| FunctionTypeReferenceSpacing | on | no | 0 | 6 | 0 | 0 |
| IfElseBracing | off | no | 1 | 11 | – | 26 |
| IfElseWrapping | off | no | 1 | 11 | – | 168 |
| ImportOrdering | on | no | 0 | 13 | 810 | 810 |
| Indentation | on | no | 2 | 14 | 119,339 | 119,339 |
| Kdoc | on | no | 0 | 8 | 131 | 131 |
| KdocWrapping | on | no | 1 | 11 | 0 | 0 |
| MaximumLineLength | on | no | 1 | 14 | 3,556 | 3,556 |
| MixedConditionOperators | on | no | 0 | 5 | 39 | 39 |
| ModifierListSpacing | on | no | 1 | 12 | 0 | 0 |
| ModifierOrdering | on | no | 0 | 5 | 2 | 2 |
| MultiLineIfElse | on | no | 1 | 12 | 259 | 259 |
| MultilineExpressionWrapping | off | no | 2 | 14 | – | 18,392 |
| MultilineLoop | on | no | 1 | 11 | 36 | 36 |
| NoBlankLineBeforeRbrace | on | no | 0 | 5 | 84 | 84 |
| NoBlankLineInList | off | no | 0 | 5 | – | 73 |
| NoBlankLinesInChainedMethodCalls | on | no | 0 | 6 | 0 | 0 |
| NoConsecutiveBlankLines | on | no | 0 | 5 | 130 | 130 |
| NoConsecutiveComments | off | no | 0 | 4 | – | 48 |
| NoEmptyClassBody | on | no | 0 | 5 | 0 | 0 |
| NoEmptyFile | on | no | 0 | 4 | 6 | 6 |
| NoEmptyFirstLineInClassBody | off | no | 1 | 11 | – | 1,604 |
| NoEmptyFirstLineInMethodBlock | on | no | 0 | 6 | 22 | 22 |
| NoLineBreakAfterElse | on | no | 0 | 5 | 0 | 0 |
| NoLineBreakBeforeAssignment | on | no | 0 | 6 | 0 | 0 |
| NoMultipleSpaces | on | no | 0 | 5 | 190 | 190 |
| NoSemicolons | on | no | 0 | 5 | 6 | 6 |
| NoSingleLineBlockComment | off | no | 1 | 10 | – | 55 |
| NoTrailingSpaces | on | no | 0 | 5 | 232 | 232 |
| NoUnitReturn | on | no | 0 | 5 | 4 | 4 |
| NoUnusedImports | on | no | 0 | 6 | 46 | 46 |
| NoWildcardImports | on | no | 1 | 13 | 13,275 | 13,275 |
| NullableTypeSpacing | on | no | 0 | 5 | 0 | 0 |
| PackageName | on | no | 0 | 4 | 1 | 1 |
| ParameterListSpacing | on | no | 0 | 12 | 4 | 4 |
| ParameterListWrapping | on | no | 1 | 14 | 755 | 755 |
| ParameterWrapping | on | no | 1 | 18 | 5 | 5 |
| PropertyName | on | no | 1 | 10 | 446 | 446 |
| PropertyWrapping | on | no | 1 | 15 | 35 | 35 |
| SpacingAroundAngleBrackets | on | no | 0 | 5 | 0 | 0 |
| SpacingAroundColon | on | no | 0 | 5 | 175 | 175 |
| SpacingAroundComma | on | no | 0 | 5 | 5 | 5 |
| SpacingAroundCurly | on | no | 1 | 11 | 84 | 84 |
| SpacingAroundDot | on | no | 0 | 6 | 4 | 4 |
| SpacingAroundDoubleColon | on | no | 0 | 5 | 0 | 0 |
| SpacingAroundKeyword | on | no | 0 | 5 | 57 | 57 |
| SpacingAroundOperators | on | no | 0 | 5 | 11 | 11 |
| SpacingAroundParens | on | no | 0 | 5 | 30 | 30 |
| SpacingAroundRangeOperator | on | no | 0 | 5 | 4 | 4 |
| SpacingAroundSquareBrackets | on | no | 0 | 6 | 0 | 0 |
| SpacingAroundUnaryOperator | on | no | 0 | 5 | 0 | 0 |
| SpacingBetweenDeclarationsWithAnnotations | on | no | 0 | 6 | 40 | 40 |
| SpacingBetweenDeclarationsWithComments | on | no | 0 | 6 | 43 | 43 |
| SpacingBetweenFunctionNameAndOpeningParenthesis | on | no | 0 | 6 | 0 | 0 |
| StatementWrapping | on | no | 1 | 12 | 570 | 570 |
| StringTemplate | on | no | 0 | 5 | 88 | 88 |
| StringTemplateIndent | off | no | 1 | 14 | – | 747 |
| ThenSpacing | on | no | 0 | 6 | 0 | 0 |
| TrailingCommaOnCallSite | on | no | 0 | 15 | 5,477 | 5,477 |
| TrailingCommaOnDeclarationSite | on | no | 0 | 16 | 3,688 | 3,688 |
| TryCatchFinallySpacing | off | no | 1 | 11 | – | 59 |
| TypeArgumentComment | on | no | 0 | 5 | 0 | 0 |
| TypeArgumentListSpacing | on | no | 1 | 12 | 0 | 0 |
| TypeParameterComment | on | no | 0 | 5 | 0 | 0 |
| TypeParameterListSpacing | on | no | 1 | 12 | 17 | 17 |
| UnnecessaryParenthesesBeforeTrailingLambda | on | no | 0 | 6 | 5 | 5 |
| ValueArgumentComment | on | no | 0 | 5 | 0 | 0 |
| ValueParameterComment | on | no | 0 | 5 | 23 | 23 |
| WhenEntryBracing | off | no | 1 | 11 | – | 1,341 |
| Wrapping | on | no | 1 | 15 | 741 | 741 |

### libraries (3)

| Rule | Default | Needs analysis | Config keys | LOC | Corpus, default | Corpus, `--all-rules` |
|---|---|---|--:|--:|--:|--:|
| ForbiddenPublicDataClass | on | no | 1 | 23 | 562 | 562 |
| LibraryCodeMustSpecifyReturnType | on | yes | 1 | 43 | – | – |
| LibraryEntitiesShouldNotBePublic | on | no | 0 | 31 | 9,143 | 9,143 |

### ruleauthors (2)

| Rule | Default | Needs analysis | Config keys | LOC | Corpus, default | Corpus, `--all-rules` |
|---|---|---|--:|--:|--:|--:|
| LeakingSessionBoundType | off | yes | 0 | 56 | – | – |
| UseEntityAtName | on | no | 0 | 38 | 0 | 0 |

## 2. What "detekt-identical" means

Everything here is alpha.6 unless a "1.23.8:" note says otherwise.

### Config

- YAML via snakeyaml-engine 2.10: duplicate keys rejected, non-map root is an error. `C/config/YamlConfig.kt`.
- `--config a;b` (path separator: `;` on Windows, `:` elsewhere): last file wins, per key; lists and maps are not
  merged. `--config-resource` is read only when `--config` is absent. `C/tooling/ProcessingSpecSettingsBridge.kt`,
  `C/config/CompositeConfig.kt`.
- No `--config`: the default config alone. With `--config` and no `--build-upon-default-config`: the user's file
  alone, and rules are enumerated from **the config's keys** (`C/RuleDescriptor.kt:43-77`), so a rule missing from
  the file does not run and rule order follows the file.
- `--all-rules`: `active` = declared value, else true (`C/config/AllRulesConfig.kt`). `--auto-correct` absent: every
  `autoCorrect` reads false. `--disable-default-rulesets` drops the nine core providers.
- Plugin `config/config.yml` files are appended to the default config as text and parsed as one document
  (`C/tooling/DefaultConfigProvider.kt`).
- `config:` block: `validation`, `warningsAsErrors`, `checkExhaustiveness`, `excludes`. Validation errors
  ("Property 'x' is misspelled or does not exist." with a Levenshtein suggestion, wrong nesting, list-vs-string)
  exit 3; deprecations and exhaustiveness are warnings. `C/config/validation/`.
- Engine keys: rule set `active` (default true), rule `active` (default false), `includes`/`excludes` on both
  levels (Java `glob:` on `./<path relative to --base-path>`; ktrs already has `java_glob.rs`), `severity`,
  `aliases`, `autoCorrect`, `ignoreAnnotated`, `ignoreFunction`. `RuleName/suffix` keys make extra instances of a
  rule. CLI `--includes/--excludes` match the same path **without** `./`.
- Values: `by config(default)` supports String, Boolean, Int, `List<String>`, `ValuesWithReason`; strings are
  coerced by the default's type; regexes are compiled per rule with `String::toRegex` (Java regex: reuse
  `KotlinRegex` from ktrs-compose). `detekt-api/src/main/kotlin/dev/detekt/api/ConfigProperty.kt`,
  `C/config/BaseConfig.kt`. A wrong primitive type is not a validation error; it throws when the rule reads it (exit 1).
- 1.23.8: `build.maxIssues`/`weights`, `output-reports`, comma-separated string lists, `,`/`;` separators on the
  CLI, path filters on absolute paths.

### Execution

- Phases: validate config → parse → load rules → analyze → report. `C/tooling/Lifecycle.kt`.
- Per file: filter by `includes`/`excludes`, one fresh rule instance per file, skip rules suppressed at file level,
  autocorrect rules first. `C/Analyzer.kt:64-88`.
- Findings stay in file order → rule order (config key order) → report order. Core never sorts or deduplicates;
  `--parallel` keeps the order. HTML and Markdown sort on their own.
- A rule exception aborts the run: message "Analyzing <path> led to an exception. …", stack trace, exit 1.
- Syntax errors are not checked; rules run on the error-recovering PSI (unlike ktlint, which rejects the file).
- Files: `.kt`/`.kts` under `--input`, walked with `kotlin.io.path.walk()`; final order unverified.

### Suppression

- `@Suppress`/`@SuppressWarnings` (matched by the annotation's type text) on the finding's element or any
  `KtAnnotated` ancestor up to the file. Ids: rule instance id, `all`/`All`/`ALL`, the rule set id,
  `<set>.<id>`, `<set>:<id>`, each configured alias; `detekt.`/`detekt:` prefixes are stripped anywhere in the
  argument's source text. `ForbiddenSuppress` can't be suppressed. `C/suppressors/Suppressions.kt`.
- Aliases come from the config (`aliases:` in the default file), not from the rule class, so they vanish when a
  config replaces the default.
- `ignoreAnnotated` (FQ-name globs; in light mode the FQ name is guessed from imports and the package) and
  `ignoreFunction` (signatures). `C/suppressors/`, `detekt-psi-utils/…/AnnotationExcluder.kt`, `FunctionMatcher.kt`.
- 1.23.8: only the first `Suppress` annotation per element is read.

### Baseline

- `<SmellBaseline><ManuallySuppressedIssues/><CurrentIssues><ID>…</ID></CurrentIssues></SmellBaseline>`, StAX
  writer with two-space indent. `C/baseline/`.
- ID = `<rule instance id>:<file name without directory>:<signature>`. No line or column. 1.23.8: `<id>:<signature>`.
- Signature (`detekt-api/…/internal/Signatures.kt`): enclosing class names joined by `.`, then `$`, then a
  per-kind rendering of the element's source text (functions without the parameter list, classes with supertypes,
  anything else its full text), whitespace collapsed. It must be ported exactly: it is also SARIF's fingerprint.
- `--create-baseline` needs `--baseline`, keeps `ManuallySuppressedIssues`, writes sorted IDs, writes nothing when
  there are no findings and no file, and never fails the run.

### Findings and locations

- Row = rule instance id, severity (`Error` default, from config only), message, start line:col, end line:col,
  text range, base-path-relative path, signature. `detekt-api/…/Issue.kt`, `Location.kt`, `Entity.kt`.
- Line/column from `PsiDiagnosticUtils` on the LF-normalized text, 1-based (columns in UTF-16 units: confirmed by the spike, "Spike result"; before that unverified in
  this checkout; ktrs offsets are UTF-8, `PositionInTextLocator` is the existing UTF-16 table).
- Reported element: `Entity.from(element)`, `Entity.atName(decl)` (name identifier), or
  `Entity.atPackageOrFirstDecl(file)`.
- Rule ids are class names; messages are format strings in the rule (Kotlin `Int`/`Double` formatting).

### Reports

| `--report` id | Module | Bytes depend on |
|---|---|---|
| `checkstyle` | `detekt-report-checkstyle` | Nothing else: files in first-occurrence order, rows in analyzer order, `source="detekt.<id>"`, all non-ASCII as hex character references. ktrs has a checkstyle writer for ktlint. |
| `sarif` | `detekt-report-sarif` | sarif4k 0.6.0 pretty printing; lists every rule instance (active or not) with help URIs holding the detekt version; `partialFingerprints["signature/v1"]` = SHA-1 of the signature. |
| `html` | `detekt-report-html` | Version and UTC timestamp (mask); own sort; metrics; code snippets read from disk. |
| `markdown` | `detekt-report-markdown` | Same as html. `--help` advertises `md`, which matches no report and writes nothing. |

- No `txt` report in 2.0. 1.23.8 ids: `txt`, `xml`, `html`, `md`, `sarif`.
- Unknown report ids are ignored; report-writing errors are swallowed. `C/reporting/OutputFacade.kt`.
- Console: only `LiteIssuesReport` is on by default: `e: <absolute path>:<line>:<col> <message> [<id>]`, then a
  blank line, then `Analysis failed with N issues.` A clean run prints nothing. Five more console reports
  (grouped, complexity, project statistics, notifications) are off by default; the metrics behind them are eleven
  processors in `detekt-metrics`. ANSI colour is off on Windows.
- 1.23.8 Lite line: `<path>:<line>:<col>: <message> [<id>]`; "Analysis failed with N weighted issues."

### CLI

- JCommander 1.85; flags in `detekt-cli/src/main/kotlin/dev/detekt/cli/CliArgs.kt` (26 options, one hidden
  `--run-rule`, free args go to the Kotlin compiler). `--report id:path` splits on the first `:`.
  `--fail-on-severity error|warning|info|never`, case-insensitive. `--generate-config <path>`. `--version`.
- Exit codes (`detekt-cli/…/Main.kt:72-75`): 0 clean or `--help`; 1 argument error (message + usage) or any
  unexpected exception (stack trace on stderr); 2 issues found; 3 invalid config (validator only; a YAML syntax
  error is exit 1).
- 1.23.8: `--max-issues` instead of `--fail-on-severity`, no `--analysis-mode`, boolean `--generate-config`.
- Not reproducible byte for byte: JVM stack traces on the exit-1 paths (same as the ktlint drop-in: mask them).

### detekt's rule tests as goldens

- 249 spec files, about 3,996 tests in the rule modules; about 1,700 belong to syntax-only rules (141 spec files),
  all with inline snippets. Per set: comments 204, complexity 114, empty-blocks 61, exceptions 70, naming 164,
  performance 45, potential-bugs 78, style 866, libraries 21, ruleauthors 8, ktlint wrapper 90.
- Every test funnels through `Rule.visitFile(root, languageVersionSettings): List<Finding>`
  (`detekt-api/…/Rule.kt:41-48`), reached from `Rule.lint(content)` in
  `detekt-test/src/main/kotlin/dev/detekt/test/RuleExtensions.kt`. Rules are built with
  `TestConfig("key" to value)` or `Config.empty`.
- So the ktlint recipe carries over: patch `RuleExtensions.kt` (and `TestConfig` to expose its pairs) with `sed`,
  call a `CaseRecorder`, run the JUnit console launcher. The recorder writes what the real rule returned, not what
  the test asserted; that matters because 2,062 assertions are `isEmpty()` and 1,268 are `hasSize(n)`, while only
  390 check a location.
- Per case: `<case>.input.kt`, `.options` (rule, file name, typed config pairs, language version), `.findings`
  (`line:col\tendLine:endCol\tstart:end\t<signature>\t<message>`).
- Differences from ktlint's goldens: `TestConfig` values are raw objects with no YAML coercion (the `.options`
  format must keep types); the test-only suppression filter is a simplified one (record before it); `lint` needs
  the Kotlin compiler on the classpath but no project classpath.
- Engine behaviour (config layering, suppression, baseline, reports, CLI) has its own upstream tests: detekt-core
  39 specs / 301 tests, detekt-cli 9 / 95. Port those as Rust tests, as was done for ktfmt's and ktlint's CLI tests.

## 3. What ktrs has, and the gaps

| Need | Have | Gap |
|---|---|---|
| Visitor rules on a read-only tree | `ktrs-psi`: 149 typed classes, 541 accessors, `KtVisitorVoid` with 117 `visit_*` methods and the upstream super chain, `KtTreeVisitorVoid`, JVM oracle on fixtures and corpus (`tools/psi-accessors`). No arena seeding (11.3% of parse saved). | Scope is "what ktfmt calls". Missing accessors below. Pinned to 2.4.20, detekt runs 2.4.10. `PsiElement` holds an `Rc<Tree>`: one file per thread, fine for the per-file worker model. |
| The same accessors with ktlint's semantics | `ktrs_ast::psi`: 69 classes, 373 accessors on the mutable arena, including `is_public`, `is_local`, `name`, `declarations`, `annotation_entries`, `has_modifier`. No visitor. | Use as the template for the ktrs-psi additions, not as the base. |
| Line/column | `PositionInTextLocator` (UTF-16, 1-based) in ktrs-lint. | UTF-8 offset → UTF-16 column on the immutable tree; move the table somewhere both engines can use. |
| Suppression | ktlint's `SuppressionLocator`. | Different id grammar and scope: new, small (`Suppressions.kt` is 50 lines). |
| Config | `ktrs-editorconfig`. | YAML: nothing in the workspace (no YAML crate, no reader). New. |
| Reports | Hand-written, byte-exact checkstyle, SARIF, HTML, JSON, baseline XML writers and a baseline XML reader for ktlint (`crates/ktrs-cli/src/ktlint/reporter/`, `baseline/`). | detekt's formats differ in every one; the writing technique and the XML reader carry over. Markdown is new. |
| CLI | Clikt-compatible parser, Java glob, file walking, parallel runner, console capture for tests, `java_launcher.rs`. | JCommander semantics and usage text. |
| Full mode, unknown plugins | `ktlint_jar.rs`: SHA-pinned download to the cache dir, `java` lookup, argv pass-through, `exec`. | Same pattern with the 85 MB detekt jar. |
| ktlint rule set | ktlint 1.8.0 rules natively, lint and format (research/26); `standard_rule_provider(id)` + `EditorConfigOverride` already run one rule with given properties (`crates/ktrs-lint/tests/golden/engine.rs:50-77`). | detekt bypasses ktlint's engine: one traversal per rule on a copy, no ktlint suppression, no rule ordering, a point location, detekt names and config keys. Needs a bare single-rule traversal entry and a 100-row mapping table. The 1.8 rules would run with Kotlin 2.4.10 PSI semantics, a combination not tested so far. |
| Goldens, corpus diff, known diffs, CI | `tests/golden/runner.rs` (already shared with compose), `xtask lint-diff`, `tools/parity/known-diffs/`, `parity.yml`. | A detekt probe and an `xtask detekt-diff`. |
| Build tools, IDE | ktlint-gradle drop-in pattern (same FQNs, one process per task), `ktrs serve`, `ktrs-project`, `ktrs-lsp`. | Nothing detekt-specific. `ktrs-project` does not detect the detekt plugin. |

### PSI gap, estimated

Method: every `.member` in the 137 syntax-only rule files plus `detekt-psi-utils`, `detekt-metrics` and
`detekt-api` that is also a member name in the compiler's `psi-api`/`psi-impl` sources, matched by snake-cased name
against the functions of ktrs-psi. Name-level, so approximate (a few false matches each way).

- About 280 distinct PSI member names are used; about 157 exist in ktrs-psi; of the remainder about 95 are real
  PSI/psiUtil members, and about 40 of those exist on the `ktrs_ast::psi` side as a reference implementation.
- All 51 `visit*` overrides the rules use exist in `KtVisitorVoid`.
- 42 of the 137 rules call nothing that is missing; 31 miss one name, 22 two, 42 three or more.
- Missing, by number of rules using it (top of the list): `nameAsSafeName` 21, `isPrivate` 12, `isTopLevel` 10,
  `isInterface` 10, `statements` 9, `declarations` 9, `getStrictParentOfType` as a named helper 8, `docComment` 8,
  `isPublic` 6, `isLocal` 6, `containingClassOrObject` 6, `superTypeListEntries` 5, `packageDirective` 5,
  `getNonStrictParentOfType` 5, `containingClass` 5, `getCallNameExpression` 5, `anyDescendantOfType` 5, `isData` 4,
  `isExtensionDeclaration` 4, `findDescendantOfType` 4, `properties` 4, `KtPsiUtil.safeDeparenthesize` 3.
- Long tail, one or two rules each: modifier predicates (`isInner`, `isSealed`, `isAbstract`, `isExternal`,
  `hasExpectModifier`…), `KtClass.companionObjects`, `secondaryConstructors`, `findFunctionByName`, `getSuperNames`,
  `KtParameter.ownerFunction`/`isVarArg`/`isPropertyParameter`, `KtStringTemplateExpression.entries`/
  `hasInterpolation`/`plainContent`, `KtPsiUtil.areParenthesesUseless`/`deparenthesize`, constant node types, and
  the KDoc API (`KDoc.getDefaultSection`, `getAllSections`, `findTagsByName`, `KDocTag.knownTag`/`getSubjectName`/
  `getContent`), which the `comments` set needs and ktrs-psi only traverses today.

Sample read in full (18 rules and 2 helpers): UndocumentedPublicClass, EmptyFunctionBlock, EmptyCatchBlock,
CyclomaticComplexMethod (+ `CyclomaticComplexity`), TooManyFunctions, NestedBlockDepth, LongMethod (+ `linesOfCode`),
SwallowedException, TooGenericExceptionCaught, MatchingDeclarationName, VariableNaming, ForEachOnRange,
EqualsAlwaysReturnsTrueOrFalse, MagicNumber, UnusedPrivateClass, plus the import/visitor census of all 137. Findings:

- Rules are short (median about 30 lines) and are plain visitors: typed accessors, ancestor/descendant queries,
  text comparisons. Nothing in the sample needs more than the tree.
- Most missing members are one-liners over the modifier list or the child list. The ones with real logic:
  `isLocal`/`KtPsiUtil.isLocal`, `fqName` (UnusedPrivateClass), `isPublic` with inherited visibility,
  `areParenthesesUseless`, the KDoc section/tag API, `linesOfCode` (token walk + line table).
- JVM-isms to reproduce: `HashMap` iteration order (LongMethod: unordered upstream, compare as a set),
  `String.toDouble()` and number rendering in messages (MagicNumber), `Locale` lowercasing, Java regex, object
  identity (`===`, `IdentityHashMap`) which maps to element ids.
- `nested visitors`: several rules run a second visitor over a subtree (CyclomaticComplexity, FunctionDepthVisitor,
  UnusedClassVisitor); `accept` on a subtree exists in ktrs-psi.

## 4. Parity plan

Mirrors the ktlint gates (research/19). Each gate gets a ratchet or a known-diffs file and a job in `parity.yml`.

| Gate | What | How |
|---|---|---|
| PSI accessors | Every accessor added to ktrs-psi | Extend `tools/psi-accessors` (Java report + Rust mirror), regenerate fixture hashes, corpus `compare`. Existing gate, no new harness. |
| Goldens | detekt's own rule tests, syntax-only rules | `tools/detekt-tests/extract-goldens.sh` (JVM, testbox, background): patch `RuleExtensions.kt`/`TestConfig.kt`, `CaseRecorder`, JUnit console launcher, into `testdata/detekt/<RuleName>/`. `cargo test -p ktrs-detekt --test golden` with `tests/golden-passing.txt`, reusing `tests/golden/runner.rs`. Rules not ported are skipped and counted. |
| Corpus diff | Rows vs the jar on `tools/corpus/REVISIONS` | `tools/detekt-oracle/detekt-probe.sh` (JVM, testbox, background, about 1 to 3 min per run): a `DetektProbe` on `detekt-tooling`'s API writing `rows.tsv` (file, line, col, end, offsets, rule, severity, signature, message) and `failed.tsv`. `cargo detekt-diff [default|all-rules]` (xtask) compares per file as a multiset; `--counts` prints per-rule totals. Runs: default config, `--all-rules`, and each with the plugin jars once the ktlint set is ported. |
| Reports and CLI | stdout, stderr, exit code, report files | `tools/detekt-oracle/cli-diff.sh`, the scenario runner of the ktlint one: config layering, validation errors (exit 3), baseline create/filter, each report id, `--generate-config`, `--version`, suppression forms, `--includes/--excludes`, argument errors. Masks: stack frames, html/markdown timestamp. Fixtures avoid LongMethod order. |
| Engine unit tests | detekt-core's and detekt-cli's specs | Ported to `cargo test -p ktrs-detekt` / `-p ktrs-cli`. |
| Holdout | 20 unseen repos | `tools/parity/detekt-compare.sh` beside the ktlint/ktfmt ones in `tools/holdout/run.sh`. detekt/detekt itself is already in the holdout. |
| Known diffs | Accepted mismatches | `tools/parity/known-diffs/detekt.tsv` in the `check-known.sh` format (stale entries fail). |

Expected sources of diffs, to check in the spike rather than assume:

- Parser pin: jar 2.4.10 vs ktrs 2.4.20. For ktlint this showed up only as KDoc lexer differences. Here the
  `comments` rules read KDoc structure, so the exposure is larger. Options are the `EmbeddedKotlin`-style switch or
  known-diff rows.
- Files with syntax errors: detekt analyzes them; ktrs's tree must match the compiler's error recovery there
  (corpus-diff already covers recovery for the parser gate).
- Columns for non-BMP characters and tabs; BOM; CRLF input.
- Order: only as a multiset per file unless the spike shows the order is otherwise stable.

## 5. Product shape

| Option | For | Against |
|---|---|---|
| `detekt` drop-in binary, flag-exact detekt-cli 2.0 | Same shape as `ktfmt` and `ktlint` (memory: cli-shape-decision); testable against the jar by `cli-diff.sh`; pre-commit, CI scripts and a later Gradle plugin all call it. | JCommander usage text and error paths to reproduce. |
| `ktrs detekt` subcommand only | Less surface. | Not a drop-in; nothing to diff against except rows. |
| Both: `ktrs detekt` is the drop-in itself | What `ktrs ktlint` does today; one implementation. | None beyond the first row. |

Recommendation: **both, one implementation** (`src/bin/detekt.rs` + `ktrs_cli::detekt`, `ktrs detekt` = same entry).

Analysis rules: **hand the run to the jar**, never skip with a notice.

- In light mode there is nothing to skip: detekt itself does not run those rules, and says so only under `--debug`
  (print the same debug lines).
- Hand off, decided right after argument parsing: `--analysis-mode full`; a `--plugins` jar that is not a
  fingerprinted native one (the three first-party jars of the pinned release, once ported); `--config-resource`
  naming anything but the built-in default; `--auto-correct` until the ktlint set is native.
- Same mechanics as `ktlint_jar.rs`: `detekt-cli-<version>-all.jar` from the GitHub release, SHA-256 pinned,
  `KTRS_DETEKT_JAR` override, one "downloading detekt … (once)" line.
- A user asking for full mode has a classpath, hence a JDK, so the hand-off costs them nothing they don't pay today.
- compose-rules' detekt flavour (`io.nlopez.compose.rules:detekt`, 834 code-search hits) uses the Analysis API:
  hand-off, not a port.

Gradle plugin drop-in (`dev.detekt`, 1.x `io.gitlab.arturbosch.detekt`): **later**, after the CLI passes cli-diff.

- Upstream's tasks already build a detekt-cli argument list and call `Main.buildRunner(args…)`
  (`detekt-gradle-plugin/src/main/kotlin/dev/detekt/gradle/invoke/DetektInvoker.kt`), so a drop-in can pass the same
  list to the `detekt` binary; plugin main sources are 2,195 lines.
- `detektMain`/`detektTest` (per-compilation tasks) run in full mode: those would hand off, so the win is the
  plain `detekt` task. `detektPlugins` jars reach the binary as `--plugins`.
- The ktlint-gradle drop-in (research/29) is the template: new plugin id, upstream FQNs, TestKit ports, a parity
  script against the real plugin.
- Until then a three-line `Exec` task or pre-commit hook covers light mode.

## 6. Phased plan

Sizes are Rust lines, estimated at about 1.3× the Kotlin they port (ktrs-compose: 5.3k Rust for about 4k Kotlin).

| Phase | Content | Size | Exit criterion |
|---|---|--:|---|
| 0. Spike | `crates/ktrs-detekt`: `Config` (in-memory + composite + all-rules), rule trait over `KtVisitorVoid`, descriptors, analyzer loop, suppression, `Entity`/`Location`/`Signatures`. ktrs-psi additions for the 27 rules. `DetektProbe` + `xtask detekt-diff`. Golden extractor + runner. Rules: the 15 `empty-blocks` rules and the 12 top-firing ones listed in section 1. | 4–5k Rust, 0.6k JVM/shell | (a) rows identical to the jar on the corpus for the 27 rules, default config, as per-file multisets; (b) their goldens pass (a few hundred cases; not counted yet); (c) rule pass over the corpus costs less than the parse. |
| 1. CLI | YAML reader + validation, `detekt` binary and `ktrs detekt`, console reports, checkstyle, SARIF, markdown, html, metrics processors, baseline, `--generate-config`, jar hand-off, `cli-diff.sh`, ported core/cli specs. | 4–5k | cli-diff scenarios identical; exit codes 0/1/2/3. |
| 2a. `style`, default-on | 23 remaining default-on syntax-only rules. | 1.5k | corpus default run for the set: 0 diffs. |
| 2b. `naming` | 17 rules (11 on; 2,324 default findings). | 0.7k | same |
| 2c. `complexity` + `detekt-metrics` | 11 rules (6 on; 1,023). | 1.0k | same |
| 2d. `exceptions` | 11 rules (9 on; 835). | 0.5k | same |
| 2e. `potential-bugs`, `performance`, `coroutines` | 11 + 5 + 1 rules (8 + 2 + 0 on; 11 findings). | 0.7k | goldens (they barely fire) |
| 2f. Default-off rules | The other 28 `style` rules, then `comments` (10, needs the KDoc PSI API; 12,203 findings under `--all-rules`). | 2.5k | `--all-rules` corpus run: 0 diffs outside known-diffs |
| 3. Plugin sets | `ktlint` wrapper (100 rules over the 1.8 port, `--auto-correct`), `libraries` 2, `ruleauthors` 1; jar fingerprints. | 2–3k | plugin corpus runs (176,650 ktlint rows): 0 diffs; `-ac` trees identical |
| 4. Integrations | Gradle plugin drop-in, `ktrs-project` detection, `ktrs migrate` swap, LSP diagnostics, README. | as research/29 | TestKit ports + parity script |
| Later, on demand | 1.23.8 mode (146 syntax-only core rules, own CLI and config dialect, no `formatting` set). | 3–4k | own oracle runs |

Order within phase 2 is by default-on syntax-only count weighted by corpus findings: style (25 on, 20,928),
empty-blocks (15, 299; in the spike because it is 192 lines), naming (11, 2,324), exceptions (9, 835),
potential-bugs (8, 11), complexity (6, 1,023), performance (2, 0). After 2d the default light run is complete
except 10 rules that produce 11 findings on the corpus.

Total for core light mode (phases 0 to 2): about 15k lines of Rust. For scale, ktrs-lint is 23k and ktrs-compose 5k.

Expected speed, not measured: the jar needs 64 s (10 cores) to 176 s (one core) for the corpus in light mode, most
of it session setup; ktrs parses the corpus in about 1 s single-threaded (research/15). The spike's criterion (c)
turns this into a number.

## 7. Decisions for the owner

| # | Decision | Recommendation | Trade-off |
|---|---|---|---|
| 1 | Do it at all, and with what scope? | Yes, light mode only: the 134 syntax-only core rules, analysis rules never ported. | Covers detekt's default CLI mode and the Gradle `detekt` task exactly; users of `detektMain` (full mode) gain nothing but a pass-through, and upstream keeps moving rules to the Analysis API (12 so far). |
| 2 | Which version is the spec? | `2.0.0-alpha.6`, engine built with a version switch; 1.23.8 mode later if users ask. | Matches the Kotlin and ktlint versions ktrs already ports and the branch that will become stable; but about 85% of today's users are on 1.23.8 and can't switch a drop-in in until they migrate. |
| 3 | First deliverable | The phase-0 spike with criteria (a) to (c), no CLI. | Proves row parity and the PSI gap cheaply before the CLI work; nothing user-visible for one phase. |
| 4 | CLI shape | `detekt` drop-in binary and `ktrs detekt`, one implementation, flag-exact. | Consistent with ktfmt/ktlint and diffable against the jar; costs a JCommander port. |
| 5 | `--analysis-mode full`, unknown `--plugins` jars | Hand the whole run to the SHA-pinned detekt jar. | Output stays detekt's; needs a JDK and an 85 MB download on first use. A "skipped" notice would be simpler but is not what detekt prints. |
| 6 | PSI base for the rules | `ktrs-psi` (immutable tree, existing visitor and JVM oracle), extended by about 95 members. | No arena seeding and a corpus-scale oracle; duplicates helper bodies that already exist in `ktrs_ast::psi`. |
| 7 | Kotlin 2.4.10 (jar) vs 2.4.20 (ktrs parser) | Keep the 2.4.20 parser; record KDoc-lexer rows as known diffs; add a switch only where the spike measures a need. | No second parser mode; the `comments` set may carry a few accepted diffs until detekt moves to 2.4.20. |
| 8 | YAML reader | An event-level pure-Rust YAML parser crate plus our own port of snakeyaml-engine's core-schema resolution and duplicate-key check. | First YAML dependency in a workspace that hand-rolls its formats, against roughly 3k lines to port a YAML scanner/parser; malformed-YAML output is a JVM stack trace upstream, so byte parity is not at stake there. |
| 9 | `ktlint` rule set (`detekt-rules-ktlint-wrapper`) | Native in phase 3, hand-off before that. | `detekt-formatting` appears in 1,716 version catalogs (about 30% of detekt's), and ktrs already has the 1.8 rules; but it is 100 mapping rows plus a traversal mode ktlint itself never uses. |
| 10 | Output order | Compare rows as per-file multisets; byte-compare report files only on CLI fixtures. | Upstream order is not deterministic for at least LongMethod, so stricter is unachievable; an ordering bug in ktrs could hide until a fixture catches it. |
| 11 | Upstream quirks (`-r md:` writes nothing, swallowed report errors, CLI vs config glob path forms, aliases lost with a replacing config) | Reproduce them; parity is the spec. | Predictable for people switching; ships known upstream bugs until upstream fixes them and the pin moves. |
| 12 | Gradle plugin drop-in | After the CLI (phase 4). | Most detekt runs are Gradle, so adoption waits; but the plugin is a thin argument builder over the CLI and is cheap once the CLI is exact. |
| 13 | Pin policy while 2.0 is alpha | Follow each alpha through `upstream.yml` until 2.0.0 final, regenerate goldens per bump. | Seven alphas in 11 months with renames between them means recurring port work; freezing on alpha.6 would leave the drop-in matching a build nobody keeps. |

## Reproduce

- Inventory: clone `detekt/detekt` at `v2.0.0-alpha.6`; providers `detekt-rules-*/src/main/kotlin/**/*Provider.kt`,
  defaults `detekt-core/src/main/resources/default-detekt-config.yml`, marker `grep -rl RequiresAnalysisApi`.
- Corpus runs (testbox, under the lock, background): `~/work/detekt-probe/run.sh` and `run2.sh`:
  `java -Xmx12g -jar detekt-cli-2.0.0-alpha.6-all.jar --input corpus --base-path corpus --report checkstyle:<out>
  [--parallel] [--all-rules] [--plugins <3 jars>] [--excludes '**/sourceFiles/**'] [--fail-on-severity never]`;
  per-rule counts = `grep -o 'source="[^"]*"' <out> | sort | uniq -c`.
- Adoption counts: `gh api search/code` with `"io.gitlab.arturbosch.detekt" filename:libs.versions.toml` (5,696),
  `"dev.detekt" filename:libs.versions.toml` (1,030), `"detekt-formatting" filename:libs.versions.toml` (1,716),
  `"detekt-rules-ktlint-wrapper"` (363), `filename:detekt.yml` (6,400), `filename:detekt-baseline.xml` (2,772),
  `"io.nlopez.compose.rules:detekt"` (834), `"buildUponDefaultConfig"` (9,104). Lower bounds, public repos only.

## Spike result (2026-10-09)

Phase 0 as defined in section 6, on branch `worktree-agent-acb240cca722776d9`. Verdict: **go**.

| Criterion | Result |
|---|---|
| (a) rows identical to the jar on the corpus, per file as multisets | **Go.** 6,123 of 6,123 files identical, default config: 24,015 rows of the 27 rules on both sides, every column (line, UTF-16 column, end, UTF-16 offsets, rule, severity, signature, message). Same against the `--all-rules` oracle. No known diffs, no panics. |
| (b) their goldens pass | **Go.** 374 of 374 cases recorded from 19 upstream spec files (224 upstream tests), exact row order (LongMethod: as a set). |
| (c) rule pass cheaper than the parse | **Go.** 27 rules: 0.53 s against 0.77 s of parsing (6,123 files, 30.8 MB, one thread, best of 3; three runs: 0.69 parses each). The jar needs 22.9 s (10 cores, probe API, parallel) for the 76 default rules. |

The "27 rules" of section 6 count EmptyFunctionBlock twice (it is in `empty-blocks` and in the top twelve). Ported: those
26 plus FunctionParameterNaming, the next rule by findings (192).

### What was built

- `crates/ktrs-detekt` (4.8k lines; conventions in `src/lib.rs`): `api` (Config, config properties, Rule, Finding, Entity,
  Location, Issue, signatures), `engine` (rule descriptors, Analyzer, `@Suppress`, path filters), `config` (YAML subset,
  YamlConfig, CompositeConfig, AllRulesConfig, DisabledAutoCorrectConfig, bundled default config), `psi` and `metrics`
  (the detekt-psi-utils and detekt-metrics parts the rules call), `rules` (5 rule sets, 27 rules, one file per upstream file).
- `crates/ktrs-psi`: about 45 accessors and psiUtil helpers (list in its `src/lib.rs`), each with a line in the JVM oracle
  (`tools/psi-accessors/src/Members.java`); fixture hashes regenerated, 680 of 680 in both modes (Kotlin 2.4.20).
- `tools/sync-detekt.sh` (pin, sources, jar with SHA check), `tools/detekt-oracle` (`DetektProbe.kt` on detekt-tooling's
  API, `detekt-probe.sh`, smoke files), `tools/detekt-tests/extract-goldens.sh` (recording `Rule.lint`, `specs.txt`),
  `testdata/detekt/<RuleName>/`, `cargo detekt-diff`, `examples/{detekt_probe,bench}`.
- `java_glob.rs` moved from ktrs-cli to ktrs-editorconfig (both engines match paths with it).

### Findings

1. **Columns and offsets are UTF-16 units** (the unverified point of section 2). Smoke file `Utf16.kt`: the same block is
   at column 41 after `é` and 42 after an emoji; a line of 116 code points and 121 UTF-16 units is over the 120 limit.
   Also observed: a BOM is not part of the text, CRLF and lone CR are line breaks, an empty file reports at 1:1.
   `crates/ktrs-detekt/tests/smoke.rs` pins these against the jar's rows (`tests/data/smoke.jvm.tsv`).
2. **A full walk per rule would be too slow; a sparse one is exact and cheap.** Upstream walks the tree once per rule. A
   rule only acts on the kinds its `visit*` overrides receive, and the flat tree lets a visitor jump between those
   (`src/visitor.rs`): same order, same pruning, same state. `detekt_visitor!` derives the kinds from the overridden
   methods. The full walk was not built or measured; the estimate before writing it was several parses for 27 rules.
3. **Cost per rule has a floor.** Alone, the cheapest rules take 0.03 s each (4% of the parse): a rule instance, its
   config reads and its node list per file. Together the 27 take 0.53 s, not the 1.3 s their single runs add up to,
   because the index is shared. At this rate 134 rules would cost about 2.6 parses; the per-file fixed cost
   (instantiation, suppression check, list build) is the thing to cut during phase 2. Slowest alone: WildcardImport
   0.16 s (builds an `ImportPath` per import), LongMethod 0.14 s, CyclomaticComplexMethod 0.10 s.
4. **Quirks reproduced** (all visible in goldens or corpus rows): `FqName.startsWith(root)` is false; MagicNumber strips
   literal suffixes one after the other, so `0x0d` reads as `0x0`; `linesOfCode` skips comments by exact class, so a
   KDoc's `/**` and `*/` lines count; MaxLineLength's "raw string" exemption applies to any string template that is
   the last element of the line by `textOffset`; `-0` is a magic number (`Double.equals`); enum entries take their
   super types from the initializer list (the only corpus diff of the first run: two signatures).
5. **Rust is about 2.2 times the Kotlin**, not 1.3: 2,100 lines for 950 lines of rule code (config accessors and
   constructors are explicit).
6. **The 2.4.10 / 2.4.20 parser difference did not show** in these rules; none reads KDoc structure. The `comments` set
   will be the test.
7. **Operational:** a Bazel server started under `flock ~/bench.lock` inherits the lock's descriptor and held the
   testbox lock for about 100 minutes after its job ended. Jobs that start daemons need `flock -o`.

### Stubbed or unverified

- YAML: a line-based subset (block maps and sequences, one-line flow sequences, plain and quoted scalars, comments,
  core-schema scalars). No anchors, tags, block scalars, flow maps, multi-line scalars or several documents: those are
  errors. Decision 8's parser and resolver are phase 1.
- `ignoreAnnotated` and `ignoreFunction` (AnnotationSuppressor, FunctionSuppressor): not ported; a config that sets
  either panics. No ported rule's default uses them.
- Regexes go through `KotlinRegex` (the `regex` crate): no lookaround or backreferences. MaxLineLength's two fixed
  patterns are rewritten without them. A user pattern that needs them panics.
- `URL(...).toURI()` (MaxLineLength) is approximated (`kotlin.rs`); covered by a dozen smoke lines and the corpus's
  5,368 rows.
- AllRulesConfig takes an empty deprecated-rules list (the jar's `deprecation.properties` is generated at build time).
- `languageVersionSettings`, the "requires type resolution" debug lines, config validation, baseline, reports, CLI:
  not started.
- Row order was compared as multisets only. The golden runner checks order per rule; order across rule sets follows the
  jar's service file and is assumed, not checked.
- `KtScript.getName()` (from the file name) is not in ktrs-psi.
- Of the workspace gates only these ran on this branch (testbox): `cargo check -p ktrs-psi -p ktrs-detekt -p xtask
  -p ktrs-cli --all-targets`, `cargo test -p ktrs-psi`, `cargo test -p ktrs-detekt`, `cargo test -p ktrs-fmt --test golden`.

### What the full port needs that the spike showed

- A backtracking Java-regex engine or a dependency for one (`fullyQualifiedNameGlobToRegex` uses lookahead; user
  patterns can use anything).
- A shared home for the Java-isms now spread over crates: `KotlinRegex` (ktrs-lint), `java_glob` (ktrs-editorconfig),
  the golden runner's `run_all`/`ratchet` (rewritten in `tests/golden/main.rs`).
- Rule names of the 93 analysis rules (for `RuleInstance` lists in SARIF and the debug lines), without their code.
- `file.rs` in ktrs-psi is over 300 lines: split it.
- parity.yml: an oracle job (`sync-detekt.sh`, `detekt-probe.sh corpus`, cached by the pin and the corpus REVISIONS), then
  `cargo detekt-diff default` and `all-rules`; the goldens already run in `cargo test -p ktrs-detekt` and need no JVM.
  About 30 lines next to the ktlint jobs. Not added yet.

### Revised sizes

| Phase | Section 6 | Revised | Why |
|---|--:|--:|---|
| 0. Spike | 4–5k Rust, 0.6k JVM/shell | 5.6k Rust + 0.7k in ktrs-psi, 0.6k JVM/shell (done) | |
| 1. CLI | 4–5k | 5–6k | plus the suppressors, the YAML resolver and the regex engine |
| 2a–2f. Remaining 107 core rules | 6.9k | 10–11k | 2.2x of 4.4k Kotlin, plus about 1k of psi-utils and metrics |
| Core light mode, phases 0–2 | 15k | 21–23k | |

Phases 3 and 4 are unchanged.

### Reproduce the spike

```sh
tools/sync-detekt.sh
tools/detekt-oracle/detekt-probe.sh corpus target/detekt-oracle/default              # JVM, background
tools/detekt-oracle/detekt-probe.sh corpus target/detekt-oracle/all-rules --all-rules
cargo detekt-diff default && cargo detekt-diff all-rules
tools/detekt-tests/extract-goldens.sh                                                # JVM, background
cargo test -p ktrs-detekt --release
cargo run -p ktrs-detekt --release --example bench corpus 3 --per-rule
```
