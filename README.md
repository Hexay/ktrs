# ktrs

Fast, native Kotlin tooling in Rust. The goal is ktfmt-identical formatting and ktlint-compatible
linting without starting a JVM.

**Status: milestone 1, parser.** A lossless Kotlin parser whose tree matches the Kotlin compiler's
PSI node for node, verified against the compiler's own parser fixtures and against real-world code.

## Formatting

Output is byte-identical to ktfmt 0.64. Two binaries, from one build:

```sh
cargo install --path crates/ktrs_cli --profile dist   # installs `ktrs` and `ktfmt`

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
