# 17 — Upstream `ktrs()` step for diffplug/spotless: PR design (2026-09-30)

Studied `diffplug/spotless` main @ `2c56489` (depth-1 clone in the session scratchpad). `S/` = spotless root,
`L/` = `S/lib/src/main/java/com/diffplug/spotless/`. Our side: `java/src/main/java/io/github/hexay/ktrs/`.
Unblocked 2026-10-06: `io.github.hexay:ktrs` is on Maven Central from 0.4.0 (spotless tests use `TestProvisioner.mavenCentral()`).

## How ktfmt is wired today

- `L/kotlin/KtfmtStep.java`: `create(version, Provisioner, @Nullable Style, @Nullable KtfmtFormattingOptions)` →
  `FormatterStep.create(NAME, roundtrip, KtfmtStep::equalityState, State::createFormat)`. Roundtrip state holds
  `JarState.Promised` (`JarState.from("com.facebook:ktfmt:" + version, provisioner)`); equality `State` holds the
  resolved `JarState` (its `FileSignature` of the jars), version, style, options. Style/option support is validated
  per ktfmt version with `BadSemver`; `< 0.51` goes through a reflection-only compat path.
- Glue source set `S/lib/src/ktfmt/java/com/diffplug/spotless/glue/ktfmt/` (`KtfmtFormatterFunc`, `KtfmtStyle`,
  `KtfmtFormattingOptions`, `KtfmtTrailingCommaManagementStrategy`): compiled `compileOnly` against ktfmt, packed into
  the lib jar, and *redefined inside the JarState classloader* by `L/FeatureClassLoader.java` (any
  `com.diffplug.spotless.glue.*` class); `com.diffplug.spotless.*` resolves from the build-tool loader. `KtfmtStep`
  reaches the glue by reflection (`loadClass` + `getConstructor(...)`).
- `S/lib/build.gradle.kts`: `needsGlue` list (alphabetic), `"ktfmtCompileOnly"(libs.ktfmt)`, and
  `buildConfigField("VERSION_KTFMT", libs.ktfmt.version)` in `KotlinBuildConfig` (the default version).
  `S/gradle/libs.versions.toml`: `ktfmt = "com.facebook:ktfmt:0.64"`. `S/.github/scripts/update_renovate_changelogs.py`
  maps `("libraries","ktfmt"): "ktfmt"` so Renovate bumps write the CHANGES lines.
- Gradle: `S/plugin-gradle/.../BaseKotlinExtension.java` (shared by `kotlin {}` and `kotlinGradle {}`):
  `ktfmt()` / `ktfmt(version)` → `KtfmtConfig` with `metaStyle() googleStyle() kotlinlangStyle() dropboxStyle()`
  returning `ConfigurableStyle`, and `configure(Consumer<KtfmtStep.KtfmtFormattingOptions>)`; each call `replaceStep`.
- Maven: `S/plugin-maven/.../kotlin/Ktfmt.java` (`FormatterStepFactory`; `@Parameter` version, style (string →
  `Style.valueOf`), maxWidth, blockIndent, continuationIndent, removeUnusedImports, trailingCommaManagementStrategy),
  registered by `Kotlin.addKtfmt`.
- Lifetime: the `FormatterFunc` is built lazily once per step per `Formatter`
  (`L/FormatterStepEqualityOnStateSerialization.java:48-81`) and, if it is a `FormatterFunc.Closeable`, closed with
  the `Formatter` (`try (Formatter formatter = buildFormatter())`, `SpotlessTaskImpl.java:101`; Maven:
  `FormattersHolder`). `SpotlessCache.clearOnce` closes cached JarState classloaders between builds.

## Native / external precedents

| step | binary source | per-file cost | lifetime |
|---|---|---|---|
| `NativeCmdStep`, `ClangFormatStep` | user's PATH / `ForeignExe` (version-checked) | one process per file (`ProcessRunner`) | `FormatterFunc.Closeable.of(runner, …)` |
| `BiomeStep` | downloaded from GitHub releases into `downloadDir` (+ `.sha256` sidecar), or `pathToExe` | one process per file | `Closeable.of(runner, …)` |
| npm (`PrettierFormatterStep`, …) | `npm install` into a build dir | one long-lived HTTP server | `Closeable.ofDangerous(() -> endServer(...), func)` |

No step resolves a native binary *from a Maven artifact*; ktrs would be the first. That is the cleanest option for
Spotless: resolution, caching, offline mode, mirrors and Renovate all come from the existing `JarState`/`Provisioner`
path, with no downloader code in Spotless. The npm steps are the precedent for "one server per Formatter, closed by
the Closeable" — exactly what `Ktrs.create()` + `close()` gives.

## 1. API shape

**A new `ktrs()` step, not an option on `ktfmt()`.** Reasons: (a) the version argument names a different artifact —
`ktfmt("0.64")` is a ktfmt version, ktrs versions are independent and each pins one ktfmt version (0.64 now);
(b) ktrs supports only the current ktfmt surface (no `DROPBOX`/`DEFAULT`, no `< 0.51` compat), so a ktfmt option would
need a ktfmt→ktrs version map and a pile of "not with native" errors; (c) Spotless already models alternative
implementations of one style as separate steps (`googleJavaFormat` / `palantirJavaFormat`); (d) maintainers own no
risk in `KtfmtStep`. Cost: the switch is "one word + the version string".

Mirror the ktfmt DSL exactly, *reusing ktfmt's option types* so existing `configure {}` blocks and imports
(`KtfmtStep.TrailingCommaManagementStrategy`) compile unchanged:

```kotlin
// before
spotless { kotlin { ktfmt("0.64").kotlinlangStyle().configure { it.setMaxWidth(120) } } }
// after
spotless { kotlin { ktrs("0.2.0").kotlinlangStyle().configure { it.setMaxWidth(120) } } }
```

```xml
<ktfmt><style>KOTLINLANG</style><maxWidth>120</maxWidth></ktfmt>   <!-- before -->
<ktrs><style>KOTLINLANG</style><maxWidth>120</maxWidth></ktrs>     <!-- after; <version> optional -->
```

Option mapping (`KtrsOptions`): style `META|GOOGLE|KOTLINLANG` → `KtrsOptions.of(Style)` (null → META; `DROPBOX`,
`DEFAULT` → `IllegalArgumentException`); `maxWidth` → `withMaxWidth`; `blockIndent` → `withBlockIndent`;
`continuationIndent` → `withContinuationIndent`; `removeUnusedImports` → `withRemoveUnusedImports`;
`trailingCommaManagementStrategy` `NONE|ONLY_ADD|COMPLETE` → `withTrailingCommas`. **Not exposed in v1:**
`withEditorConfig` — ktfmt's step has no equivalent, and Spotless can't fingerprint which `.editorconfig` files a
run read, so up-to-date checks would go stale (ktlint's step solves this with an explicit `FileSignature`; do that
in a follow-up if asked). Gradle omits `dropboxStyle()`.

## 2. File-by-file changes (spotless repo)

lib
- `gradle/libs.versions.toml`: `ktrs = "io.github.hexay:ktrs:0.2.0"` (in the formatter block, alphabetic after `ktfmt`).
- `lib/build.gradle.kts`: `buildConfigField("VERSION_KTRS", libs.ktrs.version)` in `KotlinBuildConfig`; `"ktrs"` in
  `needsGlue` (after `"ktlint"`); `// ktrs` + `"ktrsCompileOnly"(libs.ktrs)` in the GLUE block.
- `lib/src/ktrs/java/com/diffplug/spotless/glue/ktrs/KtrsFormatterFunc.java` (new, ~40 lines):
  `public final class KtrsFormatterFunc implements FormatterFunc.Closeable`; ctor
  `(String style, @Nullable Integer maxWidth, @Nullable Integer blockIndent, @Nullable Integer continuationIndent,
  @Nullable Boolean removeUnusedImports, @Nullable String trailingCommas)` builds a `KtrsOptions` and a lazy
  `Ktrs.create()`; `apply(input)` → `ktrs.format(input, options)`; `close()` → `ktrs.close()`. Plain strings in the
  ctor (no glue enums) keep the reflection to one `getConstructor` call.
- `lib/src/main/java/com/diffplug/spotless/kotlin/KtrsStep.java` (new, ~110 lines), shape copied from `KtfmtStep`
  minus the compat path: `NAME = "ktrs"`, `MAVEN_COORDINATE = "io.github.hexay:ktrs:"`, `create(Provisioner)`,
  `create(String, Provisioner)`, `create(String, Provisioner, @Nullable KtfmtStep.Style,
  @Nullable KtfmtStep.KtfmtFormattingOptions)`, `defaultVersion()` → `KotlinBuildConfig.VERSION_KTRS`; roundtrip
  `KtrsStep implements Serializable` with `JarState.Promised`; equality `State(version, JarState, style, options)`
  validating `BadSemver.version(version) >= 0.2` ("`ktrs serve` needs 0.2.0+") and the style; `createFormat()`
  loads `com.diffplug.spotless.glue.ktrs.KtrsFormatterFunc` from `jarState.getClassLoader()`.
- `lib/src/main/java/com/diffplug/spotless/kotlin/KtfmtStep.java`: add package-private getters on
  `KtfmtFormattingOptions` (`maxWidth()`, `blockIndent()`, …) so `KtrsStep` can read the shared options bean
  (today only `KtfmtStep`'s nested `State` reads the private fields). Alternative if maintainers object: a
  `KtrsStep.Options` copy — but then Gradle users must change their `configure {}` imports.
- `.github/scripts/update_renovate_changelogs.py`: `("libraries", "ktrs"): "ktrs",`.

plugin-gradle
- `BaseKotlinExtension.java`: `ktrs()` → `ktrs(KtrsStep.defaultVersion())`; `ktrs(String version)` → `KtrsConfig`,
  a sibling of `KtfmtConfig` (`metaStyle googleStyle kotlinlangStyle`, `configure(Consumer<KtfmtStep.KtfmtFormattingOptions>)`,
  inner `ConfigurableStyle`, `createStep()` → `KtrsStep.create(version, provisioner(), style, options)`). Javadoc
  one-liner linking https://github.com/Hexay/ktrs. Covers `kotlin {}` and `kotlinGradle {}`; predeclare works for free.

plugin-maven
- `kotlin/Ktrs.java` (new): copy of `Ktfmt.java` with `KtrsStep`.
- `kotlin/Kotlin.java`: `public void addKtrs(Ktrs ktrs) { addStepFactory(ktrs); }`.

docs / changelogs
- `README.md`: freshmark row `lib('kotlin.KtrsStep') +'{{yes}} | {{yes}} | {{no}} | {{no}} |',` after `KtfmtStep`, and
  the matching rendered table row (both tables are hand-kept in sync by `spotlessApply`/freshmark).
- `plugin-gradle/README.md`: TOC line `[Kotlin](#kotlin) ([ktfmt](#ktfmt), [ktrs](#ktrs), …)`; `ktrs() // has its own
  section below` in the kotlin block; new `### ktrs` section after `### ktfmt`: homepage + changelog links, one line
  "native, byte-for-byte port of ktfmt; ktrs `0.2.x` formats like ktfmt `0.64`; bundles binaries for linux/macos/
  windows × x86_64/aarch64, falls back to `-Dktrs.executable`", and the kotlin snippet above with all five setters.
- `plugin-maven/README.md`: TOC, `<ktrs />` line in the `<kotlin>` example, `### ktrs` section with homepage /
  changelog / code link and the full `<ktrs>` XML with `<!-- optional -->` comments (copy of the ktfmt one).
- `CHANGES.md`, `plugin-gradle/CHANGES.md`, `plugin-maven/CHANGES.md`, under `## [Unreleased]` → `### Added` (add the
  heading if absent), each ending `([#NNNN](https://github.com/diffplug/spotless/pull/NNNN))` — added in a follow-up
  commit once the PR number exists (PULL_REQUEST_TEMPLATE.md):
  - lib: "Add `KtrsStep`, a native port of ktfmt `0.64` ([ktrs](https://github.com/Hexay/ktrs)) that accepts ktfmt's
    styles and options."
  - gradle: "Add `ktrs()` to `kotlin`/`kotlinGradle`: same styles and `configure {}` options as `ktfmt()`, formats
    via a native binary."  maven: "Add `<ktrs>` to `<kotlin>`, with the same options as `<ktfmt>`."
- PR hygiene (CONTRIBUTING.md / template): Apache-2.0 header on new files, `./gradlew spotlessApply spotbugsMain`,
  "allow edits from maintainers", no force-push.

## 3. Resolution via JarState, and how the glue calls us

Yes — `JarState.from("io.github.hexay:ktrs:" + version, provisioner)`. Our POM has no runtime dependencies
(`spotless-lib` is `compileOnly`), so the classpath is exactly one jar. Size is a non-issue relative to ktfmt:
`ktfmt-0.64` pulls `kotlin-compiler-embeddable-2.3.20.jar` alone at 59.7 MB (+ guava, google-java-format, jna,
ec4j), vs ~15-20 MB for ktrs with all six binaries; it is downloaded once into the Gradle/Maven cache and shared by
all builds. Per-platform classifier jars (`io.github.hexay:ktrs:0.2.0:linux-x86_64`) would cut that to ~3 MB but add
platform detection to Spotless — not worth it in v1. The JarState's `FileSignature` already changes with the
version, so no `Implementation-Version` in the equality state.

Glue call path: `KtrsStep.State.createFormat()` → reflective `new KtrsFormatterFunc(style, …)` inside the
FeatureClassLoader → `Ktrs.create()` (bundled binary, extracted by `NativeBinary` to the user cache dir) →
`ktrs.format(code, options)` per file (no path: Spotless already prefixes lints with the file, and
`FormatterFunc.NeedsFile` throws on `NO_FILE_SENTINEL`, which `StepHarness` lint tests use) → `Formatter.close()` →
`ktrs.close()` stops the `ktrs serve` child. One server per step per task run; `Ktrs.shared()` must **not** be used
(see 4.2).

## 4. Changes to our `java/` API first

1. **Memoize `NativeBinary.locate()`** (`NativeBinary.java:22-38`). Every `Ktrs.create()` reads the whole bundled
   binary into memory, SHA-256s it, then reads and hashes the extracted copy again (`extract`, line 52). With the
   upstream design that runs once per Spotless task per build. Cache the resolved `Path` in a static volatile
   (per classloader = per JarState, which is exactly the right scope), and trust the hash-named directory when the
   target exists with the expected size (the atomic move already rules out partial files).
2. **Stop recommending `Ktrs.shared()` for build tools.** Its shutdown hook references the instance → class →
   FeatureClassLoader, so every `SpotlessCache.clearOnce` in a Gradle daemon leaks a closed classloader plus its
   idle `ktrs serve` processes until the daemon exits. Change our own `spotless/KtrsStep.formatter` to
   `FormatterFunc.Closeable.ofDangerous(ktrs::close, code -> ktrs.format(code, state.options))` with
   `Ktrs ktrs = Ktrs.create()`, and reword the `shared()` javadoc ("for callers without a lifecycle; build tools
   should own and close a `Ktrs`"). Once upstream ships, deprecate `io.github.hexay.ktrs.spotless.KtrsStep` in favour
   of `ktrs()` (keep it for Spotless versions without the step).
3. **Error text without a path** (`crates/ktrs-cli/src/serve.rs:95,157-163`). With no `path` header the message is
   `<stdin>:3:5: error: …`; Spotless shows `src/Foo.kt … <stdin>:3:5: …`. ktfmt's `ParseError.message` has no file
   prefix. Emit the bare `3:5: error: …` when `path` is absent (serve is only used by the JVM wrapper, so no CLI
   parity is at stake). Optional: expose line/column on `KtrsException` so a later step could throw `Lint.atLine`.
4. **Indent validation parity**: `KtrsOptions.positive()` rejects `0` for `blockIndent`/`continuationIndent`; the
   ktfmt step passes any value to `FormattingOptions` (only the unused glue setters check `< 0`). Check what ktfmt
   0.64 does with 0 and match it, so `configure {}` blocks that work with `ktfmt()` never fail with `ktrs()`.
5. Nice-to-have: `public static final String KTFMT_VERSION = "0.64"` on `Ktrs` (for README/tests and a future
   Spotless parity assertion), and have `ServerProcess.start` warn when the hello's version differs from the jar's
   (only possible with `-Dktrs.executable`).

No signature change is needed: `format(String, KtrsOptions)`, `create()`, `close()` and the `with*` builders cover
the glue; nothing Spotless-specific belongs in `Ktrs`.

## 5. Test plan (mirrors `KtfmtStepTest`, reuses `testlib/src/main/resources/kotlin/ktfmt/*`)

Verified this session against our debug `ktrs serve` (scratchpad `check_resources.py`): every pair below is
byte-identical to spotless's checked-in ktfmt expectations, so no new resources are needed.

`testlib/src/test/java/com/diffplug/spotless/kotlin/KtrsStepTest.java` (extends `ResourceHarness`):
- `behavior`: `KtrsStep.create(mavenCentral())`, `basic.dirty` → `basic.clean`.
- `behaviorWithOptions`: GOOGLE + `maxWidth 100`, `basic.dirty` → `basic.clean`.
- `maxWidth`: `maxWidth 120`, `max-width.dirty` → `max-width.clean`.
- `continuation`: `continuation.dirty` → `continuation.clean`.
- `trailingCommasOnlyAdd`: KOTLINLANG + `ONLY_ADD` → `trailing-commas-only-add.clean`.
- `trailingCommasComplete`: KOTLINLANG + `COMPLETE`, `trailing-commas.dirty` → `trailing-commas.clean`.
- `dropboxStyleRejected` / `versionBefore_0_2_rejected`: `assertThatThrownBy(() -> StepHarness.forStep(step))`
  `.isInstanceOf(IllegalArgumentException/IllegalStateException).hasMessageContaining(...)`.
- `parityWithKtfmt_0_64`: for each `*.dirty` × {META, GOOGLE, KOTLINLANG}, `KtrsStep` output == `KtfmtStep.create("0.64",
  …)` output (pin `0.64`, not `KtfmtStep.defaultVersion()`, which Renovate will move ahead of ktrs).
- `syntaxError`: `StepHarness.forStep(step).expectLintsOf("fun f( {").toBe(...)` (selfie) — asserts the message, locks point 4.3.
- `equality`: `SerializableEqualityTester` — same version equal; change version, style, and one option each → different.

`plugin-gradle/src/test/java/com/diffplug/gradle/spotless/KotlinExtensionTest.java`:
- `integrationKtrsWithPublicApi` (build.gradle.kts, `import com.diffplug.spotless.kotlin.KtfmtStep.TrailingCommaManagementStrategy`,
  `ktrs().googleStyle().configure { it.setMaxWidth(100); it.setTrailingCommaManagementStrategy(NONE) }`) → `basic.clean`.
- `testWithCustomMaxWidthDefaultStyleKtrs` (Groovy, `ktrs().configure { options -> options.maxWidth = 120 }`) → `max-width.clean`.
- `KotlinGradleExtensionTest`: `kotlinGradle { ktrs() }` on a `.kts` file (idempotence on an already-clean script).

`plugin-maven/src/test/java/com/diffplug/spotless/maven/kotlin/KtrsTest.java` (extends `MavenIntegrationHarness`):
`testKtrs` (`<ktrs/>`, two files), `testContinuation`, `testKtrsWithMaxWidthOption` (120), `testKtrsStyleWithTrailingCommas`
(`<style>KOTLINLANG</style><trailingCommaManagementStrategy>ONLY_ADD</trailingCommaManagementStrategy>`).

CI: spotless runs Gradle/Maven tests on ubuntu-latest (JDK 17/21/25/26) and windows-latest — both bundled
(`linux-x86_64` is static musl, `windows-x86_64` msvc), so no `@Tag` like `ClangTest` is needed. Run locally with
`./gradlew :testlib:test --tests com.diffplug.spotless.kotlin.KtrsStepTest` (and the plugin equivalents).

## Sequencing

1. Land 4.1-4.4 in `java/` + `serve.rs`; release ktrs 0.2.x to Maven Central.
2. Open a spotless issue first ("native ktfmt-compatible step") to get a maintainer nod on the new-step shape.
3. PR with sections 2 + 5; follow-up commit adds the three CHANGES entries with the PR number.
