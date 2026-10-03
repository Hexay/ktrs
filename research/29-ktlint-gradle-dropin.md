# 29 — ktlint-gradle drop-in: `io.github.hexay.ktrs.ktlint` (2026-10-03)

Decision: memory `ktlint-build-plugins-decision`. Second plugin id in `java/gradle-plugin` (beside the ktfmt-gradle
drop-in `io.github.hexay.ktrs`), mirroring **JLLeitschuh/ktlint-gradle v14.2.0** (latest release, 2026-03-12, tag
commit `4f887d2a46713a03c57215b08c8d90f8c5e95703`). Switching is changing the plugin id; build scripts keep
`ktlint { … }`, task names, configurations and `org.jlleitschuh.gradle.ktlint.*` types.

## Mirrored

- Plugin class `org.jlleitschuh.gradle.ktlint.KtlintPlugin` (+ `KtlintBasePlugin`), so `plugins.withType<KtlintPlugin>`
  / `apply<KtlintPlugin>()` compile. Applies to `kotlin`, `org.jetbrains.kotlin.js`, `…multiplatform`, Android
  (`org.jetbrains.kotlin.android` and AGP 9 built-in Kotlin), as upstream (`KotlinSourceSetsApplier.kt`,
  `android/AndroidPluginsApplier.kt`). Upstream's KMP `androidJvm` branch builds a lambda it never calls: ported as is.
- Extension `ktlint { version, relative, verbose, debug, android, outputToConsole, coloredOutput, outputColorName,
  ignoreFailures, enableExperimentalRules, additionalEditorconfig, baseline, reporters { reporter(…); customReporters
  { … } }, kotlinScriptAdditionalPaths { include(…) }, filter { … } }`; `ktlint-plugins.properties` (`ktlint-version`).
- Tasks: `ktlintCheck`/`ktlintFormat`, `runKtlint{Check,Format}Over<SourceSet>SourceSet` and `…OverKotlinScripts`,
  `ktlint<SourceSet>SourceSet{Check,Format}` / `ktlintKotlinScript{Check,Format}` (report tasks, which fail the
  build), `loadKtlintReporters`, `ktlintGenerateBaseline`, `addKtlint{Check,Format}GitPreCommitHook` (hook script
  byte-identical); same groups, descriptions, `check` wiring, `mustRunAfter`s, `internalKtlintGitFilter`.
- Public types: `KtlintExtension` (+ `ReporterExtension`, `KScriptExtension`), `reporter.ReporterType`,
  `reporter.CustomReporter`, `tasks.{BaseKtLintCheckTask, KtLintCheckTask, KtLintFormatTask, GenerateReportsTask,
  GenerateBaselineTask}`, `KtlintInstallGitHookTask`. Configurations `ktlint`, `ktlintRuleset`, `ktlintReporter`,
  `ktlintBaselineReporter`.
- Report files `build/reports/ktlint/<reportTask>/<reportTask>.<ext>` (`reportsOutputDirectory` configurable), console
  rows `<abs path>:<line>:<col> <detail>[ (<rule>)]` at warn, failure text "KtLint found code style violations. Please
  see the following reports:" + `- <report>` lines, "KtLint failed to parse file: <path>", baseline message.
- Format task's restored-file up-to-date check (snapshot of pre-format hashes), `@CacheableTask`s, configuration cache.

## How it runs

Each lint/format task runs **one** `ktrs ktlint` process over all its files (`KtrsKtlint.kt`, argfile past 24k
characters): the `ktlint` drop-in behind the bundled `ktrs` binary (`ktrs ktlint <args>` = `src/bin/ktlint.rs` without
the Windows `java` launcher wildcard expansion; `crates/ktrs-cli/src/ktrs.rs`). Binary: `-Pktrs.executable` /
`-Dktrs.executable`, else the one bundled in `io.github.hexay:ktrs` (`KtrsExecutable.locate()`). Working dir = the
project dir; `JAVA_HOME` = the Gradle JVM's `java.home` (for the jar hand-off).

- Options (`KtlintCommand.kt`): `--ktlint-version=1.8` for "1.8.0" (default), `=2.0` for "2.0.0-ALPHA-4", any other
  version fails `loadKtlintReporters`/the lint task naming both; `--format`, `--relative`, `--color`,
  `--color-name`, `--baseline` (when the file exists), `--editorconfig=<additionalEditorconfig as [*.{kt,kts}]>`,
  `-R` rule sets, one `--reporter=<id>[?group_by_file][,artifact=…],output=…` per reporter plus `json` for the console.
- Report task: copies the run's reports to the upstream names, prints the json rows (deduplicated), fails.
- `ktlintGenerateBaseline`: one run with the `baseline` reporter over every check task's sources (still dependsOn them).
- Rule sets (`JarServices.kt`): `ktlintRuleset` minus ktlint's own runtime groups (`com.pinterest*`, `io.github.ktlint*`,
  `org.jetbrains*`, `io.github.oshai`, `org.slf4j`, `org.ec4j`, `ch.qos.logback`); nothing if no JAR declares a rule
  set provider, the JAR itself if it is the only one, else all merged into one JAR (services concatenated) so a rule
  set's dependencies load. Custom reporters: `artifact=` a `ktlintReporter` JAR declaring `ReporterProviderV2`.

## Deviations

1. Not incremental: a task that runs relints all its files (upstream: changed files plus previously failing ones).
   Outcomes and UP-TO-DATE behave the same; report tasks are UP-TO-DATE more often than upstream's.
2. The baseline is applied by the lint task (CLI `--baseline`), so lint tasks rerun when it appears or changes.
3. Reports are ktlint CLI reporter output. On Windows: paths use `/` (ktlint CLI) where ktlint-gradle writes `\`, and
   the plain reporter's colored dir/file split differs accordingly. `html`/`sarif` list files in the CLI's (parallel)
   processing order; `plain`/`json`/`checkstyle` sort, so they match.
4. Format-task reports follow `ktlint --format`: only errors left unfixed, detail suffixed " (cannot be auto-corrected)",
   repeated per format pass when the CLI repeats them. Upstream lists every error its format run met, fixed ones
   included, without suffix.
5. Console rows of **check** tasks lack upstream's " (cannot be auto-corrected)" suffix (the json report has no error
   status). Rows come in ktlint's json order (by path), not file-walk order. Format-task rows match.
6. `additionalEditorconfig` becomes ktlint's `--editorconfig` defaults: a project `.editorconfig` setting the same
   property wins (upstream: the map overrides).
7. `relative = true`: paths relative to the project dir (upstream: root dir); same in single-project builds.
8. The failure is a plain `GradleException` (upstream nests it under "A failure occurred while executing
   …ConsoleReportWorkAction"); the parse failure message adds ktlint's detail line.
9. `ktlint`/`ktlintRuleset`/`ktlintReporter` get no ktlint artifacts added (ktrs is the ktlint); `workerMaxHeapSize`
   is accepted and unused; `android` and `enableExperimentalRules` do nothing, as upstream with ktlint 1.x; `debug`
   logs the command lines and the CLI output at warn. `ReporterType` has no `availableSinceVersion`.
10. Rule sets: only what ktlint's `-R` can load — rule set classes plus their dependencies merged into one JAR, with
    ktlint's own classes from the ktlint jar. compose-rules' `-all.jar` runs natively; its Maven artifact
    (`io.nlopez.compose.rules:ktlint` + `common-ktlint`) merges to different class bytes (19 of 197 entries), so it
    goes through the ktlint jar hand-off (JVM; downloads the ktlint 1.8.0 jar once).
11. Git hooks find `.git` by walking up (no jgit); same script and messages.
12. Plugin implementation class is upstream's FQN (`org.jlleitschuh.gradle.ktlint.KtlintPlugin`), unlike the ktfmt
    drop-in's `io.github.hexay.ktrs.gradle.KtrsPlugin`: build logic does reference `KtlintPlugin` by type.

## Verification

- TestKit (`java/gradlew -p java :ktrs-gradle-plugin:test --tests 'org.jlleitschuh.*'`, slow: background): upstream's
  functional tests ported to `src/test/kotlin/org/jlleitschuh/gradle/ktlint/` (12 classes: plugin, sources/filters/git
  filter, versions, reporters, baseline, editorconfig, supported versions × {1.8.0, 2.0.0-ALPHA-4} incl. disabled
  rules, configuration cache, build cache, configuration avoidance, git hooks, multiplatform) — 83 tests, all pass.
  Not ported: Android, Kotlin/JS (Kotlin 2.4 rejects the plugin), 3rd-party reporter (network), "force dependency
  versions", `UnsupportedGradleTest`, `KtLintClassesUsageScopeTest`. Rust: `crates/ktrs-cli/tests/ktrs_ktlint.rs`.
- Parity vs the real plugin (14.2.0, ktlint 1.8.0, Gradle 9.8.0, Windows): `tools/ktlint-gradle/parity.sh` (both sides
  on the same project; `compare.py --slashes` diffs console/outcomes/reports/sources ignoring `\` vs `/` and ANSI;
  `rows.py` compares console rows). Results (2026-10-03):

| scenario | what | result |
|---|---|---|
| check-all | 5 reporters, `ktlintCheck` ×2 | outcomes equal (2nd run UP-TO-DATE); 56/56 rows equal but 4 lacking the suffix (dev. 5); txt/json/xml/html identical; sarif identical |
| format | `ktlintFormat` ×3 | sources identical; 6/6 rows identical; plain report: dev. 4; report tasks UP-TO-DATE where upstream reruns |
| baseline | `ktlintGenerateBaseline`, `ktlintCheck` | baseline.xml identical; check results identical; lint tasks rerun (dev. 2) |
| options | verbose, relative, no color, ignoreFailures, additionalEditorconfig, filter, plain_group_by_file+checkstyle | check reports identical, 33/33 rows (5 suffix); format reports: dev. 4 |
| compose-maven | `ktlintRuleset "io.nlopez.compose.rules:ktlint:0.6.7"` | 34/34 rows (3 suffix), reports identical (via jar hand-off) |
| compose-all | `ktlintRuleset files(ktlint-compose-0.6.7-all.jar)` | same, natively |
| realcode | okhttp `commonJvmAndroid` (152 files, its `.editorconfig`) + ktlint-rule-engine (29), check + format | 2475/2475 rows (20 suffix); check txt/json/xml identical; html/sarif same content, file order differs (dev. 3); all 181 formatted sources byte-identical |

## Open items

- A status-carrying console output (e.g. a hidden json variant with the error status) would remove deviation 5;
  needs a reporter in `crates/ktrs-cli/src/ktlint/reporter` (out of scope here: CLI parity work in progress there).
- compose-rules' Maven artifact natively: add the merged-JAR fingerprint (`ktlint-0.6.7.jar` + `common-ktlint-0.6.7.jar`
  `io/nlopez/compose/` entries: `38b700317ae4830319bc7a28adfba0316ca9a7f6beecffe6f46cde50e529995b`) to
  `ktrs_compose::NATIVE_JARS`, after checking the 19 differing classes are the same source.
- Not verified: Android projects, multi-project `relative`, custom reporter JARs (network), Linux/macOS parity runs.
