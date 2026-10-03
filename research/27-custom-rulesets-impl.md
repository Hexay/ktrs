# 27 — Custom rule sets: jar hand-off and native compose-rules (2026-10-03)

Implements research/25's decision (memory `custom-ruleset-decision`): compose-rules runs natively, every other
custom JAR goes to the real ktlint jar. Works in both ktlint modes (2.0.0-ALPHA-4 default, 1.8.0; research/26).

## A. Jar hand-off (`crates/ktrs-cli/src/ktlint/ktlint_jar.rs`, `jar_providers.rs`)

- Trigger, decided right after argument parsing and before stdin is read (`jvm_only_jar`): a `-R` JAR that
  declares a rule set provider the active version loads (1.8: `RuleSetProviderV3`; 2.0: `RuleSetV2Provider` or
  `RuleSetProviderV3`) and is not a native compose-rules release, or a `--reporter=…,artifact=` JAR declaring
  `ReporterProviderV2`. Not for the git-hook subcommands (they load no JARs). A missing path or a JAR without a
  provider stays native, with upstream's exact error (no JVM needed for those).
- Run: `java [Java 24+: --sun-misc-unsafe-memory-access=allow --enable-native-access=ALL-UNNAMED] -Xmx512m -jar
  <jar> <argv minus --ktlint-version>` in the same working directory; stdin/stdout/stderr inherited (Unix: `exec`),
  the exit code returned. Same flags as the release launcher script. An `@argfile` whose expansion holds
  `--ktlint-version` (`ktlint/hand_off_args.rs`) is replaced by a temporary argfile with the rest of its tokens
  (Clikt-quoted; still read by the jar's Clikt, so no launcher wildcard expansion applies), deleted after the run
  (no `exec` then); other argfiles pass through untouched. The jar's tokenizer was checked on quotes, `\`, spaces
  and `@@`.
- `ktrs lint -R <jar>[,<jar>]` (repeatable, unlike the drop-in's last-wins `-R`; `ktrs_lint.rs`): same loading code
  and trigger (`-R` and `--reporter=…,artifact=` JARs). A hand-off is meaningful because `ktrs lint` is the ktlint
  command with other flag names: it runs the jar with the equivalent ktlint argv (`ktlint_argv`: `--relative`,
  `--reporter=`, `--ruleset=a,b`, `--baseline=`, `--editorconfig=`, `--limit=`, `--stdin[-path]`, `--format`,
  patterns with a leading `@` doubled), so the output is what `ktrs lint` would print natively. `--list-rules`
  lists the native compose rules too and is a usage error with a JVM-only JAR. On Windows the hand-off's patterns
  go through the `java` launcher's wildcard expansion, which native `ktrs lint` doesn't do (same files in practice).
- The jar: the `ktlint` release asset (self-executing fat jar) of 1.8.0 or 2.0.0-ALPHA-4, picked by the run's
  `KtlintVersion`, downloaded with `curl` on first use to `%LOCALAPPDATA%\ktrs`, `~/Library/Caches/ktrs` or
  `$XDG_CACHE_HOME/ktrs` (`~/.cache/ktrs`) as `ktlint-<version>.jar`, SHA-256 pinned in source, via a per-process
  temp file and rename. `KTRS_KTLINT_JAR=<path>` overrides it (not checked; the user picks the version).
- `java`: `$JAVA_HOME/bin/java`, else `PATH`. Missing java, a failed download or a bad digest: one `ktrs: …` line on
  stderr naming the JAR that needed the JVM and what to do, exit 1. The first-run download prints one
  `ktrs: downloading ktlint … (once)` line on stderr — the only output the hand-off adds.

### Build tools

| Path | Covered? |
|---|---|
| `ktlint -R x.jar` (CLI, scripts, CI) | yes |
| pre-commit `ktlint` hook with `-R` in `args` | yes (it runs the binary) |
| Spotless `ktlint(v).customRuleSets(...)` | no: Spotless runs ktlint in-process on the JVM; nothing reaches the binary. The `java/` wrapper only provides a ktfmt step (`KtrsStep`) |
| ktlint-gradle `ktlintRuleset(...)`, kotlinter | no: in-process JVM (Gradle workers); `java/gradle-plugin` is a ktfmt-gradle drop-in, no ktlint tasks |
| Maven antrun `com.pinterest.ktlint.Main`, gantsign plugin | no: in-process JVM |

Those integrations keep working as before (they use real ktlint); covering them needs a ktrs ktlint step in
Spotless/Gradle first (research/08 B2/B3), which would then reuse this hand-off.

### Verified

`tools/ktlint-oracle/cli-diff.sh` with `RULESET_JAR=<jar>` adds 7 `ruleset_jar_*` scenarios (lint, -F, json,
stdin, stdin -F, `ktlint_compose = disabled`, generateEditorConfig). With compose-rules **0.6.6** (not native →
hand-off): `ruleset*` scenarios 11/11 identical vs ktlint 2.0.0-ALPHA-4 and 11/11 vs 1.8.0, on Windows and on
testbox (Linux, `exec` path). Unit tests cover argv filtering, launcher options, the missing-java error; CLI tests
the hand-off attempt for rule set and reporter JARs.

## B. Native compose-rules (`crates/ktrs-compose`)

- Pin: compose-rules **v0.6.7** (latest; builds against ktlint 1.8.0), `tools/sync-compose-rules.sh`
  (`third_party/compose-rules` + `tools/compose-rules/lib/ktlint-compose-0.6.7-all.jar`); `upstream.yml` opens an
  issue when a newer release ships.
- Detection (`ktrs-cli/src/ktlint/compose_jar.rs`, `zip_directory.rs`): SHA-256 of the sorted
  `name\tcrc32\tsize` lines of the JAR's `io/nlopez/compose/` file entries (directories excluded since
  2026-10-03), read from the zip central directory (no inflating), matched against `ktrs_compose::NATIVE_JARS`.
  That identifies rule set id and release by content; renamed files still match. Per release two fingerprints: the
  `-all.jar`, and the Maven `ktlint` + `common-ktlint` JARs merged (the Gradle plugin's `-R` JAR; same bytecode up
  to the `-all.jar`'s `shadow/` psiUtil relocation, research/29). The thin Maven JAR alone goes to the hand-off.
- Loading mirrors the jar: standard rules then the 34 compose rules (provider order); in 2.0 mode the
  `RuleSetProviderV3` deprecation WARN and debug lines of `LoadRuleProviders`.
- Port layout mirrors upstream, one Rust file per Kotlin file (`core/`, `core/util/`, `rules/`, `ktlint/` with the
  `KtlintRule` adapter, config, `.editorconfig` properties, the 34 `*Check`s); conventions in its `lib.rs`. PSI
  added to `crates/ktrs-ast/src/psi/` (KtAnnotated, KtCallExpression, KtParameter, KtPsiFactory subset, `getChildren`
  semantics, visibility, …; tests `crates/ktrs-ast/tests/psi_accessors.rs`). Two Kotlin 2.2.21 (1.8 jar) vs 2.4.10
  (2.0 jar) PSI differences are switched on the run's version (`EmbeddedKotlin`): `isIdentifier` on Latin-1
  letters (PreviewNaming's `setName` quoting) and `getContextParameters`.
- Engine fixes found on the way (apply to any rule set): CodeFormatter's "Format was not able to resolve all
  violations…" warnings (both cases) are now logged; 1.8 orders a custom rule set after the standard rules and
  before the `VisitorModifier` ones (now from the merged 1.8 `RuleProviderSorter`); 2.0 `generateEditorConfig`
  with a 1.x rule set loaded throws `requireSingularIdentities`' IllegalArgumentException like the jar
  (`legacy_rule_set.rs`; 1.x properties are wrapped per rule instance, so shared ones have distinct identities).

## Parity

Goldens: `tools/compose-rules-tests/extract-goldens.sh` records compose-rules' own ktlint tests on ktlint 1.8.0
(1.8 `CaseRecorder`) and replays every case on 2.0.0-ALPHA-4 (`ReplayOn20.kt`; `.2_0.*` files where 2.0 differs —
none at this pin). 238 cases in 33 rule dirs (LambdaParameterEventTrailing has no upstream ktlint test; covered by
the corpus). `cargo test -p ktrs-compose --release --test golden`: **476/476** (238 × both modes).

Real code, testbox, `tools/compose-rules/parity.sh` (ktlint-compare.sh, `-R ktlint-compose-0.6.7-all.jar` on both
sides, style ktlint_official, lint rows + `-F` trees; branch at 6c76ea2's code):

| tree | mode | lint rows (jar) | only jar | only ktrs | `-F` files differing |
|---|---|--:|--:|--:|--:|
| dev corpus (6,123 files) | 1.8, compose only | 99 | 0 | 0 | 0 |
| | 1.8, with standard | 214,378 | 1 | 0 | 0 |
| | 2.0, compose only | 99 | 0 | 0 | 0 |
| | 2.0, with standard | 213,106 | 0 | 0 | 0 |
| held-out (15,287 files: compose-samples, accompanist, circuit, mosaic, …) | 1.8, compose only | 586 | 0 | 0 | 0 |
| | 1.8, with standard | 828,982 | 3 | 0 | 1 |
| | 2.0, compose only | 586 | 0 | 0 | 0 |
| | 2.0, with standard | 822,375 | 0 | 0 | 0 |

Exit codes equal everywhere. The 1.8 differences are the known KDoc-lexer rows of research/26 (standard
no-multi-spaces / no-trailing-spaces; Json.kt's ` * ` lines in `-F`), not compose. Held-out compose rows cover 25 of
the 34 rules (preview-public 120, parameter-naming 114, modifier-missing 83, … , 4 vm-injection).
cli-diff with 0.6.7 (native): identical but for `ruleset_jar_format`, whose CodeFormatter WARN the jar logs from
`[pool-1-thread-N]` and ktrs from `[main]` (thread assignment is nondeterministic upstream; already a known 1.8
deviation in research/26); stdin -F matches.

## How to run

- `tools/sync-compose-rules.sh`; `cargo test -p ktrs-compose --release` (goldens, `GOLDEN_FILTER`, `UPDATE_PASSING=1`).
- Goldens (JVM, ~10 min on testbox, background): `tools/sync-ktlint.sh && tools/compose-rules-tests/extract-goldens.sh`.
- Real code (testbox, background, ~1 h): `tools/compose-rules/parity.sh ~/work/ktlint-bench/bin/ktlint-1.8.0
  ~/work/ktlint-bench/bin/ktlint-2.0.0-ALPHA-4 target/release/ktlint <out> ~/work/ktlint-bench/corpus ~/work/holdout/tree`.
- CLI: `RULESET_JAR=<jar> [KTLINT_VERSION=1.8 JAR=…] tools/ktlint-oracle/cli-diff.sh` (a non-pinned compose release
  exercises the hand-off).
- Bumping the pin: change `COMPOSE_RULES_TAG`, port the upstream diff, regenerate goldens, add the new `-all.jar`'s
  and merged Maven JARs' fingerprints to `NATIVE_JARS` (the `compose_jar` unit tests check both and print the
  merged one), rerun parity.sh.

## Remaining

- Build tools (Spotless, ktlint-gradle, kotlinter, Maven) can't reach either path until ktrs has a ktlint step there.
- Only the pinned release is native; older 0.4/0.5/0.6 releases hand off (could be added per release if their rule
  code is unchanged, by fingerprint + goldens).
- Fixed 2026-10-03: `ktrs lint` has `-R` (native compose-rules or hand-off, section A); a `--ktlint-version` inside
  an `@argfile` no longer reaches the jar. Tests: `crates/ktrs-cli/tests/ktrs_lint.rs`,
  `ktlint/hand_off_args/tests.rs`.
- 2.0 `generateEditorConfig` crash text: the JVM's lambda identities (`$$Lambda/0x…@…`) can't be reproduced; ktrs
  prints the same shape with made-up ids (cli-diff masks them).
- Absolute Windows glob outside the working directory (`C:/x/src/*.kt` run from `C:/y`): the jar lints the files, the
  `ktlint` binary says "No files matched". Not ktlint logic: the Windows `java` launcher expands `*`/`?` in arguments
  before `main` (the jar logs no `walkFileTree`; it gets `C:/x/src/A.kt` and takes the existing-file shortcut).
  ktlint's own `fileSequence` (same in ktrs) walks from the working directory, since a glob makes `Path.resolve` throw
  on Windows. Fixed 2026-10-03: both drop-ins expand arguments like the launcher (`ktrs-cli/src/java_launcher.rs`,
  JDK 21 `cmdtoargs.c` + `LauncherHelper.expandArgs`); 14 wildcard cases (abs/relative, `\`, `?`, no match, `**`)
  match the ktlint 2.0 and ktfmt 0.64 jars on Windows.
