# ktrs

Rust port of Kotlin tooling: parser first, then ktfmt (byte-exact), then ktlint rules.
Milestone 1: the parser's tree must be identical to the Kotlin compiler's PSI (`DebugUtil.psiToString`).

## Layout

- `crates/ktrs-syntax` — `SyntaxKind` (generated), the flat preorder `Tree` (element = index; see
  research/06-tree-library.md for why not rowan), `psi_dump` printer.
- `crates/ktrs-lexer` — ports of `Kotlin.flex` / `KDoc.flex`.
- `crates/ktrs-parser` — `PsiBuilder` semantics + port of `KotlinParsing`/`KotlinExpressionParsing`/`KDocParser`.
  Throughput: `cargo run -p ktrs-parser --release --example bench [dir] [reps]` (CPU-cycle based, robust
  to a busy machine; `corpus-diff`'s MB/s sums wall time across all cores and swings with load).
- `crates/ktrs-psi` — typed PSI views with the compiler's accessor semantics (scope: what ktfmt calls).
  Porting conventions and the full API list: `crates/ktrs-psi/src/lib.rs` docs.
- `crates/ktrs-fmt` — ktfmt port. Throughput: `cargo run -p ktrs-fmt --release --example bench [dir] [reps] [filter]
  [threads]`; compare runs by its "format = N parses" line (stable under machine load), not MB/s.
- `crates/ktrs-cli` — `ktrs fmt` and the `ktfmt` drop-in (1:1 port of ktfmt's `cli/`: flags, messages, exit
  codes). The root package `ktrs` owns the two binaries (so `cargo install --path .` and pre-commit work);
  releases build `--profile dist` (`.github/workflows/release.yml`).
  Adoption gaps and integrations: research/08-drop-in-replacement.md. `ktrs serve` is the build-tool
  server (protocol in `crates/ktrs-cli/src/serve.rs`).
- `crates/ktrs-wasm` + `site/` — the browser playground (plain Wasm exports, no bindgen); build and
  preview: `tools/release/build-site.sh && py -3 -m http.server -d target/site`.
- `java/` — `io.github.hexay:ktrs`: JVM wrapper around `ktrs serve` (bundled binaries, Spotless `KtrsStep`).
  Tests: `cargo build --bins`, then `java/gradlew -p java test` (JAVA_HOME = tools/jdk/*).
- `crates/ktrs-ast` — mutable arena AST with IntelliJ `TreeElement` semantics, seeded from `Tree` (for ktlint);
  conventions in `src/lib.rs`. `crates/ktrs-lint` — ktlint 2.0.0-ALPHA-4 engine + ported rules; status
  research/15-ktlint-spike.md. Upstream rule tests: `tools/ktlint-tests/extract-rule-tests.py` -> `testdata/ktlint/`.
- `tools/psi-accessors/psi-accessors.sh` — JVM oracle for ktrs-psi (`one|hashes|dump <dir> [--fixture] [--script]`);
  Rust mirror: `cargo run -p ktrs-psi --release --example psi_accessors -- one|hashes|compare|dump ...`.
- `xtask` — `cargo xtask codegen` regenerates `ktrs-syntax/src/generated/kinds.rs` from `kinds.tsv`.
- `tools/psi-dump/psi-dump.sh` — JVM oracle on the pinned compiler: `one <file>`, `tree <in> <out>`, `kinds`,
  `bench <dir> <warmup> <reps>` (warm single-thread baseline to compare with the ktrs `bench` example).
- `tools/sync-kotlin.sh` — sparse-checks-out the pinned Kotlin sources to `third_party/kotlin` (gitignored)
  and vendors parser fixtures into `testdata/kotlin/`.
- `testdata/kotlin/{psi,lexer}` — upstream fixtures (Apache-2.0), `<name>.kt` + expected `<name>.txt`.
  Input convention: CRLF->LF and trailing newlines stripped (matches upstream's test framework).
- `tools/fetch-corpus.sh` — real-world repos into `corpus/` (gitignored, commits in `corpus/REVISIONS`).

## Parity gates (all must stay green)

- `cargo test -p ktrs-parser --release` — fixture ratchet `tests/passing.txt` (`UPDATE_PASSING=1` rewrites).
- `cargo corpus-diff` (run from repo root) — our dump vs the compiler's for every corpus file; also
  prints MB/s and the slowest files. Oracle dumps are built once, in the background:
  `tools/psi-dump/psi-dump.sh tree corpus target/oracle/corpus`.
- `cargo test -p ktrs-psi --release` — PSI accessor reports vs the JVM on the fixtures (hashes in
  `crates/ktrs-psi/tests/data`; regeneration commands in `tests/fixtures.rs`). Corpus, in the background:
  `psi-accessors.sh hashes corpus [--script] > target/psi-accessors/corpus[-script].jvm.hashes`, then
  `psi_accessors compare corpus target/psi-accessors/corpus[-script].jvm.hashes [--script]`.
- `cargo test -p ktrs-fmt --test golden` — ktfmt's own test cases in `testdata/ktfmt/<suite>/`, ratchet
  `tests/golden-passing.txt`. Regenerate (JVM, background): `tools/ktfmt-oracle/extract-goldens.sh`.
- `cargo fmt-diff [meta|google|kotlinlang]` (repo root) — byte diff vs real ktfmt on the corpus; oracle built
  in the background by `tools/ktfmt-oracle/ktfmt-oracle.sh <style> corpus target/ktfmt-oracle/<style>`.

- `cargo test -p ktrs-ast -p ktrs-lint --release` — primitives, seeded dump == `psi_dump`, ktlint's rule tests,
  allocation counts. Corpus vs the JVM ktlint oracle: `tools/ktlint-tests/oracle-diff.sh` (`cargo ktlint-probe`).
- `cargo test -p ktrs-cli` — ktfmt's CLI tests, ported. `tools/ktfmt-oracle/cli-diff.sh` (JVM, ~2 min) — the
  `ktfmt` binary vs the ktfmt jar on stdout/stderr/exit code/files (`ONLY=<regex>`, `KEEP=1`).

## Rules

- **Kotlin pin is `v2.4.20` everywhere** (psi-dump.sh, sync-kotlin.sh). Bump them together, then rerun
  `psi-dump.sh kinds > crates/ktrs-syntax/kinds.tsv` and `cargo xtask codegen`.
- **Release version** lives in `Cargo.toml` (workspace version + every `workspace.dependencies` ktrs
  entry) and `pyproject.toml` (the pre-commit launcher fetches `v<version>`); the release workflow
  rejects a tag that doesn't match. The JVM jar takes it from the tag.
- Never hand-edit `generated/`. Kind names = compiler field names (`KtTokens.FUN_KEYWORD` -> `FUN_KEYWORD`).
- Port 1:1: one Rust fn per Java method, `snake_case` of the Java name, same order within the file, so
  upstream diffs map onto our code. Keep upstream control flow even where it looks odd.
- Parity is the spec. When our dump differs from the fixture/oracle, the oracle is right.
- Input CRLF is normalized to LF before lexing (same as IntelliJ, ktfmt, ktlint).
- Run `psi-dump.sh`, gradle-like JVM steps, and full corpus diffs in the background (they exceed 60s).
