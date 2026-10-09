//! Fuzzer finds (research/24-fuzzing.md). Expected results are the ktfmt 0.65 jar's
//! `Formatter.format(META_FORMAT, KotlinCode(input, FileType.SCRIPT))`.

use ktrs_fmt::{FileType, FormatError, META_FORMAT, format};

/// Finding 2: a comment's `Doc.Tok` range is `[-1, 0)`, so `makeKToIJ` maps k = -1.
#[test]
fn comment_only_input_with_stray_whitespace_formats() {
    for (input, expected) in [
        (" // c", " // c\n"),
        ("/* c */ ", "/* c */\n"),
        (" /* c */", " /* c */\n"),
        ("  // comment  // comment", "  // comment  // comment\n"),
    ] {
        assert_eq!(format(input, FileType::Script, &META_FORMAT).as_deref(), Ok(expected), "{input:?}");
    }
}

/// Finding 6: the unused import's directive swallows the trailing comment, so removing it first
/// leaves its `;` past the end of the text, where `StringBuilder.replace` throws.
#[test]
fn removing_an_import_around_its_own_semicolon_throws_like_string_builder() {
    for (input, range) in [
        ("import a; /* x */", "Range [8, 0) out of bounds for length 0"),
        ("import a;// x\n", "Range [8, 1) out of bounds for length 1"),
    ] {
        let expected = format!("java.lang.StringIndexOutOfBoundsException: {range}");
        assert_eq!(format(input, FileType::Script, &META_FORMAT), Err(FormatError::Runtime(expected)), "{input:?}");
    }
}

/// Finding 4: a lambda reparse stops before its `)`; the parser's `AssertionError` beats the parse error before it.
#[test]
fn missed_tokens_throw_the_parsers_assertion_error() {
    for input in ["{fun<)]<T:@( {})", "val a = )\nfun f() = {fun<)]<T:@( {})"] {
        let error = format(input, FileType::Script, &META_FORMAT).unwrap_err();
        assert!(matches!(error, FormatError::MissedTokens(_)), "{error:?}");
        assert_eq!(error.to_string(), "java.lang.AssertionError: Tokens [RPAR] were not inserted into the tree. Language: kotlin");
    }
}

/// Finding 7: `visitElement` turns any other exception thrown below it into a `FormattingError`
/// whose message is the stack trace (here its first line).
#[test]
fn parse_error_below_visit_element_becomes_formatting_error() {
    let error = format("foo {} {}", FileType::Script, &META_FORMAT).unwrap_err();
    assert!(matches!(error, FormatError::Formatting(_)), "{error:?}");
    assert_eq!(
        error.to_string(),
        "1:1: error: org.jetbrains.kotlinx.ktfmt.format.ParseError: 1:8: error: Maximum one trailing lambda is allowed"
    );

    // Reached through visitor overrides only: the ParseError escapes as is.
    let error = format("fun f() {\n  foo {} {}\n}\n", FileType::Script, &META_FORMAT).unwrap_err();
    assert!(matches!(error, FormatError::Parse(_)), "{error:?}");
    assert_eq!(error.to_string(), "2:10: error: Maximum one trailing lambda is allowed");
}
