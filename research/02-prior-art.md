# 02 — Prior art: native (non-JVM) Kotlin formatters, linters, parsers

Researched 2026-09-25 via web search, crates.io API, GitHub web search, docs.rs. Star counts and
activity are snapshots from that date. The GitHub API was rate-limited, so some repo facts come from
page summaries; items marked (unverified) had no second source.

## TL;DR

- **Nobody has shipped a credible native Kotlin formatter or linter yet.** There are three small
  Rust attempts from 2026 (`ktfmt-rs`, `ktlint-rs`, the archived `formatter-kotlin`). Each has 0–1
  stars, one author, and parity that is either unproven or not claimed. They look fast-built and
  probably AI-assisted. None of them publishes a measured parity score against ktfmt output.
- **The JVM incumbents are fixing startup instead of rewriting.** ktfmt merged GraalVM native-image
  support on 2026-06-30 (PR #584 claims "up to 100x faster"; it is still unreleased in the
  changelog). ktlint ships native-image binaries starting with `2.0.0-ALPHA-4` (2026-08-20). Block's
  `kotlin-formatter` adds a JVM daemon.
- **Strategic risk:** at KotlinConf'26 (May 2026), JetBrains and Meta announced they are
  "standardizing ktfmt and making it a core part of Kotlin". The repo moved to `github.com/Kotlin/ktfmt`,
  and the Kotlin Toolchain (Amper) plans a `format` command. So the style we would clone is becoming
  *the* official Kotlin style, which is good for the target's stability. It also means JetBrains owns
  the reference implementation and may ship its own fast path, most likely native-image.
- **Parsing is the hard part.** The tree-sitter Kotlin grammars have known correctness gaps (one
  fork measures a 61.2% structural match against JetBrains PSI, and it has file-eating ERROR bugs).
  The JVM LightTree parser already runs at about 18.5 MB/s on one thread. Native wins come from
  **startup and parallelism**, not raw parse speed.

## Direct prior art (Kotlin-specific, non-JVM)

| Project | Lang | Status | Stars / activity | Relevance | Link |
|---|---|---|---|---|---|
| **ktfmt-rs** ("Ultrafast Kotlin Formatter", "part of the detekt ecosystem") | Rust, handwritten lexer/parser/CST (no tree-sitter dep) | v0.2.0 and v0.3.0 both published 2026-03-08. **GitHub repo now 404**, and detekt's org page doesn't list it. Looks abandoned or pulled. | 52 crates.io downloads; ~5.5K SLoC; repo ships a `CLAUDE.md` | **Highest name overlap.** Despite the name it isn't ktfmt-compatible: it has a rule list (140-col default, editorconfig) and makes no ktfmt parity claim. v0.3.0 fixed "infinite loops in parser on destructuring lambdas". Squats the `ktfmt-rs` / `ktfmt` crate names. | [crates.io](https://crates.io/crates/ktfmt-rs) · [lib.rs](https://lib.rs/crates/ktfmt-rs) · [docs.rs src](https://docs.rs/crate/ktfmt-rs/latest/source/) |
| **ktlint-rs** (qdsfdhvh) | Rust, custom parser | Active: 419 commits, v0.1.20 on 2026-08-25, all commits within ~2 weeks in Aug 2026 | 1 star | **Closest to "ktlint in Rust".** Claims all 101 ktlint 1.8.0 rules plus 148 detekt rules are "registered", but its README says "behavior remains unverified until differential fixtures pass". Claims 24x on nowinandroid (0.29s vs 7.1s) and 17x on okhttp. Uses an oracle-diff approach against a pinned ktlint JVM. | [github](https://github.com/qdsfdhvh/ktlint-rs) |
| **formatter-kotlin** (AkaraChen) | Rust + Topiary + tree-sitter-kotlin-ng | **Archived 2026-07-17** after 6 commits | 0 stars | Goal was to "fully replace ktfmt", using ktfmt's ~425 test snippets as the parity bench. Explicitly out of scope: line-length-aware wrapping, chaining, import ordering, trailing commas, KDoc. Those are exactly the hard parts. **This is the failed Topiary route.** | [github](https://github.com/AkaraChen/formatter-kotlin) |
| kotlin-parser (vyfor) | Rust, handwritten | WIP; 9 commits, last activity ~Sep 2024 | 2 stars | Abandoned early parser | [github](https://github.com/vyfor/kotlin-parser) · [crates.io](https://crates.io/crates/kotlin-parser) |
| kotlin-rs (gaultier) | Rust compiler | Archived 2021-02; 1,604 commits; handled expressions and functions only (no classes or generics) | 16 stars | Old abandoned attempt. Shows how big Kotlin's grammar is. | [github](https://github.com/gaultier/kotlin-rs) |
| kotlinrs (MenkeTechnologies) | Rust lexer/parser → fusevm | Active, toy | 1 star | Not tooling | [github](https://github.com/MenkeTechnologies/kotlinrs) |
| AStyle fork (AdHoc-Protocol) | C++ | Active; 7 commits | 0 stars | A lexer-level "kotlin mode" aiming at ktlint-ish style. No real parser. | [github](https://github.com/AdHoc-Protocol/AStyle) |
| kotlin-auto-formatter (hovinen) | Kotlin/JVM | Archived 2024-06 | 21 stars | Earlier column-aware formatter that lost to ktfmt and ktlint | [github](https://github.com/hovinen/kotlin-auto-formatter) |
| **kmp-lsp** (Hessesian; formerly crate `kotlin-lsp`) | Rust + tree-sitter | Very active (949 commits) | 194 stars | A no-JVM LSP for Kotlin, Java and Swift: symbols, hover, rename. **No formatting.** Evidence of demand for no-JVM Kotlin tooling, and a possible integration partner. Fork: qdsfdhvh/kotlin-lsp. | [github](https://github.com/Hessesian/kmp-lsp) |
| mutant-kraken | Rust (tree-sitter) | 2025 | small | Kotlin mutation testing. Shows tree-sitter-kotlin is good enough for tolerant tooling. | [crates.io](https://crates.io/crates/mutant-kraken) |
| searchdeadcode | Rust | 2026 | small | Dead code finder for Kotlin/Java | [github](https://github.com/KevinDoremy/SearchDeadCode) |

### Wrappers and orchestrators (not native implementations)

| Project | Lang | What it does | Stars | Link |
|---|---|---|---|---|
| kempt (Zac Sweers) | Rust | Pre-commit pipeline that **downloads and runs** ktfmt, GJF (native ~20ms vs jar ~500ms), cargo fmt. Author is a ktfmt contributor. | 61 | [github](https://github.com/ZacSweers/kempt) |
| poly (Goldziher) | Rust | Universal linter/formatter. Kotlin shells out to ktfmt if present, otherwise falls back to a tree-sitter tier. | 18 | [github](https://github.com/Goldziher/poly) |
| linthis | Rust | Shells out to ktlint/detekt | 7 | [github](https://github.com/zhlinh/linthis) |
| block/kotlin-formatter | Kotlin/JVM | ktfmt wrapper with configurable width, IntelliJ plugin, Gradle plugin, and an **experimental daemon** (lockfile port, 1h idle shutdown) to hide JVM startup. v1.6.2. | 60 | [github](https://github.com/block/kotlin-formatter) · [Block blog](https://engineering.block.xyz/blog/adopting-ktfmt-and-detekt) |
| Zed Kotlin extension | — | Uses ktfmt (JVM) as the default formatter; issue asks for ktlint | — | [github](https://github.com/zed-extensions/kotlin) |
| dprint | Rust | No Kotlin plugin. Only the generic `exec` plugin, which can wrap ktfmt. | — | [dprint.dev/plugins](https://dprint.dev/plugins/) |
| Biome / oxc | Rust | No Kotlin on either roadmap. Biome's 2026 roadmap is web languages only. | — | [Biome 2026](https://biomejs.dev/blog/roadmap-2026/) |
| Topiary | Rust | No Kotlin language config shipped | — | [github](https://github.com/topiary/topiary) |
| ast-grep | Rust | Supports Kotlin through its own `tree-sitter-kotlin-sg` grammar. Has a catalog of Kotlin lint rules (structural YAML). | 10k+ | [catalog](https://ast-grep.github.io/catalog/kotlin/) |

### Parsers available to a Rust project

| Grammar | Downloads | Notes |
|---|---|---|
| `tree-sitter-kotlin-ng` (tree-sitter-grammars) | 2.5M | Updated 2025-01. Known bugs: identifiers with keyword prefixes (`in1`, `open_file`) become ERROR at line start (issue #19); a one-line `object : X { fun m() {} }` collapses the rest of the file into ERROR. |
| `tree-sitter-kotlin-sg` (ast-grep) | 2.2M | Updated 2026-05 |
| `tree-sitter-kotlin` (fwcd) | 417k | Updated 2024-08. The jkumz fork cross-validates against JetBrains PSI fixtures and gets **74/121 (61.2%) structural match among clean parses**. |
| `arborium-kotlin` | 131k | Repackaged tree-sitter |

**Takeaway:** tree-sitter-kotlin is fine for editors, search and lint heuristics. It is **not safe for a
formatter that rewrites whole files**, because one misparse can corrupt everything after it. Ruff, Biome
and rustfmt all use handwritten, lossless, error-resilient parsers. Air is the one exception: it uses
tree-sitter-r feeding a rowan tree, but R's grammar is far smaller than Kotlin's.

## JVM incumbents and JetBrains / Meta activity

| Item | Status (2026-09) | Link |
|---|---|---|
| **ktfmt → Kotlin org** | KotlinConf'26 keynote: "JetBrains and Meta have started the process of standardizing `ktfmt` and making it a core part of Kotlin." Repo is now `Kotlin/ktfmt` (1.3k stars, 807 commits). Issue #705 asks for a "final standalone release while it's being upstreamed into the kotlin toolchain". | [keynote](https://blog.jetbrains.com/kotlin/2026/05/kotlinconf26-keynote-highlights/) · [#705](https://github.com/Kotlin/ktfmt/issues/705) |
| **ktfmt native-image** | PR #584 by sgammon: opened 2026-01-12, **merged 2026-06-30**, claims "up to 100x faster" for small inputs. The only code change is lazy init of `KotlinCoreEnvironment`. Listed under Unreleased in the CHANGELOG. An earlier GraalVM attempt failed on resource loading (oracle/graal#8502). | [PR](https://github.com/facebook/ktfmt/pull/584) |
| ktfmt perf | v0.64 cut allocations for "~6-7%" speedup. Old Meta figures: 3,500 files in 5.9s (ktfmt) vs 14.8s (ktlint). | [HN](https://news.ycombinator.com/item?id=33330965) |
| **ktlint native** | `2.0.0-ALPHA-4` (2026-08-20): "add GraalVM native-image binaries to release (linux, macOS, Windows)" (#3284). ktlint has been community-maintained since leaving Pinterest (`io.github.ktlint`). ~6.7k stars. | [CHANGELOG](https://github.com/ktlint/ktlint/blob/master/CHANGELOG.md) |
| Kotlin Toolchain (Amper) | Alpha at KotlinConf'26. v0.11 blog: "In the future, it will even let you format your code". No `format` command yet. | [0.11](https://blog.jetbrains.com/amper/2026/06/kotlin-toolchain-0-11/) |
| JetBrains kotlin-lsp | Alpha at KotlinConf'26. Built on the IntelliJ engine plus proprietary Air/Fleet parts, so it's heavy and JVM-based. Formats with IntelliJ's formatter. | [github](https://github.com/Kotlin/kotlin-lsp) · [docs](https://kotlinlang.org/docs/kotlin-lsp.html) |
| IntelliJ CLI formatter | `format.sh` boots a headless IDE; very slow per invocation. Users ask JetBrains for better DX. | [docs](https://www.jetbrains.com/help/idea/command-line-formatter.html) |
| K2 LightTree parser | Parser benchmark on 8,013 files / 35.2 MB: **lighttree 18.5 MB/s (1 thread), 291 MB/s (16 threads)**; PSI 10.8 / 166 MB/s. KT-77993 is about further parser optimization. | [congo-kotlin-bench](https://github.com/CodeLaser/congo-kotlin-bench) · [KT-77993](https://youtrack.jetbrains.com/issue/KT-77993) |
| detekt | 2.0 type resolution regressions: issue #8882 reports ~1 min → ~24 min on a KMP monorepo. The non-type-resolved rules are the portable part. | [#8882](https://github.com/detekt/detekt/issues/8882) |
| Diktat | Rules built on top of ktlint (100+ inspections). Low activity. | [github](https://github.com/saveourtool/diktat) |

## Analogues: what success looked like

| Project | Replaces | Parity achieved / how measured | Speed | Team / funding / timeline |
|---|---|---|---|---|
| **Ruff formatter** | Black | **>99.9% of lines identical** on Black-formatted Django and Zulip. Django: 34/2,772 files differ; Zulip similarity index 0.99727. They built an **ecosystem similarity-index CI check** and a tracking issue of intentional deviations (#5828). | 30x Black; Zulip 250k LOC in 0.10s vs 3.20s | Linter launched ~Aug 2022; formatter beta 2023-10-24 (about 1 year after the linter). Led by Micha Reiser (ex-Rome) and konstin. **Adapted Biome's printer.** Astral: $4M seed (Accel, 2023, 3 people), later unannounced A/B (a16z), ~19 staff. **Acquired by OpenAI (announced 2026-03-19).** |
| **Biome** (ex-Rome) | Prettier | ~85% → **>96–97% of Prettier's test-suite snapshots** in about 3 weeks, driven by a $20k Prettier bounty (Nov 2023, v1.4.0). A diff harness against Prettier snapshots was the key infrastructure. | ~25–35x Prettier | Rome Tools raised $4.5M, then laid everyone off in 2023. Community fork Aug 2023, now funded by Open Collective and sponsors. Handful of core members. `biome_formatter` IR/printer is language-agnostic and **reused by Ruff and Air**. |
| **oxfmt** (VoidZero) | Prettier | Alpha (Jan 2026) >95% → **100% of Prettier JS/TS conformance tests** at beta (Feb 2026). Reports remaining divergences upstream to Prettier. | >30x Prettier, 3x Biome | VC-funded (VoidZero); reused the mature oxc parser |
| **Air** (Posit) | styler (R) | New style, not a clone, so no parity target | "Format entire projects in under a second" | Announced 2025-02-21 by 2 people (Davis Vaughan, Lionel Henry). **tree-sitter-r → rowan → biome_formatter.** Ships inside Positron. |
| swift-format (Apple) vs SwiftFormat (Lockwood) | — | Two coexisting styles. Apple's won distribution by being **bundled in Xcode 16 / the toolchain**. | SwiftFormat is faster | Shows distribution beats speed |
| palantir-java-format / GJF native-image | JVM GJF | Identical output (same code) | **Startup >10x better, but throughput worse:** palantir's large batch took 1m20s native vs 30s JVM, so spotless keeps JVM on Java 21+. GJF native ~20ms startup vs ~500ms jar. | Cheap to build: a build config, not a rewrite |

## Lessons for our project

1. **We would not be duplicating a serious effort, but we'd be entering a crowded field of weak
   attempts.** `ktfmt-rs` took the obvious name and vanished. `ktlint-rs` is two weeks old and
   unverified. Credibility will come from *published, reproducible parity numbers*, which none of them
   have.
2. **Beat native-image, not the JVM.** The honest competitor is ktfmt/ktlint native-image (~20–50ms
   startup, released or soon to be). Rust still wins on (a) throughput: native-image is *slower* than
   the JIT on large batches (palantir data), while Rust plus rayon should beat both; (b) binary size,
   embedding, WASM, and LSP latency; (c) a single binary for formatting *and* linting. Benchmark
   against native-image builds, not `java -jar`.
3. **Pick ktfmt as the formatting oracle.** It's deterministic, has no options, and is becoming the
   official Kotlin style, so the target is stable and blessed. Copy Ruff's method: run the similarity
   index over real repos (ktor, okhttp, nowinandroid, detekt, kotlinx.*, Exposed, arrow) in CI, keep a
   documented intentional-deviations list, and aim for >99.9% of lines. Reuse ktfmt's test snippets
   (~425+) as a conformance suite, the way Biome used Prettier's snapshots.
4. **Write a handwritten, lossless, error-resilient parser; don't format off tree-sitter.**
   formatter-kotlin (Topiary) died on wrapping and chaining. tree-sitter-kotlin has 61% PSI structural
   match and file-eating ERROR bugs. ktfmt-rs hit infinite parser loops. Validate the parser against
   the Kotlin compiler's PSI test fixtures and a corpus of 8k+ files (the congo-kotlin-bench corpus
   is a ready template). A formatter must refuse to rewrite on any parse error.
5. **Reuse Biome's formatter IR/printer (or Ruff's fork of it)** instead of building our own. Ruff and
   Air both did this. ktfmt is itself a google-java-format Doc/Level printer, so mapping ktfmt's
   block/break semantics onto Biome's `group`/`indent`/`soft_line_break` IR is the core design task.
6. **Sequence formatter first, then lint.** Ruff shipped the linter first, but Kotlin's formatting
   pain (ktfmt/ktlint in pre-commit and CI) is where startup cost hurts most. For lint, target ktlint's
   *standard* rules (mostly syntactic) with the oracle-diff method ktlint-rs sketched. Avoid detekt's
   type-resolved rules, which need the compiler.
7. **Distribution and politics matter as much as speed.** swift-format won by being bundled.
   JetBrains may bundle native-image ktfmt into the Kotlin Toolchain. Engage early with the Kotlin/ktfmt
   maintainers (hick209, Zac Sweers, AbdullinAM), position the project as a compatible
   implementation of the standard style, and offer upstream divergence reports as oxfmt does with
   Prettier.
8. **Calibrate effort.** Ruff's formatter reached 99.9% about a year after the linter, with 2–3
   senior full-time engineers plus funding. Biome's last 11 points of parity took a funded bounty
   sprint with about a dozen contributors. Air took 2 people roughly a year, but for a much smaller
   grammar and a new style. Expect a Kotlin-grade parser plus a ktfmt-parity formatter to take on the
   order of **1–2 engineer-years** to reach >99% line parity.
9. **Low-cost wins exist.** kmp-lsp (194 stars, tree-sitter, no JVM) shows demand for no-JVM Kotlin
   tooling. A fast formatter would be an easy integration target for it, Zed, Helix, kempt and poly,
   all of which currently shell out to the JVM.

## Sources

- https://lib.rs/crates/ktfmt-rs · https://crates.io/crates/ktfmt-rs · https://docs.rs/crate/ktfmt-rs/latest/source/
- https://github.com/qdsfdhvh/ktlint-rs · https://github.com/AkaraChen/formatter-kotlin · https://github.com/vyfor/kotlin-parser · https://github.com/gaultier/kotlin-rs
- https://github.com/Hessesian/kmp-lsp · https://github.com/ZacSweers/kempt · https://github.com/Goldziher/poly · https://github.com/block/kotlin-formatter
- https://github.com/facebook/ktfmt/pull/584 · https://github.com/Kotlin/ktfmt · https://github.com/Kotlin/ktfmt/issues/705 · https://github.com/oracle/graal/issues/8502
- https://blog.jetbrains.com/kotlin/2026/05/kotlinconf26-keynote-highlights/ · https://blog.jetbrains.com/amper/2026/06/kotlin-toolchain-0-11/
- https://github.com/ktlint/ktlint/blob/master/CHANGELOG.md · https://github.com/detekt/detekt/issues/8882
- https://github.com/CodeLaser/congo-kotlin-bench · https://github.com/jkumz/tree-sitter-kotlin · https://github.com/tree-sitter-grammars/tree-sitter-kotlin/issues/19
- https://astral.sh/blog/the-ruff-formatter · https://github.com/astral-sh/ruff/issues/5828 · https://simonwillison.net/2026/Mar/19/openai-acquiring-astral/
- https://biomejs.dev/blog/biome-wins-prettier-challenge/ · https://prettier.io/blog/2023/11/27/20k-bounty-was-claimed/ · https://biomejs.dev/blog/announcing-biome/
- https://oxc.rs/blog/2026-02-24-oxfmt-beta.html · https://tidyverse.org/blog/2025/02/air/ · https://github.com/posit-dev/air
- https://github.com/palantir/palantir-java-format · https://mjtsai.com/blog/2024/11/06/swift-format-in-xcode-16/
- https://ast-grep.github.io/catalog/kotlin/ · https://github.com/zed-extensions/kotlin · https://dprint.dev/plugins/ · https://biomejs.dev/blog/roadmap-2026/
