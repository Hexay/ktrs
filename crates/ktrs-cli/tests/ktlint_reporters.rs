//! Ports of the ktlint reporter tests (`ktlint-cli-reporter-*/src/test`): plain, plain-summary, json,
//! checkstyle, format. The rest: ktlint_reporters_files.rs.

mod ktlint_reporters_support;

use ktrs_cli::ktlint::console::{LINE_SEPARATOR, Printer};
use ktrs_cli::ktlint::reporter::checkstyle::CheckStyleReporter;
use ktrs_cli::ktlint::reporter::format::{FormatReporter, FormatReporterProvider};
use ktrs_cli::ktlint::reporter::json::JsonReporter;
use ktrs_cli::ktlint::reporter::plain::{PlainReporter, PlainReporterProvider};
use ktrs_cli::ktlint::reporter::plain_summary::PlainSummaryReporter;
use ktrs_cli::ktlint::reporter::{Color, FILE_SEPARATOR, ReporterV2};
use ktlint_reporters_support::*;

#[test]
fn plain_reports_normal_rule_violations() {
    let (out, captured) = Printer::buffer();
    let mut reporter = PlainReporter::with_defaults(out);
    reporter.on_lint_error("file-1.kt", &e(1, 1, "rule-1", "description-error-at-position-1:1", CAN));
    reporter.on_lint_error("file-1.kt", &e(2, 1, "rule-2", "description-error-at-position-2:1", FIXED));
    reporter.on_lint_error("file-2.kt", &e(1, 10, "rule-1", "description-error-at-position-1:10", CAN));
    reporter.on_lint_error("file-2.kt", &e(2, 20, "rule-2", "description-error-at-position-2:20", CAN));
    reporter.on_lint_error("file-3.kt", &e(1, 1, "rule-1", "description-error-at-position-1:1", FIXED));
    reporter.after_all();
    assert_eq!(
        captured.text(),
        lines(
            "file-1.kt:1:1: description-error-at-position-1:1 (rule-1)\n\
             file-2.kt:1:10: description-error-at-position-1:10 (rule-1)\n\
             file-2.kt:2:20: description-error-at-position-2:20 (rule-2)\n\
             \n\
             Summary error count (descending) by rule:\n  rule-1: 2\n  rule-2: 1\n"
        )
    );
}

const PARSE_DETAIL: &str = "Not a valid Kotlin file (18:51 unexpected tokens (use ';' to separate expressions on the same line)) (cannot be auto-corrected) ()";

fn feed_other(reporter: &mut dyn ReporterV2) {
    reporter.on_lint_error("file-1.kt", &e(18, 51, "", PARSE_DETAIL, PARSE));
    reporter.on_lint_error("file-2.kt", &e(18, 51, "", PARSE_DETAIL, PARSE));
    reporter.on_lint_error("file-3.kt", &e(18, 51, "", "Something else", CAN));
    reporter.after_all();
}

#[test]
fn plain_reports_other_violations() {
    let (out, captured) = Printer::buffer();
    feed_other(&mut PlainReporter::with_defaults(out));
    assert_eq!(
        captured.text(),
        lines(&format!(
            "file-1.kt:18:51: {PARSE_DETAIL} ()\nfile-2.kt:18:51: {PARSE_DETAIL} ()\nfile-3.kt:18:51: Something else ()\n\n\
             Summary error count (descending) by rule:\n  Not a valid Kotlin file: 2\n  Unknown: 1\n"
        ))
    );
}

#[test]
fn plain_colored_output() {
    let (out, captured) = Printer::buffer();
    let color = Color::DarkGray;
    let mut reporter = PlainReporter::new(out, false, true, color, false);
    reporter.on_lint_error(&format!("{FILE_SEPARATOR}one-fixed-and-one-not.kt"), &e(1, 1, "rule-1", "<\"&'>", CAN));
    assert_eq!(
        captured.text(),
        format!(
            "{}one-fixed-and-one-not.kt{}1{} <\"&'> {}{LINE_SEPARATOR}",
            color.paint(FILE_SEPARATOR),
            color.paint(":"),
            color.paint(":1:"),
            color.paint("(rule-1)")
        )
    );
}

#[test]
fn plain_grouped_by_file() {
    let (out, captured) = Printer::buffer();
    let mut reporter = PlainReporter::new(out, true, false, Color::DarkGray, false);
    feed_standard(&mut reporter, "/");
    for f in ["/one-fixed-and-one-not.kt", "/two-not-fixed.kt", "/all-corrected.kt"] {
        reporter.after(f);
    }
    assert_eq!(
        captured.text(),
        lines(
            "/one-fixed-and-one-not.kt\n  1:1 <\"&'> (rule-1)\n/two-not-fixed.kt\n  1:10 I thought I would again (rule-1)\n  \
             2:20 A single thin straight line (rule-2)\n"
        )
    );
}

#[test]
fn plain_pads_columns() {
    let (out, captured) = Printer::buffer();
    let mut reporter = PlainReporter::new(out, false, false, Color::DarkGray, true);
    reporter.on_lint_error("f.kt", &e(1, 7, "r", "d", CAN));
    assert_eq!(captured.text(), lines("f.kt:1:7   : d (r)\n"));
}

#[test]
fn plain_provider_color_names() {
    assert!(PlainReporterProvider::get(Printer::buffer().0, &opts(&[("color_name", "RED")])).is_ok());
    for opt in [opts(&[]), opts(&[("color_name", "")]), opts(&[("color_name", "GARBAGE_INPUT")])] {
        let error = PlainReporterProvider::get(Printer::buffer().0, &opt).err().unwrap();
        assert_eq!(error, "java.lang.IllegalArgumentException: Invalid color parameter.");
    }
}

#[test]
fn plain_summary_counts_autocorrected_and_not() {
    let (out, captured) = Printer::buffer();
    let mut reporter = PlainSummaryReporter::new(out);
    reporter.on_lint_error("file-1.kt", &e(1, 1, "rule-1", "d (cannot be auto-corrected)", CAN));
    reporter.on_lint_error("file-1.kt", &e(2, 1, "rule-2", "d", FIXED));
    reporter.on_lint_error("file-2.kt", &e(1, 10, "rule-1", "d (cannot be auto-corrected)", CAN));
    reporter.on_lint_error("file-2.kt", &e(2, 20, "rule-2", "d (cannot be auto-corrected)", CAN));
    reporter.on_lint_error("file-3.kt", &e(1, 1, "rule-1", "d", FIXED));
    for f in ["file-1.kt", "file-2.kt", "file-3.kt"] {
        reporter.after(f);
    }
    reporter.after_all();
    assert_eq!(
        captured.text(),
        lines(
            "Count (descending) of autocorrected errors by rule:\n  rule-1: 1\n  rule-2: 1\n\n\
             Count (descending) of errors not autocorrected by rule:\n  rule-1: 2\n  rule-2: 1\n"
        )
    );
}

#[test]
fn plain_summary_reports_other_violations() {
    let (out, captured) = Printer::buffer();
    feed_other(&mut PlainSummaryReporter::new(out));
    assert_eq!(
        captured.text(),
        lines("Count (descending) of errors not autocorrected by rule:\n  Not a valid Kotlin file: 2\n  Unknown: 1\n")
    );
}

#[test]
fn json_report_generation() {
    let (out, captured) = Printer::buffer();
    let mut reporter = JsonReporter::new(out);
    feed_standard(&mut reporter, "/");
    reporter.after_all();
    let error = |line, col, message: &str, rule| {
        format!(
            "            {{\n                \"line\": {line},\n                \"column\": {col},\n                \
             \"message\": \"{message}\",\n                \"rule\": \"{rule}\"\n            }}"
        )
    };
    let expected = format!(
        "[\n    {{\n        \"file\": \"/one-fixed-and-one-not.kt\",\n        \"errors\": [\n{}\n        ]\n    }},\n    \
         {{\n        \"file\": \"/two-not-fixed.kt\",\n        \"errors\": [\n{},\n{}\n        ]\n    }}\n]\n",
        error(1, 1, "<\\\"&'>", "rule-1"),
        error(1, 10, "I thought I would again", "rule-1"),
        error(2, 20, "A single thin straight line", "rule-2"),
    );
    assert_eq!(captured.text(), lines(&expected));
}

#[test]
fn json_proper_escaping() {
    let (out, captured) = Printer::buffer();
    let mut reporter = JsonReporter::new(out);
    reporter.on_lint_error("src\\main\\all\\corrected.kt", &e(4, 7, "rule-7", "\\n\n\r\t\"", CAN));
    reporter.after_all();
    let text = captured.text();
    assert!(text.contains(r#""file": "src\\main\\all\\corrected.kt","#), "{text}");
    assert!(text.contains(r#""message": "\\n\n\r\t\"","#), "{text}");
}

#[test]
fn checkstyle_report_generation() {
    let (out, captured) = Printer::buffer();
    let mut reporter = CheckStyleReporter::new(out);
    feed_standard(&mut reporter, "/");
    reporter.after_all();
    assert_eq!(
        captured.text(),
        lines(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<checkstyle version=\"8.0\">\n    \
             <file name=\"/one-fixed-and-one-not.kt\">\n        <error line=\"1\" column=\"1\" severity=\"error\" \
             message=\"&lt;&quot;&amp;&apos;&gt;\" source=\"rule-1\" />\n    </file>\n    \
             <file name=\"/two-not-fixed.kt\">\n        <error line=\"1\" column=\"10\" severity=\"error\" \
             message=\"I thought I would again\" source=\"rule-1\" />\n        <error line=\"2\" column=\"20\" \
             severity=\"error\" message=\"A single thin straight line\" source=\"rule-2\" />\n    </file>\n</checkstyle>\n"
        )
    );
}

const CAN_NOT: (&str, &str) = ("rule-1", "This error can *not* be autocorrected");

#[test]
fn format_reporter_format_running() {
    let (out, captured) = Printer::buffer();
    let mut reporter = FormatReporter::new(out, true, false, Color::DarkGray);
    let cannot = e(1, 1, CAN_NOT.0, CAN_NOT.1, CANNOT);
    reporter.on_lint_error("/path/to/some-file-1.kt", &cannot);
    reporter.on_lint_error("/path/to/some-file-2.kt", &cannot);
    reporter.on_lint_error("/path/to/some-file-2.kt", &cannot);
    reporter.on_lint_error("/path/to/some-file-3.kt", &e(1, 1, CAN_NOT.0, CAN_NOT.1, FIXED));
    for i in 1..=4 {
        reporter.after(&format!("/path/to/some-file-{i}.kt"));
    }
    assert_eq!(
        captured.text(),
        lines(
            "/path/to/some-file-1.kt: Format not completed (1 violation needs manual fixing)\n\
             /path/to/some-file-2.kt: Format not completed (2 violations need manual fixing)\n\
             /path/to/some-file-3.kt: Format completed (all violations have been fixed)\n\
             /path/to/some-file-4.kt: Format not needed (no violations found)\n"
        )
    );
}

#[test]
fn format_reporter_lint_running() {
    let (out, captured) = Printer::buffer();
    let mut reporter = FormatReporter::new(out, false, false, Color::DarkGray);
    reporter.on_lint_error("/path/to/some-file-1.kt", &e(1, 1, "some-rule", "This error can be autocorrected", CAN));
    reporter.after("/path/to/some-file-1.kt");
    reporter.after("/path/to/some-file-2.kt");
    assert_eq!(
        captured.text(),
        lines(
            "/path/to/some-file-1.kt: Format required (all violations can be autocorrected)\n\
             /path/to/some-file-2.kt: Format not needed (no violations found)\n"
        )
    );
}

#[test]
fn format_reporter_colored() {
    let (out, captured) = Printer::buffer();
    let color = Color::DarkGray;
    let mut reporter = FormatReporter::new(out, true, true, color);
    let file = format!("{FILE_SEPARATOR}some-file-name.kt");
    reporter.on_lint_error(&file, &e(1, 1, CAN_NOT.0, CAN_NOT.1, CANNOT));
    reporter.after(&file);
    assert_eq!(
        captured.text(),
        format!(
            "{}some-file-name.kt{} Format not completed (1 violation needs manual fixing){LINE_SEPARATOR}",
            color.paint(FILE_SEPARATOR),
            color.paint(":")
        )
    );
}

#[test]
fn format_reporter_provider_options() {
    let get = |pairs: &[(&str, &str)]| FormatReporterProvider::get(Printer::buffer().0, &opts(pairs)).err();
    let iae = |m: &str| Some(format!("java.lang.IllegalArgumentException: {m}"));
    assert_eq!(get(&[]), iae("Format is not specified in config options"));
    assert_eq!(get(&[("format", "true"), ("color_name", "RED")]), None);
    assert_eq!(
        get(&[("format", "invalid"), ("color_name", "RED")]),
        iae("The string doesn't represent a boolean value: invalid")
    );
    assert_eq!(get(&[("format", "true")]), iae("Invalid color parameter."));
    assert_eq!(get(&[("format", "true"), ("color_name", "")]), iae("Invalid color parameter."));
    assert_eq!(get(&[("format", "true"), ("color_name", "GARBAGE_INPUT")]), iae("Invalid color parameter."));
}
