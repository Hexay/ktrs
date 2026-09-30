//! ktlint's own unit tests for the ported rules (`<Rule>Test.kt`), vendored as data by
//! `py -3 tools/ktlint-tests/extract-rule-tests.py <TestClass>...` into testdata/ktlint/. Same assertions
//! as KtLintAssertThat: lint violations of the rule under test (any order, distinct), the format result,
//! and for `hasNoLintViolations` also no format errors and unchanged code.

use std::collections::BTreeSet;
use std::path::Path;
use std::{env, fs};

use ktrs_lint::rules::standard_rule_provider;
use ktrs_lint::{AutocorrectDecision, Code, KtLintRuleEngine};

#[derive(Default)]
struct Case {
    name: String,
    flags: Vec<String>,
    code: Option<String>,
    formatted: Option<String>,
    violations: Vec<String>,
}

fn parse_cases(data: &str) -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::new();
    let mut section = "";
    let mut lines: Vec<&str> = Vec::new();
    for line in data.lines().chain(["#### end"]) {
        let Some(header) = line.strip_prefix("#### ") else {
            lines.push(line);
            continue;
        };
        if let Some(case) = cases.last_mut() {
            let body = Some(lines.join("\n"));
            match section {
                "code" => case.code = body,
                "formatted" => case.formatted = body,
                "violations" => case.violations = lines.iter().map(|l| l.to_string()).collect(),
                _ => {}
            }
        }
        lines.clear();
        section = "";
        match header.split_once(' ') {
            Some(("case", name)) => cases.push(Case { name: name.to_owned(), ..Case::default() }),
            Some(("flag", flag)) => cases.last_mut().unwrap().flags.push(flag.to_owned()),
            _ => section = header,
        }
    }
    cases
}

fn run(test_class: &str, rule_id: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../testdata/ktlint/{test_class}.txt"));
    let data = fs::read_to_string(&path).unwrap().replace("\r\n", "\n");
    let engine = KtLintRuleEngine::new(vec![standard_rule_provider(rule_id).unwrap()]);
    let mut failures = Vec::new();
    let mut ran = 0;
    for case in parse_cases(&data) {
        if case.flags.iter().any(|f| f == "disabled" || f.starts_with("unsupported")) {
            continue;
        }
        ran += 1;
        let code = Code::from_snippet(case.code.as_deref().unwrap(), false);
        let mut lint = BTreeSet::new();
        engine
            .lint(&code, &mut |e| {
                lint.insert(format!("{}:{}\t{}\t{}", e.line, e.col, if e.can_be_auto_corrected { "auto" } else { "manual" }, e.detail));
            })
            .unwrap();
        let expected: BTreeSet<String> = case.violations.iter().cloned().collect();
        if lint != expected {
            failures.push(format!("{}: lint\n  expected {expected:?}\n  actual   {lint:?}", case.name));
        }
        let mut format_errors = 0;
        let formatted = engine
            .format(&code, &mut |_| {
                format_errors += 1;
                AutocorrectDecision::AllowAutocorrect
            })
            .unwrap();
        let no_violations = case.flags.iter().any(|f| f == "no-violations");
        if no_violations && (formatted != code.content || format_errors != 0) {
            failures.push(format!("{}: format changed a clean snippet\n{formatted}", case.name));
        }
        let skip_format = case.flags.iter().any(|f| f == "additional-rules");
        if let Some(expected) = case.formatted.as_ref().filter(|_| !skip_format)
            && formatted != *expected
        {
            failures.push(format!("{}: format\n--- expected\n{expected}\n--- actual\n{formatted}", case.name));
        }
    }
    println!("{test_class}: {ran} cases run");
    assert!(ran > 0);
    assert!(failures.is_empty(), "{} of {ran} cases failed:\n{}", failures.len(), failures.join("\n\n"));
}

#[test]
fn no_semicolons_rule_test() {
    run("NoSemicolonsRuleTest", "no-semi");
}

#[test]
fn spacing_around_comma_rule_test() {
    run("SpacingAroundCommaRuleTest", "comma-spacing");
}

#[test]
fn multi_line_if_else_rule_test() {
    run("MultiLineIfElseRuleTest", "multiline-if-else");
}
