# ktrs

Fast, native Kotlin tooling in Rust. The goal is ktfmt-identical formatting and ktlint-compatible
linting without starting a JVM.

**Status:** the formatter is done: output identical to ktfmt 0.64 on 6,121 of 6,123 real-world files
(the other two are rejected by both), 7-100x faster than the ktfmt jar end to end ([numbers](#performance)).
Underneath is a
lossless Kotlin parser whose tree matches the Kotlin compiler's PSI node for node. Linting
(ktlint-compatible) is next.

## Formatting

Output is byte-identical to ktfmt 0.64. One install gives two binaries, `ktrs` and `ktfmt`:

```sh
curl -fsSL https://raw.githubusercontent.com/Hexay/ktrs/master/install.sh | sh   # prebuilt, Linux/macOS
cargo install ktrs                                                                # from source (crates.io)
```

Windows: unzip a release from the GitHub releases page. As a [pre-commit](https://pre-commit.com) hook:

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

### Gradle (Spotless) and the JVM

`io.github.hexay:ktrs` is a small jar with the native binaries for Linux, macOS and Windows (x86-64
and ARM) bundled in it, and no dependencies. Its Spotless step (Spotless 7+) replaces `ktfmt()`:

```kotlin
// build.gradle.kts
buildscript { dependencies { classpath("io.github.hexay:ktrs:0.2.0") } }

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
