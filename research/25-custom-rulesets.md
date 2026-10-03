# 25 — Custom ktlint rule sets in the wild

Question: which custom rule sets do real projects load into ktlint, and how concentrated is it? Decides: hand-port,
publish a Rust rule API, or hand `-R` runs to the jar. Data gathered 2026-10-02.

## Method and confidence

- GitHub legacy code search (`gh api search/code`, text-match fragments), 25 queries, all pages (≤1000/query),
  deduplicated by repo. Scripts were throwaway (scratchpad); rerun by repeating the queries below.
- Coordinates/versions pulled from match fragments by regex; a fragment often misses the next line, so per-set
  attribution is a lower bound (e.g. 305 Spotless `customRuleSets(` repos, set identifiable in 82).
- Cross-checks: Maven Central search (`search.maven.org`, "ktlint"), deps.dev dependents, GitHub repo/release API,
  shallow clone of mrmans0n/compose-rules @ v0.6.7.
- **Confidence: medium on ratios, low on absolutes.** Legacy code search indexes a subset of public default
  branches (it missed mrmans0n/compose-rules' own provider file), and private/enterprise repos — where most
  in-house rule sets live — are invisible. Treat counts as "≥ N public repos", roughly 2–5x undercounted.

## Repo counts by rule set (public, sampled)

| Rule set | Repos | Paired ktlint | Notes |
|---|---:|---|---|
| `io.nlopez.compose.rules:ktlint` (compose-rules) | **~360** | 1.1–1.8 | active; versions seen: 0.4.16 (ktlint 1.3) 22, 0.4.22 (1.4) 18, 0.6.7 (1.8) 16, 0.4.27 (1.7) 12, 0.5.x/0.6.x (1.8) ~55 |
| `com.twitter.compose.rules:ktlint` (predecessor) | ~45 | 0.48.2 | frozen at 0.0.26 (2023); users stuck on ktlint 0.4x |
| **compose-rules, either coordinate** | **~410** | | ≈ 85–90% of all identifiable third-party rule-set use |
| In-house module (`project(":…rules")` + own `RuleSetProviderV3/V2/V1`) | ~70 | 1.x (V3) 67, 0.4x (V1/V2) ~30 | excl. ~12 tooling repos (ktlint-gradle, kotlinter, maven plugins, IntelliJ plugin) |
| `com.hendraanggrian.rulebook:rulebook-ktlint` | 13 | 1.x | one author's own repos; 53 rules |
| diktat (`-R diktat.jar`, old) | ~5 | 0.4x | diktat now ships its own CLI on ktlint's engine (570★) |
| `com.github.RubixDev:ktlint-ruleset-mc-preprocessor` | 5 | 1.x | Minecraft-modding niche |
| `ktlint-ruleset-experimental` (ktlint's own, pre-0.48 artifact) | 68 (28 w/ version) | 0.34–0.47 | obsolete: merged into standard (`ktlint_experimental`), which ktrs ports |
| `ktlint-ruleset-test` | 17 | — | only lockfiles/verification metadata (transitive), not a real use |
| Long tail, 1 repo each | ~15 | mixed | room-/strikt-ktlint-plugin (F43nd1r), ktlint-extras, today.wstd, kantis, sembozdemir compose, expo, eo, ridi, … |

Other signals: `io.nlopez.compose.rules:detekt` appears in ~250 version catalogs (≈ the ktlint variant's ~250 in
catalogs) — Compose teams split evenly between detekt and ktlint for these checks. Maven Central has only ~8
ktlint rule-set artifacts besides compose-rules, most dead (0–2★, last push 2018–2020).

## How they are wired

| Mechanism | Repos with custom set | Note |
|---|---:|---|
| jlleitschuh ktlint-gradle `ktlintRuleset(...)` | 194 (125 compose, ~15 in-house `project()`, 13 rulebook, 9 README copies) | |
| Spotless `.ktlint(v).customRuleSets(...)` | 305 (≥74 compose; most of the rest undetermined) | ktlint 1.8.0 most common pin |
| kotlinter (`ktlint(...)`/buildscript) | ≤42 (kotlinter + compose-rules in same file) | upper bound |
| CLI `-R` / `--ruleset` | ~40 (≥8 real compose jar uses, rest docs/templates) | `ktlint-compose-0.6.x-all.jar` |
| Maven (antrun `com.pinterest.ktlint.Main` 138 repos; gantsign plugin 249) | ~6 pom.xml with compose-rules | rare |
| pre-commit | 1 | negligible |

Version pairing overall: **1.x dominates** (compose-rules has targeted ktlint 1.8.0 since v0.5.0, 2025-12); 0.x
survives via twitter compose-rules and old experimental-ruleset builds; **2.0: zero** real users (one fragment
mentions ALPHA-4). ktlint 2.0 still loads 1.3–1.8 jars via `ktlint-com-pinterest-backward-compatibility`
(research/22), so 1.x-API jars are what a JVM fallback must accept.

## Top set: mrmans0n/compose-rules (ktlint module)

| Metric | Value |
|---|---|
| Rules | 34 (`ComposeRuleSetProvider`), 4 autocorrect (ModifierWithoutDefault, PreviewNaming, PreviewPublic, ViewModelInjection) |
| Kotlin LOC (non-blank, non-comment) | ~3.1k shared `rules/common` (rules + core utils, also used by detekt) + ~0.85k ktlint glue |
| Release cadence | 20 releases Dec 2025–Sep 2026 (~2/month); 735★; builds against ktlint 1.8.0 / Kotlin 2.2.21 embeddable |
| Config | 12 rules read `.editorconfig` properties (allowlists, naming patterns) |
| Semantic analysis | none — purely syntactic (no BindingContext/resolve) |
| PSI style | **PSI-heavy**: 0 `ASTNode` uses in rule code; dispatch on `node.psi` (KtFile/KtClassOrObject/KtFunction/…); 45 distinct `Kt*` classes, 10 `psiUtil` helpers (`parents`, `siblings`, `isPublic`, `visibilityModifierType`, `startOffset`…); autocorrect builds nodes with `KtPsiFactory` |
| Fit with ktrs-psi | 40 of 42 imported `Kt*` classes already exist in `crates/ktrs-psi` by name (missing: `KtAnnotated`, `KtPsiFactory`); accessor coverage not audited — ktrs-psi is scoped to what ktfmt calls |

Twitter compose-rules (#2) is the same codebase's 2023 ancestor (ktlint 0.48 API); porting #1 covers its users'
rules once they migrate. No #3 has meaningful reach (rulebook = one author).

## In-house rule sets (sample of 10 public modules, 1.x API)

| Repo | Rules | LOC | Kt* refs | ASTNode-ish refs |
|---|---:|---:|---:|---:|
| airbnb/viaduct | 5 | 261 | 9 | 24 |
| aws/aws-kotlin-repo-tools (style) | 4 | 87 | 10 | 22 |
| espoon-voltti/evaka | 4 | 92 | 16 | 6 |
| tolgee/tolgee-platform | 2 | 118 | 4 | 9 |
| pubnub/kotlin | 2 | 115 | 2 | 19 |
| navikt/pensjonsbrev | 1 | 134 | 0 | 18 |
| expo/expo | 1 | 93 | 2 | 14 |
| partiql (V1 API) / gematik (V1) | 2 / 1 | 78 / 69 | 0 / 8 | 20 / 12 |
| HedvigInsurance/android | 1 | 48 | 0 | 3 |

Pattern: 1–5 rules, <300 LOC, mostly `ASTNode`/`elementType` walking with a few `Kt*` casts; typical bodies ban an
import/annotation/call or enforce a naming convention. Each is unique — no porting leverage — but each is small
enough that a Rust author could rewrite it against `ktrs-ast` in an hour if an API existed.

## Recommendation

1. **Hand-port compose-rules' 34 ktlint rules as a built-in, opt-in rule set** (`compose:*` ids, same
   `.editorconfig` keys, enabled when the build config names `io.nlopez.compose.rules:ktlint` or via
   `ktlint_compose = enabled`). It is ~85–90% of identifiable third-party use, purely syntactic, and ktrs-psi
   already names 40/42 of its PSI classes. Cost is in accessor coverage + `psiUtil` helpers + a `KtPsiFactory`
   subset for 4 autocorrects (~4k LOC source; pin to a compose-rules version and track its ~monthly releases via the
   upstream watch). This is the C4 item in research/08.
2. **Everything else (`-R` with an unknown jar, in-house modules, Spotless `customRuleSets` without compose):
   delegate the whole run to the real ktlint jar** (the drop-ins already know the flags; the Gradle/Spotless
   integrations can fall back to the JVM step). The tail is ~70+ public repos of tiny, unique sets plus an
   invisible private majority — not portable by us.
3. **Defer a public Rust rule API.** No demand signal: in-house sets are 1–5 rules, owners won't rewrite them in
   Rust for a speedup they only feel on lint. Revisit only if users ask after (1)+(2) ship; when it comes, expose the
   `ktrs-ast` node API (ASTNode-style), not typed PSI — that is what in-house rules actually use.
4. Skip twitter compose-rules (0.48 API, frozen), `ktlint-ruleset-experimental` (already standard), diktat (own CLI).
