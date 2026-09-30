# ktrs

Fast, native Kotlin tooling in Rust. The goal is ktfmt-identical formatting and ktlint-compatible
linting without starting a JVM.

**Status:** the formatter is done: output identical to ktfmt 0.64 on 6,121 of 6,123 real-world files
(the other two are rejected by both), 7-100x faster than the ktfmt jar end to end ([numbers](#performance)).
Underneath is a
lossless Kotlin parser whose tree matches the Kotlin compiler's PSI node for node. The linter is a
port of ktlint 2.0 with identical output on the same files ([details](#linting)).

## Formatting

Output is byte-identical to ktfmt 0.64. Try it without installing anything in the
[playground](https://hexay.github.io/ktrs/) (the formatter compiled to WebAssembly, running in your
browser). One install gives two binaries, `ktrs` and `ktfmt`:

```sh
curl -fsSL https://raw.githubusercontent.com/Hexay/ktrs/master/install.sh | sh   # prebuilt
cargo install ktrs                                                                # from source (crates.io)
brew tap hexay/ktrs https://github.com/Hexay/ktrs && brew install hexay/ktrs/ktrs  # Homebrew
```

The script also works on Windows under Git Bash; otherwise unzip a release from the releases page.
In GitHub Actions:

```yaml
- uses: Hexay/ktrs@v0.2.0          # installs ktrs and ktfmt on PATH (Linux, macOS, Windows)
- run: ktrs fmt --check --style kotlinlang
```

As a [pre-commit](https://pre-commit.com) hook (no Rust needed: the hook downloads the release
binaries for its `rev` on first run):

```yaml
- repo: https://github.com/Hexay/ktrs
  rev: v0.2.0
  hooks:
    - id: ktrs-fmt          # or `ktrs-fmt-check`, or `ktfmt` with ktfmt's flags in `args`
      args: [--style, kotlinlang]
```

Usage:

```sh

ktrs fmt                              # format every .kt/.kts under the current directory (meta style)
ktrs fmt --style kotlinlang src/      # styles: meta (default), google, kotlinlang
ktrs fmt --check                      # CI: list files that would change, exit 1 if any
ktrs fmt - < Foo.kt                   # stdin to stdout, for editors

ktfmt --kotlinlang-style --set-exit-if-changed src/   # drop-in: ktfmt's own flags and messages
```

The `ktfmt` binary accepts ktfmt's CLI exactly (flags, `@argfile`, `-` for stdin, exit codes,
`--enable-editorconfig`), so anything that runs the ktfmt jar can run it instead.

### Editors

Editors format the buffer through stdin; `--stdin-name` passes its path so `.editorconfig` applies
(add `--style google` or `--style kotlinlang` as needed). Formatting a file takes ~15 ms.

- **Neovim** ([conform.nvim](https://github.com/stevearc/conform.nvim)):
  ```lua
  formatters_by_ft = { kotlin = { "ktrs" } },
  formatters = { ktrs = { command = "ktrs", args = { "fmt", "--editorconfig", "--stdin-name", "$FILENAME", "-" } } },
  ```
- **Helix** (`languages.toml`):
  ```toml
  [[language]]
  name = "kotlin"
  formatter = { command = "ktrs", args = ["fmt", "-"] }
  auto-format = true
  ```
- **Zed** (`settings.json`):
  ```json
  "languages": { "Kotlin": { "formatter": { "external": {
    "command": "ktrs", "arguments": ["fmt", "--editorconfig", "--stdin-name", "{buffer_path}", "-"] } } } }
  ```
- **Emacs** ([apheleia](https://github.com/radian-software/apheleia)):
  ```elisp
  (push '(ktrs . ("ktrs" "fmt" "--editorconfig" "--stdin-name" filepath "-")) apheleia-formatters)
  (setf (alist-get 'kotlin-mode apheleia-mode-alist) 'ktrs)
  ```
- **VS Code** ([Custom Local Formatters](https://marketplace.visualstudio.com/items?itemName=jkillian.custom-local-formatters)):
  ```json
  "customLocalFormatters.formatters": [
    { "command": "ktrs fmt --editorconfig --stdin-name \"${file}\" -", "languages": ["kotlin"] } ]
  ```

### Gradle (Spotless) and the JVM

`io.github.hexay:ktrs` is a small jar with the native binaries for Linux, macOS and Windows (x86-64
and ARM) bundled in it, and no dependencies, served from this repository's Maven repo on GitHub Pages.
Its Spotless step (Spotless 7+) replaces `ktfmt()`:

```kotlin
// build.gradle.kts
buildscript {
    repositories { maven("https://hexay.github.io/ktrs/maven") }
    dependencies { classpath("io.github.hexay:ktrs:0.2.0") }
}

spotless {
    kotlin {
        addStep(io.github.hexay.ktrs.spotless.KtrsStep.create(io.github.hexay.ktrs.KtrsOptions.kotlinlang()))
    }
}
```

`KtrsOptions` mirrors ktfmt's options (`meta()`, `google()`, `kotlinlang()`, then `withMaxWidth`,
`withBlockIndent`, `withContinuationIndent`, `withRemoveUnusedImports`, `withTrailingCommas`, and
`withEditorConfig(true)` to honour `.editorconfig`). From other JVM code, `Ktrs.create()` gives a
thread-safe formatter: `ktrs.format(code, KtrsOptions.google())`. Both keep long-lived `ktrs serve`
processes, so a build starts the binary once, not once per file.

Without the jar, Spotless's generic step runs the binary once per file:
`nativeCmd("ktfmt", "/path/to/ktfmt", listOf("--kotlinlang-style", "-"))`.

### Gradle plugin (drop-in for ktfmt-gradle)

`io.github.hexay.ktrs` replaces [cortinico's ktfmt-gradle](https://github.com/cortinico/ktfmt-gradle)
0.27.0. Swap the plugin id and keep the rest of the build as it is (the `ktfmt { }` block, the
`ktfmtCheck`/`ktfmtFormat*` tasks, `--include-only`, and `com.ncorti.ktfmt.gradle.*` imports):

```kotlin
plugins {
    // id("com.ncorti.ktfmt.gradle") version "0.27.0"
    id("io.github.hexay.ktrs") version "0.2.0"
}
```

Until it is on the Gradle Plugin Portal, add this repository in `settings.gradle.kts`:

```kotlin
pluginManagement {
    repositories {
        gradlePluginPortal()
        maven("https://hexay.github.io/ktrs/maven")
    }
}
```

(From the Portal, its `io.github.hexay:ktrs` dependency comes from Maven Central.) Formatting runs in
long-lived `ktrs` processes shared by the whole build. `useClassloaderIsolation`,
`processIsolationJvmArgs` and `ktfmtClasspath` are accepted and ignored, and
`debuggingPrintOpsAfterFormatting` only logs a warning. To use another binary than the bundled one,
set the Gradle property `ktrs.executable` to its path.

## Linting

A port of ktlint 2.0 (pinned at 2.0.0-ALPHA-4): its engine, all 105 standard rules, reporters and
CLI. On the same 6,123 files, violations and `--format` output are identical to ktlint's in all three
code styles (`ktlint_official`, `intellij_idea`, `android_studio`) and with experimental rules on,
down to the files where ktlint itself crashes. The same install adds a `ktlint` binary:

```sh
ktrs lint                             # check every .kt/.kts under the current directory
ktrs lint --format src/               # fix what can be autocorrected, report the rest
ktrs lint --reporter json - < Foo.kt  # stdin; reporters: plain, json, checkstyle, sarif, html, ...

ktlint --relative "src/**/*.kt" "!src/**/generated/**"   # drop-in: ktlint's own flags and messages
```

The `ktlint` binary accepts ktlint's CLI exactly (patterns with `!` negation, `-F`, `--stdin`,
`--patterns-from-stdin`, `--baseline`, `--editorconfig`, every built-in reporter, exit codes and
the git hook subcommands). JVM rule sets and reporters (`-R`, `artifact=`) can't be loaded. As
pre-commit hooks: `id: ktrs-lint` (or `ktrs-lint-format`, or `ktlint` with ktlint's flags in `args`).

## Performance

The `ktfmt` binary against the ktfmt 0.64 jar, both run from the command line the way users run them
(same flags, same files, identical output). Median of 5 alternating runs after a warm-up, on a Windows 11
laptop (Intel Core Ultra, 22 threads, JDK 21):

| Scenario | ktrs | ktfmt 0.64 (JVM) | Speedup |
|---|---|---|---|
| Seven open-source projects (6,123 files, 31 MB), format in place | 3.69 s | 24.55 s | **7x** |
| One project (okhttp, 617 files), format in place | 465 ms | 9.58 s | **21x** |
| okhttp, format in place, 1 core | 1.48 s | 40.79 s | **28x** |
| okhttp, CI check (`-n --set-exit-if-changed`) | 316 ms | 9.08 s | **29x** |
| Pre-commit: 10 changed files | 26 ms | 2.57 s | **99x** |
| Editor: one 8 KB file on stdin | 16 ms | 1.49 s | **96x** |

Small runs are dominated by JVM startup; large ones by formatting work, where ktrs is still several
times faster per core and uses all of them.

Through Spotless (`spotlessApply` on okhttp's 573 files, identical output), the formatter's share
after subtracting a Spotless run that only trims whitespace (54 s cold, 2.6-5 s warm on this machine):

| Spotless step | ktrs (`KtrsStep`) | ktfmt 0.64 (`ktfmt()`) |
|---|---|---|
| Fresh Gradle daemon, as in CI | ~4 s (58 s total) | ~52 s (106 s total) |
| Warm daemon, repeated runs | ~2 s (6.5 s total) | ~6 s (10 s total) |

Spotless formats one file at a time, and a warm JVM has already compiled ktfmt, so the gap is
widest in CI, where every build starts cold.

- **1 core**: both processes are restricted to one CPU from launch (the affinity mask is inherited, so
  the JVM also sizes its GC and JIT threads for one CPU, as in a 1-CPU container). The JVM's JIT
  compiler then competes with the formatting for that core, which is why it slows down more than ktrs.
- **Identical output** means byte-identical files on 6,121 of the 6,123. The other two are Exposed's
  code-generator templates (`package {{packageName}}`), which are not valid Kotlin: both tools reject
  them with the same error, `Package name must be a '.'-separated identifier list` at 1:8. The binary is 1.6 MB with no runtime; the jar is 71 MB plus
a JRE. Reproduce with `python3 tools/bench/e2e.py` (needs the corpus from `tools/fetch-corpus.sh` and
the jar, which `tools/ktfmt-oracle/extract-goldens.sh` downloads).

## Development

```sh
tools/sync-kotlin.sh                  # pinned upstream sources + vendored fixtures
tools/psi-dump/psi-dump.sh one X.kt   # reference PSI tree from the real compiler (needs a JDK)
cargo xtask codegen                   # regenerate SyntaxKind from crates/ktrs-syntax/kinds.tsv
cargo test
```

## License

Dual-licensed under MIT or Apache-2.0, at your option. Contains code ported from, and test data
copied from, the Kotlin compiler and IntelliJ Platform (Apache-2.0); see [NOTICE](NOTICE).
