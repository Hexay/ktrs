#!/usr/bin/env bash
# Differential test of the `ktlint` drop-in (crates/ktrs-cli/src/ktlint) against the ktlint 2.0.0-ALPHA-4 CLI jar.
#   cli-diff.sh [path/to/ktlint-binary]   (default: target/release/ktlint, else target/debug/ktlint)
# Each scenario runs both tools on a fresh copy of the same fixture tree, from inside it, and compares
# stdout, stderr (log timestamps, the fixture's path and JVM stack frames masked), the exit code and the
# tree left behind. The fixture's root .editorconfig disables the standard rules ktrs hasn't ported
# (`ktrs lint --list-rules` vs the jar's RuleIds.java), so rule differences don't hide CLI ones.
# ONLY=<regex> selects scenarios, KEEP=1 keeps outputs, VERBOSE=1 prints diffs. Exit 1 on any mismatch.
# KTLINT_VERSION=1.8: against the 1.8.0 jar (JAR=<path>, default lib/ktlint-cli-1.8.0-all.jar, fetched), ktrs in
# 1.8 mode through the fixture's `ktrs_ktlint_version = 1.8` (research/26-ktlint-18-mode.md). Known 1.8 mismatch:
# rep_summary_format (`-F` runs 2.0's rule order). RULESET_JAR=<jar> adds the `ruleset_jar_*` scenarios
# (research/27-custom-rulesets-impl.md).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
version=${KTLINT_VERSION:-2.0}
if [[ $version == 1.8 ]]; then
  jar=${JAR:-$here/lib/ktlint-cli-1.8.0-all.jar}
  [[ -f $jar ]] || curl -sfL --create-dirs -o "$jar" \
    https://repo1.maven.org/maven2/com/pinterest/ktlint/ktlint-cli/1.8.0/ktlint-cli-1.8.0-all.jar
  provider=com.pinterest.ktlint.ruleset.standard.StandardRuleSetProvider
else
  jar=${JAR:-$here/lib/ktlint-cli-2.0.0-ALPHA-4-all.jar}
  [[ -f $jar ]] || "$repo/tools/sync-ktlint.sh" >&2
  provider=io.github.ktlint.core.ruleset.standard.StandardRuleSetProvider
fi
java=java
bundled=$(ls -d "$here"/../jdk/*/bin 2>/dev/null | head -1 || true)
if [[ -n $bundled ]]; then java="$bundled/java"; fi
ours=${1:-$(ls "$repo"/target/release/ktlint "$repo"/target/release/ktlint.exe "$repo"/target/debug/ktlint \
  "$repo"/target/debug/ktlint.exe 2>/dev/null | head -1)}
ours="$(cd "$(dirname "$ours")" && pwd)/$(basename "$ours")"
ktrs="$(dirname "$ours")/ktrs"
[[ -x $ktrs ]] || ktrs="$ktrs.exe"
command -v cygpath >/dev/null && jar=$(cygpath -m "$jar")
scratch=$(mktemp -d)
if [[ -n ${KEEP:-} ]]; then echo "outputs in $scratch"; else trap 'rm -rf "$scratch"' EXIT; fi

"$java" -cp "$jar" "$here/RuleIds.java" "$provider" 2>/dev/null | grep -a '^standard:' | sort > "$scratch/jar-rules"
"$ktrs" lint --list-rules --ktlint-version="$version" | tr -d '\r' | sort > "$scratch/our-rules"
unported=$(comm -23 "$scratch/jar-rules" "$scratch/our-rules")
echo "rules: $(wc -l < "$scratch/our-rules") ported, $(echo "$unported" | grep -c .) disabled on both sides"

fixture() {
  local w=$1
  mkdir -p "$w/src/sub" "$w/build" "$w/.hidden" "$w/ec/sub"
  {
    # `[*]`: an explicitly named non-Kotlin file is linted too.
    printf 'root = true\n\n[*]\n'
    [[ $version == 1.8 ]] && printf 'ktrs_ktlint_version = 1.8\n'
    for id in $unported; do printf 'ktlint_%s = disabled\n' "${id/:/_}"; done
  } > "$w/.editorconfig"
  printf 'fun foo( ) {\n  val x=1;\n}\n' > "$w/src/A.kt"
  printf 'import java.util.*\n\nval y = "a\\tb<&>" // \xc2\xa7 %%d\n' > "$w/src/sub/B.kt"
  printf 'val z = 1\n' > "$w/src/sub/CTest.kt"
  printf 'fun bad( {\n' > "$w/src/Bad.kt"
  printf 'plugins {\n  id("x")\n}\n' > "$w/src/build.gradle.kts"
  printf 'fun crlf( ) {\r\n  val a=1\r\n}\r\n' > "$w/src/Crlf.kt"
  printf 'fun foo( ) = 1\n' > "$w/build/C.kt"
  printf 'fun foo( ) = 1\n' > "$w/.hidden/D.kt"
  printf 'class Other {}\n' > "$w/src/Other.java"
  printf 'fun f() {\n    val a = 1\n}\n' > "$w/ec/sub/E.kt"
  printf '[*.kt]\nindent_size = 2\n' > "$w/ec/.editorconfig"
  printf '[*.kt]\nktlint_standard_no-semi = disabled\n' > "$w/alt-editorconfig"
  printf -- '--relative\nsrc/A.kt\n' > "$w/args.txt"
  printf 'fun a( ) = 1\n' > "$w/stdin.kt"
}

pass=0 fail=0
# scenario <name> <stdin-file-or-empty> <setup-snippet-or-empty> <args...>
scenario() {
  local name=$1 stdin=$2 setup=$3; shift 3
  [[ -z ${ONLY:-} || $name =~ $ONLY ]] || return 0
  local tool dir
  for tool in jar ours; do
    dir="$scratch/$name/$tool"
    mkdir -p "$dir"
    fixture "$dir/w"
    [[ -n $setup ]] && (cd "$dir/w" && eval "$setup")
    local cmd=("$ours")
    [[ $tool == jar ]] && cmd=("$java" -jar "$jar")
    local input=/dev/null
    [[ -n $stdin ]] && input="$dir/w/$stdin"
    (cd "$dir/w" && "${cmd[@]}" "$@" < "$input" > "$dir/out.raw" 2> "$dir/err.raw"; echo $? > "$dir/code")
    local w_abs w_win; w_abs="$(cd "$dir/w" && pwd)"; w_win=$w_abs
    command -v cygpath >/dev/null && w_win=$(cygpath -m "$w_abs")
    for s in out err; do
      # JVM lambda identities (`RuleKt$$Lambda/0x…@1a2b`) differ per run; on Windows the tree shows as C:/… and C:\….
      grep -av $'^\tat \|^\t\.\.\. [0-9]* more' "$dir/$s.raw" \
        | sed -E -e 's/^[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3} /HH:MM:SS.mmm /' -e "s#$w_abs#<W>#g" -e "s#$w_win#<W>#g" \
        -e "s#${w_win//\//\\\\}#<W>#g" -e 's/\$\$Lambda\/0x[0-9a-f]+@[0-9a-f]+/$$Lambda@<id>/g' \
        -e 's/ktlint-backup\.[-0-9a-f]+/ktlint-backup.<hash>/' > "$dir/$s"
    done
    # Reports written into the tree can hold absolute paths too.
    grep -rlaF --exclude-dir=.git "$w_abs" "$dir/w" | while read -r f; do sed -i "s#$w_abs#<W>#g" "$f"; done
  done
  local a="$scratch/$name/jar" b="$scratch/$name/ours" bad=()
  cmp -s "$a/code" "$b/code" || bad+=("exit $(cat "$a/code") vs $(cat "$b/code")")
  cmp -s "$a/out" "$b/out" || bad+=(stdout)
  cmp -s "$a/err" "$b/err" || bad+=(stderr)
  diff -rq -x .git "$a/w" "$b/w" > /dev/null || bad+=("tree: $(diff -rq -x .git "$a/w" "$b/w" | head -3 | tr '\n' ';')")
  if ((${#bad[@]})); then
    fail=$((fail + 1)); echo "MISMATCH $name: ${bad[*]}"
    if [[ -n ${VERBOSE:-} ]]; then diff "$a/out" "$b/out" | head -20; diff "$a/err" "$b/err" | head -20; fi
  else
    pass=$((pass + 1))
  fi
}

git_repo='git init -q . && git config user.email t@t && git config user.name t'
baseline_xml='printf "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<baseline version=\"1.0\">\n    <file name=\"src/A.kt\">\n        <error line=\"1\" column=\"9\" source=\"standard:function-signature\" />\n        <error line=\"2\" column=\"10\" source=\"standard:no-semi\" />\n    </file>\n</baseline>\n" > bl.xml'

# Clikt: help, version, usage errors.
scenario help "" "" --help
scenario help_short "" "" -h
scenario version "" "" --version
scenario version_short "" "" -v
scenario help_gen "" "" generateEditorConfig --help
scenario help_pre_commit "" "" installGitPreCommitHook -h
scenario help_pre_push "" "" installGitPrePushHook --help
scenario unknown "" "" --bogus src
scenario typo "" "" --formt src
scenario typo_relative "" "" --relativ src
scenario short_group_unknown "" "" -Fz src
scenario flag_with_value "" "" --color=yes src
scenario missing_value "" "" --limit
scenario limit_zero "" "" --limit=0 src
scenario limit_not_int "" "" --limit=abc src
scenario limit_separate "" "" --limit 3 --relative src
scenario log_level_bad "" "" --log-level=foo src
scenario log_level_none "" "" --log-level=none src
scenario log_level_warn "" "" -l warn src
scenario log_level_attached "" "" -lerror src
scenario double_dash "" "" -- --relative
scenario argfile "" "" @args.txt
scenario argfile_missing "" "" @nope.txt
scenario interspersed "" "" src --relative
scenario help_wins "" "" --bogus --help
scenario version_wins "" "" --bogus -v
scenario help_wins_sub "" "" generateEditorConfig --bogus -h
# Options 2.0 removed: 1.8 still declares --code-style (an error when used); the other two were never registered.
scenario code_style "" "" --code-style=ktlint_official src
scenario code_style_bad "" "" --code-style=foo src
scenario code_style_help "" "" --code-style=foo --help
scenario code_style_typo "" "" --codestyle src
scenario disabled_rules_option "" "" --disabled_rules=no-semi src
scenario experimental_option "" "" --experimental src

# Patterns and file selection.
scenario no_args "" ""
scenario dot "" "" .
scenario dir "" "" src
scenario dir_relative "" "" --relative src
scenario dir_trailing_slash "" "" --relative src/
scenario file "" "" src/A.kt
scenario file_dot "" "" ./src/A.kt
scenario file_dot_relative "" "" --relative ./src/A.kt
scenario files_two "" "" --relative src/A.kt src/sub/B.kt
scenario file_and_missing "" "" --relative src/A.kt src/nope.kt
scenario missing "" "" nope.kt
scenario missing_dir "" "" nope/
scenario java_file "" "" src/Other.java
scenario glob "" "" --relative 'src/**/*.kt'
scenario glob_star "" "" --relative 'src/*.kt'
scenario glob_braces "" "" --relative 'src/**/*.{kt,kts}'
scenario glob_question "" "" --relative '?rc/*.kt'
scenario glob_class "" "" --relative 'src/[AB]*.kt'
scenario glob_negation "" "" --relative 'src/**/*.kt' '!src/**/*Test.kt'
scenario glob_negation_dir "" "" --relative 'src/**/*.kt' '!src/sub/**'
scenario negation_only "" "" --relative '!src/sub/**'
scenario parent_path "" "" --relative ../w/src/A.kt
scenario parent_glob "" "" --relative '../w/src/*.kt'
scenario hidden_explicit "" "" --relative .hidden/D.kt
scenario hidden_dir "" "" --relative .hidden
scenario build_dir "" "" --relative build
scenario kts "" "" --relative src/build.gradle.kts
scenario crlf "" "" --relative src/Crlf.kt
scenario editorconfig_nested "" "" --relative ec

# Lint and format.
scenario format "" "" -F src
scenario format_relative "" "" --format --relative src
scenario format_file "" "" -F src/A.kt
scenario format_crlf "" "" -F --relative src/Crlf.kt
scenario format_ignore "" "" -F --ignore-autocorrect-failures --relative src
scenario lint_ignore "" "" --ignore-autocorrect-failures --relative src
scenario format_force_lint "" "" -F --force-lint-after-format --relative src
scenario format_limit "" "" -F --limit=1 --relative src/A.kt
scenario limit "" "" --limit=2 --relative src

# Reporters.
scenario rep_plain "" "" --reporter=plain --relative src
scenario rep_group "" "" '--reporter=plain?group_by_file' --relative src
scenario rep_pad "" "" '--reporter=plain?pad' --relative src
scenario rep_group_pad "" "" '--reporter=plain?group_by_file&pad' --relative src
scenario rep_summary "" "" --reporter=plain-summary --relative src
scenario rep_summary_format "" "" --reporter=plain-summary -F --relative src
scenario rep_json "" "" --reporter=json --relative src
scenario rep_json_abs "" "" --reporter=json src
scenario rep_checkstyle "" "" --reporter=checkstyle --relative src
scenario rep_sarif "" "" --reporter=sarif --relative src
scenario rep_sarif_abs "" "" --reporter=sarif src
scenario rep_html "" "" --reporter=html --relative src
scenario rep_html_format "" "" --reporter=html -F --relative src
scenario rep_format "" "" --reporter=format --relative src
scenario rep_format_F "" "" --reporter=format -F --relative src
scenario rep_format_color "" "" --reporter=format --color --relative src
scenario rep_baseline "" "" --reporter=baseline --relative src
scenario rep_multiple "" "" --reporter=plain --reporter=json,output=out/r.json --reporter=checkstyle,output=cs.xml --relative src
scenario rep_output_abs "" "" --reporter=json,output=r.json src
scenario rep_duplicate "" "" --reporter=json --reporter=json --relative src/A.kt
scenario rep_unknown "" "" --reporter=nope src
scenario rep_unknown_after_output "" "" --reporter=json,output=first.json --reporter=nope src
scenario rep_artifact_missing "" "" --reporter=custom,artifact=nope.jar src
scenario rep_artifact_not_jar "" "" --reporter=custom,artifact=src/A.kt src
scenario rep_query_decode "" "" '--reporter=plain?group_by_file=true&x=a%20b' --relative src/A.kt
scenario color "" "" --color --relative src
scenario color_name "" "" --color --color-name=RED --relative src
scenario color_name_bad "" "" --color-name=FOO --relative src
scenario color_group "" "" --color '--reporter=plain?group_by_file' --relative src

# Stdin.
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
scenario stdin_baseline stdin.kt "" --stdin --baseline=bl.xml
scenario stdin_unicode src/sub/B.kt "" --stdin -F
scenario stdin_percent pct.kt "printf 'val a = \"%%d\"\\n' > pct.kt" --stdin -F
scenario stdin_percent_literal pct.kt "printf 'val a = \"100%%%%\"\\n' > pct.kt" --stdin -F
scenario stdin_obsolete_property stdin.kt "printf 'disabled_rules = no-semi\\nktlint_disabled_rules = x\\n' >> .editorconfig" --stdin
scenario stdin_and_patterns "" "" --stdin --patterns-from-stdin
scenario stdin_ignored_args stdin.kt "" --stdin src/A.kt
scenario patterns_nul pats "printf 'src/A.kt\0src/sub/B.kt\0' > pats" --relative --patterns-from-stdin
scenario patterns_nul_dedup pats "printf 'src/A.kt\0src/sub/B.kt\0src/A.kt\0' > pats" --relative --patterns-from-stdin=
scenario patterns_newline pats "printf 'src/A.kt\nsrc/sub/B.kt\n' > pats" --relative --patterns-from-stdin=$'\n'
scenario patterns_default_nul pats "printf 'src/A.kt\nsrc/sub/B.kt\n' > pats" --relative --patterns-from-stdin
scenario patterns_empty "" "" --patterns-from-stdin
scenario patterns_merged pats "printf 'src/A.kt\0' > pats" --relative --patterns-from-stdin src/sub
scenario patterns_comma pats "printf 'src/A.kt,src/sub/B.kt' > pats" --relative --patterns-from-stdin=,

# Baseline, .editorconfig defaults, rule sets.
scenario baseline_create "" "" --relative --baseline=bl.xml src
scenario baseline_create_abs "" "" --baseline=out/bl.xml src
scenario baseline_valid "" "$baseline_xml" --relative --baseline=bl.xml src
scenario baseline_valid_abs "" "$baseline_xml" --baseline=bl.xml src
scenario baseline_format "" "$baseline_xml" -F --relative --baseline=bl.xml src
scenario baseline_garbage "" "printf 'garbage<' > bl.xml" --relative --baseline=bl.xml src
scenario baseline_truncated "" "printf '<baseline><file name=\"a\">' > bl.xml" --relative --baseline=bl.xml src
scenario baseline_empty "" ": > bl.xml" --relative --baseline=bl.xml src
scenario baseline_bad_line "" "printf '<baseline><file name=\"src/A.kt\"><error line=\"x\" column=\"1\" source=\"s\"/></file></baseline>' > bl.xml" --relative --baseline=bl.xml src
scenario editorconfig_missing "" "" --relative --editorconfig=nothere src/A.kt
scenario editorconfig_file "" "" --relative --editorconfig=alt-editorconfig src/A.kt
scenario editorconfig_dir "" "" --relative --editorconfig=ec src/A.kt
scenario ruleset_missing "" "" -R nothere.jar src
scenario ruleset_not_jar "" "" -R src/A.kt src
scenario ruleset_debug "" "" -R src/A.kt --log-level=debug src
scenario ruleset_two "" "" --ruleset=src/A.kt,nothere.jar src
# A real rule set JAR (RULESET_JAR=<path>): a compose-rules release ktrs runs natively, or any other, which ktrs
# hands to the ktlint jar (this script's $jar, through KTRS_KTLINT_JAR).
if [[ -n ${RULESET_JAR:-} ]]; then
  rs=$RULESET_JAR; command -v cygpath >/dev/null && rs=$(cygpath -m "$rs")
  export KTRS_KTLINT_JAR="$jar"
  [[ -n $bundled ]] && export JAVA_HOME="${bundled%/bin}"
  compose_kt='printf "@Composable\nfun Screen(modifier: Modifier, text: String) {\n    Text(text)\n    Text(text)\n}\n\n@Preview\n@Composable\nfun ScreenPreview(m: Modifier = Modifier) {\n    Row(modifier = m) {}\n}\n" > src/Compose.kt'
  scenario ruleset_jar_lint "" "$compose_kt" -R "$rs" --relative src
  scenario ruleset_jar_format "" "$compose_kt" -R "$rs" -F --relative src
  scenario ruleset_jar_json "" "$compose_kt" -R "$rs" --reporter=json src
  scenario ruleset_jar_stdin src/Compose.kt "$compose_kt" -R "$rs" --stdin
  scenario ruleset_jar_stdin_format src/Compose.kt "$compose_kt" -R "$rs" --stdin -F
  scenario ruleset_jar_disabled "" "$compose_kt && printf 'ktlint_compose = disabled\n' >> .editorconfig" -R "$rs" --relative src
  scenario ruleset_jar_gen "" "" -R "$rs" generateEditorConfig --code-style=ktlint_official
fi

# Subcommands.
scenario gen_missing "" "" generateEditorConfig
scenario gen_bad "" "" generateEditorConfig --code-style=foo
scenario gen_extra "" "" generateEditorConfig --code-style=ktlint_official extra
scenario gen_ktlint "" "" generateEditorConfig --code-style=ktlint_official
scenario gen_android "" "" generateEditorConfig --code-style android_studio
scenario gen_intellij "" "" generateEditorConfig --code-style=INTELLIJ_IDEA
scenario gen_after_args "" "" src generateEditorConfig --code-style=ktlint_official
scenario hook_no_git "" "" installGitPreCommitHook
scenario hook_pre_commit "" "$git_repo" installGitPreCommitHook
scenario hook_pre_push "" "$git_repo" installGitPrePushHook
scenario hook_backup "" "$git_repo && printf 'old hook' > .git/hooks/pre-commit" installGitPreCommitHook
scenario hook_hooks_path "" "$git_repo && git config core.hooksPath myhooks" installGitPrePushHook

echo "cli-diff: $pass identical, $fail mismatched of $((pass + fail)) scenarios"
((fail == 0))
