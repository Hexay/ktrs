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
  codes); `ktrs lint` and the `ktlint` drop-in (`src/ktlint/`, 1:1 port of ktlint-cli + reporters; deviations in
  its `mod.rs`). The root package `ktrs` owns the binaries (so `cargo install --path .` and pre-commit work);
  releases build `--profile dist` (`.github/workflows/release.yml`).
  Adoption gaps and integrations: research/08-drop-in-replacement.md. `ktrs serve` is the build-tool
  server (protocol in `crates/ktrs-cli/src/serve.rs`).
- `action.yml` — the GitHub Action: installs via `install.sh`; its `check` input runs `tools/action/check.sh`
  (`ktrs fmt --check`/`ktrs lint` with `--reporter github`, `--changed-since`).
- `crates/ktrs-lsp` — `ktrs lsp` (lsp-server, sync): ktlint diagnostics/fixes/suppressions, ktfmt or ktlint formatting;
  protocol and settings in its `src/lib.rs` docs; tests `cargo test -p ktrs-cli --test lsp` (fixes vs our `ktlint -F`).
- `crates/ktrs-wasm` + `site/` — the browser playground (plain Wasm exports, no bindgen); build and
  preview: `tools/release/build-site.sh && py -3 -m http.server -d target/site`.
- `editors/vscode` — the VS Code extension `hexay.ktrs` (TypeScript client of `ktrs lsp`; settings mirror the table in
  `crates/ktrs-lsp/src/lib.rs`). Test: `cargo build --bins`, then `npm test` in it (downloads VS Code, background it).
  Release: `tools/release/package-vscode.sh` (one VSIX per target with its binary + universal).
- `editors/intellij` — IntelliJ/Android Studio plugin `io.github.hexay.ktrs` (platform LSP via `ktrs-lsp.xml`, LSP4IJ via
  `ktrs-lsp4ij.xml`; settings mirror VS Code's). Test: `cargo build --bins`, then `./gradlew test buildPlugin verifyPlugin`
  in it (JDK 21, ~4 GB RAM: testbox, background).
- `npm/cli` — `@ktrs/cli` (JS launcher of the `@ktrs/cli-<os>-<cpu>` binary packages); release generates all seven,
  versions stamped from the tag: `node tools/release/package-npm.mjs <archives dir> <out> <tag>`.
- `docker/Dockerfile` — `ghcr.io/hexay/ktrs` (distroless static + release musl binaries, no compile); context:
  `tools/release/docker-context.sh <archives dir> <out>`.
- Package managers: `Formula/` (brew tap) and `bucket/` (Scoop) are committed by the release workflow; winget manifests
  go to microsoft/winget-pkgs only with `WINGET_TOKEN`. Generators `tools/release/{homebrew-formula,scoop-manifest,
  winget-manifests}.sh <tag> <SHA256SUMS>` (try one on a past release's sums). `cargo binstall`: root `Cargo.toml` metadata.
  Bazel: `multitool-lock.sh` (same arguments) prints the release's rules_multitool lockfile; `bazel/e2e/rules_lint/test.sh
  <lockfile>` runs rules_lint on it (`bazel/e2e/local-lock.sh <bin dir>` for built binaries; Bazel: testbox or CI; research/35).
- `java/` — `io.github.hexay:ktrs`: JVM wrapper around `ktrs serve` (bundled binaries, Spotless `KtrsStep` and
  `KtrsKtlintStep`: research/28). Tests: `cargo build --bins`, then `java/gradlew -p java test` (JAVA_HOME = tools/jdk/*).
  `java/gradle-plugin` — `io.github.hexay.ktrs`, ktfmt-gradle 0.27.0 drop-in (same DSL/tasks/FQNs), and
  `io.github.hexay.ktrs.ktlint`, ktlint-gradle 14.2.0 drop-in over `ktrs ktlint` (research/29; parity harness
  `tools/ktlint-gradle/parity.sh`), and `io.github.hexay.ktrs.kotlinter`, kotlinter-gradle 5.7.0 drop-in (research/34; parity
  `tools/kotlinter/parity.sh`); tests: `java/gradlew -p java :ktrs-gradle-plugin:test` (TestKit, slow: background it).
  `java/maven-plugin` — `io.github.hexay:ktrs-ktlint-maven-plugin`, gantsign ktlint-maven-plugin 3.7.1 drop-in (research/31;
  parity `tools/ktlint-maven/parity.sh`). Spotless Maven `implementation=` swap: `java/src/spotlessMaven` (research/30;
  parity `tools/spotless-maven/parity.sh`). Rule set/reporter JAR handling shared by all plugins: `KtlintJars` in the root jar.
- `crates/ktrs-ast` — mutable arena AST with IntelliJ `TreeElement` semantics, seeded from `Tree` (for ktlint);
  conventions in `src/lib.rs`. `crates/ktrs-lint` — ktlint 2.0.0-ALPHA-4 engine + ported rules; status
  research/15-ktlint-spike.md; corpus counts per rule research/18-ktlint-corpus-counts.md.
  ktlint 1.8 mode (`ktrs_ktlint_version = 1.8` / `--ktlint-version=1.8`; lint rows, CLI, rule-major `-F` order):
  research/26-ktlint-18-mode.md. Default stays 2.0; a 2.0 change touching a switch keeps the 1.8 branch.
  `crates/ktrs-compose` — compose-rules (pin `tools/sync-compose-rules.sh`) as a native rule set, used when `-R` loads
  that release's jar; other `-R`/reporter jars hand the run to the real ktlint jar (`ktrs-cli/src/ktlint/ktlint_jar.rs`).
  Status, gates, pin bump: research/27-custom-rulesets-impl.md.
  `crates/ktrs-editorconfig` — ec4j 1.2.0 port (ktlint's `.editorconfig` semantics; ktfmt still uses ec4rs).
- `crates/ktrs-detekt` — detekt `v2.0.0-alpha.6` light mode over ktrs-psi (pin `tools/sync-detekt.sh`): engine + 27 rules,
  no CLI; conventions `src/lib.rs`, status research/33 "Spike result". Gates: `cargo test -p ktrs-detekt --release`
  (goldens `testdata/detekt/`, ratchet `tests/golden-passing.txt`; regenerate: `tools/detekt-tests/extract-goldens.sh`, JVM,
  background); `cargo detekt-diff [default|all-rules]` vs `tools/detekt-oracle/detekt-probe.sh corpus
  target/detekt-oracle/<run> [--all-rules]` (JVM, background). Cost: `cargo run -p ktrs-detekt --release --example bench`.
- `crates/ktrs-project` — static detection of a build's ktfmt/ktlint setup (Gradle Kotlin/Groovy DSL, convention
  plugins, version catalog, Maven) for `ktrs lsp`; sources and limits in `src/lib.rs` docs; spot check:
  `cargo run -p ktrs-project --example detect -- <file or dir>...`. Its `src/migrate/` backs `ktrs migrate` (swaps
  in `coords.rs`; must mirror README "Integrations"); tests `cargo test -p ktrs-project --test migrate`
  (`UPDATE_MIGRATED=1` rewrites `tests/fixtures/migrated/`), CLI `cargo test -p ktrs-cli --test ktrs_migrate`.
- `tools/psi-accessors/psi-accessors.sh` — JVM oracle for ktrs-psi (`one|hashes|dump <dir> [--fixture] [--script]`);
  Rust mirror: `cargo run -p ktrs-psi --release --example psi_accessors -- one|hashes|compare|dump ...`.
- `crates/kt-syntax` — the public parser library (the only crate with a stable API; `ktrs-*` are internals): facade
  over syntax/lexer/parser, design and limits research/36. Tests `cargo test -p kt-syntax` (dump == `psi_dump` on the fixtures).
- `xtask` — `cargo xtask codegen` regenerates `ktrs-syntax/src/generated/kinds.rs` and `kt-syntax`'s from `kinds.tsv`.
- `tools/psi-dump/psi-dump.sh` — JVM oracle on the pinned compiler: `one <file>`, `tree <in> <out>`, `kinds`,
  `bench <dir> <warmup> <reps>` (warm single-thread baseline to compare with the ktrs `bench` example).
- `tools/sync-kotlin.sh` — sparse-checks-out the pinned Kotlin sources to `third_party/kotlin` (gitignored)
  and vendors parser fixtures into `testdata/kotlin/`.
- `testdata/kotlin/{psi,lexer}` — upstream fixtures (Apache-2.0), `<name>.kt` + expected `<name>.txt`.
  Input convention: CRLF->LF and trailing newlines stripped (matches upstream's test framework).
- `tools/fetch-corpus.sh` — real-world repos at the pins in `tools/corpus/REVISIONS` into `corpus/` (gitignored).
- `tools/bench/public.sh` — the README's benchmark (hyperfine vs sha256-pinned jars, pins `tools/bench/REVISIONS`;
  CI: `.github/workflows/bench.yml`); research/23.
- `fuzz/` (own workspace, nightly + cargo-fuzz, Linux) and `tools/fuzz/{fuzz,diff}.sh` — fuzzing and the differential
  runner vs the jars; findings and status research/24.

## Parity gates (all must stay green)

- `cargo test -p ktrs-parser --release` — fixture ratchet `tests/passing.txt` (`UPDATE_PASSING=1` rewrites).
- `cargo corpus-diff` (run from repo root) — our dump vs the compiler's for every corpus file; also
  prints MB/s and the slowest files. Oracle dumps are built once, in the background:
  `tools/psi-dump/psi-dump.sh tree corpus target/oracle/corpus`.
- `cargo test -p ktrs-psi --release` — PSI accessor reports vs the JVM on the fixtures (hashes in
  `crates/ktrs-psi/tests/data`; regeneration commands in `tests/fixtures.rs`). Corpus, in the background:
  `psi-accessors.sh hashes corpus [--script] > target/psi-accessors/corpus[-script].jvm.hashes`, then
  `psi_accessors compare corpus target/psi-accessors/corpus[-script].jvm.hashes [--script]`.
- `cargo test -p ktrs-fmt --test golden` — ktfmt's own file-based cases in `testdata/ktfmt/<group>/`, ratchet
  `tests/golden-passing.txt`. Regenerate (JVM, background): `tools/ktfmt-oracle/extract-goldens.sh`.
- `cargo fmt-diff [meta|google|kotlinlang]` (repo root) — byte diff vs real ktfmt on the corpus; oracle built
  in the background by `tools/ktfmt-oracle/ktfmt-oracle.sh <style> corpus target/ktfmt-oracle/<style>`.

- `cargo test -p ktrs-ast -p ktrs-lint --release` — primitives, seeded dump == `psi_dump`, allocation counts, and
  `--test golden`: ktlint's rule tests in `testdata/ktlint/<rule-id>/` (lint rows, format rows, text from the real
  engine), ratchet `tests/golden-passing.txt`; unported rules are skipped and counted. Regenerate (JVM, testbox,
  background): `tools/ktlint-tests/extract-goldens.sh`.
- `cargo lint-diff [ktlint_official|intellij_idea|android_studio]` (repo root) — ktrs-lint vs ktlint on the corpus: lint
  rows of the ported rules; format too with a `--rules <ported>` oracle (`--oracle DIR`); `--counts` = per-rule
  totals; `--experimental` = `ktlint_experimental = enabled`. Oracle (JVM, testbox, background): `KTLINT_CODE_STYLE=<style>
  [KTLINT_EXPERIMENTAL=enabled] tools/ktlint-oracle/ktlint-probe.sh corpus target/ktlint-oracle/<style>[-experimental]
  --rules <ported>`; status and commands research/19-ktlint-parity.md. Pass-by-pass tree diff: `tools/ktlint-tests/oracle-diff.sh` (`cargo ktlint-probe`).
- `cargo test -p ktrs-compose --release` — compose-rules' own tests as goldens (`testdata/compose-rules/`, both ktlint
  modes; regenerate: `tools/compose-rules-tests/extract-goldens.sh`, JVM, background). Real code (testbox, background):
  `tools/compose-rules/parity.sh` (jar+`-R` vs ktrs+`-R`, lint and `-F`, 1.8 and 2.0).
- `tools/holdout/run.sh` (testbox, background, hours) — held-out corpus: the `ktlint`/`ktfmt` binaries vs the jars on
  20 repos never used for fixes (`tools/holdout/REVISIONS`), via `tools/parity/{ktlint,ktfmt}-compare.sh`; research/21.
  `ktlint-compare.sh` also diffs two ktlint versions (1.8 vs 2.0: research/22).
- `cargo test -p ktrs-cli` — ktfmt's and ktlint's CLI and reporter tests, ported. `tools/ktfmt-oracle/cli-diff.sh` (JVM, ~2 min) — the
  `ktfmt` binary vs the ktfmt jar on stdout/stderr/exit code/files (`ONLY=<regex>`, `KEEP=1`).
  `tools/ktfmt-oracle/range-diff.sh [dir] [files] [seed] [binary]` (JVM, background) — partial formatting
  (`--lines`, `--offset`/`--length`) vs the jar on random ranges over a corpus sample.
  `tools/ktlint-oracle/cli-diff.sh [ktlint-binary]` (JVM, testbox, background) — same for `ktlint` vs the ktlint jar;
  unported rules are disabled on both sides; accepted mismatches in `tools/parity/known-diffs/ktlint-cli.tsv`.
- `.github/workflows/parity.yml` — the gates above that need JVM oracles or the corpus (not the holdout), nightly +
  `workflow_dispatch`; oracles cached by pin hash; accepted diffs `tools/parity/known-diffs/`. release.yml refuses a
  tag whose commit has no green run: `gh workflow run parity.yml --ref <ref>` first.

## Rules

- **Kotlin pin is `v2.4.20` everywhere** (psi-dump.sh, sync-kotlin.sh). Bump them together, then rerun
  `psi-dump.sh kinds > crates/ktrs-syntax/kinds.tsv` and `cargo xtask codegen`.
- **Release version** lives in `Cargo.toml` (workspace version + every `workspace.dependencies` ktrs
  entry), `pyproject.toml` (the pre-commit launcher fetches `v<version>`) and `editors/vscode/package.json`
  (+ its `package-lock.json`, via `npm version <v> --no-git-tag-version`); the release workflow
  rejects a tag that doesn't match. The JVM jar takes it from the tag.
- Never hand-edit `generated/`. Kind names = compiler field names (`KtTokens.FUN_KEYWORD` -> `FUN_KEYWORD`).
- Port 1:1: one Rust fn per Java method, `snake_case` of the Java name, same order within the file, so
  upstream diffs map onto our code. Keep upstream control flow even where it looks odd.
- Parity is the spec. When our dump differs from the fixture/oracle, the oracle is right.
- Input CRLF is normalized to LF before lexing (same as IntelliJ, ktfmt, ktlint).
- Run `psi-dump.sh`, gradle-like JVM steps, and full corpus diffs in the background (they exceed 60s).
