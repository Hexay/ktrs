# 10 — Is a native ktlint-compatible linter worth building?

Researched 2026-09-30 via GitHub REST/GraphQL (`gh api`), WebSearch/WebFetch, and a local + testbox benchmark of the
ktlint release binaries. Background not repeated here: prior art (02), ktlint scope/LOC and popularity (04 §2, §3,
§6), ktlint CLI/config surface and competition (08 §3, §4).

## TL;DR

- **Verdict: useful, for CI/pre-commit/editor users of plain ktlint, but only as a second product after ktfmt and
  only if scoped to the standard ruleset.** ktlint is still the most used Kotlin style tool (§1), its JVM cost is
  real and measurable (§2), and the upstream "native" answer is currently **broken**: the 2.0.0-ALPHA-4 native binaries
  crash on every Kotlin input on Linux and Windows (§2). Nobody else has a verified native ktlint (§5).
- **For whom:** CLI/CI/pre-commit/editor users (`ktlint` binary, `--stdin -F`), Bazel/`rules_lint`, and teams hit by
  embedded-compiler classpath clashes (ktlint 1.8.0 breaks in Gradle with Kotlin 2.4, §4). **Not** detekt
  `formatting` users or anyone with `-R` custom rulesets (compose-rules, in-house) unless we port those (§6).
- **Biggest risk:** the target is soft. The official-style momentum is behind ktfmt (JetBrains + Meta, §1), ktlint has
  **one maintainer** and a 2.0 that has been alpha for 13 months and changes rule traversal order. We would be
  chasing 104 rules × 3 code styles for a tool whose formatting role is shrinking, while the adapters that reach most
  users (Spotless, ktlint-gradle, kotlinter, IntelliJ plugin) all run ktlint in-process on the JVM.
- **Cheaper alternative worth weighing first:** a "lint-only after ktfmt" product (the ~17 non-formatting ktlint rules
  plus compose-rules checks) is a fraction of the work, and it is exactly the gap left for ktfmt users.

## 1. Trajectory after "ktfmt is the official style"

| Signal | Finding | Source (date) |
|---|---|---|
| Official formatter | "JetBrains and Meta have started the process of standardizing `ktfmt` and making it a core part of Kotlin." | [KotlinConf'26 keynote](https://blog.jetbrains.com/kotlin/2026/05/kotlinconf26-keynote-highlights/) (2026-05) |
| Toolchain `format` | Keynote: Toolchain covers "formatting code". Toolchain 0.12.0–0.12.2 release notes (2026-08-25 → 09-15) contain **no format/lint command**. | [releases](https://github.com/JetBrains/kotlin-toolchain/releases), [0.12 blog](https://blog.jetbrains.com/kotlin/2026/09/kotlin-toolchain-0-12-multiplatform-library-publishing-wasm-apps-and-more/) |
| ktfmt releases | Still 0.64 (2026-06-24), JAR only. #705 (2026-09-24) asks for "a final standalone release while it's being upstreamed into the kotlin toolchain". | [ktfmt releases](https://github.com/Kotlin/ktfmt/releases), [#705](https://github.com/Kotlin/ktfmt/issues/705) |
| JetBrains on linting | **No statement about an official linter** in any KotlinConf'26/Toolchain post found. The JetBrains lint story is IntelliJ inspections via Kotlin LSP (alpha, JVM, IntelliJ-based) and Qodana. Toolchain 0.12 adds *Android Lint*, not a Kotlin style linter. | [kotlin-lsp docs](https://kotlinlang.org/docs/kotlin-lsp.html), [0.12 blog](https://blog.jetbrains.com/kotlin/2026/09/kotlin-toolchain-0-12-multiplatform-library-publishing-wasm-apps-and-more/) |
| Practitioner read | "no single one of [ktlint, detekt, ktfmt] has been predominantly used until now"; ktfmt "will be maintained as a built-in code formatter". | [zenn KotlinConf report](https://zenn.dev/hktechno/articles/042c6fc20a808d?locale=en) (2026-06) |
| Split pattern | Block/Square: ktfmt for formatting + detekt for linting, ktlint removed. Allegro (2026-03): stays on ktlint, did not roll out detekt. | [Block](https://engineering.block.xyz/blog/adopting-ktfmt-and-detekt), [Allegro](https://blog.allegro.tech/2026/03/static-code-analysis-kotlin.html) |
| Usage, code search | `libs.versions.toml`: ktlint **6,288**, detekt 7,168, ktfmt 858 (04 §3 had ktlint 5,952 / ktfmt 801 on 09-25; the delta is index noise, not growth). `.pre-commit-config.yaml`: ktlint 182, ktfmt 153. | GitHub code search, 2026-09-30 |
| Usage, downloads | ktlint 1.8.0 CLI asset: **790,434** downloads since 2025-11-14. JetBrains plugins: detekt 546,502, Ktlint 459,276, ktfmt 113,769. | `gh api` releases; plugins.jetbrains.com API, 2026-09-30 |
| Stars | ktlint 6,747, detekt 7,077 (2026-09-30) vs 6,733 / 7,031 on 2026-08-20: flat. | [pistack](https://www.pistack.xyz/posts/2026-08-20-kotlin-linting-tools-detekt-ktlint-spotless-comparison/) (2026-08-20), `gh api` |
| Maintenance | Paul Dingemans (2026-06-18): "Currently I am the only maintainer … Next to this I am also maintaining the Ktlint Intellij Plugin." ktlint-gradle is moving into the `ktlint` org. 41 commits since ALPHA-4, mostly rule fixes. | [ktlint#3324](https://github.com/ktlint/ktlint/issues/3324), commits 2026-08-21→09-28 |

**Read:** no evidence of decline yet (ktlint's installed base is still ~7x ktfmt's), but no growth either, and the
formatting half of ktlint is now the non-official style. What survives the ktfmt move is (a) the installed base that
will not reformat a whole repo to ktfmt, and (b) ktlint as a *lint* gate (naming, filename, wildcard imports,
KDoc, comment placement), which ktfmt does not do. ktlint's `ktlint_official` style is itself contested
([#2703 "ktlint has become too opinionated"](https://github.com/ktlint/ktlint/issues/2703)).

## 2. ktlint 2.0 and native binaries: measured

| Item | Status |
|---|---|
| Latest stable | **1.8.0 (2025-11-14)**. No 1.8.x patch exists. 1.8.0 is **incompatible with Kotlin 2.4** in-process ("Extensions storage is not registered"); the maintainer's answer is "resolved in 2.0.0-ALPHA-4" ([#3403](https://github.com/ktlint/ktlint/issues/3403), 2026-09-28). |
| Latest 2.0 | **2.0.0-ALPHA-4 (2026-08-21)**, prerelease; earlier alphas were never published. No beta/RC. Master is `2.0.0-ALPHA-5-SNAPSHOT`. Breaking: per-node rule traversal (#3252), `RuleV2`, coordinates `io.github.ktlint`. ([CHANGELOG](https://github.com/ktlint/ktlint/blob/master/CHANGELOG.md)) |
| Native assets | linux-x86-64 62 MB, darwin-arm64 54 MB, windows-x86-64 57 MB. **Downloads: 111 / 20 / 49** vs 200 for the ALPHA-4 JAR (2026-09-30). No linux-arm64, no darwin-x64. |
| Native works? | **No.** Both the Windows binary (local) and the Linux binary (testbox) print `ExceptionInInitializerError … IStubFileElementType … Could not find 'theUnsafe' field in the Unsafe class` on `echo 'class Foo' \| ktlint --stdin` and on every repo; only `--version` works. No upstream issue reports it (search 2026-09-30). The PR's reflection metadata covers only ServiceLoader providers ([#3284](https://github.com/ktlint/ktlint/pull/3284)). |
| Native custom rulesets | Not supported (04 §2, 08 §3); nothing on master changes that. |
| Published native perf numbers | **None** found for ktlint (web search 2026-09-30). ktfmt's native PR claims "up to 100x" on small inputs only (04 §6). |

JVM baseline, testbox (Linux, 11 pinned cores, Java 21), lint only, `--relative '**/*.kt'`, best of 2:

| Repo | Files / MB | 1.8.0 wall / CPU / RSS | 2.0.0-ALPHA-4 JVM wall / CPU / RSS |
|---|---|---|---|
| `echo 'class Foo' \| --stdin` | 1 | 0.80 s / 1.5 s / 151 MB | — (native crashes: 0.02 s) |
| nowinandroid | 310 / 1.1 | 3.0 s / 20 s / 452 MB | 3.5 s / 24 s / 609 MB |
| okhttp | 573 / 4.5 | 8.0 s / 50 s / 860 MB | 10.9 s / 59 s / 1.25 GB |
| kotlinx.coroutines | 1,039 / 4.0 | 7.6 s / 55 s / 738 MB | 9.5 s / 65 s / 877 MB |
| ktor | 2,423 / 10.3 | 12.1 s / 74 s / 620 MB | 17.0 s / 94 s / 866 MB |

- ktlint JVM lints at **~0.14 MB per CPU-second** (ktor); 2.0 alpha is ~20–40% *slower* and uses more memory than 1.8.
  For scale, `ktrs_fmt` formats at ~5.3 MB/s on one thread (08 §3), so a Rust linter doing comparable tree work has a
  ~30x CPU headroom, in line with ktlint-rs's unverified 17–24x wall claims (§5). Windows (local, busy laptop):
  okhttp 20–58 s wall with 1.8.0.
- Raw numbers: testbox `~/work/research10-klbench.log`.

## 3. detekt

| Item | Finding |
|---|---|
| Release | 2.0.0-alpha.6 (2026-08-04); alphas since 2025-09-04; last stable 1.23.8 (2025-02-21). ([releases](https://github.com/detekt/detekt/releases)) |
| ktlint wrapper in 2.0 | Kept: module `detekt-rules-ktlint-wrapper` bundles a shaded ktlint (`ktlint-repackage`) and moved to JVM 17+ in alpha.6. The 2023 removal poll ended in "We'll keep detekt-formatting" ([discussion #5997](https://github.com/detekt/detekt/discussions/5997)). |
| Wrapper speed | [#7880](https://github.com/detekt/detekt/issues/7880) "Formatting rule set is really slow" (2025-01): ktlint rules are the slowest detekt rules. [#9291](https://github.com/detekt/detekt/pull/9291) (2026-04) found "88 PSI copies and 88 AST walks per file" on a 720-file project and made the ruleset ~3x faster; **reverted** 2026-06-10 ([#9380](https://github.com/detekt/detekt/pull/9380)). Still open. |
| detekt speed | 2.0 type resolution: ~1 min → ~24 min on an 80k-LOC KMP monorepo ([#8882](https://github.com/detekt/detekt/issues/8882), 2025-11). |
| Would a native ktlint help? | Only if the user drops `detekt-formatting` and runs our binary as a separate step. The wrapper links ktlint as a JVM library; detekt can't call a Rust binary without a new rule-provider shim. Users like it for detekt's IDE plugin + single config (#5997 comments), so expect low pull. |

## 4. Pain points (quantified where possible)

| Complaint | Number | Source (date) |
|---|---|---|
| JVM startup per `--stdin` call | ~1.2–1.5 s per file; project generation 45 s → **8 m 20 s** with ktlint on 428 files; asks for a batch mode | [ktlint#2754](https://github.com/ktlint/ktlint/issues/2754) (2024-07) |
| Pre-commit hook scans whole tree when no files changed | slow no-op commits; maintainer recommends the IDE plugin and says pre-commit is "almost useless" | [ktlint#2976](https://github.com/ktlint/ktlint/issues/2976) (2025-04) |
| Gradle-driven git hook | **1 m 49 s** through Gradle vs ~1 s calling ktlint directly | [ktlint-gradle#328](https://github.com/JLLeitschuh/ktlint-gradle/issues/328) (2020-01) |
| Classloader isolation | kotlinter 5.0: `clean lintKotlin` 13 s → 25 s (classloader) / 19 s (process); metaspace leak runs the daemon out of memory | [kotlinter#420](https://github.com/jeremymailen/kotlinter-gradle/pull/420) (2024-12) |
| Embedded compiler vs project Kotlin | ktlint 1.8.0 + Kotlin 2.4.20 breaks ktlint-gradle; workaround pins `org.jetbrains.kotlin` in `ktlint*` configurations | [ktlint#3403](https://github.com/ktlint/ktlint/issues/3403) (2026-09-28) |
| Spotless + ktlint mismatch | Spotless reports max-line-length errors the ktlint CLI does not | [spotless#1913](https://github.com/diffplug/spotless/issues/1913), cited in [ktlint#3324](https://github.com/ktlint/ktlint/issues/3324) (2026-06) |
| Memory | 0.45–1.25 GB RSS on 1–10 MB repos (§2) | testbox, 2026-09-30 |

No 2026 Reddit/HN thread found that complains about ktlint speed specifically. The loud complaints are about
per-invocation startup, Gradle integration overhead and classpath conflicts, **not** steady-state throughput.
Incremental Gradle runs are already sub-second (kotlinter#420).

## 5. Competitors (native)

| Project | State 2026-09-30 |
|---|---|
| ktlint-rs (qdsfdhvh) | v0.1.20 (2026-08-25) is still latest; **0 commits since 2026-08-25** (last 2 that day); 1 star; last push 2026-08-25. README claims 101 ktlint IDs "registered" plus 148 detekt rules, 24x/17x on nowinandroid/okhttp, but its matrix still says "Registration only; behavior remains unverified until differential fixtures pass". **No published differential parity numbers.** [repo](https://github.com/qdsfdhvh/ktlint-rs) |
| ktlint native (upstream) | Crashes on all input in ALPHA-4 (§2). Fixable with more reflection metadata, so expect it to work in ALPHA-5 or later. Throughput will then be JIT-less (see 02 palantir data). |
| New since 2026-09-25 | None. GitHub repo searches (Rust + ktlint/detekt/Kotlin linter, created after 2026-08-15/09-01) return only ktlint-rs, `linthis` (shells out to ktlint/detekt) and SearchDeadCode (dead code, not style). |

## 6. Who would switch, and how

| Integration | Today | Native adoption path | Blocked by custom rulesets? |
|---|---|---|---|
| ktlint CLI / CI scripts | `ktlint` JAR (790k downloads of 1.8.0) | Drop-in binary with the 08 §4 CLI surface and exit codes | Only if `-R` is used |
| pre-commit / git hooks | ktlint's `installGitPreCommitHook`, third-party hook repos (182 configs) | Binary hook, no JVM; biggest perceived win (1 s → ~20 ms startup) | Rarely |
| Editors (conform, apheleia, none-ls, Zed) | `ktlint --stdin -F`, `--reporter=json --stdin` | stdin parity; apheleia defaults to ktlint (08 §2) | No |
| Bazel `rules_lint` / rules_kotlin | drives ktlint binary | Hermetic binary | Sometimes |
| Spotless `ktlint()` | in-process, `customRuleSets(...)` | `nativeCmd` today (one process per file, no path → no `.editorconfig`); first-class step needs an upstream PR | Yes, for compose-rules users |
| ktlint-gradle / kotlinter | worker/process isolation, in-process engine | Needs an "executable" mode upstream or our own plugin; ktlint-gradle is joining the ktlint org (#3324), so upstream is plausible but single-maintainer bandwidth is thin | Yes (`ktlintRuleset`/`ktlint(...)` deps) |
| IntelliJ Ktlint plugin (459k) | bundles ktlint JARs, warm JVM | Little latency to win in-IDE; would need a fork that shells out | Plugin supports external rulesets |
| detekt `formatting` | shaded ktlint library | Not reachable without dropping the wrapper (§3) | n/a |

- **compose-rules**: v0.6.7 (2026-09-24), 735 stars, still built per ktlint version. About 1 in 7 ktlint catalogs pull
  it (04 §2). Every Compose/Android shop in that slice is blocked until we port its ktlint checks (~5k LOC, 04 §7).
- **In-house rulesets** (e.g. `KilgoreT/ktlint-rules`, created 2026-09-25) cannot be served by any native binary.

## 7. Recommendation

1. Don't start a full ktlint port now. Ship ktfmt (Phase A/B in 08 §5) and measure pull first.
2. If lint work starts, target **ktlint 1.8.0 standard rules, `ktlint_official` + `intellij_idea`**, CLI/pre-commit/
   editor channels only, and publish a differential parity number from day one (the thing ktlint-rs never did).
3. Cheaper first step: a native **lint-only** ruleset for ktfmt users (the ~17 lint-only ktlint rules + compose-rules
   ktlint checks). It complements the official formatter instead of competing with it.
4. Re-check before committing: ktlint ALPHA-5 native (working? how fast?), ktlint-rs activity, and whether the Kotlin
   Toolchain ships a `lint` alongside `format`.

## Sources

- https://github.com/ktlint/ktlint/releases · https://github.com/ktlint/ktlint/blob/master/CHANGELOG.md · https://github.com/ktlint/ktlint/pull/3284
- https://github.com/ktlint/ktlint/issues/3324 · /3403 · /2754 · /2976 · /2703 · https://github.com/ktlint/ktlint/pull/3398
- https://github.com/detekt/detekt/releases · https://github.com/detekt/detekt/discussions/5997 · https://github.com/detekt/detekt/issues/7880 · /8882 · https://github.com/detekt/detekt/pull/9291 · /9380
- https://github.com/jeremymailen/kotlinter-gradle/pull/420 · https://github.com/JLLeitschuh/ktlint-gradle/issues/328 · https://github.com/diffplug/spotless/issues/1913
- https://github.com/qdsfdhvh/ktlint-rs · https://github.com/mrmans0n/compose-rules/releases · https://github.com/nbadal/ktlint-intellij-plugin/releases
- https://blog.jetbrains.com/kotlin/2026/05/kotlinconf26-keynote-highlights/ · https://blog.jetbrains.com/kotlin/2026/09/kotlin-toolchain-0-12-multiplatform-library-publishing-wasm-apps-and-more/ · https://github.com/JetBrains/kotlin-toolchain/releases
- https://github.com/Kotlin/ktfmt/releases · https://github.com/Kotlin/ktfmt/issues/705 · https://kotlinlang.org/docs/kotlin-lsp.html
- https://zenn.dev/hktechno/articles/042c6fc20a808d · https://blog.allegro.tech/2026/03/static-code-analysis-kotlin.html · https://engineering.block.xyz/blog/adopting-ktfmt-and-detekt · https://www.pistack.xyz/posts/2026-08-20-kotlin-linting-tools-detekt-ktlint-spotless-comparison/
- plugins.jetbrains.com API (plugins 15057, 14912, 10761) and GitHub code search, 2026-09-30
