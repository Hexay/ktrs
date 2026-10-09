<div align="center">

# ktrs

**Kotlin formatting and linting in Rust, without starting a JVM.**

[![CI](https://github.com/Hexay/ktrs/actions/workflows/ci.yml/badge.svg)](https://github.com/Hexay/ktrs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ktrs.svg)](https://crates.io/crates/ktrs)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](#license)
[![Playground](https://img.shields.io/badge/try%20it-playground-7f52ff.svg)](https://hexay.github.io/ktrs/)

<img src="assets/benchmark.svg" alt="Format okhttp: ktrs 0.18 s vs ktfmt 3.30 s. Lint okhttp: ktrs 0.34 s vs ktlint 11.56 s." width="720">

</div>

ktrs is a native replacement for [ktfmt](https://github.com/facebook/ktfmt) and
[ktlint](https://github.com/pinterest/ktlint). It produces the same output, it is 13-190x faster, and
it ships as small native binaries with no runtime.

- ⚡ **Fast.** Under 10 ms per file in an editor or pre-commit hook, against roughly a second of JVM
  startup. On whole projects it uses every core and is still at least 13x faster.
- 🎯 **Identical output.** Byte-identical to ktfmt 0.64 on 6,121 of 6,123 real-world files (both
  tools reject the other two). Lint violations and `--format` output match ktlint 2.0 in all three
  code styles, and ktlint 1.8 in a compatibility mode.
- 🔌 **Drop-in.** The `ktfmt` and `ktlint` binaries accept the originals' flags, messages and exit
  codes, so existing scripts, hooks and CI keep working.
- 🧩 **Fits your setup.** Integrations for GitHub Actions, pre-commit, Spotless, ktfmt-gradle and
  ktlint-gradle drop-in plugins, and Neovim, Helix, Zed, Emacs, VS Code and IntelliJ/Android Studio.
- 🌳 **Built on a faithful parser.** ktrs includes a lossless Kotlin parser whose tree matches the
  Kotlin compiler's PSI node for node.

Try the formatter in your browser in the **[playground](https://hexay.github.io/ktrs/)**, which runs
it as WebAssembly.

## Installation

```sh
curl -fsSL https://raw.githubusercontent.com/Hexay/ktrs/master/install.sh | sh   # prebuilt binaries
cargo binstall ktrs                                                               # prebuilt, via cargo-binstall
cargo install ktrs                                                                # from crates.io
brew tap hexay/ktrs https://github.com/Hexay/ktrs && brew install hexay/ktrs/ktrs  # Homebrew
scoop bucket add ktrs https://github.com/Hexay/ktrs && scoop install ktrs/ktrs     # Scoop (Windows)
npm install --save-dev @ktrs/cli                                                  # npm (npx ktlint, npx ktfmt)
docker run --rm -v "$PWD:/src" ghcr.io/hexay/ktrs ktlint "**/*.kt"                 # Docker (linux amd64/arm64)
```

One install puts three binaries on your PATH: `ktrs`, `ktfmt` and `ktlint`. On Windows, the
install script also works under Git Bash, or download a zip from
[Releases](https://github.com/Hexay/ktrs/releases). The Docker image takes the binary name as its first argument,
works in `/src`, and has no Java, so `ktlint -R` with a rule set other than compose-rules doesn't run there; on
Linux, add `--user "$(id -u):$(id -g)"` to keep fixed files owned by you.

## Usage

```sh
ktrs fmt                              # format every .kt/.kts under the current directory
ktrs fmt --style kotlinlang src/      # styles: meta (default), google, kotlinlang
ktrs fmt --check                      # CI: list files that would change, exit 1 if any
ktrs fmt - < Foo.kt                   # stdin to stdout, for editors

ktrs lint                             # check every .kt/.kts under the current directory
ktrs lint --format src/               # autocorrect what can be fixed, report the rest
ktrs lint --reporter json - < Foo.kt  # reporters: plain, json, checkstyle, sarif, html, ...
```

### Drop-in for ktfmt and ktlint

The `ktfmt` and `ktlint` binaries accept the original command lines exactly, so anything that runs
the jars can run them instead:

```sh
ktfmt --kotlinlang-style --set-exit-if-changed src/
ktlint --relative "src/**/*.kt" "!src/**/generated/**"
```

- `ktfmt` supports all of ktfmt's flags, plus `@argfile`, `-` for stdin and `--enable-editorconfig`.
- `ktlint` supports `!` negation in patterns, `-F`, `--stdin`, `--patterns-from-stdin`, `--baseline`,
  `--editorconfig`, every built-in reporter, and the git hook subcommands.
- ktlint implements 2.0.0-ALPHA-4, with its engine and all 105 standard rules.

**ktlint 1.8.** Most teams still run ktlint 1.x, and 2.0 changed rule results, autocorrect order,
exit codes and baseline matching ([research/22](research/22-ktlint-1x-gap.md)). To get 1.8.0
behaviour, add this to `.editorconfig` (the ktlint jars ignore it), or pass `--ktlint-version=1.8`:

```ini
[*.{kt,kts}]
ktrs_ktlint_version = 1.8
```

This mode ports 1.8's rule differences, its rule-by-rule autocorrect order and its CLI. On 21,000
files it matches the 1.8.0 jar except for a few KDoc whitespace rows, where 1.8's older Kotlin lexer
splits trailing spaces differently ([research/26](research/26-ktlint-18-mode.md)).

**Custom rule sets.** `-R` jars are supported, in `ktlint` and `ktrs lint`.
[compose-rules](https://github.com/mrmans0n/compose-rules) 0.6.7, by far the most used rule set,
runs natively with output identical to the jar (the `-all.jar` and the Maven artifacts). Any other
rule set or reporter jar hands the whole run to the real ktlint jar (downloaded once and checked by
SHA-256; needs Java), so it works at JVM speed ([research/27](research/27-custom-rulesets-impl.md)).

**Limits.**

- kotlinter runs ktlint inside the JVM and doesn't use these binaries.
- The `ktfmt` binary accepts Kotlin 2.4 syntax (e.g. `companion { }` blocks) that ktfmt 0.64, built
  on Kotlin 2.3, rejects.

## Integrations

### Migrating

`ktrs migrate` switches a build's ktfmt-gradle, ktlint-gradle, ktlint-maven-plugin and Spotless
setup to the ktrs drop-ins described below, editing only ids, coordinates and versions in place.
Setups it can't rewrite, such as kotlinter, get a `note:` saying what to change by hand.

```sh
ktrs migrate            # print the edits as a diff; exit 1 if there are any
ktrs migrate --write    # apply them
```

### GitHub Actions

```yaml
- uses: Hexay/ktrs@v0.5.0          # Linux, macOS and Windows
- run: ktrs fmt --check --style kotlinlang
- run: ktrs lint
```

### pre-commit

No Rust needed: on first run, the hook downloads the release binaries for its `rev`.

```yaml
- repo: https://github.com/Hexay/ktrs
  rev: v0.5.0
  hooks:
    - id: ktrs-fmt          # also: ktrs-fmt-check, ktfmt (with ktfmt's flags in `args`)
      args: [--style, kotlinlang]
    - id: ktrs-lint         # also: ktrs-lint-format, ktlint (with ktlint's flags in `args`)
```

### Editors

`ktrs lsp` is a language server for ktlint diagnostics, quick fixes and ktfmt or ktlint formatting.
It runs next to your Kotlin language server and takes its setup from the Gradle or Maven build. In VS
Code, install the `hexay.ktrs` extension, which bundles it. In IntelliJ IDEA 2025.3+ and Android
Studio, install the `ktrs` plugin (`io.github.hexay.ktrs`, also bundling it; the IDEs without
the platform LSP client need LSP4IJ), or the release's `ktrs-intellij-<version>.zip` from disk. Editor
plugins that already run `ktlint` or `ktfmt` (conform.nvim, nvim-lint, none-ls, ALE, apheleia,
flycheck-kotlin, Helix, Zed, VS Code's mskelton.ktlint, Block's IntelliJ Kotlin Formatter) work
unchanged with the drop-in binaries; their exact invocations are diffed against the jars. Configs for
both: [docs/editors.md](docs/editors.md).

### Gradle

**ktfmt-gradle drop-in.** The `io.github.hexay.ktrs` plugin replaces
[ktfmt-gradle](https://github.com/cortinico/ktfmt-gradle) 0.27.0. Change only the plugin id. The
`ktfmt { }` block, the `ktfmtCheck`/`ktfmtFormat*` tasks, `--include-only` and the
`com.ncorti.ktfmt.gradle.*` imports keep working.

```kotlin
plugins {
    id("io.github.hexay.ktrs") version "0.5.0"   // was: id("com.ncorti.ktfmt.gradle") version "0.27.0"
}
```

**ktlint-gradle drop-in.** The `io.github.hexay.ktrs.ktlint` plugin replaces
[ktlint-gradle](https://github.com/JLLeitschuh/ktlint-gradle) 14.2.0 the same way: the `ktlint { }`
block, `ktlintCheck`/`ktlintFormat` and the per-source-set, baseline and git hook tasks,
`ktlintRuleset(...)` and the `org.jlleitschuh.gradle.ktlint.*` types keep working. Console output,
reports and formatted files match the original with ktlint 1.8.0
([research/29](research/29-ktlint-gradle-dropin.md)).

```kotlin
plugins {
    id("io.github.hexay.ktrs.ktlint") version "0.5.0"   // was: id("org.jlleitschuh.gradle.ktlint") version "14.2.0"
}
```

`version` defaults to `"1.8.0"`; `"2.0.0-ALPHA-4"` selects 2.0, and other versions fail the build.
compose-rules runs natively; other rule sets run the task through the real ktlint jar.

**Spotless.** `KtrsStep` replaces `ktfmt()` and `KtrsKtlintStep` replaces `ktlint()` (Spotless 7+):

```kotlin
buildscript {
    repositories { mavenCentral() }
    dependencies { classpath("io.github.hexay:ktrs:0.5.0") }
}

spotless {
    kotlin {
        addStep(io.github.hexay.ktrs.spotless.KtrsStep.create(io.github.hexay.ktrs.KtrsOptions.kotlinlang()))
        // or, instead of ktlint("1.8.0").editorConfigOverride(...).customRuleSets(...):
        addStep(io.github.hexay.ktrs.spotless.KtrsKtlintStep.create(io.github.hexay.ktrs.KtlintOptions.defaults()
            .withEditorConfigPath(rootProject.file(".editorconfig"))
            .withEditorConfigOverride(mapOf("ktlint_code_style" to "ktlint_official"))))
    }
}
```

`KtrsKtlintStep` gives the same results as Spotless's `ktlint("1.8.0")` step
([research/28](research/28-spotless-ktlint-step.md)). `withCustomRuleSets(files)` takes jar files;
only compose-rules is supported there, other rule sets need Spotless's `ktlint()`.

<details>
<summary>JVM API and options</summary>

The `io.github.hexay:ktrs` jar has no dependencies. It bundles the native binaries for Linux, macOS
and Windows (x86-64 and ARM) and keeps long-lived `ktrs serve` processes, so a build starts the
binary once, not once per file.

- `KtrsOptions` mirrors ktfmt's options: start from `meta()`, `google()` or `kotlinlang()`, then
  chain `withMaxWidth`, `withBlockIndent`, `withContinuationIndent`, `withRemoveUnusedImports`,
  `withTrailingCommas` and `withEditorConfig(true)`.
- From other JVM code, `Ktrs.create()` returns a thread-safe formatter:
  `ktrs.format(code, KtrsOptions.google())`, or `ktrs.ktlint(code, KtlintOptions.defaults(), path)`
  for ktlint's formatted code and remaining violations.
- The plugin accepts `useClassloaderIsolation`, `processIsolationJvmArgs` and `ktfmtClasspath` but
  ignores them. `debuggingPrintOpsAfterFormatting` only logs a warning.
- To use a different binary from the bundled one, set the Gradle property `ktrs.executable`.
- Without the jar, Spotless's generic step runs the binary once per file:
  `nativeCmd("ktfmt", "/path/to/ktfmt", listOf("--kotlinlang-style", "-"))`.

</details>

### Maven

**ktlint-maven-plugin drop-in.** `io.github.hexay:ktrs-ktlint-maven-plugin` replaces gantsign's
[ktlint-maven-plugin](https://github.com/gantsign/ktlint-maven-plugin) 3.7.1. Change only the
coordinates: the goals (`mvn ktlint:check`, `ktlint:format`, the `ktlint` site report), parameters,
`ktlint.*` properties, `<reporters>` and rule sets in the plugin's `<dependencies>` keep working, and
console output, reports and formatted files match the original
([research/31](research/31-ktlint-maven-dropin.md)).

```xml
<plugin>
  <groupId>io.github.hexay</groupId>                   <!-- was: com.github.gantsign.maven -->
  <artifactId>ktrs-ktlint-maven-plugin</artifactId>    <!-- was: ktlint-maven-plugin -->
  <version>0.5.0</version>
  <executions><execution><goals><goal>check</goal></goals></execution></executions>
</plugin>
```

`<ktlintVersion>` (property `ktrs.ktlintVersion`) defaults to `1.8.0`; `2.0.0-ALPHA-4` selects 2.0.

**Spotless.** Add `implementation=` to the existing `<ktfmt>` or `<ktlint>` element and the jar as a
plugin dependency; the other options stay as they are (Maven 3.9+, Java 17+):

```xml
<plugin>
  <groupId>com.diffplug.spotless</groupId>
  <artifactId>spotless-maven-plugin</artifactId>
  <configuration>
    <kotlin>
      <ktfmt implementation="io.github.hexay.ktrs.spotless.maven.KtrsKtfmt"><style>KOTLINLANG</style></ktfmt>
      <!-- or <ktlint implementation="io.github.hexay.ktrs.spotless.maven.KtrsKtlint">…</ktlint> -->
    </kotlin>
  </configuration>
  <dependencies>
    <dependency><groupId>io.github.hexay</groupId><artifactId>ktrs</artifactId><version>0.5.0</version></dependency>
  </dependencies>
</plugin>
```

Output is identical to stock `<ktfmt>` 0.64 and `<ktlint>` 1.8.0 (`tools/spotless-maven/parity.sh`).
`<version>` must be left out or match (ktfmt `0.64`; ktlint `1.8.0` or `2.0.0-ALPHA-4`).

## Performance

Each tool is run from the command line the way users run it: same flags, same files, identical
output. Timings are wall time, the median of 5 runs after a warm-up, measured with
[hyperfine](https://github.com/sharkdp/hyperfine) on Linux (Xeon E-2136, 10 CPUs, JDK 21).

**Formatting**, the `ktfmt` binary against the ktfmt 0.64 jar:

| Scenario | ktrs | ktfmt 0.64 | Speedup |
|---|--:|--:|--:|
| Editor: one 8 KB file on stdin | <5 ms | 791 ms | **>150x** |
| Pre-commit: 10 changed files | 16 ms | 743 ms | **47x** |
| CI check on okhttp (617 files) | 183 ms | 3.87 s | **21x** |
| Format okhttp in place | 176 ms | 3.30 s | **19x** |
| Format okhttp in place, 1 core | 505 ms | 12.46 s | **25x** |
| Format 7 projects (6,123 files, 31 MB) | 813 ms | 10.91 s | **13x** |

**Linting**, the `ktlint` binary against the ktlint 2.0.0-ALPHA-4 jar:

| Scenario | ktrs | ktlint 2.0 | Speedup |
|---|--:|--:|--:|
| Editor: one 8 KB file on stdin | <10 ms | 1.06 s | **>100x** |
| Lint one file | <10 ms | 845 ms | **>80x** |
| Lint okhttp (617 files) | 344 ms | 11.56 s | **34x** |
| Autocorrect okhttp (`-F`) | 640 ms | 123.0 s | **192x** |
| Lint okhttp, 1 core | 1.37 s · 49 MB | 39.36 s · 366 MB | **29x** |
| Lint 7 projects (6,123 files) | 1.72 s · 268 MB | 49.73 s · 553 MB | **29x** |
| Autocorrect 7 projects (`-F`) | 4.00 s | 338.9 s | **85x** |

The ktlint 1.8.0 jar is 10-30% faster than 2.0 on these runs. ktrs's 1.8 mode visits the tree once
per rule, as 1.8 does, and is up to 2x slower than its 2.0 mode.

JVM startup dominates small runs. On large runs ktrs is still several times faster per core, and it
uses every core. The binary is a few MB with no runtime, compared with a 71 MB jar plus a JRE.

<details>
<summary>Methodology and Spotless numbers</summary>

- **Corpus.** The 7 projects are okhttp, kotlinx.coroutines, nowinandroid, ktlint, ktfmt, Exposed
  and ktor, pinned in [`tools/bench/REVISIONS`](tools/bench/REVISIONS).
- **1 core.** Both processes are pinned to one CPU from launch. The JVM then sizes its GC and JIT
  threads for one CPU, as it would in a 1-CPU container, and its JIT competes with the work.
- **Identical output.** ktfmt rejects 2 of the 6,123 files, Exposed's `{{packageName}}`
  code-generator templates, and ktrs rejects them with the same error.
- **Reproduce.** `tools/bench/public.sh` (Linux or macOS; needs hyperfine, Java and a
  `cargo build --profile dist`) fetches the corpus, downloads the jars and checks their SHA-256, and
  runs every scenario above; `--only quick` runs a subset. The `bench` workflow runs it on a GitHub
  runner for each release tag or on demand. Method, noise and the ktlint 1.8 figures are in
  [research/23](research/23-public-bench.md).

**Spotless.** `spotlessApply` on okhttp's 573 files gives identical output. The figures are the
formatter's share, after subtracting a Spotless run that only trims whitespace:

| Spotless step | ktrs (`KtrsStep`) | ktfmt 0.64 (`ktfmt()`) |
|---|--:|--:|
| Fresh Gradle daemon, as in CI | ~4 s | ~52 s |
| Warm daemon, repeated runs | ~2 s | ~6 s |

</details>

## How correctness is checked

Parity with the original tools is the spec. The ported test suites run in CI, and the corpus diffs
(`cargo corpus-diff`, `cargo fmt-diff`, `cargo lint-diff`) compare against the real tools on ~6,000
files:

- **Parser.** The tree must match the Kotlin compiler's PSI (`DebugUtil.psiToString`) on the
  compiler's own test fixtures and on every corpus file.
- **Formatter.** ktfmt's test suite is ported. The output is also diffed byte for byte against the
  ktfmt jar on the corpus in the meta, google and kotlinlang styles.
- **Linter.** ktlint's rule tests are ported. Violations and `--format` output are diffed against
  the ktlint jar on the corpus in the `ktlint_official`, `intellij_idea` and `android_studio` code
  styles, with and without experimental rules.
- **CLIs.** The `ktfmt` and `ktlint` binaries are compared with the jars on stdout, stderr, exit
  code and written files across a scenario suite.
- **Held-out corpus.** To check that the corpus work didn't overfit, the CLIs also run against a
  second corpus that never drove a fix. It has 15,287 files from 20 other projects, pinned in
  [`tools/holdout/REVISIONS`](tools/holdout/REVISIONS). In the first run:
  - All 2.2M lint violations matched ktlint in all three styles.
  - ktfmt output matched on every file in all three styles.
  - `--format` output differed on 3 files, from one rule. That bug is now fixed.

  Details are in [research/21](research/21-holdout.md); `tools/holdout/run.sh` reruns it. The
  ktlint 1.8 mode and compose-rules are checked against their jars on both corpora too.
- **Fuzzing.** `fuzz/` has cargo-fuzz targets for the parser, ktfmt and ktlint, and
  `tools/fuzz/diff.sh` runs mutated real-world files through the binaries and the jars and reports
  any difference. The first runs found 4 bugs, now fixed, including exponential parser memory on
  deeply nested generic-looking input ([research/24](research/24-fuzzing.md)).
- **Upstream releases.** A weekly workflow opens an issue when ktfmt, ktlint, compose-rules or
  Kotlin publishes a release newer than the pinned version.

## Maintenance

ktrs is maintained by [@Hexay](https://github.com/Hexay). It tracks ktfmt 0.64, ktlint
2.0.0-ALPHA-4 (plus 1.8.0 in compatibility mode), compose-rules 0.6.7 and the Kotlin 2.4.20 parser. New upstream releases are ported and re-checked against
the parity gates before a ktrs release. If ktrs output ever differs from ktfmt or ktlint on your
code, that's a bug: please [open an issue](https://github.com/Hexay/ktrs/issues) with the file, or
a snippet that reproduces it, and the command line you used.

## Contributing

```sh
tools/sync-kotlin.sh                  # pinned upstream sources + vendored test fixtures
tools/psi-dump/psi-dump.sh one X.kt   # reference PSI tree from the real compiler (needs a JDK)
cargo xtask codegen                   # regenerate SyntaxKind from crates/ktrs-syntax/kinds.tsv
cargo test
```

[`CLAUDE.md`](CLAUDE.md) lists the crate layout and every parity gate. Design notes are in
[`research/`](research/).

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. ktrs
contains code and test data ported from the Kotlin compiler, the IntelliJ Platform, ktfmt,
google-java-format, ec4j (Apache-2.0), ktlint and ktfmt-gradle (MIT). See [NOTICE](NOTICE).
