//! Port of ktfmt's `cli/ParsedArgsTest.kt` (v0.64).

mod common;

use std::fs;

use common::{TempDir, strings, write_text};
use ktrs_cli::ktfmt::{ParseResult, ParsedArgs, parse_options, process_args};
use ktrs_fmt::{FormattingOptions, GOOGLE_FORMAT, KOTLINLANG_FORMAT, META_FORMAT};

fn parse(args: &[&str]) -> ParseResult {
    parse_options(&strings(args))
}

fn assert_succeeds(parse_result: ParseResult) -> ParsedArgs {
    match parse_result {
        ParseResult::Ok(parsed) => parsed,
        other => panic!("expected ParseResult::Ok, got {other:?}"),
    }
}

fn assert_error(parse_result: ParseResult) {
    assert!(matches!(parse_result, ParseResult::Error(_)), "{parse_result:?}");
}

fn assert_show_message(parse_result: ParseResult) {
    assert!(matches!(parse_result, ParseResult::ShowMessage(_)), "{parse_result:?}");
}

/// Upstream's `parseResultOk` with its defaults: META_FORMAT, no flags, unused imports removed.
fn parse_result_ok(
    file_names: &[&str],
    formatting_options: FormattingOptions,
    dry_run: bool,
    set_exit_if_changed: bool,
) -> ParseResult {
    ParseResult::Ok(ParsedArgs {
        file_names: strings(file_names),
        formatting_options: FormattingOptions { remove_unused_imports: true, ..formatting_options },
        dry_run,
        set_exit_if_changed,
        stdin_name: None,
        editor_config: false,
        quiet: false,
    })
}

#[test]
fn unknown_flags_return_an_error() {
    assert_error(parse(&["--unknown"]));
}

#[test]
fn unknown_flags_starting_with_at_return_an_error() {
    assert_error(parse(&["@unknown"]));
}

#[test]
fn parse_options_uses_default_values_when_args_are_empty() {
    let parsed = assert_succeeds(parse(&["foo.kt"]));
    assert_eq!(parsed.formatting_options, META_FORMAT);
}

#[test]
fn parse_options_recognizes_meta_style() {
    let parsed = assert_succeeds(parse(&["--meta-style", "foo.kt"]));
    assert_eq!(parsed.formatting_options, META_FORMAT);
}

#[test]
fn parse_options_recognizes_google_style() {
    let parsed = assert_succeeds(parse(&["--google-style", "foo.kt"]));
    assert_eq!(parsed.formatting_options, GOOGLE_FORMAT);
}

#[test]
fn parse_options_recognizes_dry_run() {
    assert!(assert_succeeds(parse(&["--dry-run", "foo.kt"])).dry_run);
}

#[test]
fn parse_options_recognizes_n_as_dry_run() {
    assert!(assert_succeeds(parse(&["-n", "foo.kt"])).dry_run);
}

#[test]
fn parse_options_recognizes_set_exit_if_changed() {
    assert!(assert_succeeds(parse(&["--set-exit-if-changed", "foo.kt"])).set_exit_if_changed);
}

#[test]
fn parse_options_defaults_to_removing_imports() {
    assert!(assert_succeeds(parse(&["foo.kt"])).formatting_options.remove_unused_imports);
}

#[test]
fn parse_options_recognizes_do_not_remove_unused_imports_to_removing_imports() {
    let parsed = assert_succeeds(parse(&["--do-not-remove-unused-imports", "foo.kt"]));
    assert!(!parsed.formatting_options.remove_unused_imports);
}

#[test]
fn parse_options_recognizes_enable_editorconfig() {
    assert!(assert_succeeds(parse(&["--enable-editorconfig", "foo.kt"])).editor_config);
}

#[test]
fn parse_options_recognizes_quiet() {
    assert!(assert_succeeds(parse(&["--quiet", "foo.kt"])).quiet);
}

#[test]
fn parse_options_recognizes_stdin_name() {
    let parsed = assert_succeeds(parse(&["--stdin-name=my/foo.kt", "-"]));
    assert_eq!(parsed.stdin_name.as_deref(), Some("my/foo.kt"));
}

#[test]
fn parse_options_accepts_stdin_name_with_empty_value() {
    let parsed = assert_succeeds(parse(&["--stdin-name=", "-"]));
    assert_eq!(parsed.stdin_name.as_deref(), Some(""));
}

#[test]
fn parse_options_rejects_stdin_name_without_value() {
    assert_error(parse(&["--stdin-name"]));
}

#[test]
fn parse_options_rejects_dash_and_files_at_the_same_time() {
    assert_error(parse(&["-", "File.kt"]));
}

#[test]
fn parse_options_rejects_stdin_name_when_not_reading_from_stdin() {
    assert_error(parse(&["--stdin-name=foo", "file1.kt"]));
}

#[test]
fn parse_options_recognises_help() {
    assert_show_message(parse(&["--help"]));
}

#[test]
fn parse_options_recognises_h() {
    assert_show_message(parse(&["-h"]));
}

#[test]
fn arg_help_overrides_all_others() {
    assert_show_message(parse(&["--style=google", "@unknown", "--help", "file.kt"]));
}

#[test]
fn parse_options_recognises_version() {
    assert_show_message(parse(&["--version"]));
}

#[test]
fn parse_options_recognises_v() {
    assert_show_message(parse(&["-v"]));
}

#[test]
fn arg_version_overrides_all_others() {
    assert_show_message(parse(&["--style=google", "@unknown", "--version", "file.kt"]));
}

// Upstream matches the JVM's FileNotFoundException text "(No such file or directory)"; the OS
// message differs per platform, so this checks the error kind.
#[test]
fn process_args_use_the_at_file_option_with_non_existing_file() {
    let e = process_args(&strings(&["@non-existing-file"])).expect_err("expected an IO error");
    assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn process_args_use_the_at_file_option_with_file_containing_arguments() {
    let root = TempDir::new("parsed_args");
    let file = root.path().join("existing-file");
    write_text(&file, "--google-style\n--dry-run\n--set-exit-if-changed\nFile1.kt\nFile2.kt\n");

    let canonical_path = fs::canonicalize(&file).unwrap();
    let result = process_args(&[format!("@{}", canonical_path.display())]).unwrap();
    let parsed = assert_succeeds(result);

    assert_eq!(parsed.formatting_options, GOOGLE_FORMAT);
    assert!(parsed.dry_run);
    assert!(parsed.set_exit_if_changed);
    assert_eq!(parsed.file_names, strings(&["File1.kt", "File2.kt"]));
}

#[test]
fn parses_multiple_args_successfully() {
    let test_result = parse(&["--google-style", "--dry-run", "--set-exit-if-changed", "File.kt"]);
    assert_eq!(test_result, parse_result_ok(&["File.kt"], GOOGLE_FORMAT, true, true));
}

#[test]
fn last_style_in_args_wins() {
    let test_result = parse(&["--google-style", "--kotlinlang-style", "File.kt"]);
    assert_eq!(test_result, parse_result_ok(&["File.kt"], KOTLINLANG_FORMAT, false, false));
}

#[test]
fn error_when_parsing_multiple_args_and_one_is_unknown() {
    let test_result = parse(&["@unknown", "--google-style", "File.kt"]);
    assert_eq!(test_result, ParseResult::Error("Unexpected option: @unknown".to_owned()));
}
