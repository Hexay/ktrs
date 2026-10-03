//! The `ktfmt` CLI on fuzzer finds (research/24-fuzzing.md, findings 6 and 7); expected stderr is the
//! ktfmt 0.64 jar's without stack frames (`tools/fuzz/differs.sh`).

mod common;

use common::{LS, TempDir, arg, read_text, run, write_text};

#[test]
fn exception_other_than_parse_or_formatting_error_fails_the_file_silently() {
    let root = TempDir::new("fuzz");
    let file = root.path().join("t.kt");
    write_text(&file, "import a; /* x */");
    let r = run("", &[&arg(&file)]);
    assert_eq!((r.exit_code, r.out.as_str(), r.err.as_str()), (1, "", ""));
    assert_eq!(read_text(&file), "import a; /* x */");
}

/// Finding 4: the parser's `AssertionError` (logged by IntelliJ's `DefaultLogger`, then uncaught).
#[test]
fn missed_tokens_print_the_log_and_the_uncaught_assertion_error() {
    let root = TempDir::new("fuzz");
    let file = root.path().join("t.kt");
    write_text(&file, "{fun<)]<T:@( {})");
    let r = run("", &[&arg(&file)]);
    let message = "Tokens [RPAR] were not inserted into the tree. Language: kotlin";
    assert_eq!((r.exit_code, r.out.as_str()), (1, ""));
    assert_eq!(
        r.err,
        format!(
            "ERROR: {message}\nDetails:\nmissedTokensFragment.txt\n{{fun<)]<T:@( {{}}){LS}\
             Exception in thread \"main\" java.lang.AssertionError: {message}{LS}"
        )
    );
    assert_eq!(read_text(&file), "{fun<)]<T:@( {})");
}

#[test]
fn parse_error_below_visit_element_is_reported_as_formatting_error() {
    let root = TempDir::new("fuzz");
    let file = root.path().join("t.kts");
    write_text(&file, "foo {} {}");
    let r = run("", &[&arg(&file)]);
    let error = "1:1: error: com.facebook.ktfmt.format.ParseError: 1:8: error: Maximum one trailing lambda is allowed";
    assert_eq!(r.exit_code, 1);
    assert_eq!(r.err, format!("{}:{error}{LS}com.google.googlejavaformat.FormattingError: {error}{LS}", arg(&file)));
}
