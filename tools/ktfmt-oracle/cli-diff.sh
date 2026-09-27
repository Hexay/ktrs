#!/usr/bin/env bash
# Differential test of the `ktfmt` drop-in (crates/ktrs_cli) against the real ktfmt CLI jar.
#   cli-diff.sh [path/to/ktfmt-binary]   (default: target/debug/ktfmt[.exe])
# Each scenario runs both tools on a fresh copy of the same fixture tree, from inside it, and
# compares stdout (bytes), stderr (sorted lines: files are formatted in parallel), the exit code
# and the tree left behind. Prints one line per mismatch and a summary; exit 1 on any mismatch.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
jar=$(ls "$here"/lib/ktfmt-*-with-dependencies.jar 2>/dev/null | head -1 || true)
[[ -n $jar ]] || { echo "no oracle jar; run tools/sync-ktfmt.sh" >&2; exit 1; }
java=java
bundled=$(ls -d "$here"/../jdk/*/bin 2>/dev/null | head -1 || true)
if [[ -n $bundled ]]; then java="$bundled/java"; fi
ours=${1:-$(ls "$repo"/target/debug/ktfmt.exe "$repo"/target/debug/ktfmt 2>/dev/null | head -1)}
ours="$(cd "$(dirname "$ours")" && pwd)/$(basename "$ours")"
command -v cygpath >/dev/null && jar=$(cygpath -m "$jar")
scratch=$(mktemp -d)
# KEEP=1 leaves each scenario's outputs in $scratch/<name>/{jar,ours} for inspection.
if [[ -n ${KEEP:-} ]]; then echo "outputs in $scratch"; else trap 'rm -rf "$scratch"' EXIT; fi

unformatted='fun   f( a:Int,b :String ) {
if(a>0){println( b )}
}
'
long='class Foo {
fun bar(first: Int, second: String, third: Long, fourth: Double, fifth: Boolean) { baz(first, second) }
}
'

fixture() {
  local w=$1
  mkdir -p "$w/src/ok" "$w/src/deep/x" "$w/ec"
  printf '%s' "$unformatted" > "$w/src/A.kt"
  printf 'fun f() = println("hello, world")\n' > "$w/src/ok/B.kt"
  printf '%s' "$unformatted" > "$w/src/deep/x/C.kts"
  printf 'fun    f1 (  ' > "$w/src/broken.kt"
  printf 'val x = foo(bar { } { zap = 2 })' > "$w/src/lambdas.kt"
  printf 'class Other {}\n' > "$w/src/Other.java"
  printf '\xef\xbb\xbf%s' "$unformatted" > "$w/src/bom.kt"
  printf 'fun   f( ) {\r\nprintln( 1 )\r\n}\r\n' > "$w/src/crlf.kt"
  printf 'fun   f( ) = 1\n' > "$w/.kt"
  printf -- '--google-style\n-n\nsrc/A.kt\nsrc/ok/B.kt\n' > "$w/args.txt"
  local c
  for c in plain width80 width_off indent3 indent_tab tab_style ij_indent ij_cont ij_cont_both \
      comma_none comma_upper comma_bad unset nested glob_braces glob_case root_stop bad_int; do
    mkdir -p "$w/ec/$c/sub"
    printf '%s' "$long" > "$w/ec/$c/F.kt"
    printf '%s' "$long" > "$w/ec/$c/sub/G.kt"
  done
  ec() { printf "$2" > "$w/ec/$1/.editorconfig"; }
  ec width80 'root = true\n[*]\nmax_line_length = 80\n'
  ec width_off 'root = true\n[*]\nmax_line_length = off\n'
  ec indent3 'root = true\n[*.kt]\nindent_size = 3\n'
  ec indent_tab 'root = true\n[*]\nindent_size = tab\ntab_width = 6\n'
  ec tab_style 'root = true\n[*]\nindent_style = tab\ntab_width = 8\n'
  ec ij_indent 'root = true\n[*]\nindent_size = 3\nij_kotlin_indent_size = 5\n'
  ec ij_cont 'root = true\n[*]\nij_continuation_indent_size = 6\n'
  ec ij_cont_both 'root = true\n[*]\nij_continuation_indent_size = 6\nij_kotlin_continuation_indent_size = 3\n'
  ec comma_none 'root = true\n[*]\nktfmt_trailing_comma_management_strategy = none\n'
  ec comma_upper 'root = true\n[*]\nktfmt_trailing_comma_management_strategy = COMPLETE\n'
  ec comma_bad 'root = true\n[*]\nktfmt_trailing_comma_management_strategy = sometimes\n'
  ec unset 'root = true\n[*]\nindent_size = 4\n[sub/*]\nindent_size = unset\n'
  ec nested 'root = true\n[*]\nindent_size = 4\nmax_line_length = 60\n'
  printf '[*]\nindent_size = 8\n' > "$w/ec/nested/sub/.editorconfig"
  ec glob_braces 'root = true\n[*.{kt,kts}]\nindent_size = 6\n'
  ec glob_case 'root = true\n[*.KT]\nindent_size = 3\n'
  ec root_stop '[*]\nindent_size = 7\n'
  ec bad_int 'root = true\n[*]\nindent_size = 0\nmax_line_length = -5\ntab_width = x\n'
  unset -f ec
}

pass=0 fail=0
# scenario <name> <stdin-file-or-empty> <args...>; ONLY=<regex> selects scenarios by name.
scenario() {
  local name=$1 stdin=$2; shift 2
  [[ -z ${ONLY:-} || $name =~ $ONLY ]] || return 0
  local tool dir
  for tool in jar ours; do
    dir="$scratch/$name/$tool"
    mkdir -p "$dir"
    fixture "$dir/w"
    local cmd=("$ours")
    [[ $tool == jar ]] && cmd=("$java" -jar "$jar")
    (cd "$dir/w" && "${cmd[@]}" "$@" < "${stdin:-/dev/null}" > "$dir/out" 2> "$dir/err.raw"; echo $? > "$dir/code")
    # A JVM stack trace's frames can't be matched; its first line (the exception) can.
    grep -av $'^\tat ' "$dir/err.raw" | sort > "$dir/err"
    # Files are formatted in parallel, so only stdin output has a defined line order.
    if [[ $name == stdin* ]]; then cp "$dir/out" "$dir/out.cmp"; else sort "$dir/out" > "$dir/out.cmp"; fi
  done
  local a="$scratch/$name/jar" b="$scratch/$name/ours" bad=()
  cmp -s "$a/code" "$b/code" || bad+=("exit $(cat "$a/code") vs $(cat "$b/code")")
  cmp -s "$a/out.cmp" "$b/out.cmp" || bad+=(stdout)
  cmp -s "$a/err" "$b/err" || bad+=(stderr)
  diff -rq "$a/w" "$b/w" > /dev/null || bad+=("tree: $(diff -rq "$a/w" "$b/w" | head -3 | tr '\n' ';')")
  if ((${#bad[@]})); then
    fail=$((fail + 1)); echo "MISMATCH $name: ${bad[*]}"
    [[ -n ${VERBOSE:-} ]] && diff "$a/err" "$b/err" | head -10 && diff "$a/out" "$b/out" | head -10
  else
    pass=$((pass + 1))
  fi
}

input="$scratch/input.kt"
printf '%s' "$unformatted" > "$input"
formatted="$scratch/formatted.kt"
printf 'fun f() = println("hello, world")\n' > "$formatted"
broken="$scratch/broken.kt"
printf 'fun    f1 (  ' > "$broken"

scenario help "" --help
scenario help_short "" -h
scenario version "" --version
scenario no_args ""
scenario unknown "" --bogus src
scenario at_later "" src @args.txt
scenario stdin "$input" -
scenario stdin_kotlinlang "$input" --kotlinlang-style -
scenario stdin_google_keep "$input" --google-style --do-not-remove-unused-imports -
scenario stdin_dry "$input" -n -
scenario stdin_dry_clean "$formatted" --dry-run -
scenario stdin_exit "$input" --set-exit-if-changed -
scenario stdin_exit_clean "$formatted" --set-exit-if-changed -
scenario stdin_broken "$broken" -
scenario stdin_name "$broken" --stdin-name=pkg/Foo.kt -
scenario stdin_name_bad "" --stdin-name src
scenario stdin_and_files "" - src/A.kt -
scenario stdin_name_no_value "" --stdin-name= -
scenario single_file "" src/A.kt
scenario single_java "" src/Other.java
scenario single_missing "" nope.kt
scenario dir "" src
scenario dir_quiet "" --quiet src
scenario dir_dry "" -n src
scenario dir_dry_exit "" -n --set-exit-if-changed src
scenario dir_exit "" --set-exit-if-changed src/ok
scenario multi_mixed "" src/A.kt src/Other.java src/ok
scenario dot "" .
scenario missing_dir "" nope1 nope2
scenario broken_file "" src/broken.kt
scenario lambdas "" src/lambdas.kt
scenario argfile "" @args.txt
scenario argfile_missing "" @missing.txt
scenario ec_off "" ec
# One file per run: with many files, ktfmt 0.64's shared ec4j cache races (files fail with
# "Could not load .editorconfig" or are silently skipped), which we don't reproduce.
for c in plain width80 width_off indent3 indent_tab tab_style ij_indent ij_cont ij_cont_both \
    comma_none comma_upper comma_bad unset nested glob_braces glob_case root_stop bad_int; do
  scenario "ec_$c" "" --enable-editorconfig "ec/$c/F.kt"
  scenario "ec_${c}_sub" "" --enable-editorconfig --kotlinlang-style "ec/$c/sub/G.kt"
done

echo "cli-diff: $pass passed, $fail mismatched"
((fail == 0))
