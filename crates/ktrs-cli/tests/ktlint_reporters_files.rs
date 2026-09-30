//! Ports of the ktlint reporter tests: html, sarif, baseline (and the provider lookup).

mod ktlint_reporters_support;

use std::path::PathBuf;

use ktrs_cli::ktlint::console::{LINE_SEPARATOR, Printer};
use ktrs_cli::ktlint::reporter::html::HtmlReporter;
use ktrs_cli::ktlint::reporter::sarif::{SarifReporter, sanitize};
use ktrs_cli::ktlint::reporter::{KtlintCliError, ReporterEnvironment, ReporterV2, get_reporter};
use ktlint_reporters_support::*;

const HTML_HEAD: &str = "<html>\n<head>\n<link href=\"https://fonts.googleapis.com/css?family=Source+Code+Pro\" \
    rel=\"stylesheet\" />\n<meta http-equiv=\"Content-Type\" Content=\"text/html; Charset=UTF-8\">\n<style>\nbody {\n    \
    font-family: 'Source Code Pro', monospace;\n}\nh3 {\n    font-size: 12pt;\n}</style>\n</head>\n<body>\n";

fn html(errors: &[(&str, KtlintCliError)]) -> String {
    let (out, captured) = Printer::buffer();
    let mut reporter = HtmlReporter::new(out);
    for (file, error) in errors {
        reporter.on_lint_error(file, error);
    }
    reporter.after_all();
    captured.text()
}

#[test]
fn html_empty_report() {
    assert_eq!(html(&[]), lines(&format!("{HTML_HEAD}<p>Congratulations, no issues found!</p>\n</body>\n</html>\n")));
}

#[test]
fn html_issues_and_corrected() {
    let broken = |status| e(1, 1, "rule-1", "rule-1 broken", status);
    let body = |corrected| {
        format!(
            "{HTML_HEAD}<h1>Overview</h1>\n<p>Issues found: 1</p>\n<p>Issues corrected: {corrected}</p>\n<h3>/file1.kt</h3>\n\
             <ul>\n<li>(1, 1): rule-1 broken  (rule-1)</li>\n</ul>\n</body>\n</html>\n"
        )
    };
    assert_eq!(html(&[("/file1.kt", broken(CAN))]), lines(&body(0)));
    assert_eq!(html(&[("/file1.kt", broken(CAN)), ("/file2.kt", broken(FIXED))]), lines(&body(1)));
}

#[test]
fn html_escapes_special_symbols() {
    let text = html(&[
        ("/file1.kt", e(1, 1, "rule-1", "Error message contains a generic type like List<Int> (cannot be auto-corrected)", CAN)),
        ("/file1.kt", e(2, 1, "rule-2", "Error message contains special html symbols like a<b>c\"d'e&f (cannot be auto-corrected)", CAN)),
    ]);
    assert!(text.contains("<li>(1, 1): Error message contains a generic type like List&lt;Int&gt; (cannot be auto-corrected)  (rule-1)</li>"));
    assert!(text.contains(
        "<li>(2, 1): Error message contains special html symbols like a&lt;b&gt;c&quot;d&apos;e&amp;f (cannot be auto-corrected)  (rule-2)</li>"
    ));
}

#[test]
fn sarif_report_generation() {
    let home = std::env::temp_dir();
    let working_directory = sanitize(&home.to_string_lossy());
    let (out, captured) = Printer::buffer();
    let mut reporter = SarifReporter::new(out, Some(home.clone()));
    reporter.before_all();
    feed_standard(&mut reporter, &working_directory);
    reporter.after_all();
    let actual: String = captured.text().chars().filter(|c| !c.is_whitespace()).collect();
    let result = |uri: &str, col, line, text: &str, rule: &str| {
        format!(
            r#"{{"level":"error","locations":[{{"physicalLocation":{{"artifactLocation":{{"uri":"{uri}","uriBaseId":"%SRCROOT%"}},"region":{{"startColumn":{col},"startLine":{line}}}}}}}],"message":{{"text":"{text}"}},"ruleId":"{rule}"}}"#
        )
    };
    let results = [
        result("one-fixed-and-one-not.kt", 1, 1, r#"<\"&'>"#, "rule-1"),
        result("one-fixed-and-one-not.kt", 1, 2, "And if you see my friend", "rule-2"),
        result("two-not-fixed.kt", 10, 1, "I thought I would again", "rule-1"),
        result("two-not-fixed.kt", 20, 2, "A single thin straight line", "rule-2"),
        result("all-corrected.kt", 1, 1, "I thought we had more time", "rule-1"),
    ];
    let v = ktrs_cli::ktlint::reporter::KTLINT_VERSION;
    let expected = format!(
        r#"{{"$schema":"https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json","version":"2.1.0","runs":[{{"originalUriBaseIds":{{"%SRCROOT%":{{"uri":"file://{working_directory}"}}}},"results":[{}],"tool":{{"driver":{{"downloadUri":"https://github.com/ktlint/ktlint/releases/tag/{v}","fullName":"ktlint","informationUri":"https://github.com/ktlint/ktlint/","language":"en","name":"ktlint","organization":"ktlint","rules":[],"semanticVersion":"{v}","version":"{v}"}}}}}}]}}"#,
        results.join(",")
    );
    let expected: String = expected.chars().filter(|c| !c.is_whitespace()).collect();
    assert_eq!(actual, expected);
}

#[test]
fn sarif_empty_report_layout() {
    let (out, captured) = Printer::buffer();
    let mut reporter = SarifReporter::new(out, Some(PathBuf::from("/home/ubuntu")));
    reporter.before_all();
    reporter.after_all();
    let text = captured.text();
    assert!(text.starts_with("{\n  \"$schema\": "), "{text}");
    assert!(text.contains("\n      \"results\": [],\n"), "{text}");
    assert!(text.ends_with(&format!("\n}}\n{LINE_SEPARATOR}")), "{text}");
}

#[test]
fn baseline_reporter_report_generation() {
    let (out, captured) = Printer::buffer();
    let env = ReporterEnvironment { user_home: None, working_dir: std::env::current_dir().unwrap() };
    let mut reporter = get_reporter("baseline", out, &opts(&[]), &env).unwrap().unwrap();
    feed_standard(reporter.as_mut(), "");
    reporter.after_all();
    assert_eq!(
        captured.text(),
        lines(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<baseline version=\"1.0\">\n    \
             <file name=\"one-fixed-and-one-not.kt\">\n        <error line=\"1\" column=\"1\" source=\"rule-1\" />\n    \
             </file>\n    <file name=\"two-not-fixed.kt\">\n        <error line=\"1\" column=\"10\" source=\"rule-1\" />\n        \
             <error line=\"2\" column=\"20\" source=\"rule-2\" />\n    </file>\n</baseline>\n"
        )
    );
}

#[test]
fn baseline_reporter_relativizes_absolute_paths() {
    let (out, captured) = Printer::buffer();
    let cwd = std::env::current_dir().unwrap();
    let env = ReporterEnvironment { user_home: None, working_dir: cwd.clone() };
    let mut reporter = get_reporter("baseline", out, &opts(&[]), &env).unwrap().unwrap();
    let file = cwd.join("src").join("A.kt");
    reporter.on_lint_error(&file.to_string_lossy(), &e(1, 1, "r", "d", CAN));
    reporter.after_all();
    assert!(captured.text().contains("<file name=\"src/A.kt\">"), "{}", captured.text());
}

/// The order a JVM `ConcurrentHashMap` iterated these keys in (JDK 21, recorded on the testbox).
#[test]
fn java_concurrent_hash_map_iteration_order() {
    let expected = "139,169,174,0,168,26,105,173,25,35,111,141,89,99,106,70,36,90,55,71,135,60,140,66,107,61,65,32,42,94,31,95,\
        134,175,170,144,20,108,19,133,180,104,59,136,142,146,29,110,172,138,102,5,196,7,164,166,112,28,115,163,167,57,58,100,23,\
        78,88,127,183,34,24,148,33,87,82,77,54,62,128,53,83,176,8,84,30,103,195,49,143,199,27,131,151,187,179,123,171,159,160,\
        51,124,75,119,191,154,125,16,122,190,157,79,121,69,15,45,81,91,120,56,80,46,12,156,22,6,11,40,86,85,21,41,9,155,74,52,\
        4,118,50,76,184,189,1,147,188,194,17,116,152,158,130,150,186,192,113,63,97,114,129,3,38,18,48,178,198,2,68,47,13,67,132,\
        177,162,14,92,197,93,43,72,44,98,10,64,181,182,73,149,161,96,145,185,153,37,101,109,39,126,137,193,117,165";
    let mut map = ktrs_cli::ktlint::reporter::java_map::JavaConcurrentHashMap::new();
    for i in 0..200 {
        map.get_or_put(&format!("/tmp/w/src/dir{}/File{i}.kt", i % 7), || i);
    }
    let order: Vec<String> = map.iter().map(|(_, v)| v.to_string()).collect();
    assert_eq!(order.join(","), expected);
}

#[test]
fn unknown_reporter_id() {
    let env = ReporterEnvironment { user_home: None, working_dir: PathBuf::from(".") };
    assert!(get_reporter("nope", Printer::buffer().0, &opts(&[]), &env).is_none());
}
