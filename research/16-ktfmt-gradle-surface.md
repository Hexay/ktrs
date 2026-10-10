# ktfmt-gradle 0.27.0 public surface (for `io.github.hexay.ktrs`)

Status 2026-10-10: ktrs formats as ktfmt 0.65 everywhere; ktfmt-gradle 0.27.0 bundles ktfmt 0.64. The drop-in keeps
this plugin's DSL, tasks and FQNs, but its output is 0.65's: the first `ktfmtFormat` after switching is the ktfmt
0.64 to 0.65 diff (README "ktfmt version").

Source: `cortinico/ktfmt-gradle` tag `0.27.0` (latest tag, 2026-08-03; commit a37dda0), cloned `--depth 1`.
Paths below: `G/` = `plugin-build/plugin/src/main/java/com/ncorti/ktfmt/gradle/`,
`T/` = `plugin-build/plugin/src/test/java/com/ncorti/ktfmt/gradle/`. Binary API dump: `plugin-build/plugin/api/plugin.api`.

## 1. Plugin, versions, source discovery

- Id `com.ncorti.ktfmt.gradle`, class `com.ncorti.ktfmt.gradle.KtfmtPlugin`, maven `com.ncorti.ktfmt.gradle:plugin`
  (`plugin-build/gradle.properties:1-8`). Declares configuration-cache compatible (`plugin-build/plugin/build.gradle.kts:82`).
- Built with: Kotlin 2.4.10 (apiVersion/languageVersion 2.0, jvmTarget 17), Gradle 9.6.1 wrapper, AGP 9.3.1 compileOnly,
  ktfmt 0.64 compileOnly non-transitive (`gradle/libs.versions.toml:1-11`, `plugin/build.gradle.kts:12-25,46-59`).
  No explicit minimum-Gradle check anywhere. Runtime needs JDK 17+ (bytecode 17; README's "JDK 11" is stale).
  Kotlin Gradle plugin compat is tested for 1.7.20, 1.9.10, 1.9.20, 2.0.0 (`T/PluginVersionCompatibilityTest.kt:24`).
  README's "AGP 4.1+" is stale: code uses `com.android.build.api.dsl.CommonExtension.sourceSets` and
  `AndroidSourceDirectorySet.directories` (`G/KtfmtAndroidUtils.kt:36-42`).
- `apply` (`G/KtfmtPlugin.kt:30-82`):
  - creates extension `ktfmt` (31); a resolvable `ktfmt` configuration (usage java-runtime, not visible/consumable) with
    `com.facebook:ktfmt:0.64` added from the bundled resource `ktfmt-version.txt` (34-51; resource written by
    `persistKtfmtVersion`, `plugin/build.gradle.kts:98-107`). No DSL to change the ktfmt version.
  - `tasks.withType(KtfmtBaseTask).configureEach` wires `ktfmtClasspath`, `formattingOptionsBean`
    (= `ktfmtExtension.toFormattingOptions()`, eagerly `.get()`s every extension property at task config time),
    `useClassloaderIsolation` (53-57). Applies to user-registered tasks of these types too.
  - registers `ktfmtFormat`/`ktfmtCheck` (59-60) and the scripts tasks (62) unconditionally, before any Kotlin plugin.
  - reacts to: `kotlin` id (= kotlin-jvm) (64), `org.jetbrains.kotlin.js` (78) -> `KotlinProjectExtension.sourceSets`,
    each `kotlin.sourceDirectories` (84-96); `org.jetbrains.kotlin.multiplatform` (79-81, 98-138);
    `com.android.application|library|test|dynamic-feature` (73-76) -> Android path, **skipped if KMP is already
    applied at that moment** (66-72; order-dependent).
- KMP (`G/KtfmtPlugin.kt:98-138`): every `KotlinSourceSet` becomes task suffix `"kmp <name>"` ->
  `ktfmtCheckKmpCommonMain` etc. Source sets whose name starts with `android` are skipped unless
  `com.android.kotlin.multiplatform.library` is applied (107-116; that plugin's android target is `KotlinPlatformType.jvm`,
  so its source sets go through the normal KMP path). For each target with `platformType == androidJvm` (legacy
  `com.android.library` + KMP) it runs the Android path with `isKmpProject = true` -> `ktfmtCheckKmpMain`, `KmpDebug`, ...
- Android (`G/KtfmtAndroidUtils.kt:12-43`): `extensions.findByName("android") as? CommonExtension`;
  `sourceSets.configureEach` -> dirs = `kotlin.directories + java.directories` wrapped in `project.files(Callable)`.
  This is the DSL source-set container, so names are `main`, `debug`, `release`, `test`, `testDebug`, `androidTest`,
  `androidTestDebug`, `testFixtures`, `testFixturesDebug`, ... (`T/KtfmtPluginTest.kt:89-175`). README's
  `...JavaSource` naming is stale. Sets extra property `ktfmt.android.tasks.created` (19-22) but nothing reads it.
- Per-source-set creation (`G/KtfmtPluginUtils.kt:35-64`):
  - skipped names: `generatedByKsp*`, `aot`, `aotTest` (22-32; checked on the raw name, so `"kmp generatedByKsp..."` is not skipped).
  - task name = `ktfmtCheck`/`ktfmtFormat` + name split on spaces, each word's first char uppercased, joined (115-123, 140-148).
  - source = `provider { srcDir.toList().filterNot { it.absolutePath.matches(srcSetPathExclusionPattern.get()) } }`
    (159-168): the regex is matched against each **source directory's absolute path** (whole-string `matches`), not files.
  - includes `**/*.kt`, `**/*.kts` (`G/KtfmtPlugin.kt:156`), so `.java` in Android java dirs are ignored.
  - every `org.jetbrains.kotlin.gradle.tasks.KotlinCompile` `mustRunAfter(check, format)` (52-54);
    aggregate `ktfmtFormat`/`ktfmtCheck` `dependsOn` it (56-57); `check` dependsOn the check task when
    `LifecycleBasePlugin` is applied (59-63).
- Scripts (`G/KtfmtPluginUtils.kt:66-107`): `ktfmtCheckScripts`/`ktfmtFormatScripts`, source =
  `fileTree(projectDir).filter { ext == "kts" && parentFile == projectDir }` — only top-level `*.kts` (includes the
  module's own `build.gradle.kts`/`settings.gradle.kts`). Same mustRunAfter / aggregate / `check` wiring.
  No other `.kts` handling: `.kts` inside source dirs go through the normal tasks. `Formatter.format(options, code)` gets
  no file name; ktfmt parses everything as a script anyway (`crates/ktrs-fmt/src/format/parser.rs:1`).

## 2. `ktfmt { }` extension (`G/KtfmtExtension.kt`)

Abstract class, managed properties, conventions set in `init` (8-17):

| member | type | default |
|---|---|---|
| `maxWidth` (20) | `Property<Int>` | 100 |
| `blockIndent` (32) | `Property<Int>` | 2 |
| `continuationIndent` (44) | `Property<Int>` | 4 |
| `trailingCommaManagementStrategy` (51) | `Property<com.ncorti.ktfmt.gradle.TrailingCommaManagementStrategy>` (`NONE`, `ONLY_ADD`, `COMPLETE`; `G/TrailingCommaManagementStrategy.kt`) | `ONLY_ADD` |
| `removeUnusedImports` (54) | `Property<Boolean>` | true |
| `srcSetPathExclusionPattern` (63) | `Property<kotlin.text.Regex>` | `^(.*[\\/])?build([\\/].*)?$` (128-129) |
| `debuggingPrintOpsAfterFormatting` (69) | `Property<Boolean>` | false |
| `useClassloaderIsolation` (75) | `Property<Boolean>` | false (= process isolation) |
| `manageTrailingCommas` (77-88) | `var Boolean`, **@Deprecated** | setter: true->COMPLETE, false->NONE; getter: strategy != NONE |
| `googleStyle()` (92-96) | fun | blockIndent 2, continuationIndent 2, trailing COMPLETE |
| `kotlinLangStyle()` (103-107) | fun | blockIndent 4, continuationIndent 4, trailing COMPLETE |

- Defaults = ktfmt's Meta style (2/4/ONLY_ADD). There is **no `metaStyle()`** and no `dropboxStyle()` (removed 0.19.0,
  `CHANGELOG.md`). Style methods do not touch maxWidth/removeUnusedImports. Groovy DSL works via the generated setters
  (`maxWidth = 80`), Kotlin DSL via `.set(..)` or `=` assignment.
- `toFormattingOptions()` (internal, 109-117) -> public `FormattingOptionsBean` data class, `Serializable`
  (`G/FormattingOptionsBean.kt`, fields maxWidth, blockIndent, continuationIndent, trailingCommaManagementStrategy,
  removeUnusedImports, debuggingPrintOpsAfterFormatting). It is a task `@Input`, so it is part of the cache key.
- Other public API (`plugin.api`): `KtfmtPlugin`, `KtfmtBaseTask`/`KtfmtCheckTask`/`KtfmtFormatTask`,
  `util.KtfmtResultSummary` (valid/invalid/skipped/failed file lists + `prettyPrint()`), the enum, the bean.

## 3. Tasks

| task | type | group | description |
|---|---|---|---|
| `ktfmtFormat` | plain `Task` | `formatting` | `Run Ktfmt formatter for all source sets for project '<p>'` (`G/KtfmtPlugin.kt:140-145`) |
| `ktfmtCheck` | plain `Task` | `verification` | `Run Ktfmt validation for all source sets for project '<p>'` (147-153) |
| `ktfmtCheck<Set>` | `KtfmtCheckTask` | `verification` | `Run Ktfmt formatter validation for sourceSet '<name>' on project '<p>'` (`<name>` includes `kmp ` prefix) |
| `ktfmtFormat<Set>` | `KtfmtFormatTask` | `formatting` | `Run Ktfmt formatter for sourceSet '<name>' on project '<p>'` |
| `ktfmtCheckScripts` | `KtfmtCheckTask` | `verification` | `Run Ktfmt formatter validation for script files on project '<p>'` |
| `ktfmtFormatScripts` | `KtfmtFormatTask` | `formatting` | `Run Ktfmt formatter for script files on project '<p>'` |

`KtfmtBaseTask(layout: ProjectLayout) : SourceTask` (`G/tasks/KtfmtBaseTask.kt`), `@DisableCachingByDefault` (37):
- `ktfmtClasspath: ConfigurableFileCollection` `@Classpath @InputFiles` (45)
- `formattingOptionsBean: Property<FormattingOptionsBean>` `@Input` (47)
- `includeOnly: Property<String>` `@Input`, `@Option("include-only", "A comma separate list of relative file paths to include
  exclusively. If set the task will run the processing only on such files.")`, convention `""` (49-56, 41-43)
- `useClassloaderIsolation: Property<Boolean>` `@Input` (58); `processIsolationJvmArgs: ListProperty<String>` `@Input` (60;
  new in 0.27.0, no convention, not on the extension — set per task: `tasks.withType<KtfmtBaseTask> { processIsolationJvmArgs... }`)
- `getSource()` `@InputFiles @PathSensitive(RELATIVE) @IgnoreEmptyDirectories @SkipWhenEmpty` (62-66) -> `NO-SOURCE` when empty
- `output: Provider<RegularFile>` = `build/ktfmt/<taskName>/output.txt` (68-72); `reformatFiles: Boolean` `@Internal` (76)
- protected abstract `handleResultSummary(KtfmtResultSummary)`.

`KtfmtCheckTask` (`G/tasks/KtfmtCheckTask.kt`): `@CacheableTask` (14), `output` is `@OutputFile` (22) -> UP-TO-DATE /
FROM-CACHE work. `KtfmtFormatTask` (`G/tasks/KtfmtFormatTask.kt`): `@DisableCachingByDefault(because = "Formatting tasks
modify source files and should not be cached")` (14), `output` `@Internal` (23) -> no declared outputs, so it always runs.
Neither is incremental (`InputChanges` unused): every source file is submitted every run.

Action (`KtfmtBaseTask.kt:83-99`): results go to `temporaryDir/<uuid>/`, one file per work item, then:
1. failures first (155-164): `error("Ktfmt failed to run with N failures:\n<paths relative to projectDir, \n-joined>")`
   -> task FAILED, summary file not written.
2. writes `output.txt` = `KtfmtResultSummary.prettyPrint()` (`G/util/KtfmtResultSummary.kt:12-21`):
   ```
   Ktfmt Summary:
     - Valid formatted files: N
     - Invalid formatted files: N
     - Skipped files: N
     - Failed files: N
   ```
3. check (`KtfmtCheckTask.kt:26-39`): if any unformatted -> `error("[ktfmt] Found N files that are not properly formatted:\n<relpaths>")`;
   else `info("[ktfmt] Successfully checked N files with Ktfmt")` (N = valid count).
   format (`KtfmtFormatTask.kt:27-36`): `info("[ktfmt] Successfully reformatted N files with Ktfmt")` if any changed, else
   `info("[ktfmt] All files (N) are already correctly formatted")`.

Per-file log lines (`G/tasks/worker/KtfmtWorkAction.kt`; all prefixed `[ktfmt] `, `G/util/KtfmtLogger.kt:5-25`):
- INFO `Skipping format for <abs file> because it is not included` (63); DEBUG `Checking format for <file>` (67)
- INFO `Valid formatting for: <file>` (75); format: INFO `Reformatting <file>` (80)
- check: **ERROR** `Invalid formatting for: <file>` (85), then per diff delta INFO `<file>:<line> - <msg>` where msg is
  `Line changed: <first original line>` | `Line deleted` | `Line added` (java-diff-utils, `G/util/KtfmtDiffer.kt:12-32`;
  line = delta source position + 1, diff is on whole strings split by lines)
- failure: ERROR `Failed to format file: <file> (reason = <exception message>)` + DEBUG with stack trace (89-93).

`--include-only` (`G/tasks/IncludedFilesParser.kt:5-15`): split on `,` and `:`, strip one leading `/` or `\`, `\`->`/`,
resolve against **projectDir**, `canonicalFile`; a source file not in the set is `Skipped` (`KtfmtWorkAction.kt:103-107`).
Empty/blank = everything. README's pre-commit recipe: register your own `KtfmtFormatTask` with
`source = fileTree(rootDir); include("**/*.kt")` and pass `--include-only=a.kt:b.kt` (`README.md:133-156`).

## 4. How ktfmt is invoked

- Worker API, queue per task (`KtfmtBaseTask.kt:101-125`): `processIsolation { classpath(ktfmtClasspath);
  forkOptions.jvmArgs(processIsolationJvmArgs) }` by default, `classLoaderIsolation` when `useClassloaderIsolation`.
  **One `WorkAction` per source file** (114-121), params: `sourceFile`, `formattingOptions` (bean), `includedFiles`,
  `resultDirectory`, `reformatFiles` (`KtfmtWorkAction.kt:42-48`); `queue.await()`.
- Mapping (`KtfmtWorkAction.kt:109-126`): `FormattingOptions(maxWidth, blockIndent, continuationIndent,
  trailingCommaManagementStrategy (enum 1:1), removeUnusedImports, debuggingPrintOpsAfterFormatting)` by named args;
  any other ktfmt field (e.g. `preserveLambdaBreaks`) takes ktfmt's constructor default. Call is
  `Formatter.format(options, sourceFile.readText())` (72) — no file name, no `.editorconfig`.
- Compare `original == formatted` (74). Format writes with `writeText(formatted, Charset.defaultCharset())` (81) while it read
  UTF-8 (gotcha: non-UTF-8 platform default). CRLF input: ktfmt keeps line endings, so equality semantics follow ktfmt.
- Any exception (parse error etc.) -> `KtfmtFormatFailure`, file untouched; other files still processed
  (`T/tasks/KtfmtFormatTaskIntegrationTest.kt:193-211`). Result serialized as `status,wasCorrectlyFormatted,path`
  (`G/tasks/worker/KtfmtFormatResult.kt:5-40`; breaks on paths containing `,` — upstream bug).

## 5. Other features

- No git/pre-commit hook helper beyond `--include-only` (above). No ktfmt-version DSL (fixed by plugin release; the
  `ktfmt` configuration is public-ish, so `dependencies { ktfmt("com.facebook:ktfmt:x") }` would conflict-resolve upward).
- Scripts tasks exist even with no Kotlin plugin (changelog 0.26.0; `T/KtfmtPluginTest.kt:15-26`).

## 6. Upstream tests (JUnit 5 + Truth)

ProjectBuilder unit tests: `T/KtfmtPluginTest.kt` (plain project; kotlin jvm; kotlin multiplatform; android application —
exact task names + aggregate `dependsOn` lists), `T/KtfmtExtensionTest.kt` (defaults; googleStyle/kotlinLangStyle;
toFormattingOptions), `T/tasks/KtfmtBaseTaskTest.kt` (output path `build/ktfmt/ktfmtFormatMain/output.txt`; regular files
not excluded), `T/tasks/IncludedFilesParserTest.kt` (separator normalization; comma+colon), `T/util/KtfmtPluginUtilsTest.kt`
(shouldCreateTasks main/KSP/Spring), `KtfmtDifferTest`, `KtfmtLoggerKtTest`, `worker/KtfmtFormatResultTest`.

TestKit functional tests (`GradleRunner.withPluginClasspath()`, fixture `src/test/resources/jvmProject` =
`kotlin("jvm") version "1.7.20"` + `ktfmt { kotlinLangStyle() }`; files default to `src/main/java/TestFile.kt`):

`T/tasks/KtfmtCheckTaskIntegrationTest.kt`: check task fails if there is invalid code · fails if there is not formatted code
(`[ktfmt] Invalid formatting`) · fails if ktfmt fails to parse the code · succeed if code is formatted · runs before
compilation (`--dry-run` order) · prints formatted files with --info · format task uses configuration cache correctly ·
validates all the file with a failure · skips a file if with --include-only · can check the formatting of multiple files
(10/15/30/50/100/1000) · is cacheable (`clean` + `--build-cache` -> FROM_CACHE) · up-to-date when invoked twice with
multiple different sized sourceSets · is configuration cache compatible · custom formatCheck task compatible with
configuration cache · detects source and test files in a flattened project structure · by default ignores sourceSets in
the build folder (NO_SOURCE) · does not ignore build folder with a custom exclusion pattern · ignores main sourceSets when
given as exclusion pattern · check scripts validates top-level script · ignores non top-level scripts · scripts task on a
project without any kotlin plugins.

`T/tasks/KtfmtFormatTaskIntegrationTest.kt`: format task fails if there is invalid code · formats correctly · succeeds if code
is formatted · succeeds after subsequent execution · succeeds after subsequent execution when formatting · executed again
after edit · prints formatted files with --info · uses configuration cache correctly · reformats all files even with a
failure · runs before compilation · skips a file with --include-only · can format multiple files (parameterized) · custom
format task compatible with configuration cache · kotlinLang style 4-space · googleStyle 2-space · flattened project
structure · by default does not format sourceSets in build folder · custom exclusion pattern keeps build folder · main
excluded by pattern · format scripts fails on unparsable top-level script · formats top-level script · does not format
non top-level script · formats top-level script on project without kotlin plugins.

`T/PluginVersionCompatibilityTest.kt`: applies to kotlin-jvm 1.7.20/1.9.10/1.9.20/2.0.0 and runs `ktfmtCheckMain`.

## Porting notes

- **Binary/DSL compatibility**: users write `import com.ncorti.ktfmt.gradle.tasks.*`, `KtfmtFormatTask`,
  `TrailingCommaManagementStrategy.NONE`, `tasks.withType<KtfmtBaseTask>`. A drop-in with a new id must decide whether
  to keep the `com.ncorti.ktfmt.gradle` package/class names (source-compatible) or re-home them (scripts need edits).
  Keep `ktfmtClasspath`/`useClassloaderIsolation`/`processIsolationJvmArgs` as accepted-but-ignored inputs so existing
  scripts still configure.
- **Options -> `ktrs serve`**: send all five explicit keys (`max-width`, `block-indent`, `continuation-indent`,
  `trailing-commas`, `remove-unused-imports`) with `style=meta`; `editorconfig=false` (the plugin never reads it).
  `debuggingPrintOpsAfterFormatting` has no serve key — ignore or warn. Our `preserve_lambda_breaks` must stay false.
- **Error/log wording** is tested by substring/regex: keep `[ktfmt] ` prefixes, `Invalid formatting for: `,
  `Failed to format file: <f> (reason = ...)`, `Ktfmt failed to run with N failures`, `Successfully checked/reformatted`,
  failures reported before unformatted files; the check's diff lines are INFO. Map a serve `status=error` to reason text.
- **Configuration cache**: extension values are read inside `configureEach` into a bean; source dirs via
  `provider {}`/`files(Callable)`; tasks take `ProjectLayout` by injection — no `Project` at execution. Keep a long-lived
  serve process in a `BuildService` (not a task field) to be CC-safe; per-file WorkActions are unnecessary — batch the
  whole source set through one serve connection (or a pool) inside the task action.
- **Caching**: keep check `@CacheableTask` with `@OutputFile output.txt` and path-sensitive RELATIVE sources, and format
  with no outputs (always runs). The ktrs binary identity should be an `@Input` (replaces `@Classpath ktfmtClasspath`).
- **Android**: AGP 9 removed the old variant APIs; upstream only uses DSL `CommonExtension.sourceSets` +
  `kotlin.directories`/`java.directories`, compiled against AGP 9.3.1 (`CommonExtension` non-generic there) — this also
  covers AGP 9 built-in Kotlin. Task set includes variant/build-type source sets (Debug, Release, TestDebug, ...).
  Order quirk: `hasPlugin(kmp)` is checked when the Android plugin is applied (`KtfmtPlugin.kt:66-72`).
- **KMP**: `kmp ` prefix on names; android-prefixed KMP source sets skipped unless the android-KMP-library plugin is
  present; legacy android target adds `ktfmtCheckKmpMain`-style tasks from the Android DSL.
- **Exclusion regex** is matched against source *directories'* absolute paths with Windows `\`; keep `Property<Regex>`
  (Kotlin `Regex`, so Groovy users need `kotlin.text.Regex`).
