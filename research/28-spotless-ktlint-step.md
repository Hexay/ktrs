# 28 — Spotless ktlint step: `KtrsKtlintStep` (2026-10-03)

Implements the Spotless half of memory `ktlint-build-plugins-decision`: a drop-in for Spotless's `ktlint()` step that
runs ktrs's ktlint port through `ktrs serve`.

```kotlin
spotless { kotlin { addStep(KtrsKtlintStep.create(KtlintOptions.defaults()
    .withEditorConfigOverride(mapOf("indent_size" to 2))
    .withEditorConfigPath(rootProject.file(".editorconfig")))) } }
```

## Pinned reference

Spotless tag `gradle/8.10.3` = `spotless-lib` 4.10.3 (2026-09-25), the latest release. Mirrored files:

- `lib/src/main/java/com/diffplug/spotless/kotlin/KtLintStep.java` — options and up-to-date state.
- `lib/src/ktlint/java/com/diffplug/spotless/glue/ktlint/KtlintFormatterFunc.java` — `Lint.atLine(line, ruleId,
  detail).shortcut()` from the first unfixable violation.
- `lib/src/compatKtLint1Dot0Dot0/.../KtLintCompat1Dot0Dot0Adapter.java` — engine setup, override building.
- `lib/src/compatKtLintApi/.../KtLintCompatReporting.java` — the callback throws on the first uncorrected error.
- `plugin-gradle/.../BaseKotlinExtension.java` — `KtlintConfig` (editorConfigPath default, customRuleSets).
- ktlint 1.8.0 `KtLintRuleEngine.format(code, callback: (LintError, Boolean) -> Unit)` → `CodeFormatter.format(code,
  AllAutocorrectHandler, callback)`.

## What Spotless does (1.x), and what ktrs mirrors

| Spotless | ktrs |
|---|---|
| `KtLintRuleEngine.format(code, callback)` with `AllAutocorrectHandler`: every error a rule can fix is fixed; after formatting, the callback gets the distinct errors sorted by (line, col) with `corrected` | `KtLintRuleEngine::format_reporting(code, \|_\| Allow, callback)` (new, ktrs-lint), same `CodeFormatter` path |
| callback throws on the first `!corrected` error → no formatted output; Spotless records `Lint.atLine(line, ruleId, detail)` and passes the file on unchanged | server returns all uncorrected violations; the step throws `Lint.atLine(...).shortcut()` for the first, discarding the code |
| `KtLintParseException` ("`line:col message`") → `Lint.createFromThrowable` (line from the message) | error response with the same text → `KtrsException` with that message, same lint |
| `Code.fromPath(path)` + content replaced: file name rules and `.editorconfig` lookup by the real path, `.kts` by extension | `Code::from_file_content(path, content)` |
| rule providers: `ServiceLoader<RuleSetProviderV3>` over ktlint + `customRuleSets` | standard rules + native compose-rules; any other JAR: error naming the JAR and "keep `ktlint()`" |
| `EditorConfigDefaults.load(editorConfigPath)` (Gradle plugin default: root project's `.editorconfig` if it exists) | `editorconfig-defaults=<path>`; `KtlintOptions.withEditorConfigPath` (no default, see deviations) |
| override map empty → no override; else: names from the rules' `usesEditorConfigProperties` + 7 standard properties, `ktlint_<set>` / `ktlint_<set>_<rule>` → rule (set) execution properties, anything else dropped silently; if neither the map nor any section of the defaults file sets `ktlint_code_style`, it adds `ktlint_code_style=intellij_idea` | `spotless_editor_config_override` in `crates/ktrs-cli/src/serve_ktlint.rs`, same rules; values sent as `toString()` |
| step name `ktlint`; lints print `L<line> ktlint(<rule id>) <detail>` | step name `ktlint` |
| up-to-date state: version, override (sorted), `FileSignature` of the editorconfig path, ktlint + rule set JARs | `KtlintOptions` + `FileSignature` (promised, config-cache safe) of the path and rule set JARs + jar version |

Version: `KtlintOptions.of("1.8.0")` (default) or `"2.0.0-ALPHA-4"`; anything else throws at step creation
(`ktrs matches ktlint 1.8.0 and 2.0.0-ALPHA-4, not <v>`). Spotless has no 2.0 adapter (it rejects major ≠ 1), and ktlint
2.0 dropped the overload Spotless calls, so 2.0 mode decides per error like the CLI's `--format` (fix iff it can be
fixed); the rest is the same.

## Protocol (`ktrs serve` v2)

Hello is now `ktrs-serve 2 <version>`. v1 ktfmt requests/responses are byte-identical (a ktfmt response gains no
header lines). New: `tool=ktfmt|ktlint`; ktlint keys `ktlint-version` (1.8|2.0, default 1.8), `editorconfig-defaults`,
`editorconfig-override=<name>=<value>` (repeatable), `ruleset` (repeatable), `path`. Ok responses add
`violation=<line>\t<col>\t<rule id>\t<detail>` lines (detail escapes `\\ \t \n \r`). The server keeps the engine of
the last configuration (rule sets, defaults, overrides) and rebuilds it only when that changes. Java: `Ktrs.ktlint(code,
KtlintOptions, Path) -> KtlintResult`; `ServerProcess` accepts protocol ≥ 1 and fails a ktlint call on a v1 server with
a "too old" message. An old jar (exact `ktrs-serve 1` check) refuses a v2 binary; jars bundle their binary, so only a
`-Dktrs.executable` mismatch hits that.

## Verification (2026-10-03, Windows)

- Rust: `cargo test -p ktrs-cli` all green, incl. `tests/serve_ktlint.rs` (8) and the escape unit test.
- Java `:test` (`java/gradlew -p java :test`): 33 tests, 32 pass, 1 skipped (pre-existing `NativeBinaryTest`); suites `KtlintTest`,
  `KtlintDropInParityTest` (both versions), `KtrsKtlintStepTest` (incl. Spotless's own `unsolvable.dirty` expectation
  `L1 ktlint(standard:no-empty-file) File 'unsolvable.dirty' should not be empty`), `SpotlessKtlintParityTest`.
- `SpotlessKtlintParityTest`: Spotless 4.10.3's real `KtLintStep.create("1.8.0", ...)` in the test JVM (ktlint-cli
  1.8.0 and compose-rules `io.nlopez.compose.rules:ktlint:0.6.7` resolved by Gradle) vs `KtrsKtlintStep` on the same
  files, 5 configurations (defaults; overrides incl. an unknown key → intellij_idea; overridden code style; code style
  from the editorconfig path; compose rules). Outcome compared per file: formatted text, lints, or exception message.
  Inputs: built-in samples, or `-PktlintParityCorpus=<dir> [-PktlintParityMaxFiles=N]` (first N files, LF).
  Results: samples, and the first 400 files of `corpus/nowinandroid` and of `corpus/okhttp` — 0 differences in all
  5 configurations.
- `KtlintDropInParityTest`: same inputs, `Ktrs.ktlint` vs `ktlint -F --ktlint-version=<v>` (code and distinct
  unfixed rows; a rule crash or parse error must be a row there): samples, nowinandroid, okhttp, both versions, equal.

## Deviations

- No default `editorConfigPath`: `addStep` can't see the root project. It matters only for values the file's own
  `.editorconfig` chain doesn't set, and for the `intellij_idea` switch (a `ktlint_code_style` in the root
  `.editorconfig` keeps Spotless from forcing `intellij_idea` only when it is the editorconfig path). Pass
  `withEditorConfigPath(rootProject.file(".editorconfig"))` to match `ktlint()` exactly.
- `customRuleSets` are JAR files, not Maven coordinates; only compose-rules 0.6.7 runs, any other JAR fails each file
  with the "keep `ktlint()`" message (as a Spotless lint at `LINE_UNDEFINED`, so the build fails).
- A rule crash's message is ktrs's (`Rule '<id>' throws exception …\nCaused by: <panic>`), not the JVM stack trace
  Spotless would show.
- Engine warnings (e.g. "Format was not able to resolve all violations") are dropped; Spotless logs them via SLF4J.
- `.editorconfig` files are cached for a server's life (one per formatter function, i.e. per Spotless run); ktlint's
  cache in Spotless's daemon classloader lives longer.

## Open

- 2.0 mode: the `standard:indent` rule panics (`Stack should be empty`) in format mode on comment-only
  `corpus/nowinandroid/spotless/copyright.kt` (nowinandroid `.editorconfig`); the `ktlint` drop-in does the same (rule
  crash row), 1.8 mode is fine. Not checked against the real 2.0.0-ALPHA-4 jar (needs `tools/sync-ktlint.sh`).
