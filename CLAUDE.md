# ktrs

Rust port of Kotlin tooling: parser first, then ktfmt (byte-exact), then ktlint rules.
Milestone 1: the parser's tree must be identical to the Kotlin compiler's PSI (`DebugUtil.psiToString`).

## Layout

- `crates/ktrs_syntax` — `SyntaxKind` (generated), rowan tree types, `psi_dump` printer.
- `crates/ktrs_lexer` — ports of `Kotlin.flex` / `KDoc.flex`.
- `crates/ktrs_parser` — `PsiBuilder` semantics + port of `KotlinParsing`/`KotlinExpressionParsing`/`KDocParser`.
  Throughput: `cargo run -p ktrs_parser --release --example bench [dir] [reps]` (CPU-cycle based, robust
  to a busy machine; `corpus-diff`'s MB/s sums wall time across all cores and swings with load).
- `xtask` — `cargo xtask codegen` regenerates `ktrs_syntax/src/generated/kinds.rs` from `kinds.tsv`.
- `tools/psi-dump/psi-dump.sh` — JVM oracle on the pinned compiler: `one <file>`, `tree <in> <out>`, `kinds`,
  `bench <dir> <warmup> <reps>` (warm single-thread baseline to compare with the ktrs `bench` example).
- `tools/sync-kotlin.sh` — sparse-checks-out the pinned Kotlin sources to `third_party/kotlin` (gitignored)
  and vendors parser fixtures into `testdata/kotlin/`.
- `testdata/kotlin/{psi,lexer}` — upstream fixtures (Apache-2.0), `<name>.kt` + expected `<name>.txt`.
  Input convention: CRLF->LF and trailing newlines stripped (matches upstream's test framework).
- `tools/fetch-corpus.sh` — real-world repos into `corpus/` (gitignored, commits in `corpus/REVISIONS`).

## Parity gates (both must stay green)

- `cargo test -p ktrs_parser --release` — fixture ratchet `tests/passing.txt` (`UPDATE_PASSING=1` rewrites).
- `cargo corpus-diff` (run from repo root) — our dump vs the compiler's for every corpus file; also
  prints MB/s and the slowest files. Oracle dumps are built once, in the background:
  `tools/psi-dump/psi-dump.sh tree corpus target/oracle/corpus`.

## Rules

- **Kotlin pin is `v2.4.20` everywhere** (psi-dump.sh, sync-kotlin.sh). Bump them together, then rerun
  `psi-dump.sh kinds > crates/ktrs_syntax/kinds.tsv` and `cargo xtask codegen`.
- Never hand-edit `generated/`. Kind names = compiler field names (`KtTokens.FUN_KEYWORD` -> `FUN_KEYWORD`).
- Port 1:1: one Rust fn per Java method, `snake_case` of the Java name, same order within the file, so
  upstream diffs map onto our code. Keep upstream control flow even where it looks odd.
- Parity is the spec. When our dump differs from the fixture/oracle, the oracle is right.
- Input CRLF is normalized to LF before lexing (same as IntelliJ, ktfmt, ktlint).
- Run `psi-dump.sh`, gradle-like JVM steps, and full corpus diffs in the background (they exceed 60s).
