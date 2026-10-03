# 29 — ktlint-gradle drop-in: `io.github.hexay.ktrs.ktlint` (2026-10-03)

Decision: memory `ktlint-build-plugins-decision`. Second plugin id in `java/gradle-plugin` (beside the ktfmt-gradle
drop-in `io.github.hexay.ktrs`), mirroring **JLLeitschuh/ktlint-gradle v14.2.0** (latest release, 2026-03-12, tag
commit `4f887d2a46713a03c57215b08c8d90f8c5e95703`). Switching is changing the plugin id; build scripts keep
`ktlint { … }`, task names, configurations and `org.jlleitschuh.gradle.ktlint.*` types.

## Mirrored

- Plugin class `org.jlleitschuh.gradle.ktlint.KtlintPlugin` (+ `KtlintBasePlugin`), so `plugins.withType<KtlintPlugin>`
  / `apply<KtlintPlugin>()` compile (unlike the ktfmt drop-in's `io.github.hexay.ktrs.gradle.KtrsPlugin`: build logic
  does reference `KtlintPlugin` by type). Applies to `kotlin`, `org.jetbrains.kotlin.js`, `…multiplatform`, Android
  (`org.jetbrains.kotlin.android` and AGP 9 built-in Kotlin), as upstream (`KotlinSourceSetsApplier.kt`,
  `android/AndroidPluginsApplier.kt`). Upstream's KMP `androidJvm` branch builds a lambda it never calls: ported as is.
- Extension `ktlint { version, relative, verbose, debug, android, outputToConsole, coloredOutput, outputColorName,
  ignoreFailures, enableExperimentalRules, additionalEditorconfig, baseline, reporters { reporter(…); customReporters
  { … } }, kotlinScriptAdditionalPaths { include(…) }, filter { … } }`; `ktlint-plugins.properties` (`ktlint-version`).
- Tasks: `ktlintCheck`/`ktlintFormat`, `runKtlint{Check,Format}Over<SourceSet>SourceSet` and `…OverKotlinScripts`,
  `ktlint<SourceSet>SourceSet{Check,Format}` / `ktlintKotlinScript{Check,Format}` (report tasks, which fail the build
  from a `org.jlleitschuh.gradle.ktlint.worker.ConsoleReportWorkAction`, so the failure text nests as upstream's),
  `loadKtlintReporters`, `ktlintGenerateBaseline`, `addKtlint{Check,Format}GitPreCommitHook` (hook script
  byte-identical); same groups, descriptions, `check` wiring, `mustRunAfter`s, `internalKtlintGitFilter`.
- Incremental lint/format as upstream (`BaseKtLintCheckTask.runLint`/`runFormat`): changed files (all after an
  `.editorconfig` change), skipped when only removals changed; then lint adds every file the last run linted
  (upstream keeps a result per linted file, errors or not — so in practice it relints everything it linted before),
  format adds the files its last run formatted. Format task's restored-file up-to-date check (pre-format hashes).
- Public types: `KtlintExtension` (+ `ReporterExtension`, `KScriptExtension`), `reporter.ReporterType`,
  `reporter.CustomReporter`, `tasks.{BaseKtLintCheckTask, KtLintCheckTask, KtLintFormatTask, GenerateReportsTask,
  GenerateBaselineTask}`, `KtlintInstallGitHookTask`. Configurations `ktlint`, `ktlintRuleset`, `ktlintReporter`,
  `ktlintBaselineReporter`.

## How it runs

Each lint/format task runs **one** `ktrs ktlint` process over the files it lints (`KtrsKtlint.kt`; an argfile past 24k
characters), in source order: the `ktlint` drop-in behind the bundled `ktrs` binary (`ktrs ktlint <args>` =
`src/bin/ktlint.rs` without the Windows `java` launcher wildcard expansion; `crates/ktrs-cli/src/ktrs.rs`). Binary:
`-Pktrs.executable` / `-Dktrs.executable`, else the one bundled in `io.github.hexay:ktrs` (`KtrsExecutable.locate()`).
Working dir = the project dir (baseline keys); `JAVA_HOME` = the Gradle JVM's `java.home` (for the jar hand-off).

ktlint-gradle runs ktlint's engine in-process with its own reporting; hidden, ktrs-only options of the drop-in
reproduce that (`crates/ktrs-cli/src/ktlint/gradle.rs`, table there):

- `--ktrs-gradle-events=<file>`: every error with its status (`file`/`error` lines). Check tasks print upstream's
  " (cannot be auto-corrected)" from the status. With `--format` the run formats like the engine's
  `format(code, callback: (LintError, Boolean))` (`KtLintInvocation100.invokeFormat`): all autocorrectable errors fixed,
  baseline ones too, and the reporters get every error met, fixed ones included, with lint statuses and plain details.
  Reporters get paths with the platform's separators (upstream's `File.absolutePath`; it also orders `html`).
- `--ktrs-relative-to=<root dir>` with `--relative`: report paths relative to the root project, as upstream's.
- `--ktrs-editorconfig-override=<name>=<value>` (`additionalEditorconfig`): an `EditorConfigOverride`, which wins over
  every `.editorconfig`, names resolved like `EditorConfigPropertyRegistry.find` (unknown names fail the run).

Other options (`KtlintCommand.kt`): `--ktlint-version=1.8` for "1.8.0" (default), `=2.0` for "2.0.0-ALPHA-4", any other
version fails `loadKtlintReporters`/the lint task naming both; `--color`, `--color-name`, `--baseline` (when the file
exists), `-R`, one `--reporter=<id>[?group_by_file][,artifact=…],output=…` per reporter. The report task copies the
run's reports to the upstream names. `ktlintGenerateBaseline`: one run with the `baseline` reporter over every check
task's sources (still dependsOn them).

Rule sets (`JarServices.kt`): `ktlintRuleset` minus ktlint's own runtime groups (`com.pinterest*`, `io.github.ktlint*`,
`org.jetbrains*`, `io.github.oshai`, `org.slf4j`, `org.ec4j`, `ch.qos.logback`); nothing if no JAR declares a rule set
provider, the JAR itself if it is the only one, else all merged into one JAR (services concatenated). compose-rules
runs natively both as `files("…-all.jar")` and as the Maven artifact (`io.nlopez.compose.rules:ktlint` +
`common-ktlint`, which the plugin merges): `ktrs_compose::NATIVE_JARS` holds both fingerprints per release
(directory entries no longer count, so merge order and the merger's tool don't matter). The 19 of 197 compose classes
whose bytes differ between the two are the same code: `javap -c -p` equal after normalizing constant-pool indices,
`ldc`/`ldc_w`, offsets and branch targets, except that the `-all.jar` calls `shadow/org/jetbrains/kotlin/psi/psiUtil/*`
(its relocated copy of the compiler helpers) where the Maven JARs call `org/jetbrains/kotlin/psi/psiUtil/*`. The CLI case
`-R thin.jar -R common-ktlint.jar` doesn't apply: ktlint rejects a `-R` JAR without a provider (`common-ktlint` has
none), so only the merged JAR matters. Other rule set / custom reporter JARs: the drop-in hands the run to the ktlint
jar, which ignores the hidden options; the events file then is a `json` report (no statuses: check-task rows lack the
suffix, format reports follow ktlint's CLI).

## Deviations

1. The baseline is applied by the lint task (CLI `--baseline`), so lint tasks rerun when it appears or changes
   (upstream: only the report tasks rerun). Outcomes and outputs are the same.
2. `ktlint`/`ktlintRuleset`/`ktlintReporter` get no ktlint artifacts added (ktrs is the ktlint); `workerMaxHeapSize`
   is accepted and unused; `android` and `enableExperimentalRules` do nothing, as upstream with ktlint 1.x; `debug`
   logs the command lines and the CLI output at warn. `ReporterType` has no `availableSinceVersion`.
3. Rule sets other than compose-rules (and custom reporters) go through the ktlint jar hand-off: JVM speed, the
   `json`-report fallback above, and only what ktlint's `-R` loads (the merged JAR includes dependencies, but ktlint's
   own classes come from the ktlint jar).
4. Git hooks find `.git` by walking up (no jgit); same script and messages.
5. The plugin id in Gradle's "registered by plugin '…'" lines.

## Verification

- TestKit (`java/gradlew -p java :ktrs-gradle-plugin:test`, slow: background): upstream's functional tests ported to
  `src/test/kotlin/org/jlleitschuh/gradle/ktlint/` (12 classes incl. the incremental-lint tests) — 85 tests pass; the
  ktfmt plugin's 91 too. Not ported: Android, Kotlin/JS (Kotlin 2.4 rejects the plugin), 3rd-party reporter
  (network), "force dependency versions", `UnsupportedGradleTest`, `KtLintClassesUsageScopeTest`.
  The 3 `KtfmtCheckTaskIntegrationTest` failures seen earlier also failed on master (same 3 of 27): Windows-only, the
  test helper appended CRLF lines to the LF fixture `build.gradle.kts`, which `ktfmtCheckScripts` then flagged; fixed
  in `testutil/File.kt`.
- Rust: `crates/ktrs-cli/tests/ktrs_ktlint.rs` (passthrough, gradle format events/reports, override precedence,
  relative base), `hand_off_args` (hidden options dropped/rewritten), `compose_jar` (both fingerprints).
- Parity vs the real plugin (14.2.0, ktlint 1.8.0, Gradle 9.8.0, Windows): `tools/ktlint-gradle/parity.sh` (both sides
  on the same project; `compare.py` diffs console — each run's task blocks in a stable order, Gradle noise and the
  plugin id normalized — task outcomes, report files and sources byte for byte; `rows.py` compares console rows).
  Results (2026-10-03):

| scenario | what | result |
|---|---|---|
| check-all | 5 reporters, `ktlintCheck` ×2 | identical (56 rows, reports, outcomes incl. UP-TO-DATE) |
| format | `ktlintFormat` ×3 | identical (sources, reports, 6 rows, outcomes) |
| baseline | `ktlintGenerateBaseline`, `ktlintCheck` | baseline.xml and results identical; lint tasks rerun (dev. 1) |
| options | verbose, relative, no color, ignoreFailures, additionalEditorconfig, filter, plain_group_by_file+checkstyle | identical |
| compose-maven | `ktlintRuleset "io.nlopez.compose.rules:ktlint:0.6.7"` | identical, native (no hand-off) |
| compose-all | `ktlintRuleset files(ktlint-compose-0.6.7-all.jar)` | identical, native |
| realcode | okhttp `commonJvmAndroid` (152 files, its `.editorconfig`) + ktlint-rule-engine (29), check + format | identical (2475 rows, txt/json/xml/sarif reports, 181 formatted sources) but `html` file order: same lines, order of the reporter's `ConcurrentHashMap` keyed by the absolute path, which differs between the two project dirs (`…/upstream/…` vs `…/ktrs/…`) |

## Open items

- Not verified: Android projects, multi-project `relative`, custom reporter JARs (network), Linux/macOS parity runs.
