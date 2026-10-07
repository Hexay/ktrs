# 31 — ktlint-maven-plugin drop-in: `io.github.hexay:ktrs-ktlint-maven-plugin` (2026-10-06)

Mirrors **gantsign `com.github.gantsign.maven:ktlint-maven-plugin` 3.7.1** (tag `3.7.1`, commit `5856cc39`; landscape:
research/30). Switching is changing `groupId`/`artifactId`/`version` of the `<plugin>`; goals, phases, parameters,
`ktlint.*` properties, reporters, log lines and failure messages stay.

## Shape

- `java/maven-plugin` (Gradle, `org.gradlex.maven-plugin-development` 1.0.3, vanniktech publish, `packaging=maven-plugin`,
  Maven API 3.9.0 compileOnly, Java 11). `goalPrefix` `ktlint`: `mvn ktlint:check|format|ktlint|help`.
- 1:1 port in Java, upstream names: `AbstractBaseMojo`, `CheckMojo`, `FormatMojo`, `KtlintReport`, `ReporterConfig`
  (Kotlin data-class `equals`/`hashCode`, which order the `<reporters>` set), `MavenLogReporter[Provider]`,
  `internal/{AbstractLintSupport, AbstractCheckSupport, Check, Format, Report, Sources, ModelReporter,
  KtlintReportGenerator, ...}`. Package `io.github.hexay.ktrs.maven.ktlint`.
- One extra parameter: `ktlintVersion` (property `ktrs.ktlintVersion`, default `1.8.0`; `2.0.0-ALPHA-4` = 2.0 mode; else
  the build fails naming both). Not `ktlint.version`: Maven resolves `property=` from project properties too, and poms
  already use `<ktlint.version>` for the antrun/exec pattern (research/30 §4).

## How it runs

Each goal: one `ktrs ktlint` process over the module's files (gantsign's source walk: `DirectoryScanner`, same roots,
includes, dedup), in `basedir`, with the ktlint-gradle hidden options (research/29, `crates/ktrs-cli/src/ktlint/gradle.rs`):
`--relative --ktrs-relative-to=<basedir>` (base-relative paths, platform separators = gantsign's `toRelativeString`),
`--ktrs-gradle-events=<tmp>` (every error with status; `--format` = the engine's `format(code, callback)`),
`--ktrs-editorconfig-override` for `android` / `experimental`. The `maven` reporter and the site report's model run in the
Maven JVM from the events, in gantsign's order (walk again, per file: `before`, errors, `after`); every other reporter is a
`--reporter=<id>?<properties>[,artifact=],output=` of the run. `format` counts changed files by content. Binary:
`-Dktrs.executable`, else the one bundled in `io.github.hexay:ktrs`; `JAVA_HOME` = Maven's JVM (for the hand-off).

Plugin `<dependencies>` (`${plugin}` artifacts minus the plugin's own runtime closure, listed at build time in
`plugin-artifacts.txt`, minus ktlint's runtime groups) become `-R` / `artifact=` JARs with the Gradle plugin's rules
(`JarServices`): compose-rules 0.6.7 runs natively, other rule set / reporter JARs hand the run to the ktlint jar.
ktlint's own reporter artifacts (`ktlint-cli-reporter-html`, `-sarif`, `-plain-summary`, ...) enable that id natively,
as adding them to gantsign's classpath does.

## Deviations

1. Reporters without `<output>` (stdout) print after the `maven` reporter's rows, not interleaved per error.
2. `format`: a file ktlint can't parse logs `[ERROR] <message>` without the JVM stack trace; the message is rebuilt from
   ktlint's CLI detail, which lower-cases it (first letter restored).
3. `format` writes only files it changed (gantsign rewrites every file).
4. Debug (`-X`) output: no `Discovered RuleSetProviderV3` lines, `Discovered reporter` lists the known ids sorted, and the
   per-file lines come after the run; the `ktrs ktlint` command line and its log are added.
5. A rule set / reporter JAR other than compose-rules 0.6.7 runs on the ktlint jar (JVM speed, one-time download):
   `android`/`experimental` don't apply there, and a reporter id is only checked by that jar. With several custom
   reporter JARs, each unknown reporter is given one of them.
6. A reporter's own `color`/`color_name` properties are overridden by `reporterColor`/`reporterColorName`.
7. `help` describes this plugin; `ktlint` declares `AbstractMavenReport`'s parameters under its own names (the
   descriptor generator doesn't scan dependency classes; readonly, so not user-visible).

## Verification

- Unit tests: `java/gradlew -p java :ktrs-ktlint-maven-plugin:test` (`cargo build --bins` first): reporter rows, events /
  json parsing, versions, check/format end to end on the debug binary.
- Parity: `tools/ktlint-maven/parity.sh` (Maven 3.9.16 pinned by sha512, JDK 21, Windows; compares console, exit codes,
  report files and sources; `compare.py` normalizes timings, plugin coordinates and stack traces (dev. 2)). Results
  2026-10-07, 12 scenarios, 11 identical:

| scenario | what | result |
|---|---|---|
| clean | `check`, `format` on clean sources | identical |
| violations | `check` ×5: default, `verbose`, `failOnViolation=false`+`android`+`experimental`, no tests/scripts, `skip` | identical (28 rows) |
| format | `format`, `check`, `format` | identical (sources, counts, rows) |
| reporters | plain, plain `group_by_file`, checkstyle, json, html (plugin dep), baseline | identical reports |
| options | `maven` reporter `group_by_file`+`pad`, verbose, color, excludes | identical |
| stdout | `plain` without `<output>` | same lines, different order (dev. 1) |
| unknown-reporter | `sarif` without its artifact | identical failure message |
| compose / handoff | compose-rules 0.6.7 (native) / 0.6.6 (ktlint jar), `check` (+`format`) | identical |
| parse-error | `check`, `format` with an unparsable file | identical but the stack trace (dev. 2) |
| report | `ktlint:ktlint` → `target/site/ktlint.html` | identical |
| multimodule | pom-packaged parent + 2 modules: `ktlint:check -fae`, `verify` with `format`+`check` executions | identical |

2.0 mode (`-Dktrs.ktlintVersion=2.0.0-ALPHA-4`) runs (28 rows on the sample); `1.5.0` fails with the version message.

## Open items

- Not verified: Linux/macOS, `mvn site` through maven-site-plugin (only standalone `ktlint:ktlint`), custom reporter JARs,
  2.0 mode against a reference (gantsign has none).
- `JarServices` duplicates the Gradle plugin's `JarServices.kt` (could move to the root jar).
