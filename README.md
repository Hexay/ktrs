# ktrs

Fast, native Kotlin tooling in Rust. The goal is ktfmt-identical formatting and ktlint-compatible
linting without starting a JVM.

**Status:** the formatter is done: output identical to ktfmt 0.64 on 6,121 of 6,123 real-world files
(the other two are rejected by both), about 10x faster than the ktfmt jar end to end. Underneath is a
lossless Kotlin parser whose tree matches the Kotlin compiler's PSI node for node. Linting
(ktlint-compatible) is next.

## Formatting

Output is byte-identical to ktfmt 0.64. One install gives two binaries, `ktrs` and `ktfmt`:

```sh
curl -fsSL https://raw.githubusercontent.com/ktrs/ktrs/master/install.sh | sh   # prebuilt, Linux/macOS
cargo install --git https://github.com/ktrs/ktrs ktrs                           # from source
```

Windows: unzip a release from the GitHub releases page. As a [pre-commit](https://pre-commit.com) hook:

```yaml
- repo: https://github.com/ktrs/ktrs
  rev: v0.1.0
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
`--enable-editorconfig`), so anything that runs the ktfmt jar can run it instead, for example
Spotless's generic step:

```kotlin
spotless { kotlin { nativeCmd("ktfmt", "/path/to/ktfmt", listOf("--kotlinlang-style", "-")) } }
```

## Development

```sh
tools/sync-kotlin.sh                  # pinned upstream sources + vendored fixtures
tools/psi-dump/psi-dump.sh one X.kt   # reference PSI tree from the real compiler (needs a JDK)
cargo xtask codegen                   # regenerate SyntaxKind from crates/ktrs_syntax/kinds.tsv
cargo test
```

## License

Dual-licensed under MIT or Apache-2.0, at your option. Contains code ported from, and test data
copied from, the Kotlin compiler and IntelliJ Platform (Apache-2.0); see [NOTICE](NOTICE).
