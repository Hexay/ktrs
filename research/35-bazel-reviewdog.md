# 35 — Bazel, reviewdog and Danger integrations (design, 2026-10-09)

Design, with A1 and B1 built (2026-10-09): `tools/release/multitool-lock.sh`, `bazel/e2e/rules_lint`, the README
sections and the `ci_*` scenarios of `tools/ktlint-oracle/cli-diff.sh`. Upstream facts were read at the tags named (via
`gh api`); local facts from this tree at `ed15609` (v0.5.1). What the e2e runs showed, including every item first marked
UNVERIFIED, is in A.6 and B.4; where they contradict the design text, the text was corrected. A2, A3, B2 and B3 are
not built.

## TL;DR

| | Recommendation | First deliverable | Size |
|---|---|---|---|
| Bazel | In-repo `bazel/` module `rules_ktrs` (toolchain + rules_kotlin-compatible `ktlint_*` + `ktfmt` targets), published to BCR; rules_lint stays a docs recipe plus small upstream PRs | A1: `multitool.lock.json` release asset + README recipes + an e2e workspace in CI | A1 S (1–2 d), A2 M (4–6 d), A3 S each |
| reviewdog | README recipes now; add `--diff` to `ktrs fmt`/`ktrs lint`; native `rdjsonl` with per-rule suggestions only on demand | B1: recipes + invocations added to `cli-diff.sh` | B1 S (0.5 d), B2 S (1 d), B3 M (2–3 d) |
| Danger | Documentation only | part of B1 | — |

## A. Bazel

### A.1 How Bazel Kotlin projects run ktfmt/ktlint today

| Rule set | Version | Tool it runs | Can a native binary be swapped in? |
|---|---|---|---|
| [rules_kotlin](https://github.com/bazelbuild/rules_kotlin/tree/v2.4.20) `ktlint_test`/`ktlint_fix`/`ktlint_config` (`kotlin/lint.bzl`) | v2.4.20 (2026-09-17) | ktlint **1.8.0** jar (`PINTEREST_KTLINT` in `versions.bzl`, `http_file` → `java_import` → `java_binary` `com.pinterest.ktlint.Main`) | **No.** `_ktlint_tool` is a private attr; no toolchain, no bzlmod tag. Replacing the repo still needs a jar with that main class. Needs a patch or replacement rules. |
| [rules_lint](https://github.com/aspect-build/rules_lint/tree/v2.9.1) `lint_ktlint_aspect` (`lint/ktlint.bzl`) | v2.9.1 (2026-09-18), Bazel ≥ 7.6 | `binary` label; its `ktlint` module tag fetches ktlint **1.2.1** | **Yes**, `binary = <native ktlint>` (observed, A.6). But a JDK is still an action input and toolchain, `baseline_file` is mandatory, the action is `run_shell` (bash on Windows), and there is no fix/patch output. Works on Bazel 8.8.1 only out of the box (A.6). |
| rules_lint `format_multirun` / `format_test` (`format/`) | same | `kotlin = <label>`; its `ktfmt` tag fetches ktfmt **0.46**, wrapped by the user in a `java_binary` | **Yes, zero changes.** Kotlin goes through the generic branch of `format.sh`; nothing JVM-specific. |
| [bazel_rules_detekt](https://github.com/buildfoundation/bazel_rules_detekt) | 0.8.2.x | detekt | Not ktlint/ktfmt. |
| Others | — | BCR has no `rules_ktlint`, `rules_ktfmt`, `ktlint`, `ktfmt` module; only an unmaintained 2021 repo turned up | — |

Command lines that a drop-in must reproduce:

| Caller | Invocation |
|---|---|
| rules_kotlin `ktlint_test` | `ktlint [--editorconfig=<rlocation>] [--android] [--experimental] --relative <src rlocations...>`; bash launcher, `PATH="{java_home}/bin:$PATH"`; no reporter, no `-R`, no baseline; result = stdout + exit code |
| rules_kotlin `ktlint_fix` | `ktlint --format <same flags> --relative ${BUILD_WORKSPACE_DIRECTORY}/<src>...` under `bazel run` |
| rules_lint aspect | `ktlint [--color] <args> [--editorconfig=P] [--baseline=P] [--ruleset=P] --relative` with **no file arguments** (srcs are inputs only, so ktlint's default patterns walk the cwd); run twice, stdout → `<t>.AspectRulesLintKTLint.out` and `.raw_machine_report`; the latter is converted to SARIF by rules_lint's Go parser with errorformat `%f:%l:%c: %m` |
| rules_lint format | fix: `ktfmt <kotlin_fix_args> <files...>`; check: `ktfmt --set-exit-if-changed --dry-run <files...>`; files from `git ls-files` for `*.kt *.ktm *.kts`, via `xargs -0`, cwd = the real workspace |

Consequences for the binaries (check in A1's e2e):
- rules_lint's aspect depends on the no-argument default patterns over a tree of **file symlinks** (sandbox); that
  works. Without a sandbox (`--spawn_strategy=local`, so also Windows) the execroot holds **directory** symlinks, which
  the walk does not enter: "No files matched", exit 0, the target passes unlinted (A.6). Not compared with the jar.
- rules_lint format passes `.ktm` files to `ktfmt`, which skips them silently (A.6).
- rules_lint's ktlint SARIF is rebuilt from plain text, so the plain reporter's `path:line:col: message (rule)` rows are
  the contract there.

### A.2 Options

| | (a) Docs only | (b) `rules_ktrs` module | (c) Upstream PRs to rules_lint |
|---|---|---|---|
| rules_lint format users | works today | adds `@rules_ktrs//:ktfmt` to point `kotlin =` at | built-in Kotlin default (`BUILTIN_TOOL_LABELS`) is possible |
| rules_lint lint users | works, with a JDK fetched for nothing, bash, no fix | same, tool label from the toolchain | optional JDK, srcs as arguments, patch support |
| rules_kotlin `ktlint_*` users | **not possible** (private attr) | **change the `load` line** | n/a (rules_kotlin would need a tool attr: separate PR) |
| Binary fetch | user keeps a `rules_multitool` lockfile (18 entries: 3 tools × 6 platforms) | toolchain, sha256 per platform | their multitool lockfile + daily mirror workflow |
| Upkeep for us | regenerate a snippet per release | one generated file per release, e2e CI | review latency, their release cadence |
| Size | S | M | S per PR, acceptance uncertain |

**Recommendation: (a) now as A1, then (b) as A2, then (c) as A3 where it removes friction.** Reasons:
- Only (b) reaches rules_kotlin's `ktlint_test` users, and "change the coordinates, keep the DSL" is what the Gradle
  and Maven drop-ins already do (research/29, 31).
- (a) costs a day and tests the binaries under Bazel before any Starlark is written.
- (c) alone leaves us dependent on rules_lint's pinning choices (it still ships ktlint 1.2.1 and ktfmt 0.46).
- We do not write our own lint aspect in A2: rules_lint's works with the native binary, and its helpers live under
  `//lint/private` (loading `lint_aspect.bzl` from another module works, A.6; it is still undocumented).

### A.3 What to build

**A1 — recipes and lockfile (S).**
- `tools/release/multitool-lock.sh <tag> <SHA256SUMS>`: prints a `rules_multitool` lockfile with keys `ktrs`, `ktfmt`,
  `ktlint`, each with six `kind: archive` binaries (`url`, `sha256`, `os`, `cpu`, `file: ktrs-<tag>-<target>/<tool>[.exe]`).
  Sources `release-assets.sh` like the Homebrew and Scoop generators. The release job uploads it as
  `ktrs-<tag>.multitool.lock.json`. Lockfile schema: [rules_multitool](https://github.com/bazel-contrib/rules_multitool) v1.11.1.
  **Built**; the v0.5.1 output validates against that tag's `lockfile.schema.json`. v0.5.1 itself has no such asset.
- README "Integrations → Bazel": `multitool.hub(lockfile = ...)`; `format_multirun(kotlin = "@multitool//tools/ktfmt",
  kotlin_fix_args = [...])`; `lint_ktlint_aspect(binary = "@multitool//tools/ktlint", editorconfig = ..., baseline_file = ...)`;
  a note that rules_kotlin's rules need A2.
- `bazel/e2e/rules_lint/`: a workspace with those recipes and a few fixture sources. **Built**: `test.sh <lockfile>`
  asserts the outcomes; ci.yml's `bazel` job runs it on `target/debug` binaries (`bazel/e2e/local-lock.sh`, `file://`
  urls), release.yml's on the published lockfile. Its `kt_jvm_library` is a stand-in rule of the same kind name, so the
  job needs no Kotlin compiler; the real rules_kotlin 2.4.20 rule gave the same report once (A.6).

**A2 — `rules_ktrs` (M).** Layout, in this repo (same release tag, same SHA256SUMS, binaries built in the same CI):

```
bazel/
  MODULE.bazel                 module(name = "rules_ktrs"); deps: bazel_skylib, platforms, rules_shell
  ktrs/extensions.bzl          `ktrs` extension, tag `toolchain()`; tag `local(ktrs=, ktfmt=, ktlint=)` for dev/CI
  ktrs/repositories.bzl        per-platform repo rule (download_and_extract by sha256) + WORKSPACE macro
  ktrs/toolchain.bzl           `ktrs_toolchain` -> KtrsInfo(ktrs, ktfmt, ktlint); toolchain_type
  ktrs/private/versions.bzl    GENERATED: KTRS_VERSION, KTRS_SHA256 = {target triple: hex}
  ktrs/private/toolchains_repo.bzl   `toolchain(exec_compatible_with = ...)` per platform (no download on registration)
  ktlint/defs.bzl              ktlint_test, ktlint_fix, ktlint_config: rules_kotlin's names and attrs
  ktfmt/defs.bzl               ktfmt_test, ktfmt_fix (srcs, style, editorconfigs)
  BUILD.bazel                  `:ktfmt`, `:ktlint`, `:ktrs` resolved-toolchain executables (one runfile each)
  e2e/smoke/, e2e/rules_lint/, e2e/rules_kotlin_parity/
.bcr/bazel/{metadata.template.json,source.template.json,presubmit.yml}
tools/release/bazel-versions.sh, tools/release/bazel-archive.sh
```

Rules:

| Rule | Shape |
|---|---|
| `ktlint_config` | `editorconfig`, `android_rules_enabled`, `experimental_rules_enabled` (as rules_kotlin) + `ktlint_version` (`"1.8"` default, `"2.0"`) + `editorconfigs` (label list: every `.editorconfig` discovery may read) |
| `ktlint_test` | Lint runs as a **build action** over execroot paths: `ctx.actions.run(ktlint, --relative, --reporter=plain,output=<t>.txt, --reporter=sarif,output=<t>.sarif, flags, srcs via param file)`. Outputs: both reports + an exit-code file. The test prints the text report and exits with the recorded code. Extra output group `ktrs_report`. |
| `ktlint_fix` | `bazel run`: `ktlint --format ... ${BUILD_WORKSPACE_DIRECTORY}/<src>`, as rules_kotlin |
| `ktfmt_test` / `ktfmt_fix` | Same split: action runs `ktfmt --set-exit-if-changed --dry-run [--<style>-style] [--enable-editorconfig]`, test reports; fix runs in the workspace |

ktrs changes A2 needs (hidden `--ktrs-*` options, the precedent is `crates/ktrs-cli/src/ktlint/gradle.rs`):

| Change | Why | Size |
|---|---|---|
| `--ktrs-exit-code-file=<path>`: write the exit code, exit 0 | `ctx.actions.run` cannot redirect or capture `$?`; avoids `run_shell`, so the action needs no bash on Windows | S |
| `--ktrs-editorconfig-root=<dir>`: stop the upward `.editorconfig` walk there | hermeticity, see A.5; `ResourcePropertiesService::root_directories` already exists (`crates/ktrs-editorconfig/src/resource_properties_service.rs:74`) | S for ktlint; ktfmt uses ec4rs, check separately |
| ktfmt: file for the `--dry-run` list (or reuse the exit-code file and stdout via a wrapper) | same reason as the first row | S |

Test launchers stay bash scripts in A2 (as in rules_kotlin and rules_lint), so on Windows the lint *action* is
bash-free but `bazel test` still needs bash. A `.bat` launcher is a later item.

**Per-release sha256 table.** `tools/release/bazel-versions.sh <tag> <SHA256SUMS> > bazel/ktrs/private/versions.bzl`,
the same shape as `homebrew-formula.sh` (source `release-assets.sh`, one `asset_sha` per target, whole file printed).
One module version pins one ktrs version (the rules_uv pattern), so there is no multi-version table to merge. In
`release.yml`:
1. job `release`, after `SHA256SUMS` is computed: `bazel-archive.sh` copies `bazel/`, writes the generated
   `versions.bzl`, stamps `module(version = ...)`, and tars it as `rules_ktrs-<tag>.tar.gz` (MODULE.bazel at the archive
   root, as rules_lint's `release_prep.sh` does for its sub-modules). The name must not match `ktrs-*.tar.gz`. It is
   uploaded with the other assets.
2. new job `bazel` (after `release`, like `homebrew`/`scoop`): commits `versions.bzl` to master so `git_override` works.
3. new job `bcr`, only when `BCR_PUBLISH_TOKEN` is set (like `WINGET_TOKEN`): [publish-to-bcr](https://github.com/bazel-contrib/publish-to-bcr)
   v1.5.1 reusable workflow with `module_roots: bazel`, `attest: false` (attestations need bazel-contrib's release
   workflow), `draft: true`.

**BCR requirements** ([docs](https://github.com/bazelbuild/bazel-central-registry/blob/main/docs/README.md),
[policies](https://github.com/bazelbuild/bazel-central-registry/blob/main/docs/bcr-policies.md)):
- `modules/rules_ktrs/metadata.json` (homepage, maintainers with GitHub user id, `repository: ["github:Hexay/ktrs"]`).
- Per version: `MODULE.bazel` identical to the archive's, `source.json` (`url` of a **stable uploaded release asset**,
  not the auto-generated source archive; `integrity`; `strip_prefix`), `presubmit.yml` with `bcr_test_module:
  module_path: e2e/smoke` and explicit Bazel versions and platforms.
- Dependencies must already be in BCR. Entries are add-only (fixes are `.bcr.N`). A new module needs a BCR maintainer's
  approval. Needs a fork of the registry and a classic PAT.
- A module in a monorepo subdirectory is supported (`moduleRoots`, `.bcr/<path>/` templates).
- Name: reviewers discourage official-sounding `rules_` names for generic things; `rules_ktrs` names our own tool, so it
  should pass (**UNVERIFIED**; fallback `ktrs`).

**A3 — upstream PRs (S each, after A2 has users):** rules_lint docs/example mentioning native ktlint/ktfmt; make the
JDK optional in `lint_ktlint_aspect`; pass srcs as arguments. rules_kotlin: a public tool attribute on `ktlint_*`.

### A.4 CI

| Check | Where | What |
|---|---|---|
| e2e on built binaries | new `.github/workflows/bazel.yml`: on `bazel/**` and CLI changes, plus nightly | `cargo build --bins`, then `bazel test //...` in `bazel/e2e/*` with the `local(...)` toolchain tag; matrix ubuntu/macos/windows × Bazel 7.x/8.x/9.x (`USE_BAZEL_VERSION`, `bazel-contrib/setup-bazel`) |
| Parity with rules_kotlin | `tools/bazel/parity.sh`, same job, Linux | Same fixture workspace run twice (rules_kotlin 2.4.20's `ktlint_test`/`ktlint_fix`, then ours with the load line swapped); compare test logs, exit status, fixed sources. Mirrors `tools/ktlint-gradle/parity.sh`. |
| rules_lint recipes | same job | `bazel run //:format.check`, `bazel build --aspects=...%ktlint --output_groups=rules_lint_human` in `e2e/rules_lint`; compare the `.out` files with the jar's on the same fixture |
| Released artifact | `release.yml` job after `release` | `e2e/smoke` against the downloaded `rules_ktrs-<tag>.tar.gz` (real download path, real sha256) |
| BCR presubmit | BCR's own CI | `e2e/smoke` on debian, ubuntu, macos, windows |

None of this can run on the dev machine (RAM); it is CI-only by design.

### A.5 Output and caching details

| Topic | Behaviour | Design answer |
|---|---|---|
| Hermetic tool | One static binary per platform, fetched by sha256; no JDK | Toolchain; never `PATH` |
| `.editorconfig` discovery in the sandbox | The sandbox contains only declared inputs ([sandboxing](https://bazel.build/docs/sandboxing)), as symlinks. ktrs walks lexical parents and never canonicalizes (no `canonicalize`/`read_link` in ktrs-cli, ktrs-editorconfig, ktrs-lint), so it does not follow symlinks back into the source tree. | `.editorconfig` files must be declared: `ktlint_config.editorconfigs`. An undeclared one is silently not applied; document it. |
| Walk leaves the execroot | Without `root = true` at the workspace root, the walk continues through the output base up to `/`, so a `~/.editorconfig` can leak in (the jar has the same flaw). | `--ktrs-editorconfig-root=<execroot>` passed by the rules |
| `--editorconfig=<file>` | In ktlint this only supplies defaults for properties no discovered file sets; rules_kotlin and rules_lint both pass it | Keep the flag for compatibility; discovery inputs are the separate `editorconfigs` attr |
| ktfmt | Reads `.editorconfig` only with `--enable-editorconfig` | Off unless `editorconfigs` is set |
| `HOME` | ktlint's SARIF writes `originalUriBaseIds` from `user.home`; `~` expansion uses it too (`command_line.rs:63`) | Observed (A.6): the SARIF differs with `HOME` and is `file:///` without it. A2's actions must run without `HOME` (or with a fixed one); its e2e asserts it. |
| `-R` rule sets | compose-rules 0.6.7 runs natively; any other jar hands off to the ktlint jar, which downloads and needs Java | A2 exposes no `ruleset` attr; A3+ could add compose-rules only. Hand-off inside an action is unsupported. |
| Report files | `--reporter=plain,output=` and `--reporter=sarif,output=` are declared outputs; `--relative` makes paths execroot-relative, which equals workspace-relative for source files | Output group `ktrs_report`; with `--remote_download_outputs=minimal` add `--remote_download_regex` for them |
| Checkstyle | Available as a reporter | `ktlint_config.reporters` later; not in A2 |
| Exit codes | ktlint: 0 clean, 1 violations, others for usage/IO; 1.8 and 2.0 differ (research/22) | Action always succeeds and records the code; the test fails on non-zero. A broken invocation must still fail the action: only codes 0/1 are recorded, the rest propagate. |
| `--keep_going` | A failing build action stops dependents; a failing test does not | Lint results are test results, so `bazel test -k //...` reports every target; reports are cached per target |
| rules_lint aspect | Exits 0 and writes `*.exit_code` unless `--@aspect_rules_lint//lint:fail_on_violation` | Unchanged; our binary is just the `binary` |
| Cache key | binary, srcs, declared `.editorconfig` files, flags | A change to a root `.editorconfig` invalidates every lint action (same as rules_lint's ruff) |
| Fix | Must write to the source tree | `bazel run` only (`ktlint_fix`, `ktfmt_fix`, rules_lint `format`); never in a build action |
| Windows | `ctx.actions.run` on an `.exe` needs no bash; `run_shell` and sh launchers do | See A.3 |

### A.6 Observed in the A1 e2e (testbox, Linux x86-64, 2026-10-09)

`bazel/e2e/rules_lint` with rules_lint 2.9.1, rules_multitool 1.11.1 and the v0.5.1 release binaries, Bazel 8.8.1
unless noted. `test.sh` passes with the generated v0.5.1 lockfile (17 s, 15 s repeated; the first cold run spent 329 s
in module resolution and downloads) and with a `local-lock.sh` lockfile of `file://` urls.

| Question | Observed |
|---|---|
| Does `format_multirun(kotlin = "@multitool//tools/ktfmt")` work unchanged? | Yes. `format.check` lists `src/Violation.kt` and exits 1; `format` rewrites it; `format.check` then exits 0. |
| `.ktm` files | rules_lint passes them; `ktfmt` neither lists nor rewrites them and reports nothing. |
| Does `lint_ktlint_aspect(binary = <native ktlint>)` work? | Yes: `.out`, `.out.exit_code` (1 / 0), SARIF `.report` with `src/Violation.kt` rows; `lint_test` passes or fails on the exit code; `fail_on_violation` fails the build. |
| `baseline_file = None` | Rejected at load: "Aspect attribute '_baseline_file' has no default value". A baseline file is mandatory; one without entries works. |
| No file arguments | Command line: `ktlint --color --editorconfig=.editorconfig --baseline=ktlint-baseline.xml --relative`. In the sandbox only the target's own srcs are linted (a second file of the package does not appear). |
| No sandbox (`--spawn_strategy=local`) | `No files matched [**/*.kt, **/*.kts]`, exit 0: nothing is linted and the target passes. The execroot's `src` is a directory symlink. Not compared with the jar. |
| ktlint's log lines | They go to stdout, so into `.out` (an INFO line even for a clean target). `args = ["--log-level=none"]` leaves only the report; a clean target's `.out` is then empty. |
| Is a JDK needed? | As an input, yes: 116 of the action's 120 inputs are `local_jdk` files and `JAVA_HOME` is set; the binary never runs it. With no `java` on `PATH`, analysis fails ("Cannot find Java binary bin/java ... local_jdk"). `--java_runtime_version=remotejdk_21` fixes that by downloading a 353 MB JDK. |
| The `editorconfig =` file | A declared input and applied: removing its `ktlint_standard_no-wildcard-imports = disabled` line brings the row back. |
| Undeclared nested `src/.editorconfig` | Not applied in the sandbox (it is not there). |
| Walk above the execroot | Confirmed: without `root = true`, a `.editorconfig` in a directory above Bazel's output root (`ktlint_standard = disabled`) emptied the report, exit 0. With `root = true` it is ignored. |
| rules_lint's own multitool entries | None named `ktfmt` or `ktlint`, so the shared `multitool` hub has no clash. |
| `//lint/private:lint_aspect.bzl` from another module | Loads (no `visibility()`); undocumented all the same. |
| Real `kt_jvm_library` (rules_kotlin 2.4.20) instead of the stand-in | Same report and exit code. |
| Bazel 7.7.1 | Format works; the ktlint aspect fails to load: `@rules_java//java/common/rules:java_runtime.bzl is not visible for loading from package @aspect_rules_lint//lint`. Upstream; a newer rules_java was not tried. |
| Bazel 9.3.0 | Fails in analysis: rules_lint's transitive rules_go uses the removed `CcInfo` global. Upstream; overrides were not tried. |
| `--reporter=sarif` and `HOME` (for A2) | `originalUriBaseIds.%SRCROOT%.uri` is `file://$HOME/`; `file:///` with an empty environment. Two machines differ unless the action has no `HOME`. |

Still **UNVERIFIED**: whether BCR reviewers accept the name `rules_ktrs` (only a submission can tell). Not run:
Windows and macOS, remote execution.

## B. reviewdog and Danger

### B.1 What reviewdog can consume today

[reviewdog](https://github.com/reviewdog/reviewdog) v0.21.2 (2026-09-18). `-f=ktlint` does not exist (no Kotlin entry in
[errorformat/fmts](https://github.com/reviewdog/errorformat/tree/master/fmts)).

| reviewdog input | ktrs output today | Suggestions |
|---|---|---|
| `-f=checkstyle` | `ktlint --reporter=checkstyle --relative`, `ktrs lint --reporter checkstyle` | no (format has none) |
| `-f=sarif` | `--reporter=sarif`, **without** `--relative` (B.4) | no: reviewdog maps SARIF `fixes`, but ktlint's reporter (and our 1:1 port, `reporter/sarif.rs`) emits only `startLine`/`startColumn` |
| `-efm="%f:%l:%c: %m"` | default plain reporter | no |
| `-f=diff` | none directly: `ktfmt`/`ktrs fmt`/`ktlint -F` write in place, then `git diff` | **yes**, one per hunk, no rule attribution |
| `-f=rdjsonl` / `rdjson` | none | yes |

Reporters of the drop-in (`reporter/mod.rs`): `baseline`, `checkstyle`, `format`, `html`, `json`, `plain`,
`plain-summary`, `sarif`. `ktrs lint` takes the same `--reporter`. `ktrs fmt` emits only the changed-file list
(`--check`) or rewritten files; it has no diff output.

Suggestions are rendered as GitHub "suggested changes" only by `-reporter=github-pr-review` (also
`gitlab-mr-discussion`), only inside the diff context, and only when comment range equals suggestion range; at most 30
comments per run. Fork PRs fall back to log annotations without suggestions.

**Is a native `rdjsonl` reporter worth adding?**

| Step | What | Gives | Size |
|---|---|---|---|
| B1 | Recipes: checkstyle pipe for diagnostics; format in place + [reviewdog/action-suggester](https://github.com/reviewdog/action-suggester) v1.26.2 for suggestions | Everything reviewdog offers, no code | S |
| B2 | `ktrs fmt --diff` and `ktrs lint --format --diff`: unified diff on stdout, no writes, exit 1 if non-empty | `... --diff \| reviewdog -f=diff -f.diff.strip=0 -reporter=github-pr-review` without dirtying the tree; works for GitLab; useful without reviewdog (ruff and biome have it) | S |
| B3 | `ktrs lint --reporter rdjsonl`: one diagnostic per violation with `code.value` = rule id, `source.name`, severity, and `suggestions[]` from a single-error fix | Per-rule suggestions in one pass | M |

Recommendation: **B1 and B2; B3 only if asked for.** B3's costs:
- The fix text per violation needs one engine run per error (`fix_one` in `crates/ktrs-lsp/src/ktlint.rs:134`, edits via
  `text.rs::text_edits`), which would have to move to a crate the CLI shares.
- RDFormat columns are 1-based **UTF-8 byte** offsets with an exclusive end; ktlint's are character columns.
- GitHub drops a suggestion whose range differs from the comment's, and many ktlint fixes are not local to the reported
  position, so a share of suggestions would not render. B2's hunks avoid that.
- It must live in `ktrs lint` only: the `ktlint` drop-in's reporter ids are ktlint's.

Precedent: only ruff (`--output-format rdjson`) and biome (`--reporter=rdjson`) emit it natively, both with
suggestions; neither ships a reviewdog action. shellcheck's action uses the diff route (B2's shape).

### B.2 Danger

| Plugin | Version | Input | With ktrs |
|---|---|---|---|
| [danger-ktlint](https://github.com/mataku/danger-ktlint) (Ruby) | 0.0.9 (2023, dormant) | Spawns `ktlint <files> --reporter=json --relative --log-level=none` from `PATH`, or reads a json report (`skip_lint`, `report_file`) | drop-in on `PATH` works as is |
| danger-checkstyle_format (Ruby) | 0.1.1 (2017) | checkstyle XML file | `--reporter=checkstyle,output=` |
| `io.github.bastosss77:ktlint-danger-kotlin` (danger-kotlin 1.3.4) | 0.5.1 (2025-10) | ktlint json, sarif or checkstyle file | any of the three reporters |
| `danger-kotlin-checkstyle_format`, `danger-kotlin-reporter-plugin` | 2022–2023 | checkstyle XML | same |
| danger-js `danger-plugin-lint-report` | 1.8.1 (2023) | checkstyle XML | same |

Every plugin reads a ktlint json or checkstyle file, or runs `ktlint` from `PATH`. **Nothing to build.** Add
danger-ktlint's exact command line to `tools/ktlint-oracle/cli-diff.sh` so its json stays byte-identical to the jar's,
and a short README paragraph. No Danger plugin posts suggestions.

### B.3 Action or recipes; relation to native annotations and `--changed-since`

Existing: [ScaCap/action-ktlint](https://github.com/ScaCap/action-ktlint) v1.9.0 (2024-07, stale; Docker with
OpenJDK 11, download URL hard-wired to pinterest/ktlint, checkstyle only, reviewdog v0.19.0). No reviewdog-org Kotlin
action, no ktfmt one. A composite action on [action-composite-template](https://github.com/reviewdog/action-composite-template)
is `action.yml`, a ~40-line script and four or five workflows (release bump, depup, tests).

| | Native annotations + `--changed-since` (in progress elsewhere) | reviewdog |
|---|---|---|
| Setup | none, no token, works on fork PRs | token with `pull-requests: write`; forks degrade to annotations |
| Scope | files changed since a ref | lines: `-filter-mode=added\|diff_context\|file\|nofilter` |
| Volume | GitHub's REST docs (Checks, read 2026-10-09): "limited to 10 warning annotations and 10 error annotations per step", and 50 annotations per Checks API request. A per-job cap of 50 is not in the docs. | review comments, 30 per run; Checks API reporters uncapped by those limits |
| Committable suggestions | no | yes (`github-pr-review`, GitLab MR discussions) |
| Other hosts | GitHub only | GitLab, Bitbucket, Gerrit, Gitea |

They overlap only on "show violations on a GitHub PR", where the native path is simpler and should be the README
default. reviewdog remains worth documenting for line-level filtering, suggestions and non-GitHub hosts; that is a docs
cost plus B2. `--changed-since` also helps reviewdog runs (fewer files to lint before filtering).

Recommendation: **README recipes, no separate action repo now.** If demand appears, add it as a subdirectory action in
this repo (`uses: Hexay/ktrs/reviewdog@<tag>`), which shares the release tag and `install.sh`; a separate
`Hexay/action-ktrs` only matters for a second Marketplace listing and costs its own release and depup upkeep.

### B.4 Observed for B1 (testbox, 2026-10-09)

reviewdog 0.21.2 with `-reporter=local -filter-mode=added` on a git repo with one added violating file, v0.5.1 binaries.

| Input | Observed |
|---|---|
| `ktlint --relative --reporter=checkstyle,output=ktlint.xml`, then `reviewdog -f=checkstyle < ktlint.xml` | All five rows reported; `-fail-level=error` exits 1. `ktrs lint --reporter checkstyle` piped in gives the same rows. |
| `ktlint --relative \| reviewdog -efm="%f:%l:%c: %m"` | Same rows. |
| `--reporter=sarif` with `--relative` | **Nothing reported**, exit 0: the uris are relative to the working directory but declared relative to `%SRCROOT%` = `file://$HOME/`, so reviewdog resolves them to files that are not in the diff. |
| `--reporter=sarif` without `--relative` | Rows reported (uris are relative to `$HOME`, as the base says). |
| `ktlint -F` or `ktfmt`, then `git diff \| reviewdog -f=diff -f.diff.strip=1` | One suggestion per hunk (checked with `-reporter=rdjsonl`). action-suggester itself needs a pull request and was not run. |
| danger-ktlint's command line and the report-file invocations | `cli-diff.sh` scenarios `ci_*` (8): identical to the 2.0.0-ALPHA-4 and the 1.8.0 jars. No Danger run was made. |

## Sizes

| Item | Contents | Estimate |
|---|---|---|
| A1 | lockfile generator (~40 lines sh), release step, README section, `e2e/rules_lint` + CI job | 1–2 d |
| A2 | ~600 lines Starlark, 2–3 hidden CLI options, generator + archive scripts, 3 release jobs, e2e + parity, BCR templates and first submission | 4–6 d + BCR review time |
| A3 | upstream PRs | 0.5 d each |
| B1 | README recipes (reviewdog, Danger), `cli-diff.sh` cases | 0.5 d |
| B2 | `--diff` for `ktrs fmt` and `ktrs lint --format`, tests | 1 d |
| B3 | rdjsonl reporter with suggestions | 2–3 d |
| Action | subdirectory composite action + test workflow | 1 d |

## Decisions for the owner

| # | Decision | Recommended |
|---|---|---|
| 1 | Bazel path | A1 now, A2 next, A3 opportunistically |
| 2 | Where the module lives | In this repo under `bazel/`, released with the ktrs tag; not a separate repo |
| 3 | Module name | `rules_ktrs` (fallback `ktrs` if BCR reviewers object) |
| 4 | Publish to BCR | Yes, from A2's first release; needs a registry fork and a `BCR_PUBLISH_TOKEN` secret; job skipped without it |
| 5 | Version model | One module version = one ktrs version; no multi-version table |
| 6 | WORKSPACE support | Ship `repositories.bzl` (cheap), test it on Bazel 7 only; bzlmod is the documented path |
| 7 | Default ktlint mode of the Bazel `ktlint_*` rules | `1.8` (what rules_kotlin pins; matches the settled build-plugin default), attr to select `2.0` |
| 8 | Own lint aspect vs rules_lint's | Use rules_lint's with our binary; improve it upstream (A3) |
| 9 | Hidden CLI options `--ktrs-exit-code-file`, `--ktrs-editorconfig-root` | Yes, both |
| 10 | Rule set jars in Bazel rules | Not in A2; later compose-rules only |
| 11 | Windows `bazel test` without bash | Defer; actions are bash-free from A2 |
| 12 | reviewdog | B1 + B2; no rdjsonl reporter until someone asks |
| 13 | Danger | Docs + a `cli-diff.sh` case |
| 14 | reviewdog action | No separate repo; subdirectory action later if asked |
| 15 | README default for PR feedback on GitHub | Native annotations; reviewdog as the "suggestions / other hosts" recipe |
