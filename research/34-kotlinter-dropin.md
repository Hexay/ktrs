# 34 — kotlinter drop-in: `io.github.hexay.ktrs.kotlinter` (design 2026-10-09, built 2026-10-10)

Sections 1 to 7 are the design, corrected where the implementation or an observation of kotlinter 5.7.0 differs;
decisions D1 to D10 were taken as recommended. What was observed, built and verified: sections 8 to 10. It revisits the
"No kotlinter" line of memory `ktlint-build-plugins-decision` (2026-10-03). Third plugin id in `java/gradle-plugin`,
mirroring **jeremymailen/kotlinter-gradle 5.7.0** (latest release, 2026-08-09, tag commit
`02dd68cb26a9554932a13192b5bc3e706341b44f`), which bundles **ktlint 1.8.0** (`gradle/libs.versions.toml`; written to
`/kotlinter.properties` as `ktlintVersion`). Switching is changing the plugin id; build scripts keep `kotlinter { … }`,
task names, the `ktlint` configuration and `org.jmailen.gradle.kotlinter.*` types.

Usage, GitHub code search 2026-10-09 (files, forks included, not deduplicated to repos): `org.jmailen.kotlinter` in 784
`build.gradle.kts` and 510 `libs.versions.toml`. research/25's "≤42" counted only kotlinter + compose-rules in one file.

## 1. Surface to mirror (all of upstream's `src/main`, 1,000 lines)

**Plugin.** Id `org.jmailen.kotlinter`, class `org.jmailen.gradle.kotlinter.KotlinterPlugin` (artifact
`org.jmailen.gradle:kotlinter-gradle`), constant `KTLINT_CONFIGURATION_NAME = "ktlint"`. `apply` creates the extension
and `configurations.maybeCreate("ktlint")` (resolvable, not consumable); everything else happens in `afterEvaluate`:
ktlint's eight artifacts added to `ktlint` at `ktlintVersion`, the hook task on the root project only, then per
`pluginManager.withPlugin` of `org.jetbrains.kotlin.jvm`, `…multiplatform`, `…js` (`KotlinSourceSetApplier`) and
`com.android.base` (`AndroidSourceSetApplier`): the parent tasks and the source set tasks. Last,
`tasks.withType(ConfigurableKtLintTask)` get `ktlintClasspath.from(ktlint configuration)` — custom tasks work with no
Kotlin plugin (`test-project-no-kotlin-plugin`). `org.jetbrains.kotlin.android` is not in the map: Android projects get
Android DSL source sets only.

**Extension** `kotlinter { }` (`KotlinterExtension`, plain `var`s, read lazily through `provider { }` at execution):

| property | default |
|---|---|
| `ktlintVersion: String` | `"1.8.0"` (README still shows 1.5.0) |
| `ignoreFormatFailures: Boolean` | `true` (`DEFAULT_IGNORE_FORMAT_FAILURES`) |
| `ignoreLintFailures: Boolean` | `false` (`DEFAULT_IGNORE_LINT_FAILURES`) |
| `reporters: Array<String>` | `arrayOf("checkstyle")` (`DEFAULT_REPORTER`) |

**Tasks.** `lintKotlin` ("Runs lint on the Kotlin source files.") and `formatKotlin` ("Formats the Kotlin source
files."), both group `formatting`, plain lifecycle tasks; `check` dependsOn `lintKotlin`. Per source set
`lintKotlin<Id>` (`LintTask`) and `formatKotlin<Id>` (`FormatTask`), `<Id>` = the name's first word, first char
title-cased. Root project: `installKotlinterPrePushHook` (group `build setup`, "Installs Kotlinter Git pre-push hook").
No ordering between lint and format tasks, no aggregate report task, no baseline, no `.kts`-scripts task.

**Task classes** (`org.jmailen.gradle.kotlinter.tasks`):

- `ConfigurableKtLintTask : SourceTask` — `ignoreLintFailures: Property<Boolean>` (@Input), `ktlintClasspath:
  ConfigurableFileCollection` (@Classpath), `workerJvmArgs: ListProperty<String>` (@Internal; default
  `--sun-misc-unsafe-memory-access=allow` on JDK 24+), internal `editorconfigFiles` (@InputFiles, RELATIVE, @Incremental):
  `<project dir>/.editorconfig`, then `.editorconfig` of each parent directory up to the first file with `root = true`.
  Top-level `WorkerExecutionException.hasRootCause(Class)`.
- `LintTask` — `@CacheableTask`; `reports: MapProperty<String, File>` (@OutputFiles, default empty: no reports);
  `getSource()` @InputFiles RELATIVE; `run(InputChanges)`.
- `FormatTask` — `@DisableCachingByDefault`, `outputs.upToDateWhen { false }`; `report: RegularFileProperty`
  (@OutputFile @Optional), `ignoreFormatFailures: Property<Boolean>` (@Input).
- `InstallHookTask(hookFileName)` — `gitDirPath: Property<String>` (".git"), `rootProjectDir`, abstract `hookContent`;
  `InstallPrePushHookTask`, `InstallPreCommitHookTask` (the class exists, the plugin never registers it).
- Also public: `support.ReporterType` (`checkstyle`→xml, `html`, `json`, `plain`→txt, `sarif`→sarif.json),
  `reporterFileExtension`, `support.LintFailure`, `support.KotlinterError`, `support.VersionProperties`/`versionProperties`,
  `SourceSetApplier`, `tasks.lint.LintWorkerAction`/`LintWorkerParameters`, `tasks.format.FormatWorkerAction`/`…Parameters`.
  Typed against ktlint and so not mirrorable: `reporterFor`, `reporterPathFor`, `SortedThreadSafeReporterWrapper`,
  `resolveRuleProviders`, `defaultRuleSetProviders`.

**Source sets.** Kotlin/KMP/JS: every `KotlinProjectExtension.sourceSets` entry, source = its `kotlin`
`SourceDirectorySet` (`main`, `test`, `commonMain`, `jvmMain`, …). Android: every `CommonExtension.sourceSets` entry
(`main`, `debug`, `test`, `androidTest`, flavors; not variants), `**/*.kt` under `java.directories + kotlin.directories`
(no `.kts`). Tasks are `SourceTask`s: builds use `exclude { … }`, `source(…)`, `include`. The worker skips files whose
extension is not `kt`/`kts`. The parent tasks are `tasks.register`ed once per matching plugin id, so a Kotlin plugin
plus `com.android.base` registers `lintKotlin` twice: 5.7.0 fails such a build at configuration (section 8), and so
does the drop-in.

**Engine use.** One process-isolated worker per task (`ktlintClasspath`), one static `KtLintRuleEngine(ruleProviders)`:
no `EditorConfigDefaults`, no override, so configuration is each file's `.editorconfig` chain and ktlint's defaults
(`ktlint_official`), like the CLI without `--editorconfig`. Rule sets: `ServiceLoader<RuleSetProviderV3>` over the worker
classpath, `standard` first. Custom rules: `dependencies { ktlint(project(":rules")); ktlint("g:a:v") }`.

**Lint.** Files in `source` order; `engine.lint(Code.fromFile(file))` (1.8: distinct errors sorted by line, col). Each
error: `logger.error("<file.path>:<line>:<col>: Lint error > [<rule id>] <detail>")` and every reporter. Then
`LintFailure("kotlin source <task name> failed lint check")`.

**Reporters.** `reports` name → file; the plugin sets `build/reports/ktlint/<id>-lint.<ext>`. A name outside
`ReporterType` is `IllegalArgumentException` (`valueOf`). Each reporter is ktlint's own class with constructor defaults
(plain: no grouping, no color, with its rule summary) inside `SortedThreadSafeReporterWrapper`, which replays at
`afterAll`: files sorted as strings, per file `before`, its errors, `after`. Three things follow that a plain `ktlint
--reporter` run does not do: (a) errors live in a `ConcurrentSkipListSet` ordered by (line, col) only, so **a second error
at the same position is dropped from every report** (the console still prints it); (b) paths are relative to the project
directory (`File.toRelativeString`, platform separators), except `sarif`, which gets the absolute path (ktlint's reporter
then relativizes against `user.home`); (c) file order is the string sort, not source order. And (d), observed: with
ktlint as a library its `ktlintVersion()` is null, so `sarif` has `releases/tag/null` and no `semanticVersion`/`version`.

**Format.** `engine.format(Code.fromFile(file)) { error -> … ALLOW_AUTOCORRECT }`: the 1.3+ overload whose callback runs
at emit time, in rule execution order, in every one of up to 3 passes (an unfixable error is reported again by each
further pass). Per call: `"<path>:<l>:<c>: Format fixed > [<rule>] <detail>"` at warn when `canBeAutoCorrected`, else
`"… Format could not fix > …"` at error. Changed text: `"<path>: Format fixed"` at warn, file rewritten. Any unfixable
error: `LintFailure` (same text as lint) **before the report is written**; otherwise `report` (`<id>-format.txt`) gets the
messages joined by `\n`, or `ok`.

**Failure semantics.**

| case | result |
|---|---|
| lint errors, `ignoreLintFailures = false` | task fails: `A failure occurred while executing org.jmailen.gradle.kotlinter.tasks.lint.LintWorkerAction` > `kotlin source lintKotlinMain failed lint check` |
| lint errors, `ignoreLintFailures = true` | swallowed (`hasRootCause(LintFailure)` = the class name appears in the stack trace text); task succeeds, becomes UP-TO-DATE / cacheable |
| unfixable errors in format, `ignoreFormatFailures = true` (default) | swallowed; no report file |
| same, `false` | task fails as above with `…format.FormatWorkerAction` |
| parse error, rule exception, IO | `KotlinterError("<lint|format> worker execution error while processing <file.path>: <message>")`, never ignored; the run stops at that file (reporters never reach `afterAll`: report files exist, empty) |
| `FormatTask.ignoreLintFailures` | wired from the extension, an input, never read |

**Up-to-date / caching.** `LintTask`: inputs sources (RELATIVE), `.editorconfig` chain, `ignoreLintFailures`, classpath;
outputs the report files; cacheable (FROM-CACHE restores reports); NO-SOURCE on an empty source set (`SourceTask`). It is
not incremental per file — `InputChanges` only feeds `reloadEditorConfigFile` for the worker daemon's engine cache; every
run lints all sources. `FormatTask` always runs. Config-cache compatible (upstream's TestKit runs add `--configuration-cache`).

**Git hooks.** `<rootDir>/<gitDirPath>/hooks/pre-push` (created, made executable): a new file gets `#!/bin/sh\nset -e`,
then the block `\n##### KOTLINTER HOOK START #####` / `##### KOTLINTER 5.7.0 #####` / `GRADLEW=<gradlew(.bat) path with
'/', or gradle>` / body / `##### KOTLINTER HOOK END #####\n`; an existing file gets the block appended, or replaced
between the markers. Body: `if ! $GRADLEW lintKotlin ; then echo 1>&2 "\nlintKotlin found problems, running formatKotlin;
commit the result and re-push"; $GRADLEW formatKotlin; exit 1; fi` (pre-commit: `formatKotlin`, "… had non-zero exit
status, aborting commit"). Up to date when the file contains the version line (the check itself creates the file).
Logs `Wrote hook to <file>` at quiet; "skipping hook creation because <dir> is not a directory" at warn.

## 2. Mapping onto ktrs

| piece | ktrs today | work |
|---|---|---|
| process runner, argfile, `JAVA_HOME`, `ktrs.executable` | `KtrsKtlint.kt`, `KtrsExecutable.locate()` | shared; its `[KtLint DEBUG]` logging becomes a parameter |
| ktlint version → `--ktlint-version` | `KtlintVersions.cliOption` | shared; the error text names `ktlint { version }` and the ktlint-gradle plugin: take the setting and plugin names as arguments |
| rule set JARs from a configuration | `JarServices.withoutKtlintItself`, `KtlintJars.ruleSetJars` (merge, native compose-rules, hand-off) | shared as is; the plugin wires `ktlintClasspath.from(withoutKtlintItself(ktlint))`, which matters here because `ktlint-ruleset-standard` itself declares a `RuleSetProviderV3` |
| errors of a run for the console | `--ktrs-gradle-events`, `GradleEventsReporter`, `RunErrors.kt` | lint reuses the format and reader; format needs emit-order events (below) |
| reporters | all five ported in `ktrs-cli` (incl. `html`'s hash order, `native_separators`) | new: the sorted wrapper with its (line, col) dedup, the per-reporter path flavor |
| format with a per-error callback at emit time | `KtLintRuleEngine::format(code, callback)` (ktrs-lint, "emit order, duplicates across passes included") | new CLI mode around it |
| Kotlin/Android source sets | `KotlinSourceSetsApplier.kt`, `android/AndroidPluginsApplier.kt` (same `CommonExtension` + `directories`) | new, ~40 lines each: upstream's shape differs (ids, `**/*.kt`, `com.android.base`) |
| git hook | `GitHook.kt` (ktlint-gradle's, other script) | new, 1:1 port of `GitHookTasks.kt` (no jgit there either) |
| failure nesting | `worker/ConsoleReportWorkAction` trick: a `noIsolation` `WorkAction` with upstream's FQN | same trick with `LintWorkerAction` / `FormatWorkerAction` |
| incremental lint state, baseline, `loadKtlintReporters`, report tasks | ktlint-gradle only | not needed |

**New Kotlin** (`java/gradle-plugin/src/main/kotlin/org/jmailen/gradle/kotlinter/…`, one file per upstream file, same
names and order; plus `io/github/hexay/ktrs/gradle/kotlinter/KotlinterCommand.kt`): the worker actions run `noIsolation`
and do what upstream's do, with the engine call replaced by one `ktrs ktlint` process for the task's `kt`/`kts` files.
Extra task inputs: `ktlintVersion` and the ktrs version (as `BaseKtLintCheckTask` has); `ktrs.executable` as there.

**New Rust** (`crates/ktrs-cli/src/ktlint/kotlinter.rs`, hidden option `--ktrs-kotlinter-events=<file>`, listed in
`hand_off_args`):

- lint: every `--reporter` wrapped in a port of `SortedThreadSafeReporterWrapper` (Kotlin `String` order = UTF-16 code
  units); paths relative to the working directory (= project dir) with native separators, absolute for `sarif`; events as
  today (`file` / `error` lines, engine order, no dedup, parse and rule exceptions by status).
- `--format`: `engine.format` with an allow-all callback that appends one `error` event per call (status from
  `canBeAutoCorrected`), a `formatted\t<file>` line when the text changed; no reporters. The plugin builds the console
  lines and `<id>-format.txt` from the events.
- A parse/rule exception is an `error` line with the exception's status and its `Throwable.message`, and ends the run
  as it ends upstream's worker: files are processed in parallel but their results (events, reporter calls, the
  rewritten file) are applied in order and stop there, and the reporters get no `afterAll` (empty report files). So D8's
  "ktrs still processes the rest" did not have to be taken: later files are left alone. The plugin prints the rows
  before it and throws `KotlinterError` with upstream's text.
- `sarif` in this mode is built without the manifest version (`ReporterSettings::library`).

## 3. ktlint version and output parity

kotlinter 5.7.0 runs ktlint **1.8.0** from Maven (`com.pinterest.ktlint:ktlint-rule-engine` etc., Kotlin 2.2.21 inside),
exactly ktrs's 1.8 mode (research/26), so the default maps to `--ktlint-version=1.8` with no result shift. Expected
differences are research/26's: the KDoc-whitespace rows (1 per 6,123 dev-corpus files, 3 per 15,287 held-out) and one
formatted file; a 1.8 `RunAfterRuleFilter` failure shows without the JVM stack trace.

- `ktlintVersion` is free-form upstream ("Ktlint 1.0+"); ktrs runs 1.8.0 and 2.0.0-ALPHA-4. kotlinter 5.7.0 cannot load
  2.0 at all (it hardcodes the `com.pinterest.ktlint` coordinates and packages; 2.0 is `io.github.ktlint.core`), so
  `"2.0.0-ALPHA-4"` would be a ktrs-only extra with no upstream to compare against.
- Earlier 5.x bundle older ktlint (5.0: 1.5.0, 5.1: 1.6.0; 5.2–5.6 not checked): migrating from them changes results the
  way upgrading kotlinter to 5.7.0 would.
- New parity surface: `formatKotlin`'s console and `*-format.txt` show the **raw emit order across rules and passes**.
  research/26 verified final text, sorted rows and tie order, not this stream. It is the most likely source of diffs.
- Reports: the dedup and ordering of section 1 are deterministic and portable; `html` keys are project-relative, so
  both sides hash the same strings (no `unordered` entry expected, unlike research/29's realcode).

## 4. Parity harness: `tools/kotlinter/parity.sh`

Copy of `tools/ktlint-gradle/parity.sh` (same `scenario`/`project` functions, `known_diffs.py`, output
`target/kotlinter/<scenario>/{upstream,ktrs}`, accepted diffs `tools/parity/known-diffs/kotlinter.tsv`). Upstream side:
`id 'org.jmailen.kotlinter' version '5.7.0'`. `compare.py` gets the plugin id pair as an argument instead of the
hardcoded replace; `rows.py` a second row pattern (`: Lint error > [`, `: Format (fixed|could not fix) > [`); the
"Gradle ran" probe greps `> Task :lintKotlin`. Compared per scenario: console (task blocks order-free, noise dropped),
exit status per invocation, task outcomes, every file under the project (reports, formatted sources, the hook).

| scenario | what |
|---|---|
| lint-all | 5 reporters, `lintKotlin --continue` ×2 (failed tasks rerun), sample with two errors at one position |
| lint-ignored | `ignoreLintFailures = true`, ×2 (UP-TO-DATE), then `clean` + `--build-cache` (FROM-CACHE, reports restored) |
| format | `formatKotlin --continue` ×3, default and `ignoreFormatFailures = false`; a file needing 2+ passes with an unfixable error |
| custom-tasks | `LintTask`/`FormatTask` registered by hand (reports map, `report`), `exclude { }`, no Kotlin plugin, empty source set |
| parse-error | broken file between two files with errors, lint and format |
| editorconfig | change `.editorconfig` between runs (upstream's `EditorConfigTest` cases), parent-dir `.editorconfig` |
| hook | `installKotlinterPrePushHook` ×2 on a new, an existing and an already hooked file |
| graph | `tasks --all`, `lintKotlin formatKotlin check --dry-run`, `help --task lintKotlinMain` |
| compose-maven / compose-all | `ktlint("io.nlopez.compose.rules:ktlint:0.6.7")` / `ktlint(files(…-all.jar))`: native |
| custom-rules | upstream's `test-project` (`ktlint(project(":rules"))`): jar hand-off, known diffs expected |
| kmp, android | KMP (jvm + js), upstream's `test-project-android`, and KMP + Android (testbox: needs the Android SDK) |
| realcode | research/29's template (okhttp `commonJvmAndroid` + ktlint-rule-engine, okhttp's `.editorconfig`), lint + format |

Real projects (testbox, background; pins in `tools/kotlinter/REVISIONS`; ktrs side produced by `ktrs migrate --write`, so
the migration is tested too; upstream side bumped to 5.7.0), `lintKotlin formatKotlin --continue` plus `graph`:

| project | why |
|---|---|
| navikt/mock-oauth2-server | 5.7.0, single-module JVM, `withType<LintTask> { dependsOn("formatKotlin") }` |
| JuulLabs/kable | 5.7.0 from the catalog, KMP + `com.android.kotlin.multiplatform.library` |
| LemmyNet/jerboa | 5.6.0, Android app on AGP 9, `subprojects { apply(plugin = …) }` |
| kizitonwose/Calendar | 5.3.0, KMP + `com.android.library`, `apply(plugin = libs.plugins.kotlinter.get().pluginId)` |
| line/kotlin-jdsl | catalog with two kotlinter entries (3.16.0 and 5.3.0): a `ktrs migrate` edge case |
| jeremymailen/kotlinter-gradle | its three `test-project*` builds and the CI greps on their reports |

TestKit: upstream's functional tests ported to `src/test/kotlin/org/jmailen/gradle/kotlinter/functional/` (11 classes,
about 45 tests, all with `--configuration-cache`). Not portable: `RuleSetsTest`, `ReportersTest`,
`SortedThreadSafeReporterWrapperTest` (ktlint types; the wrapper's cases become Rust unit tests), the worker-JVM half of
`WorkerJvmArgsTest`; Android classes only where an SDK exists. Rust: `crates/ktrs-cli/tests/ktrs_kotlinter.rs`.

## 5. Naming and packaging

- Plugin id **`io.github.hexay.ktrs.kotlinter`**, implementation class `org.jmailen.gradle.kotlinter.KotlinterPlugin`
  (builds do `apply<KotlinterPlugin>()`; `ktrs-project` already maps that type name).
- Third `gradlePlugin { plugins { create("ktrsKotlinter") { … } } }` entry in the existing
  **`io.github.hexay:ktrs-gradle-plugin`** artifact: no new module, publication or release step. The jar then carries a
  root resource `kotlinter.properties` and the `org.jmailen.*` classes, so it cannot share a build classpath with the
  real kotlinter (same as the two existing drop-ins and their originals).
- Plugin Portal: `publishPlugins` uploads every declared id; a new id's first version goes through the Portal's manual
  review even under an approved prefix. Unknown until tried: whether that holds the other two ids' new version in the
  same upload. The GitHub Pages Maven repo gets the marker at once either way. Plan the first release with the new id
  as one where a delayed Portal listing is acceptable, or publish the id in a patch release of its own.
- Also to touch: `release.yml`'s comment, the `java/gradle-plugin` line of CLAUDE.md, the header comment of
  `java/gradle-plugin/build.gradle.kts`, `parity.yml` (sample scenarios).

## 6. `ktrs migrate` and README

`crates/ktrs-project/src/migrate/coords.rs`: `GRADLE_PLUGINS` becomes `[PluginSwap; 3]` with `{ old_id:
"org.jmailen.kotlinter", new_id: "io.github.hexay.ktrs.kotlinter", old_artifact: "org.jmailen.gradle:kotlinter-gradle" }`;
the `KOTLINTER` advice and its `NO_DROP_IN` row go (`[…; 5]`). Ids, `version` literals, markers, `buildscript` and
convention-build coordinates and the catalog then follow from the existing generic code (`gradle_plugins::rewrite`,
`catalog_edits`). New notes, next to `check_ktlint_version` in `gradle_plugins.rs`:

- `kotlinter { ktlintVersion = "x" }` with x outside `VERSIONS_2_0_AND_1_8`: "the kotlinter drop-in runs only …".
- replaced plugin version below 5: the drop-in has the 5.x DSL; name `failBuildWhenCannotAutoFormat` → `ignoreFormatFailures`
  (inverted), `ignoreFailures` → `ignoreLintFailures`, and 4.x's `buildscript` classpath rule sets → `ktlint(…)`.
- `ktlint(…)` dependencies that are neither compose-rules nor a project: "runs through the real ktlint jar".

`crates/ktrs-project/src/gradle/findings.rs`: `KOTLINTER_PLUGINS` gains the new id (so `ktrs lsp` still detects a
migrated build). `migrate/mod.rs` docs drop "kotlinter". Tests: `tests/migrate.rs` `kotlinter-groovy` turns from a note
into a swap (regenerate with `UPDATE_MIGRATED=1`), new fixtures for Kotlin DSL + catalog + the notes;
`crates/ktrs-cli/tests/ktrs_migrate.rs`'s note test needs another no-drop-in fixture (ktlint from its jar).

README: delete the "kotlinter runs ktlint inside the JVM…" limit; "Migrating" lists kotlinter among the rewritten setups
and takes another example of a note; under "Gradle", after the ktlint-gradle paragraph:

> **kotlinter drop-in.** The `io.github.hexay.ktrs.kotlinter` plugin replaces
> [kotlinter](https://github.com/jeremymailen/kotlinter-gradle) 5.7.0: the `kotlinter { }` block,
> `lintKotlin`/`formatKotlin` and the per-source-set tasks, `installKotlinterPrePushHook`, `ktlint(...)` rule sets and the
> `org.jmailen.gradle.kotlinter.*` types keep working. `ktlintVersion` defaults to `"1.8.0"` (research/34).

plus the one-line id swap snippet and the version / rule set sentence already used for ktlint-gradle.

## 7. Size, risks, decisions

**Size.** About half of research/29's plugin: Kotlin main ~700 lines (upstream's 1,000 minus engine, reporter and rule
set code), TestKit port ~1,500 of upstream's 2,200 test lines, Rust ~250 + tests, harness ~250 (mostly blocks and
samples), migrate ~60 + fixtures. No engine or rule work.

**Planned deviations.** `ktlint` gets no ktlint artifacts (nothing resolved from Maven; `ktlintClasspath` = rule sets);
`workerJvmArgs` accepted, unused; `.editorconfig` cache reset is moot (new process per run); the ktlint-typed public
functions are absent; non-native rule sets go through the jar hand-off (lint rows without dedup or kotlinter's path
flavors, format without the "Format fixed >" rows); plugin compiled for Java 17 (upstream 11, Gradle 8.4+); the plugin id.

**Risks.**

1. Emit-order stream of format (section 3): first use of that ordering as output.
2. KMP + Android registration (section 1): unverified upstream behaviour; port what the `kmp+android` scenario shows.
3. Wrapper details: same-position dedup keeps the first error in engine order; `sarif` keyed by absolute path while
   `before`/`after` use the relative one; all to be pinned by `lint-all`.
4. Parse-error abort: upstream leaves empty report files and unlinted later files; mirrored only in console and failure.
5. `ktlintVersion` pins other than 1.8.0 are common in the wild (README sample is 1.5.0): each is a failed build after
   migration unless the user edits it; `ktrs migrate` must say so.
6. Portal review of the new id (section 5).
7. Stdout/stderr interleaving of warn and error lines may differ from a forked worker's forwarding; `compare.py`
   compares task blocks, so only within-block order is exposed.

**Decisions for the owner.**

| # | question | recommended |
|---|---|---|
| D1 | Build it, reversing the 2026-10-03 "No kotlinter"? | Yes: the smallest of the plugin drop-ins, and upstream's default ktlint is exactly 1.8 mode |
| D2 | Id and artifact | `io.github.hexay.ktrs.kotlinter`, inside `ktrs-gradle-plugin` |
| D3 | Which kotlinter to mirror | 5.7.0 only; 4.x DSL users get `ktrs migrate` notes |
| D4 | `ktlintVersion` other than 1.8.0 | `2.0.0-ALPHA-4` accepted as an extra, anything else fails naming both (same rule as the settled ktlint-gradle one), not warn-and-run-1.8 |
| D5 | Rule sets other than compose-rules | Jar hand-off with the documented output deviations, not a build failure |
| D6 | Add ktlint's artifacts to `ktlint` like upstream | No: offline-safe, nothing to resolve; document `ktlintClasspath` = rule sets |
| D7 | Hook marker `##### KOTLINTER <version> #####` | Upstream's `5.7.0` (hook bytes identical, an installed upstream hook is up to date), not the ktrs version |
| D8 | Parse/rule exception | Upstream's message and abort point on the console; ktrs still processes the rest |
| D9 | Real projects in the gate | Sample scenarios in `parity.yml`; the six real projects on the testbox per release, not nightly |
| D10 | Portal timing | Ship the new id in its own patch release, after one green testbox run |

## 8. Observed on kotlinter 5.7.0 (testbox, Linux, Gradle 9.8.0, JDK 21, 2026-10-09)

The upstream side of `tools/kotlinter/parity.sh` (`SIDES=upstream`), then both sides.

| open point | what 5.7.0 does |
|---|---|
| KMP + `com.android.library` (AGP 8.13.2, Gradle 8.14.3) | configuration fails: `Cannot add task 'lintKotlin' as a task with that name already exists.` The drop-in, same code, fails with the same text. kizitonwose/Calendar (on 5.3.0) fails this way on both sides once it is on 5.7.0: for such a build neither kotlinter 5.7.0 nor the drop-in is usable |
| KMP + `com.android.kotlin.multiplatform.library` (AGP 9.2.1) | works: one `lintKotlin`, tasks per Kotlin source set (`lintKotlinAndroidMain`, …). JuulLabs/kable is this shape |
| `com.android.library` (AGP 9, built-in Kotlin) | works; tasks per Android DSL source set, most of them NO-SOURCE. An empty directory as `ANDROID_HOME` is enough: lint and format never touch the SDK |
| empty source set | NO-SOURCE for `LintTask` and `FormatTask`, plugin-registered and hand-registered (`SourceTask`'s `@SkipWhenEmpty` survives `LintTask.getSource()`'s override); no report file |
| source with only non-Kotlin files beside Kotlin ones | skipped silently; the reporters' `before` without `after` leaves no trace in any of the five reports |
| `formatKotlin` emit order | rule execution order within a pass, pass after pass; an unfixable error once per pass (twice when the first pass fixed something). ktrs's `engine.format` callback stream is the same, row for row, on the samples and on realcode (181 files, 4,927 console rows over lint + format) |
| lint rows | every error, two or three at one position included; the reports keep the first per position |
| `sarif` / `html` of ktlint 1.8.0 | `html` as ported. `sarif` differs from the CLI's: (d) of section 1. Fixed in kotlinter mode |
| failure text | `Execution failed for task ':lintKotlinMain'.` > `There was a failure while executing work items` > `A failure occurred while executing …LintWorkerAction` > `kotlin source lintKotlinMain failed lint check`; a parse error: `lint worker execution error while processing <file>: 4:11 Expecting ')'`, one line, no separate cause line. In-process (`noIsolation`) work prints the same |
| parse error | rows of the files before it in source order, none after, the five report files empty; `formatKotlin` leaves later files unformatted |
| console grouping | a task's rows come under `> Task :x`, with `> Task :x FAILED` after them when the worker outlives Gradle's grouping window, else under one `> Task :x FAILED`: timing, merged by `compare.py` |

## 9. Built

- Rust: `crates/ktrs-cli/src/ktlint/kotlinter.rs` (`--ktrs-kotlinter-events`, `SortedThreadSafeReporterWrapper`, tests
  `kotlinter/tests.rs` and `tests/ktrs_kotlinter.rs`); `hand_off_args` turns the events file into a `json` report.
- Kotlin: `org/jmailen/gradle/kotlinter/**` (one file per upstream file), `io/github/hexay/ktrs/gradle/kotlinter/
  KotlinterCommand.kt`, `resources/kotlinter.properties`; TestKit port in `src/test/kotlin/org/jmailen/gradle/kotlinter/`
  (13 classes: upstream's functional tests incl. Android and Kotlin/JS, `GenerateHookTest`, `WorkerJvmArgsTest`, and
  `KtlintVersionTest` for D4). Not ported: `RuleSetsTest`, `ReportersTest`, `SortedThreadSafeReporterWrapperTest`
  (Rust), "workerJvmArgs are passed to the worker jvm" (replaced: accepted and unused).
- Harness: `tools/kotlinter/parity.sh` (samples), `real.sh` + `REVISIONS` (real projects through `ktrs migrate
  --write` and a locally published plugin); `tools/ktlint-gradle/compare.py` and `rows.py` shared. `compare.py` used to
  skip every directory named `kotlin`, `src/main/kotlin` included, so formatted sources were not compared; fixed (the
  ktlint-gradle scenarios stay identical with sources compared).
- `ktrs migrate`: the swap and the three notes (`migrate/kotlinter_notes.rs`); `ktrs lsp` detects the new id.

Deviations beyond section 7's: a hand-off run gets `--relative`, so its plain/checkstyle/json/html reports have
kotlinter's project-relative paths (`sarif` then too, where upstream's is home-relative); "Editorconfig changed,
resetting KtLint caches" is still logged at info, with nothing to reset.

## 10. Verification (testbox, 2026-10-10)

- `cargo test -p ktrs-cli`: all pass. `cargo test -p ktrs-project --test migrate`: 15 pass, `UPDATE_MIGRATED=1` leaves
  the committed `migrated/` trees unchanged. `cargo test -p ktrs-project`: all pass (but see open items). Rerun after
  the rebase onto master 53a27d2, with the plugin tests and the sample scenarios.
- `java/gradlew -p java :ktrs-gradle-plugin:test`: 229 tests, 0 failed, 1 skipped (53 of them the kotlinter port, Android
  classes included, with an empty `ANDROID_HOME`).
- `tools/kotlinter/parity.sh`: lint-all, lint-ignored, format, format-strict, custom-tasks, parse-error, editorconfig,
  hook, graph, compose-maven, compose-all, kmp, android, kmp-android, realcode identical (console, exit codes,
  outcomes incl. UP-TO-DATE / FROM-CACHE / NO-SOURCE, reports, sources, the hook file); custom-rules identical but for
  the accepted hand-off difference (`tools/parity/known-diffs/kotlinter.tsv`); kmp-android-library: the same
  configuration failure on both sides.
- `tools/kotlinter/real.sh`: mock-oauth2-server, kable, jerboa identical (all three are clean: every task succeeds or
  is NO-SOURCE, no rows); kotlinter's `test-project` identical but for the same hand-off difference; Calendar: the
  identical configuration failure of section 8.

Open items:

- line/kotlin-jdsl: its catalog is `./libs.versions.toml`, which `ktrs migrate` does not read; it swaps the
  `apply(plugin = …)` id and notes that no version was found, and the migrated build has no plugin. Not run.
- kotlinter's `test-project-android` and `-no-kotlin-plugin`: covered by the android and custom-tasks samples only.
- Windows and macOS parity runs; ktlint `2.0.0-ALPHA-4` under the plugin beyond `KtlintVersionTest`; a rule exception
  (only parse errors were provoked); a real project with lint errors (the pinned ones are clean, realcode stands in).
- `cargo test -p ktrs-project --test gradle` fails one old test (`binary_convention_plugin_with_helper_function`,
  detection returns nothing) when the checkout sits in a directory named `src`; the same tree passes from a directory
  named otherwise. Not looked into.
- Plugin Portal behaviour for an upload that adds an id (D10).
