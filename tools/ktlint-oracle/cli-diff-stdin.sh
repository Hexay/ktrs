# Sourced by cli-diff.sh: stdin scenarios, then the exact invocations of editor integrations (docs/editors.md) and
# of CI integrations (README).

scenario stdin stdin.kt "" --stdin
scenario stdin_format stdin.kt "" --stdin -F
scenario stdin_format_file src/A.kt "" --stdin -F
scenario stdin_broken src/Bad.kt "" --stdin
scenario stdin_broken_format src/Bad.kt "" --stdin -F
scenario stdin_script src/build.gradle.kts "" --stdin
scenario stdin_script_format src/build.gradle.kts "" --stdin -F
scenario stdin_path stdin.kt "" --stdin --stdin-path=src/Foo.kt
scenario stdin_path_kts src/build.gradle.kts "" --stdin --stdin-path=x.kts
scenario stdin_path_blank stdin.kt "" --stdin --stdin-path=
scenario stdin_json stdin.kt "" --stdin --reporter=json
scenario stdin_relative stdin.kt "" --stdin --relative
scenario stdin_relative_path src/A.kt "" --stdin --relative --stdin-path=src/sub/A.kt
scenario stdin_relative_path_json src/A.kt "" --stdin --relative --reporter=json --stdin-path=src/sub/A.kt
scenario stdin_baseline stdin.kt "" --stdin --baseline=bl.xml
scenario stdin_unicode src/sub/B.kt "" --stdin -F
scenario stdin_percent pct.kt "printf 'val a = \"%%d\"\\n' > pct.kt" --stdin -F
scenario stdin_percent_literal pct.kt "printf 'val a = \"100%%%%\"\\n' > pct.kt" --stdin -F
scenario stdin_obsolete_property stdin.kt "printf 'disabled_rules = no-semi\\nktlint_disabled_rules = x\\n' >> .editorconfig" --stdin
scenario stdin_and_patterns "" "" --stdin --patterns-from-stdin
scenario stdin_ignored_args stdin.kt "" --stdin src/A.kt
scenario stdin_dash stdin.kt "" --stdin -
scenario stdin_dash_first src/A.kt "" - --stdin -F
scenario patterns_nul pats "printf 'src/A.kt\0src/sub/B.kt\0' > pats" --relative --patterns-from-stdin
scenario patterns_nul_dedup pats "printf 'src/A.kt\0src/sub/B.kt\0src/A.kt\0' > pats" --relative --patterns-from-stdin=
scenario patterns_newline pats "printf 'src/A.kt\nsrc/sub/B.kt\n' > pats" --relative --patterns-from-stdin=$'\n'
scenario patterns_default_nul pats "printf 'src/A.kt\nsrc/sub/B.kt\n' > pats" --relative --patterns-from-stdin
scenario patterns_empty "" "" --patterns-from-stdin
scenario patterns_merged pats "printf 'src/A.kt\0' > pats" --relative --patterns-from-stdin src/sub
scenario patterns_comma pats "printf 'src/A.kt,src/sub/B.kt' > pats" --relative --patterns-from-stdin=,

# Editors; `_bad` = unfixable input. conform.nvim, none-ls formatting:
scenario ed_conform src/A.kt "" --format --stdin --log-level=none
scenario ed_conform_bad src/Bad.kt "" --format --stdin --log-level=none
# nvim-lint (parses the reporter's stream):
scenario ed_nvim_lint src/A.kt "" --reporter=json --stdin
scenario ed_nvim_lint_bad src/Bad.kt "" --reporter=json --stdin
# none-ls diagnostics:
scenario ed_none_ls src/A.kt "" --relative --reporter=json --log-level=none --stdin
scenario ed_none_ls_bad src/Bad.kt "" --relative --reporter=json --log-level=none --stdin
# ALE linter, fixer and `g:ale_kotlin_ktlint_rulesets` (space-separated `--ruleset X`):
scenario ed_ale src/A.kt "" --stdin
scenario ed_ale_bad src/Bad.kt "" --stdin
scenario ed_ale_fix src/A.kt "" --stdin --format
scenario ed_ale_ruleset_missing src/A.kt "" --ruleset nothere.jar --stdin
scenario ed_ale_ruleset_not_jar src/A.kt "" --ruleset src/A.kt --stdin --format
# apheleia (trailing `-` pattern):
scenario ed_apheleia src/A.kt "" --log-level=none --stdin -F -
scenario ed_apheleia_bad src/Bad.kt "" --log-level=none --stdin -F -
scenario ed_apheleia_clean stdin.kt "printf 'fun a() = 1\\n' > stdin.kt" --log-level=none --stdin -F -
# VS Code mskelton.ktlint / rnoro.vscode-ktlint-formatter:
scenario ed_vscode src/A.kt "" --stdin -F --log-level none --stdin-path src/A.kt
scenario ed_vscode_bad src/Bad.kt "" --stdin -F --log-level none --stdin-path src/Bad.kt
scenario ed_vscode_ec ec/sub/E.kt "" --stdin -F --stdin-path ec/sub/E.kt --log-level=none

# CI integrations (README "Bazel", "reviewdog and Danger"). danger-ktlint 0.0.9 (the PR's .kt files come first):
scenario ci_danger "" "" src/A.kt src/sub/B.kt --reporter=json --relative --log-level=none
scenario ci_danger_clean "" "" src/sub/CTest.kt --reporter=json --relative --log-level=none
scenario ci_danger_bad "" "" src/A.kt src/Bad.kt --reporter=json --relative --log-level=none
# reviewdog and the Danger plugins that read a report file:
scenario ci_report_checkstyle "" "" --relative --reporter=checkstyle,output=ktlint.xml
scenario ci_report_sarif "" "" --relative --reporter=sarif,output=ktlint.sarif
scenario ci_report_json "" "" --relative --reporter=json,output=ktlint.json
# rules_lint's lint_ktlint_aspect (no patterns: the sandbox holds only the target's srcs):
scenario ci_rules_lint "" "" --editorconfig=.editorconfig --relative
scenario ci_rules_lint_color "" "" --color --editorconfig=.editorconfig --relative
