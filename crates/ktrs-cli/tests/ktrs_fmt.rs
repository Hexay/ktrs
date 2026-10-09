//! `ktrs fmt`'s own flags (the `ktfmt` drop-in has ktfmt's tests).

mod common;

use std::io::Cursor;
use std::path::Path;

use common::{TempDir, strings, write_text};
use ktrs_cli::ktrs_fmt::{parse_fmt_args, run_with};

/// Runs `ktrs fmt <args>` with `input` on stdin: exit code and stdout.
fn fmt(input: &str, args: &[&str]) -> (i32, String) {
    let parsed = parse_fmt_args(&strings(args)).unwrap();
    let mut out = Vec::new();
    let exit_code = run_with(&parsed, Path::new("."), Cursor::new(input.as_bytes().to_vec()), &mut out, Vec::new()).unwrap();
    (exit_code, String::from_utf8(out).unwrap())
}

const CODE: &str = "fun f() {\n  val x = 1\n}\n";

#[test]
fn styles() {
    assert_eq!(fmt(CODE, &["-"]).1, CODE);
    assert_eq!(fmt(CODE, &["--style", "kotlinlang", "-"]).1, "fun f() {\n    val x = 1\n}\n");
    assert_eq!(fmt(CODE, &["--style=kotlinlang", "-"]).1, "fun f() {\n    val x = 1\n}\n");
}

#[test]
fn editorconfig_applies_to_stdin_at_the_stdin_name() {
    let dir = TempDir::new("ktrs-fmt-stdin-editorconfig");
    write_text(&dir.path().join(".editorconfig"), "root = true\n[*.kt]\nindent_size = 8\n");
    let name = dir.path().join("src").join("A.kt").display().to_string();
    let with = fmt(CODE, &["--editorconfig", "--stdin-name", &name, "-"]);
    assert_eq!(with, (0, "fun f() {\n        val x = 1\n}\n".to_owned()));
    assert_eq!(fmt(CODE, &["--stdin-name", &name, "-"]).1, CODE, "only with --editorconfig");
    assert_eq!(fmt(CODE, &["--editorconfig", "-"]).1, CODE, "no name, no lookup");
}

#[test]
fn lines_and_offsets_format_part_of_the_input() {
    let code = "fun untouched ( ) =   1\n\nfun test() {\n  val selected    =   2\n  val adjacent    =   3\n}\n";
    let selected = "fun untouched ( ) =   1\n\nfun test() {\n  val selected = 2\n  val adjacent    =   3\n}\n";
    assert_eq!(fmt(code, &["--lines", "4", "-"]), (0, selected.to_owned()));
    assert_eq!(fmt(code, &["--lines=4:4", "-"]).1, selected);
    let offset = code.find("selected").unwrap().to_string();
    assert_eq!(fmt(code, &["--offset", &offset, "--length", "0", "-"]).1, selected);
    assert_eq!(fmt(code, &["--check", "--lines", "1", "-"]).0, 1);

    assert!(parse_fmt_args(&strings(&["--lines", "x", "-"])).is_err());
    assert!(parse_fmt_args(&strings(&["--offset", "3", "-"])).is_err());
    assert!(parse_fmt_args(&strings(&["--lines", "1"])).is_err(), "the default path is a directory");
    assert!(parse_fmt_args(&strings(&["--lines", "1", "A.kt", "B.kt"])).is_err());
    assert!(parse_fmt_args(&strings(&["--lines", "1", "--changed-since", "main", "A.kt"])).is_err());
    assert!(parse_fmt_args(&strings(&["--offset", "1", "--length", "2", "--changed-since=main", "A.kt"])).is_err());
    let github = parse_fmt_args(&strings(&["--check", "--reporter", "github", "--lines", "1", "A.kt"])).unwrap();
    assert!(github.github && !github.parsed.line_ranges.is_empty());
}

#[test]
fn bad_arguments() {
    assert!(parse_fmt_args(&strings(&["--style", "pretty"])).is_err());
    assert!(parse_fmt_args(&strings(&["--bogus"])).is_err());
    assert!(parse_fmt_args(&strings(&["-", "A.kt"])).is_err());
    assert!(parse_fmt_args(&strings(&["--stdin-name", "A.kt", "A.kt"])).is_err());
    assert!(parse_fmt_args(&strings(&["--changed-since", "main", "-"])).is_err());
    assert!(parse_fmt_args(&strings(&["--changed-since"])).is_err());
    assert!(parse_fmt_args(&strings(&["--reporter", "github"])).is_err(), "only with --check");
    assert!(parse_fmt_args(&strings(&["--check", "--reporter", "sarif"])).is_err());
    assert!(parse_fmt_args(&strings(&["--check", "--reporter=plain"])).is_ok());
}
