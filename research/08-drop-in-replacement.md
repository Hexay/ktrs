# 08 — Drop-in replacement gap analysis (ktfmt first, then ktlint)

Researched 2026-09-27. Local facts come from `third_party/ktfmt` (v0.64) and `crates/ktrs-fmt`. Web facts have
inline URLs. Scope, LOC and popularity figures are in `research/04-parity-scope.md` and are not repeated here.

## TL;DR

- The library is done. `ktrs_fmt::format(&FormattingOptions, &str)` supports **every** field in Kotlin's
  `FormattingOptions` and matches 6121/6123 corpus files. What is missing is the **~500 LOC shell around it**: the
  CLI, ec4j-compatible `.editorconfig` handling, and packaging.
- The ktfmt CLI is small. `Main.kt` (193 LOC), `ParsedArgs.kt` (184) and `EditorConfigResolver.kt` (136) have
  1,202 LOC of tests (`MainTest` 564, `ParsedArgsTest` 276, `EditorConfigResolverTest` 362). The tests are
  in-process (`Main(input, out, err, args).run()`), so we can port them almost verbatim as a Rust integration
  harness.
- **Every major integration runs ktfmt in-process on the JVM**: Spotless, ktfmt-gradle and the IntelliJ plugin. A
  binary on PATH only helps CLI, pre-commit, editor and CI users unless we land adapters for those integrations.
- **Our lead is shrinking.** ktfmt main has merged GraalVM native-image support (#584, #617, #663) but has not
  released it; it is expected in 0.65. ktlint 2.0.0-ALPHA-4 already ships native binaries. Our pitch therefore
  becomes throughput plus a single hermetic binary plus a Wasm or library embed, **not** "no JVM" alone.

## 1. ktfmt CLI surface (v0.64, vendored)

### Flags (`ParsedArgs.parseOptions`)

| Flag | Effect | Notes for the port |
|---|---|---|
| `-h`, `--help` | Prints `HELP_TEXT` to **stdout** and exits 0 | Checked before any other parsing (`"--help" in args`) and wins over invalid flags. |
| `-v`, `--version` | Prints `ktfmt version 0.64` to stdout and exits 0 | Same precedence as `--help`, checked after it. Print the ktfmt version we match, so version-sniffing wrappers still work. |
| `--meta-style` / `--google-style` / `--kotlinlang-style` | Picks a preset | The last one wins. The default is META. |
| `-n`, `--dry-run` | No writes; prints the path of each file that would change to **stdout** | For stdin it prints the `--stdin-name` value or `<stdin>`. |
| `--set-exit-if-changed` | Exits 1 if any input changed | |
| `--do-not-remove-unused-imports` | Sets `removeUnusedImports = false` on the chosen preset | Applied after the style is picked, so flag order does not matter. |
| `--enable-editorconfig` | Turns on per-file `.editorconfig` overrides | Off by default. Never applied to stdin in 0.64 (see the next table). |
| `--quiet` | Suppresses the `Done formatting X` lines | Errors are still printed. |
| `--stdin-name=<v>` | Name used in stdin messages | An empty value is allowed. `--stdin-name` without `=` → `Found option '...', expected '--stdin-name=<value>'`. Using it without `-` → an error. |
| `-` | Reads stdin, writes stdout | `-` plus any file → `Cannot read from stdin and files in same run. ...` |
| `@ARGFILE` | Reads one argument per line | Only when it is the **sole** argument. Any other `@x` → `Unexpected option: @x`. |
| any other `--x` | `Unexpected option: --x` on stderr, exit 1 | |

Things 0.64 does **not** have:
- No `--max-width`, `--block-indent` or `--continuation-indent` flags. Max width only comes from the API or
  `.editorconfig`.
- No `--lines`/`--offset`. Those are in CHANGELOG `[Unreleased]` along with the package move to
  `org.jetbrains.ktfmt` (https://github.com/Kotlin/ktfmt/blob/main/CHANGELOG.md).
- Unreleased: `--enable-editorconfig --stdin-name=$PWD/src/Foo.kt -` resolves `.editorconfig` for stdin.

### Behaviour (`Main.run`/`format`)

| Behaviour | Detail |
|---|---|
| No file arguments | Prints `USAGE` to stderr and exits 1. |
| File expansion | A single argument that is a regular file is used as-is, **whatever its extension**. Otherwise each argument is walked top-down (`File.walkTopDown`) and filtered to `.kt`/`.kts`. **Gotcha:** with 2+ arguments, explicit non-`.kt` files are silently dropped. There is no `.gitignore` handling and hidden or `build/` directories are included. Nonexistent paths yield nothing. |
| Nothing matched | Prints `Error: no .kt files found` to stderr and exits 1. |
| Parallelism | `files.parallelStream()` (ForkJoin, one thread per core). Stderr line order is nondeterministic, so byte-exact stderr ordering is not a requirement. |
| I/O | Reads UTF-8, strips a leading BOM, writes UTF-8. Writes **only if the content changed** (preserves mtime). Line separators follow the input (`guessLineSeparator`); ktrs already ports this. |
| Per-file messages (stderr) | `Done formatting <path>` (unless `--quiet`).<br>IO error: `Error formatting <path>: <msg>; skipping.`<br>ParseError: `<path>:<line>:<col>: error: <desc>`.<br>FormattingError: `<path>:<diagnostic>` per diagnostic plus a JVM stack trace (we can omit the trace). |
| Error isolation | One bad file does not stop the others (`all files in args are processed, even if one of them has an error`). Done: the CLI `catch_unwind`s per file and the release profile unwinds. |
| Exit codes | Only **0** and **1**. 1 means an argument error, any per-file exception, or a change with `--set-exit-if-changed`. |
| stdin | Always writes the formatted code to stdout, changed or not. A parse error on stdin goes to stderr and exits 1. |
| `.kt` vs `.kts` | Irrelevant to formatting: `Parser.kt` always parses as `temp.kts`. So a path-less stdin pipe loses nothing except `.editorconfig`. |

### `.editorconfig` (ec4j, only with `--enable-editorconfig`)

- **Resolution:** ec4j `ResourcePropertiesService` walks from the file's absolute path up to `root = true` or the
  filesystem root. Sections are matched with EditorConfig globs (`*`, `**`, `?`, `[...]`, `{a,b}`, `{1..3}`); the
  nearer file and the later section win. Parsed files are cached in a `ConcurrentHashMap`.
- **Keys:**
  - `max_line_length` → maxWidth. `off` keeps the base value.
  - `ij_kotlin_indent_size`, then `indent_size`, then `tab_width` → blockIndent. `indent_size = tab` falls through
    to `tab_width`.
  - `ij_kotlin_continuation_indent_size`, then `ij_continuation_indent_size` → continuationIndent.
  - `ktfmt_trailing_comma_management_strategy = none|only_add|complete`. Invalid values are ignored.
- **Rust:** hand-port the glob matcher and the cascade to about 250 LOC. The `ec4j` spec test suite
  (editorconfig-core-test) is the oracle; `EditorConfigResolverTest` has 15 cases. Crates `ec4rs` or `editorconfig`
  may also do; check that their glob semantics match ec4j before adopting one.

### Options coverage

| Kotlin `FormattingOptions` field | ktrs (`formatting_options.rs`) |
|---|---|
| maxWidth, blockIndent, continuationIndent | yes |
| trailingCommaManagementStrategy (NONE/ONLY_ADD/COMPLETE) | yes (`value_of`, `Display`) |
| removeUnusedImports | yes |
| preserveLambdaBreaks | yes |
| debuggingPrintOpsAfterFormatting | field present, wired in `formatter.rs` |
| Builder / toBuilder / deprecated constructors | yes |

**Conclusion:** a 1:1 `ktfmt` binary is about 500 Rust LOC (args 150, main 150, editorconfig 200) plus a ported
test suite. Follow CLAUDE.md's "one fn per Java method" rule: `parse_options`, `process_args`,
`expand_args_to_file_names`, `run`, `format`, `resolve_formatting_options`.

## 2. How people run ktfmt, and what each path needs from us

| Path | How it runs today | What a native `ktrs` needs | Effort |
|---|---|---|---|
| **Spotless Gradle/Maven** (the main path) | `KtfmtStep` loads `com.facebook:ktfmt:<v>` into a `JarState` classloader in-process. Default version 0.64 (https://github.com/diffplug/spotless/blob/main/gradle/libs.versions.toml). Config: `ktfmt("0.64").googleStyle().configure { setMaxWidth/…/setTrailingCommaManagementStrategy }`. | (a) **Works today, no changes:** the generic `nativeCmd('ktrs', '/path', ['--google-style','-'])` step (https://github.com/diffplug/spotless/blob/main/lib/src/main/java/com/diffplug/spotless/generic/NativeCmdStep.java). Limits: one process per file, no path passed (so no `.editorconfig`), manual install. (b) **Upstream PR** `ktfmt().pathToExe(...)` or a new `ktrs()` step modelled on `biome().pathToExe`/`clangFormat().pathToExe` (plugin-gradle README), with auto-download. | a: S (docs) · b: M |
| ktfmt-gradle (cortinico 0.27.0) | Worker API, forked JVM per build, one work item per file (https://github.com/cortinico/ktfmt-gradle) | Upstream an "executable" mode, or ship our own small Gradle plugin with `ktrsCheck`/`ktrsFormat`. A batch mode (many files per process) matters here. | M |
| IntelliJ ktfmt plugin | In-process `Formatter.format` (`ktfmt_idea_plugin/.../KtfmtFormattingService.kt`) | A fork or new plugin that shells out to `ktrs -` or a daemon. Low value: in-IDE JVM latency is already warm. | M, defer |
| pre-commit | `language-formatters-pre-commit-hooks` `pretty-format-kotlin`: Python downloads the ktfmt jar and requires Java (https://github.com/macisamuele/language-formatters-pre-commit-hooks) | A `.pre-commit-hooks.yaml` in our repo using `language: rust` (cargo builds on first run), or a pip wheel with a bundled binary (`language: python`, no JVM). This is the easiest visible win. | S |
| CI / scripts | `java -jar ktfmt-*-with-dependencies.jar --set-exit-if-changed -n src` | CLI parity plus release binaries plus a GitHub Action (`setup-ktrs`). | S |
| Neovim conform.nvim | `ktfmt -` (https://github.com/stevearc/conform.nvim) | stdin parity. Users can override the command; an upstream `ktrs` entry is a one-file PR. | S |
| VS Code | Third-party extensions shell out to the ktfmt jar (e.g. https://marketplace.visualstudio.com/items?itemName=omBratteng.ktfmt-kotlin-formatter) | Configurable binary path, or our own extension (stdin/stdout). | S–M |
| Emacs apheleia, Helix, Zed | No ktfmt default. apheleia defaults to ktlint (https://github.com/radian-software/apheleia/blob/main/apheleia-formatters.el) | Upstream formatter entries; stdin parity is enough. | S |
| dprint | No Kotlin plugin (https://dprint.dev/plugins/) | `dprint-plugin-exec` works now. A **Wasm plugin** is a unique channel no JVM tool can match. | M |
| kempt (ZacSweers) | Rust pre-commit pipeline that shells out to the ktfmt jar and needs JDK 17 (https://github.com/ZacSweers/kempt) | Could link `ktrs_fmt` as a crate. This natural early adopter wants a crates.io release. | S |

Editor and daemon note: stdin/stdout parity (`-`, `--stdin-name`) covers conform, apheleia, VS Code and
nativeCmd. The unreleased `--stdin-name` editorconfig lookup is worth adopting early.

## 3. Distribution

| Channel | What it takes | Effort |
|---|---|---|
| `cargo install ktrs` / crates.io | Publish `ktrs_syntax`, `lexer`, `parser`, `psi`, `fmt` and a `ktrs` bin crate. Name check: `ktfmt-rs` is taken (an abandoned 2026-03 upload, https://crates.io/crates/ktfmt-rs). | S |
| GitHub release binaries | `cargo-dist` (or a hand-rolled matrix) for linux x64/arm64 (musl, static), macOS x64/arm64, windows x64/arm64, with checksums, a shell/PowerShell installer and `ktrs-<target>.tar.gz`. `lto = fat` is already set. | S |
| Homebrew | Tap first (`cargo-dist` generates it). homebrew-core later: it needs notability, and the existing `ktfmt` formula is a JVM wrapper on `openjdk@17` (https://github.com/Homebrew/homebrew-core/blob/main/Formula/k/ktfmt.rb). | S |
| npm / pip wrappers | Per-platform optional-dependency packages (the biome/ruff pattern). pip unlocks `language: python` pre-commit with no JVM. **npm done:** `@ktrs/cli` + six `@ktrs/cli-<os>-<cpu>` (`npm/`, release job `npm`). | S–M |
| Maven Central artifact | A jar embedding the per-OS binaries plus a tiny Java shim (`ProcessBuilder`, the pattern of biome and esbuild-java). This is what a Spotless first-class step or a Gradle plugin would resolve. JNI is possible but not worth it: per-file processes are already cheap natively. | M |
| GitHub Action / Docker image | `uses: ktrs/setup-ktrs@v1`; a `FROM scratch` image with the static binary. **Done:** `action.yml` (`uses: Hexay/ktrs@<tag>`); `ghcr.io/hexay/ktrs`, distroless static, amd64 + arm64 (`docker/Dockerfile`, release job `docker`). | S |
| Wasm (`wasm32-wasip1`) | For dprint, the web playground and a VS Code extension without per-platform binaries. | M |

**Competition and value proposition**

- **ktfmt:** v0.64 (2026-06-24) ships only JARs (https://github.com/Kotlin/ktfmt/releases). Native image work:
  - #584 merged through Meta's import bot about 2026-06-30 (https://github.com/Kotlin/ktfmt/pull/584).
  - #617 adds per-OS release jobs; #663 adds native CI (merged 2026-08-10); #693 "set up publishing" closed on
    2026-09-15.
  - Expect native binaries in **0.65**. Binaries are about 13–16 MB. The benchmark claims "up to 100x faster" on
    small inputs, with the JIT catching up at thousands of files.
- **ktlint:** 2.0.0-ALPHA-4 ships native binaries for linux-x64, darwin-arm64 and windows-x64
  (https://github.com/ktlint/ktlint/releases). Native mode cannot load custom rulesets.
- **Our numbers:**
  - Single thread: 5.3 MB/s.
  - 12 threads: 19.8 MB/s, so the 30.8 MB corpus takes about **1.6 s against 16.1 s** for the ktfmt 0.64 JVM
    (1.9 MB/s including startup), roughly 10x.
  - Native-image ktfmt has no JIT and uses Serial GC, so it is likely *slower* than the JVM on large trees.
    **Measure this once 0.65 ships.**
- **Positioning:**
  - (1) Fastest on whole-repo CI and monorepos, where ~10x holds against both JVM and native builds.
  - (2) A small static binary with no GraalVM quirks.
  - (3) Embeddable as a Rust crate or Wasm: kempt, dprint, a playground, an LSP.
  - (4) One binary for ktfmt and, later, ktlint.
  - Startup alone is no longer a moat after 0.65.
- **Version-pinning story:** users pin `ktfmt("0.6x")`. Ship "ktrs X.Y (matches ktfmt 0.64)" and chase each release
  (see 04 §1, output churn).

## 4. ktlint: surface users depend on

See 04 §2 for LOC and rule counts: 104 rule files, 87 autocorrect, 8 experimental. The docs on master list 99 rule
ids plus 7 experimental (https://github.com/ktlint/ktlint/blob/master/documentation/release-latest/docs/rules/standard.md).

**CLI** (https://github.com/ktlint/ktlint/blob/master/documentation/release-latest/docs/install/cli.md):
- Patterns: gitignore-style globs with `!` negation. The default is `**/*.kt{,s}` excluding `build/`.
- Flags:
  - `-F/--format`, `--stdin`, `--stdin-path`, `--patterns-from-stdin[=delim]`
  - `--reporter=plain|plain?group_by_file|plain-summary|json|sarif|checkstyle|html|baseline[,output=f]` (repeatable)
  - `--baseline=<xml>`, `--relative`, `--editorconfig=<path>`, `--limit`, `--color[-name]`, `-l/--log-level`
  - `-R/--ruleset=<jar>`
  - Subcommands `generateEditorConfig`, `installGitPreCommitHook` and `installGitPrePushHook`.
  - `--disabled_rules` is gone.
- Exit codes:
  - 0: ok. 1: violations remain. 2: IO error. 3: stdin is not valid Kotlin.
  - 4: stdin exception. 5: invalid path. 6: unsupported ruleset jar. 7: bad reporter config.

**Config** is `.editorconfig` only:
- `ktlint_code_style = ktlint_official` (default) `| intellij_idea | android_studio`.
- `ktlint_standard_<rule> = disabled`, `ktlint_standard = disabled` then per-rule `enabled`, and
  `ktlint_experimental = enabled`.
- About 10 rule-specific `ktlint_*` properties:
  - `ktlint_function_signature_rule_force_multiline_when_parameter_count_greater_or_equal_than`
  - `ktlint_chain_method_rule_force_multiline_when_chain_operator_count_greater_or_equal_than`
  - `ktlint_function_naming_ignore_when_annotated_with`, …
- Plus `ij_kotlin_imports_layout`, `ij_kotlin_allow_trailing_comma[_on_call_site]`, `ij_kotlin_packages_to_use_import_on_demand`,
  `indent_size/style`, `max_line_length`, `insert_final_newline` and `end_of_line`
  (https://github.com/ktlint/ktlint/blob/master/documentation/release-latest/docs/rules/configuration-ktlint.md).

**Suppression:**
- `@Suppress("ktlint:standard:<id>")`, `@Suppress("ktlint")` and `@file:Suppress(...)`.
- Also `// ktlint-suppress`-style handling inside the engine (see 04).
- `ktlint-disable` comments were **removed in 0.50.0** (2023-06). We do not need them; at most, emit a migration
  hint.

**Integrations:**
- All in-process today: Spotless `ktlint("1.8.0").editorConfigOverride(...).customRuleSets(...)` (Spotless defaults
  to `intellij_idea`!), ktlint-gradle, kotlinter, the IntelliJ Ktlint plugin and Detekt `formatting`.
- Editors use `ktlint --stdin -F` (apheleia, conform) and `--reporter=json --stdin --relative` for diagnostics
  (none-ls).

**Custom rulesets:**
- compose-rules v0.6.7 is used as `-R ktlint-compose-<v>-all.jar` and must be built against the matching ktlint
  version (https://github.com/mrmans0n/compose-rules/blob/main/docs/ktlint.md).
- We cannot load JARs, and neither can ktlint-native. So either port compose-rules' ktlint checks natively (about
  5k LOC, see 04 §7), or fall back to the JVM for projects that pass `-R`.

**Minimum viable ktlint replacement:**
- (1) All standard non-experimental rules for all 3 code styles, with byte-exact `--format` output and exact
  violation messages and positions.
- (2) Config: `.editorconfig` including `ktlint_*` enable/disable and the rule properties.
- (3) Suppression: `@Suppress`.
- (4) Reporters: plain, json, checkstyle, sarif and baseline read/write (these cover CI and code-scanning UIs).
- (5) Exit codes 0/1/2/3.
- (6) stdin plus `--stdin-path`.
- (7) Pick one version target: **2.0** (native competitor, per-node traversal) or **1.8.0** (what Spotless pins
  today). Recommendation: target 1.8.x output because it is installed everywhere, and add 2.0 when it goes stable.
- Out of scope for the MVP: html and plain-summary reporters, JAR rulesets, experimental rules.

## 5. Prioritized plan

### Phase A: a drop-in `ktfmt` binary for CLI, CI, pre-commit and editor users

| # | Step | Effort |
|---|---|---|
| A1 | `crates/ktrs-cli`, a 1:1 port of `ParsedArgs`/`Main`: flags, `@argfile`, stdin, messages, exit codes 0/1, change-only writes, BOM, rayon parallelism, per-file `catch_unwind` (the build profile keeps panics from aborting the batch). Binary name `ktrs`, with a `ktfmt`-compatible argument surface. | S |
| A2 | Port `MainTest`/`ParsedArgsTest` (≈50 cases) as Rust integration tests, plus a differential test running the JVM `ktfmt-0.64-with-dependencies.jar` against `ktrs` on the corpus with `-n --set-exit-if-changed` (stdout and exit code identical). | S |
| A3 | `.editorconfig` resolver (ec4j glob plus cascade, 4 keys + `--enable-editorconfig`), with `EditorConfigResolverTest` ported. Also honour `--stdin-name` for the lookup (ktfmt Unreleased). | M (glob exactness) |
| A4 | Close the last 2/6123 corpus diffs, or document them. | S–M |
| A5 | Release pipeline: `cargo-dist` for 6 targets, checksums, installers, Homebrew tap, crates.io, GitHub Action. | S |
| A6 | `.pre-commit-hooks.yaml` (`ktrs-format`, `ktrs-check`) plus a pip wheel so there is no Rust/JVM prerequisite. | S |
| A7 | Docs: "switching from ktfmt". Spotless `nativeCmd` recipe, conform/apheleia/VS Code snippets, the version-mapping table, and a README benchmark against JVM and (once released) native ktfmt 0.65. | S |

### Phase B: build-tool reach, where most ktfmt users are

| # | Step | Effort |
|---|---|---|
| B1 | Maven Central `ktrs-native` jar (bundled binaries plus a `ProcessBuilder` shim with a batch/stdin API). | M |
| B2 | Upstream Spotless PR: `ktfmt().pathToExe(...)` or a `ktrs()` step with auto-download from B1. Pass the file path so `.editorconfig` works. | M |
| B3 | A Gradle plugin, or an upstream executable mode for ktfmt-gradle, using one batch process per task rather than per file. | M |
| B4 | Track ktfmt releases. 0.65 (released 2026-10-07, `org.jetbrains.kotlinx:ktfmt`): ported in ktrs-fmt and the `ktfmt` binary, incl. `--lines`/`--offset`/`--length`, `.kt` vs `.kts` parsing, single-item trailing commas; gates `range-diff.sh`, `cli-diff.sh`. Not ported: `--experimental-engine` (rejected). Open: the Spotless/ktfmt-gradle/Maven drop-ins, `ktrs migrate` and the README still name 0.64, which those plugins bundle. Sections above describe the 0.64 CLI. | M, recurring |
| B5 | Optional: Wasm build → dprint plugin, playground, VS Code extension. | M |

### Phase C: ktlint (after A and B; see 04 §7 for the ~33–37k LOC estimate)

| # | Step | Effort |
|---|---|---|
| C1 | Mutable CST plus the rule engine (traversal order, `RunAfterRule`, `@Suppress`, repeat-format loop) and `.editorconfig` `ktlint_*` properties, reusing the A3 resolver. | L |
| C2 | Standard rules, largest-usage first. Autocorrect rules for `ktlint_official` and `intellij_idea` (Spotless default) before `android_studio`, with a differential corpus gate against the ktlint 1.8.0 JVM. | L |
| C3 | ktlint-compatible CLI: patterns, `-F`, stdin, reporters (plain/json/checkstyle/sarif/baseline), exit codes. | M |
| C4 | Port compose-rules' ktlint checks as a built-in optional ruleset, the main custom-ruleset lock-in. | M–L |
| C5 | Adapters: the Spotless `ktlint` path via B1/B2 mechanics; kotlinter/ktlint-gradle executable modes; LSP diagnostics. | M |

**Smallest set that makes users switch:** A1 + A2 + A5 + A7 (days, not weeks) gets CLI, CI and pre-commit users.
A3 is needed by anyone with `--enable-editorconfig`. B1 + B2 unlock the Spotless majority. Everything in Phase C
waits until ktfmt adoption shows pull.
