# 32 — IDE / editor / LSP integration landscape

2026-10-07. Builds on 08 §2 (editor table, C5) and 10 (demand); not repeated here. **F** = fact (checked today,
source linked), **I** = inference.

## TL;DR

- No ktlint/ktfmt LSP exists anywhere (GitHub search 2026-10-07). Every non-JetBrains editor integration shells out
  to the `ktlint`/`ktfmt` CLIs, so our drop-in binaries already work there; the gaps are a few CLI edge cases to
  verify, and docs.
- Official kotlin-lsp formats with the IntelliJ formatter only, with no ktfmt/ktlint and no delegation. An editor needs a second
  tool for ktlint/ktfmt either way, which is the slot a `ktrs lsp` fills.
- VS Code has no dominant ktlint/ktfmt extension: every active one is under 2.5k installs.
- IntelliJ is the largest pool (Ktlint plugin 460k), but it runs ktlint in-process on a warm JVM. Its engine seam
  depends on ktlint's per-error `AutocorrectDecision` callback, which `ktrs serve` doesn't expose.
- ktfmt 0.65 (in the changelog, untagged) adds a **GraalVM native image** and **range formatting**, and moves to
  `org.jetbrains.kotlinx`.

## 1. IntelliJ

### Ktlint plugin: nbadal/ktlint-intellij-plugin (F)

| | |
|---|---|
| Marketplace | id 15057, `com.nbadal.ktlint`, **460,075 downloads**, 0.30.5 on 2026-07-17 (IDE 251–262.*), ~70 MB. https://plugins.jetbrains.com/plugin/15057-ktlint |
| License | MIT. Co-maintained by Paul Dingemans (10). |
| Engine | In-process `KtlintRuleEngine`. `ktlint-lib` shades and relocates the ktlint jars so they don't clash with the IDE's Kotlin compiler, and ships the standard rule sets **1.0.1 … 1.8.0** (11 dirs, default `ktlint = "1.8.0"` in `gradle/libs.versions.toml`). No 2.0-ALPHA. |
| Custom rule sets | External jar paths in settings (`externalJarPaths`), loaded by a `RelocatingClassLoader`. Rule set version can come from the settings or from a shared `ktlint-plugins.properties`. |
| Features | `externalAnnotator` (lint as you type), `postFormatProcessor` (ktlint after Reformat Code), `actionOnSave` (after FormatOnSave), intentions: format, autocorrect one violation, suppress, show all. Baseline support. Modes: NOT_INITIALIZED / DISTRACT_FREE / MANUAL / DISABLED. Plus `.editorconfig` option descriptors. |
| Engine seam | `KtlintRuleEngineWrapper.executeKtlint` (`ktlint-plugin/src/main/kotlin/com/nbadal/ktlint/KtlintRuleEngineWrapper.kt`) calls `lint(code)` or `format(code) { e -> AutocorrectDecision }` with three handlers: whole file (minus baseline errors), **block** (only errors whose line start falls in a range) and **single violation**. Code is `Code.fromSnippetWithPath(documentText, path)`, so `.editorconfig` resolves from the path. |

I: a ktrs engine swap is feasible: one class plus settings, with the UI, annotator and intentions all reused. It needs
`serve` requests for "format but autocorrect only errors in range R / only error E / not baseline B", plus lint rows
with offsets. ktrs-lint already has an `AutocorrectDecision` internally (`crates/ktrs-lint/src/engine/ktlint_rule_engine.rs`),
but the serve protocol (`crates/ktrs-cli/src/serve.rs`) has no decision or range keys. Pick fork or upstream PR at the
time; Dingemans is the sole ktlint maintainer.

### ktfmt plugin: Kotlin/ktfmt `ktfmt_idea_plugin` (F)

| | |
|---|---|
| Marketplace | id 14912, `com.facebook.ktfmt_idea_plugin`, **114,330 downloads**, 1.3.0.64 on 2026-06-24 (since 223.7571). https://plugins.jetbrains.com/plugin/14912-ktfmt |
| License | Apache-2.0 (repo moved facebook → https://github.com/Kotlin/ktfmt). |
| Engine | `KtfmtFormattingService : AsyncDocumentFormattingService`, in-process `Formatter.format(options, text)`. Styles meta/google/kotlinlang/custom. No lint, no on-save of its own (uses the IDE's Reformat on save). |
| 0.64 | `getFeatures() = emptySet()`, i.e. whole file only. |
| main (0.65) | `Feature.FORMAT_FRAGMENTS`, `format(..., characterRanges=request.toRanges())`. Package `org.jetbrains.kotlinx.ktfmt`, plugin.xml `<vendor>JetBrains</vendor>`, `<id>org.jetbrains.ktfmt_idea_plugin</id>` (commit 2026-10-06). |
| ktfmt 0.65 changelog | `--lines/--line/--offset/--length` (google-java-format style), selected-range in IntelliJ, **"GraalVM native image support"**, `--stdin-name` for `.editorconfig`. https://github.com/Kotlin/ktfmt/blob/main/CHANGELOG.md |

I: the plugin's new id means JetBrains is taking ktfmt over. A first-party native ktfmt erodes the
"fast `ktfmt` binary" pitch. Our edges become ktlint, the shared server, and Wasm. Porting 0.65's range formatting is a
prerequisite for LSP `rangeFormatting` parity.

### Plugins that run an external formatter binary (F)

| Plugin | Downloads | Mechanism |
|---|---|---|
| block/kotlin-formatter (id 26482) | 7,743 | `AsyncFormattingService`. With `.idea/kotlin-formatter.properties` `kotlin-formatter.script-path=bin/kotlin-format` it runs `ProcessBuilder(scriptPath, "--set-exit-if-changed", "-")` (`idea-plugin/.../KotlinReformatService.kt`); otherwise embedded ktfmt. Apache-2.0. https://github.com/block/kotlin-formatter |
| Ruff (koxudaxi, id 20574) | 500,742 | Runs the ruff CLI, or `ruff server` via the platform LSP API **or** LSP4IJ (`only-lsp4ij.xml`). https://github.com/koxudaxi/ruff-pycharm-plugin |
| Biome (id 22761) | 214,057 | Platform LSP API (`platform.lsp.serverSupportProvider`; depends on `com.intellij.modules.ultimate`). Binary from node_modules/PATH. |
| dprint (id 18192) | 21,668 | dprint's editor-service protocol over a child process. https://github.com/dprint/dprint-intellij |

I: Block's plugin + a `bin/kotlin-format` script that runs `exec ktfmt --kotlinlang-style "$@"` (or any style flags)
is a **zero-code IntelliJ path to ktrs ktfmt today**. `--set-exit-if-changed -` is a ktfmt flag we port.

### JetBrains policy (F)

- Approval Guidelines v1.3 (effective 2026-03-31) have **no rule on bundled native binaries, runtime downloads,
  size or signing**. Review is manual for every version, and plugins must not introduce security issues or degrade
  performance. https://plugins.jetbrains.com/docs/marketplace/jetbrains-marketplace-approval-guidelines.html
- Precedent: the ktlint plugin ships about 70 MB of jars. The Ruff, Biome and dprint plugins rely on a user-installed binary.
- LSP API: free in all IntelliJ IDEA from 2025.3 (unified distribution).
  https://blog.jetbrains.com/platform/2025/09/the-lsp-api-is-now-available-to-all-intellij-idea-users-and-plugin-developers/
  The **client API is open-sourced in 2026.2** (and 2026.1.4), so Android Studio support is expected. Until then, LSP4IJ
  (Red Hat) is the Android Studio route. https://blog.jetbrains.com/platform/2026/06/open-sourcing-the-lsp-client-api-in-intellij-idea-2026-2/

## 2. LSP and terminal editors

### Kotlin language servers (F)

| | Status | Formatting |
|---|---|---|
| Kotlin/kotlin-lsp | Alpha. v263.6379.0 on 2026-10-03, 3.5k stars, Apache-2.0, IntelliJ-based, bundles its JRE (needs JDK 25). https://github.com/Kotlin/kotlin-lsp | `LSCommonFormattingProvider` → `CodeStyleManager.reformatText` (IntelliJ formatter + `.editorconfig`). Full and range. **No ktfmt/ktlint and no external-formatter hook.** Issue #219 (2026-06-09, no reply): "formatting differs slightly from `ktlintFormat`". Diagnostics are IntelliJ inspections only. |
| fwcd/kotlin-language-server | README: "can be considered **deprecated**". Last release 1.3.13 (2025-01-18), last push 2025-06-02. | Embedded ktfmt (`formatting/KtfmtFormatter.kt`). |

I: since kotlin-lsp can't run ktlint, users run a second LSP or a CLI formatter next to it. All the editors below support that.

### CLI integrations that already work with our binaries (F, source files read today)

| Tool | Exact invocation | Source |
|---|---|---|
| conform.nvim `ktlint` | `ktlint --format --stdin --log-level=none` | `lua/conform/formatters/ktlint.lua` |
| conform.nvim `ktfmt` | `ktfmt -` | `lua/conform/formatters/ktfmt.lua` |
| nvim-lint `ktlint` | `ktlint --reporter=json --stdin`, **parses stderr**, ignores exit code | `lua/lint/linters/ktlint.lua` |
| none-ls diagnostics `ktlint` | `ktlint --relative --reporter=json --log-level=none --stdin` (json) | `null-ls/builtins/diagnostics/ktlint.lua` |
| none-ls formatting `ktlint` | `ktlint --format --stdin --log-level=none` | `null-ls/builtins/formatting/ktlint.lua` |
| ALE linter/fixer | `ktlint [opts] [--ruleset X] --stdin` (+ ` --format`), plain reporter `file:line:col: msg` on **stderr** | `ale_linters/kotlin/ktlint.vim`, `autoload/ale/handlers/ktlint.vim` |
| apheleia (default for kotlin-mode / kotlin-ts-mode) | `ktlint --log-level=none --stdin -F -` (note the trailing `-` pattern) | `apheleia-formatters.el:118` |
| efmls-configs-nvim | detekt only, no ktlint/ktfmt | — |
| Helix | `language-servers = ["kotlin-language-server"]`; `kotlin-lsp` is defined but not the default. No formatter set: user adds `formatter = { command = "ktfmt", args = ["-"] }` | `languages.toml:2411` |
| Zed (zed-extensions/kotlin) | fwcd server by default, kotlin-lsp opt-in. Formatter via `"languages": {"Kotlin": {"formatter": {"external": {"command": "ktfmt", "arguments": ["-"]}}}}` | https://github.com/zed-extensions/kotlin |

Verify these in `tools/ktlint-oracle/cli-diff.sh` before advertising "zero work":
1. Reporter stream with `--stdin` (stderr, which both nvim-lint and ALE rely on).
2. `--stdin -F -`, i.e. a `-` pattern alongside `--stdin` (apheleia).
3. `--relative` with stdin.
4. ALE's `--ruleset`, which goes down the jar-fallback path.

## 3. VS Code (F, VS Marketplace API 2026-10-07)

| Extension | Installs | Last update | How |
|---|---|---|---|
| JetBrains.kotlin-server (kotlin-lsp) | 44,870 | 2026-10-03 | Platform-specific vsix (7 targets). IntelliJ formatter. |
| fwcd.kotlin | 1,566,131 | 2024-12-06 | Deprecated server, embedded ktfmt. |
| mathiasfrohlich.Kotlin | 2,131,187 | 2020-02-11 | Syntax only. |
| esafirm.kotlin-formatter | 221,437 | 2018-12-03 | npm `ktlint` package, `exec("ktlint -F <path>")` on the file. Dead. |
| rnoro.vscode-ktlint-formatter | 2,437 | 2026-05-26 | Downloads the ktlint release script (`ktlint.version`, default 1.8.0), spawns `ktlint --stdin -F --stdin-path <p> --log-level=none`. No path setting. |
| mskelton.ktlint | 494 | 2026-02-19 | `spawn('ktlint', ['--stdin','-F','--log-level','none','--stdin-path', fsPath])` from PATH. **Works with our binary as is.** |
| omBratteng.ktfmt-kotlin-formatter | 186 | 2026-05-05 | Fat jar + `java`. Settings: `ktfmt.jarPath`, `javaHome`, `extraArgs`. Not binary-swappable. |
| crdrost.ktfmt / shape-app.ktfmtter | 1,834 / 325 | 2021 / 2025 | ktfmt jar |
| Trunk.io | 218,073 | 2026-06-05 | Meta-linter that manages its own ktlint. |

No dominant extension: the leaders are dead or deprecated. VS Code lets a second extension own `editor.defaultFormatter` for
`[kotlin]` and add diagnostics alongside kotlin-lsp. I: VS Code Kotlin is a small market (kotlin-lsp at 45k), but
the ktlint/ktfmt slot in it is uncontested.

## 4. Rust formatter/linter precedents

| Tool | Server | LSP features | VS Code distribution |
|---|---|---|---|
| ruff | `ruff server`, crate **lsp-server 0.10** + `gen-lsp-types` (Cargo.toml) | Diagnostics, format, **range format**, quick fix per diagnostic, noqa insert, `source.fixAll`, organize imports, noqa hover, notebooks, config reload via file watching. https://docs.astral.sh/ruff/editors/features/ | charliermarsh.ruff, 4.74M installs. **Platform-specific vsix** (10 targets incl. alpine, armhf) with a bundled binary. Resolution order: `ruff.path`, then env, then PATH, then bundled. Untrusted workspace always uses the bundled binary. |
| Biome | `biome lsp-proxy` (daemon), crate **tower-lsp-server 0.23** | Format, range format, diagnostics, quick fixes, `source.fixAll.biome`, `source.organizeImports.biome`. Config discovery `biome.json`. https://biomejs.dev/reference/vscode/ | biomejs.biome, 855k installs. Platform targets on the marketplace, but resolves `biome.lsp.bin`, then node_modules `@biomejs/cli-*`, then PATH. Copies the binary to temp on Windows (file locks). |
| taplo | `taplo-lsp` on its own `lsp-async-stub` + lsp-types 0.93 | Format, diagnostics, schema completion/validation | even-better-toml, 4.9M installs. **Universal vsix, server compiled to Wasm** (`@taplo/lsp`) running in Node. `taplo.bundled=false` + `taplo.path` for native. |
| dprint | `dprint editor-service` (custom protocol). Its experimental LSP was **removed** in dprint-vscode 0.18.0. Core uses `deno_tower_lsp`. | Format, range format | dprint.dprint, 33.6k installs. Universal vsix, binary from PATH/node_modules/`dprint.path`. |

### Rust LSP crates (crates.io / GitHub, 2026-10-07)

| Crate | Latest | Status |
|---|---|---|
| lsp-server (rust-analyzer `lib/lsp-server`) | 0.10.0, 2026-07-16; 4.0M recent dl | Maintained with rust-analyzer. Sync, crossbeam channels, no async runtime. Used by ruff and rust-analyzer. |
| tower-lsp (ebkalderon) | 0.20.0, **2023-08-11** | Effectively unmaintained (last push 2024-08). |
| tower-lsp-server (community fork) | 0.23.0, 2025-12-07; 0.24.0-rc.1, 2026-09-11 | Active. Used by Biome. tokio, uses `ls-types`. |
| async-lsp (oxalica) | 0.2.4, 2026-04-24 | Active, small (178 stars). tower-based. |
| lsp-types (gluon) | 0.97.0, **2024-06-04** | Stale. Forks: `gen-lsp-types` 0.11 (ruff), `ls-types` 0.0.6 (tower-lsp-server). |

I: **lsp-server + gen-lsp-types** fits ktrs. It's sync like the rest of the codebase, needs no tokio, and is ruff's proven
choice. A formatter/linter server is CPU-bound per request, so async buys nothing.

## Recommendation (by reach per effort)

1. **Docs + CLI edge-case parity (S, days).** Ship an "Editors" page with exact configs for conform, nvim-lint,
   none-ls, ALE, apheleia, Helix, Zed, mskelton.ktlint and Block's IntelliJ `script-path`. Close the four cli-diff
   checks in §2. Reach: every Neovim/Vim/Emacs/Helix/Zed Kotlin user, for no new code.
2. **`ktrs lsp` (M).** Built on lsp-server. Provides:
   - ktlint diagnostics: push model, plus pull if cheap.
   - Formatting: ktfmt or ktlint `-F`, chosen by setting or `.editorconfig`.
   - Code actions: autocorrect one violation, `source.fixAll.ktlint`, insert suppression.
   - Config: `.editorconfig` lookup by document path, with reload on `didChangeWatchedFiles`.

   Add range formatting after porting ktfmt 0.65's `--lines`/ranges. The server runs next to kotlin-lsp, and it is the
   shared core for items 3–4.
3. **VS Code extension (S–M once 2 exists).** A thin client following the ruff model: platform-specific vsix with a bundled
   binary, `ktrs.path` override, published to VS Marketplace + Open VSX. A Wasm universal vsix (taplo model, via
   `crates/ktrs-wasm`) is a fallback for unsupported targets. Uncontested niche, small absolute reach.
4. **IntelliJ (M–L, defer).** Biggest audience (460k + 114k), least to gain: it's a warm JVM and both plugins are
   maintained. Cheapest path: an LSP-API plugin over `ktrs lsp`, free in IDEA ≥ 2025.3, with Android Studio once the
   2026.2 open-source client lands, or LSP4IJ before that. Alternative: upstream an "external engine" mode into nbadal's
   plugin. That needs serve-protocol autocorrect decisions (range / single error / baseline) first.
