//! Port of `BaselineTest` (ktlint-cli-reporter-baseline) plus the loader's error cases.

mod common;

use common::TempDir;
use ktrs_cli::ktlint::baseline::{Baseline, BaselineStatus, contains_lint_error, does_not_contain, load_baseline};
use ktrs_cli::ktlint::console::Console;
use ktrs_cli::ktlint::logger::{Level, Logger};
use ktrs_cli::ktlint::reporter::{KtlintCliError, Status};

const VALID: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<baseline version="1.0">
    <file name="src/main/kotlin/Foo.kt">
        <error line="1" column="1" source="standard:max-line-length" />
        <error line="2" column="1" source="standard:max-line-length" />
        <error line="4" column="9" source="standard:property-naming" />
    </file>
</baseline>
"#;

const INVALID: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<baseline version=\"1.0\">\r\n    \
    <file name=\"src/main/kotlin/Foo2.kt\">\r\n        <error line=\"1\" column=\"1\" source=\"standard:max-line-length\" />\r\n\
    </baseline>\r\n";

/// Loads `content` as `baseline.xml`; returns the baseline, stdout (logs), stderr and whether the file survived.
fn load(content: Option<&str>) -> (Baseline, String, String, bool) {
    let dir = TempDir::new("ktlint-baseline");
    if let Some(content) = content {
        std::fs::write(dir.path().join("baseline.xml"), content).unwrap();
    }
    let (console, out, err) = Console::capture(b"");
    let logger = Logger::new(console.clone(), Level::Info);
    let baseline = load_baseline("baseline.xml", dir.path(), &logger, &console);
    (baseline, out.text(), err.text(), dir.path().join("baseline.xml").exists())
}

fn ignored(line: usize, col: usize, rule: &str) -> KtlintCliError {
    KtlintCliError::new(line, col, rule, "", Status::BaselineIgnored)
}

#[test]
fn valid_baseline_is_loaded() {
    let (baseline, out, err, exists) = load(Some(VALID));
    assert_eq!(baseline.status, BaselineStatus::Valid);
    assert_eq!(baseline.path.as_deref(), Some("baseline.xml"));
    assert_eq!(
        baseline.lint_errors_per_file.get("src/main/kotlin/Foo.kt").unwrap(),
        &vec![
            ignored(1, 1, "standard:max-line-length"),
            ignored(2, 1, "standard:max-line-length"),
            ignored(4, 9, "standard:property-naming"),
        ]
    );
    assert_eq!((out.as_str(), err.as_str(), exists), ("", "", true));
}

#[test]
fn invalid_xml_is_logged_and_deleted() {
    let (baseline, out, err, exists) = load(Some(INVALID));
    assert_eq!(baseline.status, BaselineStatus::Invalid);
    assert!(out.contains(" [main] ERROR io.github.ktlint.core.cli.reporter.baseline.Baseline -- Unable to parse baseline file: baseline.xml"), "{out}");
    assert!(
        err.contains("The element type \"file\" must be terminated by the matching end-tag \"</file>\"."),
        "{err}"
    );
    assert!(!exists);
}

#[test]
fn content_before_prolog() {
    let (baseline, _, err, _) = load(Some("garbage<"));
    assert_eq!(baseline.status, BaselineStatus::Invalid);
    assert!(err.starts_with("[Fatal Error] :1:1: Content is not allowed in prolog."), "{err}");
}

/// `[Fatal Error]` positions and messages as the JDK 21 Xerces reports them (recorded on the testbox).
#[test]
fn xerces_error_positions() {
    use ktrs_cli::ktlint::baseline::xml::parse_document;
    let cases = [
        (INVALID, 5, 3, "The element type \"file\" must be terminated by the matching end-tag \"</file>\"."),
        ("  \n  x", 2, 3, "Content is not allowed in prolog."),
        ("<a><b></b>", 1, 11, "XML document structures must start and end within the same entity."),
        ("<a b=1/>", 1, 6, "Open quote is expected for attribute \"b\" associated with an  element type  \"a\"."),
        ("<a></a>x", 1, 8, "Content is not allowed in trailing section."),
        ("<a></a><b/>", 1, 9, "The markup in the document following the root element must be well-formed."),
        ("<a>&foo;</a>", 1, 9, "The entity \"foo\" was referenced, but not declared."),
        ("<a b=\"<\"/>", 1, 7, "The value of attribute \"b\" associated with an element type \"a\" must not contain the '<' character."),
    ];
    for (doc, line, col, message) in cases {
        let e = parse_document(doc).err().unwrap_or_else(|| panic!("{doc:?} parsed"));
        assert_eq!((e.line, e.col, e.message.as_str()), (line, col, message), "{doc:?}");
    }
}

#[test]
fn empty_file() {
    let (_, _, err, _) = load(Some(""));
    assert!(err.starts_with("[Fatal Error] :1:1: Premature end of file."), "{err}");
}

#[test]
fn missing_number_is_a_number_format_exception() {
    let (baseline, out, err, exists) = load(Some(r#"<baseline><file name="a"><error column="1" source="x"/></file></baseline>"#));
    assert_eq!(baseline.status, BaselineStatus::Invalid);
    assert!(out.contains("-- For input string: \"\""), "{out}");
    assert_eq!((err.as_str(), exists), ("", false));
}

#[test]
fn missing_file_is_not_found() {
    let (baseline, out, _, _) = load(None);
    assert_eq!(baseline.status, BaselineStatus::NotFound);
    assert_eq!(out, "");
}

#[test]
fn entities_are_decoded() {
    let (baseline, ..) = load(Some(r#"<baseline><file name="a&amp;b&#65;"><error line="1" column="2" source="s"/></file></baseline>"#));
    assert!(baseline.lint_errors_per_file.contains_key("a&bA"));
}

#[test]
fn contains_ignores_detail_and_status() {
    let list = vec![ignored(1, 2, "r")];
    let found = KtlintCliError::new(1, 2, "r", "some detail", Status::LintCanBeAutocorrected);
    assert!(contains_lint_error(&list, &found));
    assert!(does_not_contain(&list, &KtlintCliError::new(1, 3, "r", "", Status::LintCanBeAutocorrected)));
}
