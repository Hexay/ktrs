# 30 — Maven build plugin: landscape and shape

2026-10-06. Web/GitHub research only, nothing built. **F** = fact (sourced), **I** = inference.
GitHub counts come from the legacy REST code search (`gh api search/code`, `filename:pom.xml`). Treat them as
rough relative sizes, not as absolute numbers.

## 1. gantsign `com.github.gantsign.maven:ktlint-maven-plugin`

- **F** Latest is 3.7.1, released 2026-04-06. 3.6.0, 3.7.0 and 3.7.1 all shipped that same day, and the release
  before them was 3.5.0 on 2025-01-26. https://repo1.maven.org/maven2/com/github/gantsign/maven/ktlint-maven-plugin/
- **F** 3.7.1 bundles ktlint **1.8.0** as normal dependencies: rule-engine, rule-engine-core, cli-ruleset-core,
  ruleset-standard, and the reporters plain/json/checkstyle/baseline. Kotlin 2.2.21. `<prerequisites><maven>3.5.4`,
  maven-plugin-api 3.8.9, annotations 3.15.2.
  https://repo1.maven.org/maven2/com/github/gantsign/maven/ktlint-maven-plugin/3.7.1/ktlint-maven-plugin-3.7.1.pom
- **F** Maintenance: 65 stars and 18 open issues, mostly dependabot. Issue #684 (2025-12) asked "Is this plugin no
  longer being actively maintained?" and the maintainer replied 2026-04-06: "maintenance has been a bit sporadic".
  No ktlint 2.0 support exists. https://github.com/gantsign/ktlint-maven-plugin/issues/684
- **F** Docs: https://gantsign.com/ktlint-maven-plugin/ (`plugin-info.html`, `check-mojo.html`, `format-mojo.html`,
  `ktlint-mojo.html`, `usage.html`).
  Source: https://github.com/gantsign/ktlint-maven-plugin/tree/main/src/main/kotlin/com/github/gantsign/maven/plugin/ktlint

### Goals (all `threadSafe = true`, `requiresProject = true`)

| goal | default phase | notes |
|---|---|---|
| `check` | `verify` | throws `MojoFailureException("Kotlin source failed ktlint check.")` if there are errors and `failOnViolation` is set |
| `format` | `process-sources` | rewrites files in place |
| `ktlint` | `verify` | Maven site report (HTML), needs maven-site-plugin 3.7.0 or later |
| `help` | — | generated |

### Parameters (`AbstractBaseMojo`, used by all goals)

| name | property | default |
|---|---|---|
| `scriptRoots` (List) | `ktlint.scriptRoots` | `${project.basedir.path}` |
| `includeSources` | `ktlint.includeSources` | `true` |
| `includeTestSources` | `ktlint.includeTestSources` | `true` |
| `includeScripts` | `ktlint.includeScripts` | `true` |
| `sourcesIncludes` / `sourcesExcludes` (Set) | — | `**/*.kt` / none |
| `testSourcesIncludes` / `testSourcesExcludes` | — | `**/*.kt` / none |
| `scriptsIncludes` / `scriptsExcludes` | — | `*.kts` / none |
| `android` | `ktlint.android` | `false` (sets `ktlint_code_style=android_studio`) |
| `experimental` | `ktlint.experimental` | `false` (sets `ktlint_experimental=enabled`) |

Source roots are the read-only `${project.compileSourceRoots}` and `${project.testCompileSourceRoots}`.

Parameters specific to `check`: `reporters` (Set<ReporterConfig>), `reporterColor` (default false),
`reporterColorName` (default `DARK_GRAY`), `verbose` (`ktlint.verbose`, false), `failOnViolation`
(`ktlint.failOnViolation`, true) and `skip` (`ktlint.skip`, false). `format` has only `skip`. The `ktlint` report goal
also has `verbose`, `reporters` and `encoding` (`${project.build.sourceEncoding}`).

### Reporters

- **F** `ReporterConfig { name, output: File, properties: Properties }`. The built-in reporters are `plain`, `json`
  and `checkstyle`; the ktlint baseline reporter jar is also on the classpath. `check` always adds a Maven log
  reporter (`MavenLogReporter`). That reporter prints `dir/` + bold `name` + `:line:col: detail` through
  `log.error`, and with `verbose` it appends ` (ruleId)`. It honours `group_by_file`.
- **F** Custom reporters are added as `<dependencies>` of the plugin and found via `ServiceLoader`.
  Usage snippet: `<reporter><name>plain</name><output>${project.build.directory}/ktlint.txt</output><properties><property><name>group_by_file</name><value>true</value></property></properties></reporter>`.

### Rule sets and editorconfig

- **F** Rule sets are loaded with `ServiceLoader.load(RuleSetProviderV3::class.java)` (no explicit classloader),
  deduplicated by id. This makes plugin `<dependencies>` the way to add third-party rule sets such as compose-rules.
  The usage docs do not mention it.
- **F** The engine is built as `KtLintRuleEngine(ruleProviders, EditorConfigDefaults.load(null, …), overrides,
  isInvokedFromCli = false)`. There is no parameter for an editorconfig path or for overrides. `.editorconfig`
  files are discovered per linted file by the normal ktlint lookup.
- **F** There is no baseline support, no `--relative` (#380), no ktlint version selection, and no Java 17+
  `--add-opens` workaround inside the plugin (users add `.mvn/jvm.config`).

### Adoption

- **F** 321 `pom.xml` hits for `ktlint-maven-plugin` and 310 for `com.github.gantsign.maven`.

## 2. ktfmt on Maven

- **F** No dedicated ktfmt Maven plugin exists. Maven Central search for `a:ktfmt-maven-plugin` and `ktfmt-maven`
  returns 0 hits, and GitHub repository search for "ktfmt maven plugin" returns 0 repositories. ktfmt's own README,
  under "using Maven", says: "Consider using Spotless with the ktfmt Maven plugin" (it links Spotless).
  https://github.com/facebook/ktfmt#using-maven
- **F** Latest ktfmt is 0.64 (Central metadata, 2026-06-24). README badges now point at github.com/Kotlin/ktfmt.
- **F** Code search counts:
  - `spotless-maven-plugin ktfmt`: 593
  - `ktfmt` in pom.xml: 591
  - `ktfmt-maven-plugin`: 0
  - for scale, `spotless-maven-plugin`: 19,168 and `kotlin-maven-plugin`: 26,944
- **I** Running ktfmt in Maven effectively means Spotless `<kotlin><ktfmt/>`, with a few exec/antrun one-offs.
  There is no incumbent ktfmt-specific DSL to copy.

## 3. spotless-maven-plugin

- **F** Latest is 3.10.3 (2026-09-25). It requires Maven to run on JRE 17 or later (2.46.1 for JRE 11).
  https://github.com/diffplug/spotless/blob/main/plugin-maven/README.md
- **F** `<ktfmt>` options:
  - `version`
  - `style` (META default, GOOGLE, KOTLINLANG)
  - `maxWidth`
  - `blockIndent`
  - `continuationIndent`
  - `removeUnusedImports`
  - `trailingCommaManagementStrategy`

  Source: `plugin-maven/src/main/java/com/diffplug/spotless/maven/kotlin/Ktfmt.java`
- **F** `<ktlint>` options: `version`, `editorConfigPath` (defaults to `./.editorconfig` if it exists),
  `editorConfigOverride` (a map), and `customRuleSets` (a list of GAV strings such as
  `io.nlopez.compose.rules:ktlint:0.4.25`). Spotless presets `ktlint_code_style=intellij_idea`.
  Spotless ktlint has 125 `pom.xml` hits.
- **F** Spotless offers no plugin/SPI mechanism for external steps. `FormatterFactory.addStepFactory` is
  `protected`. `Kotlin` exposes only `addKtlint(Ktlint)`, `addKtfmt(Ktfmt)`, `addDiktat` and
  `addTableTestFormatter`. The generic public adders are `addNativeCmd`, `addJsr223`, `addReplace*`,
  `addLicenseHeader` and similar.
- **F** `<nativeCmd><name/><pathToExe/><arguments/></nativeCmd>` pipes file content through stdin and reads the
  result from stdout. It needs an absolute path to an installed binary.
- **F** `Ktfmt` and `Ktlint` (spotless maven) are `public class … implements FormatterStepFactory` and are not
  final.
- **F** Maven's configurator honours an `implementation="fqcn"` attribute on complex parameters.
  https://maven.apache.org/guides/mini/guide-configuring-plugins.html
- **I** (needs a spike) Zero-upstream route into Spotless Maven:
  `<kotlin><ktfmt implementation="io.github.hexay.ktrs.spotless.maven.KtrsKtfmt">…</ktfmt></kotlin>`, plus
  `io.github.hexay:ktrs` as a `<dependency>` of spotless-maven-plugin. `KtrsKtfmt` would subclass Spotless's
  `Ktfmt` and override `newFormatterStep` to return our existing `KtrsStep`. A `KtrsKtlint` subclass would do the
  same for ktlint.
  - Risks: plugin-realm classloading, private fields in the superclass (Plexus sets fields up the hierarchy),
    and coupling to Spotless-internal classes across Spotless releases.
  - Fallback: `<nativeCmd>` running the `ktfmt` drop-in on stdin, which needs a user-installed binary.

## 4. ktlint docs: the antrun / exec pattern

- **F** The ktlint docs for 1.0.0 through 1.5.0 show maven-antrun-plugin 3.1.0. The ktlint docs
  `documentation/release-latest/docs/install/integrations.md` at tag 1.5.0 contain:

```xml
<plugin>
    <groupId>org.apache.maven.plugins</groupId>
    <artifactId>maven-antrun-plugin</artifactId>
    <version>3.1.0</version>
    <executions>
        <execution>
            <id>ktlint</id>
            <phase>verify</phase>
            <configuration>
            <target name="ktlint">
                <java taskname="ktlint" dir="${basedir}" fork="true" failonerror="true"
                    classpathref="maven.plugin.classpath" classname="com.pinterest.ktlint.Main">
                  <arg value="src/**/*.kt"/>
                </java>
            </target>
            </configuration>
            <goals><goal>run</goal></goals>
        </execution>
        <execution>
            <id>ktlint-format</id>
            <configuration>
            <target name="ktlint">
                <java taskname="ktlint" dir="${basedir}" fork="true" failonerror="true"
                    classpathref="maven.plugin.classpath" classname="com.pinterest.ktlint.Main">
                    <jvmarg value="--add-opens=java.base/java.lang=ALL-UNNAMED"/>
                    <arg value="-F"/>
                    <arg value="src/**/*.kt"/>
                </java>
            </target>
            </configuration>
            <goals><goal>run</goal></goals>
        </execution>
    </executions>
    <dependencies>
        <dependency>
            <groupId>com.pinterest.ktlint</groupId>
            <artifactId>ktlint-cli</artifactId>
            <version>1.4.1</version>
        </dependency>
        <!-- additional 3rd party ruleset(s) can be specified here -->
    </dependencies>
</plugin>
```

  The snippet is shown without its XML comments. Invocation is `mvn antrun:run@ktlint` / `@ktlint-format`.
  https://github.com/pinterest/ktlint/blob/1.5.0/documentation/release-latest/docs/install/integrations.md
- **F** The ktlint docs at the 1.8.0 tag switched to **exec-maven-plugin 3.5.0**. That snippet:
  - runs `<executable>java</executable>` with `-classpath <classpath/> com.pinterest.ktlint.Main --format --relative`
  - binds execution `ktlint-format` to `compile`
  - uses `includePluginDependencies=true`
  - depends on `com.pinterest.ktlint:ktlint-cli:1.7.1:all`
  - runs as `mvn exec:exec@ktlint-format`
- **F** ktlint master docs (after 2.0.0-ALPHA-4, commit b5f16d7c "Update repository coordinates", 2026-05-26) show
  `io.github.ktlint.core:ktlint-cli:2.0.0:all` and main class `io.github.ktlint.core.Main`. Both docs link gantsign
  as the "dedicated" plugin.
- **F** `com.pinterest.ktlint.Main` has 145 `pom.xml` hits, `com.github.shyiko.ktlint.Main` 19, and `ktlint-cli` 75.
- **I** The exec/antrun users invoke the ktlint CLI, so our `ktlint` drop-in binary already covers them as long as
  they can swap `java -cp … Main` for an executable. A Maven goal adds little for them.

## 5. Authoring constraints

- **F** Maven compatibility plan: since October 2025, plugins should use **3.9.0** as their minimum. Maven before
  3.8.3 is EOL (the history page says 3.8.9 and earlier are EOL). Current releases are 3.10.0 (2026-09-27),
  3.9.16 and 4.0.0-rc-7 (Java 17). gantsign still declares 3.5.4.
  https://maven.apache.org/developers/compatibility-plan.html, https://maven.apache.org/docs/history.html
- **F** maven-plugin-plugin / maven-plugin-annotations is at 3.16.0 (2026-09-06).
- **F** Gradle plugin `org.gradlex.maven-plugin-development` is at 1.0.3 (2025-02-11). It succeeds
  `de.benediktritter.maven-plugin-development`, which is deprecated (last 0.4.3, 2024-02).
  - It needs Gradle 7.5 or later.
  - It wraps maven-plugin-tools 3.15.1 and generates `META-INF/maven/plugin.xml` from annotations, plus an
    optional help mojo via `helpMojoPackage`.
  - It supports the configuration cache, and its end-to-end test runs the Gradle-built plugin in a real Maven
    build.
  - Repo activity: commits through 2026-09-19 (Renovate). 38 stars.
  - Since 1.0, mojos must live in their own project, not in a separate source set.

  https://github.com/gradlex-org/maven-plugin-development
- **F** The gradlex plugin does not set `<packaging>maven-plugin</packaging>` in the published POM. The test build
  uses a plain `from components.java`.
- **I** Maven resolves a plugin from the jar's `plugin.xml`, so a POM with `jar` packaging works. Setting
  `pom { packaging = "maven-plugin" }` through vanniktech is cosmetic but matches convention.
- **I** Recommended build setup: a `java/maven-plugin` Gradle subproject with gradlex 1.0.3 and vanniktech 0.37,
  `compileOnly` maven-plugin-api 3.9.x and maven-plugin-annotations 3.16.0, depending on `io.github.hexay:ktrs`.
  A Maven sub-build would mean a second build tool in CI and in the release workflow, for no gain.
- **I** Testing: TestKit-style tests could use takari-plugin-testing, or simply run a Maven wrapper against fixture
  projects. The latter mirrors what `java/gradle-plugin` does with TestKit.

## Recommendation (I)

1. **ktlint:** mirror gantsign `ktlint-maven-plugin` 3.7.1 flag for flag. It is the only incumbent (about 320
   poms), it is sporadically maintained, and it has no ktlint 2.0 support. Keep:
   - the goals `check`, `format` and `ktlint` (report), with the same phases
   - the parameter names, `ktlint.*` user properties and defaults
   - `ReporterConfig`, `MavenLogReporter` output and the failure message
   - the plugin-`<dependencies>` rule-set mechanism (compose-rules natively, the jar fallback otherwise, as in
     `ktrs lint -R`)

   Optional extras that don't break drop-in use: `ktlintVersion` (1.8 default, matching research/29), baseline, and
   editorconfig override.
2. **ktfmt:** there is no plugin to mirror. Either ship a small ktrs-native `ktfmt` goal set (`format`/`check`, with
   options named after Spotless's `<ktfmt>`: `style`, `maxWidth`, `blockIndent`, `continuationIndent`,
   `removeUnusedImports`, `trailingCommaManagementStrategy`), or spike the Spotless `implementation=` subclass trick
   (§3). That trick would cover Spotless ktfmt (about 590 poms, the larger audience) and Spotless ktlint (125) with
   no new DSL.

## Spike result: Spotless `implementation=` swap (2026-10-06)

The swap works with no upstream change. Tested on Maven 3.9.16, JDK 21, spotless-maven-plugin 3.10.3 and ktrs 0.4.0.
The throwaway code lives in the session scratchpad and is not kept.

```xml
<plugin>
  <groupId>com.diffplug.spotless</groupId><artifactId>spotless-maven-plugin</artifactId><version>3.10.3</version>
  <configuration><kotlin>
    <ktfmt implementation="io.github.hexay.ktrs.spotless.maven.KtrsKtfmt"><style>KOTLINLANG</style></ktfmt>
    <!-- or: <ktlint implementation="io.github.hexay.ktrs.spotless.maven.KtrsKtlint">...</ktlint> -->
  </kotlin></configuration>
  <dependencies><dependency><!-- jar holding KtrsKtfmt/KtrsKtlint, depends on io.github.hexay:ktrs --></dependency></dependencies>
</plugin>
```

- `KtrsKtfmt extends com.diffplug.spotless.maven.kotlin.Ktfmt` and `KtrsKtlint extends ...kotlin.Ktlint` each override
  `newFormatterStep(FormatterStepConfig)` to return `KtrsStep.create(...)` or `KtrsKtlintStep.create(...)`. The
  implicit no-arg constructor is enough.
- The superclass fields are all `private` and have no getters. Redeclaring same-named fields in the subclass works
  without reflection, because Plexus injects into the most-derived field (the `-X` log shows `(f) style` on
  `KtrsKtfmt`). `trailingCommaManagementStrategy` keeps Spotless's `KtfmtStep.TrailingCommaManagementStrategy` type.
- `KtrsKtlint` reproduces two Spotless behaviours itself:
  - The `./.editorconfig` default, resolving the path with `FileLocator.locateFile`.
  - `customRuleSets` GAVs, resolved with `config.getProvisioner().provisionWithTransitives(false, ...)`.
- Results matched stock Spotless:
  - `spotless:check` fails and lists the file. `spotless:apply` produces output byte-identical to stock `<ktfmt>`
    0.64 KOTLINLANG (maxWidth 80) and to stock `<ktlint>` 1.8.0.
  - The up-to-date index skips the file on the next run.
  - An unfixable `no-wildcard-imports` violation gives the same lint text and failure as stock in both `check` and
    `apply`.
- No classloader problems: the ktrs native binary extracts and runs from the plugin realm.
- Risk: the swap compiles against Spotless-internal classes (`maven.kotlin.Ktfmt`/`Ktlint`, `FormatterStepConfig`,
  `KtfmtStep.TrailingCommaManagementStrategy`). Spotless can rename or remove a field without notice, and the
  shadowed field then silently stops being set.
