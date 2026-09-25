# ktrs

Fast, native Kotlin tooling in Rust. The goal is ktfmt-identical formatting and ktlint-compatible
linting without starting a JVM.

**Status: milestone 1, parser.** A lossless Kotlin parser whose tree matches the Kotlin compiler's
PSI node for node, verified against the compiler's own parser fixtures and against real-world code.

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
