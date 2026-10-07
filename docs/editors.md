# Editors

Editor plugins that run the `ktlint` or `ktfmt` CLI work with ktrs unchanged: [install ktrs](../README.md#installation)
so its `ktlint` and `ktfmt` binaries come first on `PATH`, then use the plugin's stock ktlint/ktfmt setup below.
Each invocation listed here is checked byte for byte (stdout, stderr, exit code) against the ktlint 2.0.0-ALPHA-4 and
1.8.0 jars and the ktfmt 0.64 jar by `tools/ktlint-oracle/cli-diff.sh` (`ed_*`, `stdin_*`; ALE's
`--ruleset` also as `ruleset_jar_ale*` with `RULESET_JAR`) and `tools/ktfmt-oracle/cli-diff.sh` (`stdin_*`).

ktlint behaves like 2.0.0-ALPHA-4 by default; for 1.8.0 put `ktrs_ktlint_version = 1.8` in `.editorconfig` (see the
README). Most of these plugins pass the buffer on stdin without its path, so ktlint resolves `.editorconfig` from the
working directory, as the jar does.

## Language server: `ktrs lsp`

`ktrs lsp` (stdio) runs next to your Kotlin language server and adds ktlint diagnostics, quick fixes (fix one
violation, fix all, suppress on the line or in the file) and formatting with ktfmt or ktlint. It reads the setup from
the build: ktfmt-gradle, ktlint-gradle, kotlinter, Spotless and the Maven plugins, including convention plugins and the
version catalog. When the build has none of them it shows ktlint 1.8 diagnostics and doesn't format. Editor settings
override the build; the keys are listed in `crates/ktrs-lsp/src/lib.rs`, nested as in
`{"ktrs": {"format": {"tool": "ktfmt"}, "ktfmt": {"style": "kotlinlang"}}}`.

**Neovim** (0.11+):

```lua
vim.lsp.config("ktrs", {
  cmd = { "ktrs", "lsp" },
  filetypes = { "kotlin" },
  root_markers = { "settings.gradle.kts", "settings.gradle", "pom.xml", ".git" },
  -- settings = { ktrs = { format = { tool = "ktfmt" } } },
})
vim.lsp.enable("ktrs")
```

**Helix** (`languages.toml`; keep your Kotlin server in the list):

```toml
[language-server.ktrs]
command = "ktrs"
args = ["lsp"]

[[language]]
name = "kotlin"
language-servers = ["kotlin-language-server", "ktrs"]
```

The sections below use the `ktlint`/`ktfmt` binaries instead, for plugins that run those CLIs.

## Neovim

[conform.nvim](https://github.com/stevearc/conform.nvim), formatting (`ktlint --format --stdin --log-level=none` or
`ktfmt -`):

```lua
require("conform").setup({
  formatters_by_ft = { kotlin = { "ktlint" } }, -- or { "ktfmt" }
})
```

For a ktfmt style: `formatters = { ktfmt = { prepend_args = { "--kotlinlang-style" } } }`.

[nvim-lint](https://github.com/mfussenegger/nvim-lint), diagnostics (`ktlint --reporter=json --stdin`; reads the json
report from stderr):

```lua
require("lint").linters_by_ft = { kotlin = { "ktlint" } }
vim.api.nvim_create_autocmd({ "BufWritePost", "InsertLeave" }, {
  callback = function() require("lint").try_lint() end,
})
```

[none-ls](https://github.com/nvimtools/none-ls.nvim), diagnostics and formatting
(`ktlint --relative --reporter=json --log-level=none --stdin`, `ktlint --format --stdin --log-level=none`):

```lua
local null_ls = require("null-ls")
null_ls.setup({
  sources = { null_ls.builtins.diagnostics.ktlint, null_ls.builtins.formatting.ktlint },
})
```

## Vim / Neovim: ALE

[ALE](https://github.com/dense-analysis/ale) linter and fixer (`ktlint [--ruleset X] --stdin`, fixer adds
`--format`):

```vim
let g:ale_linters = {'kotlin': ['ktlint']}
let g:ale_fixers = {'kotlin': ['ktlint']}
" optional: custom rule sets (compose-rules 0.6.7 runs natively, other jars hand off to the ktlint jar; needs Java)
let g:ale_kotlin_ktlint_rulesets = ['/path/to/ktlint-compose-0.6.7-all.jar']
```

## Emacs

[apheleia](https://github.com/radian-software/apheleia), formatting: its default for `kotlin-mode` and `kotlin-ts-mode`
is already `ktlint --log-level=none --stdin -F -`:

```elisp
(apheleia-global-mode +1)
```

To use ktfmt instead:

```elisp
(push '(ktfmt . ("ktfmt" "--kotlinlang-style" "-")) apheleia-formatters)
(setf (alist-get 'kotlin-mode apheleia-mode-alist) 'ktfmt
      (alist-get 'kotlin-ts-mode apheleia-mode-alist) 'ktfmt)
```

[flycheck-kotlin](https://github.com/emacsorphanage/flycheck-kotlin), diagnostics (`ktlint --stdin`):

```elisp
(require 'flycheck-kotlin)
(flycheck-kotlin-setup)
(add-hook 'kotlin-mode-hook #'flycheck-mode)
```

Flymake has no ktlint backend.

## Helix

`languages.toml` (Helix has no Kotlin formatter by default):

```toml
[[language]]
name = "kotlin"
formatter = { command = "ktfmt", args = ["--kotlinlang-style", "-"] }
auto-format = true
```

## Zed

`settings.json`:

```json
"languages": {
  "Kotlin": {
    "formatter": { "external": { "command": "ktfmt", "arguments": ["--kotlinlang-style", "-"] } }
  }
}
```

## VS Code

The **ktrs** extension ([`hexay.ktrs`](https://marketplace.visualstudio.com/items?itemName=hexay.ktrs), also on
[Open VSX](https://open-vsx.org/extension/hexay/ktrs)) runs `ktrs lsp` above with a bundled binary: diagnostics, quick
fixes, fix all on save and formatting. Setup and settings: [editors/vscode/README.md](../editors/vscode/README.md).

Alternatively, [mskelton.ktlint](https://marketplace.visualstudio.com/items?itemName=mskelton.ktlint), formatting
(`ktlint --stdin -F --log-level none --stdin-path <file>`; runs `ktlint` from `PATH`, no settings):

```json
"[kotlin]": { "editor.defaultFormatter": "mskelton.ktlint" }
```

## IntelliJ IDEA / Android Studio

[Block's Kotlin Formatter plugin](https://plugins.jetbrains.com/plugin/26482) runs a script with
`--set-exit-if-changed -` on the document. `.idea/kotlin-formatter.properties`:

```properties
kotlin-formatter.enabled=true
kotlin-formatter.script-path=bin/kotlin-format
```

`bin/kotlin-format` (executable; pick your style flag):

```sh
#!/bin/sh
exec ktfmt --kotlinlang-style "$@"
```

Restart the IDE after changing the properties file. For ktlint in IntelliJ, the
[ktlint plugin](https://plugins.jetbrains.com/plugin/15057-ktlint) runs ktlint in-process and does not use these
binaries.
