//! `ktrs lsp` end to end, in process (crates/ktrs-lsp). Fixes and ktlint formatting must equal the `ktlint`
//! drop-in's `-F` on the same file.

mod common;
mod lsp_support;

use common::{TempDir, read_text, strings, write_text};
use lsp_support::Client;
use lsp_types::{DiagnosticSeverity, NumberOrString, Position, Range};
use serde_json::{Value, json};

const ONE_VIOLATION: &str = "val  foo = 1\n";
const MANY_VIOLATIONS: &str = "import java.util.*\n\nfun  foo( a:Int ) {\n    println(a) ;\n}\n";

/// `ktlint --ktlint-version=1.8 -F` on a file `name` with `content`: the file's content afterwards.
fn ktlint_format(name: &str, content: &str) -> String {
    let dir = TempDir::new("lsp-oracle");
    let file = dir.path().join(name);
    write_text(&file, content);
    let report = format!("--reporter=plain,output={}", dir.path().join("report.txt").display());
    let args = ["ktlint", "--ktlint-version=1.8", "-F", "--log-level=none", &report, &file.display().to_string()];
    ktrs_cli::ktrs::run(&strings(&args));
    read_text(&file)
}

fn action<'a>(actions: &'a [(String, String, Vec<lsp_types::TextEdit>)], title: &str) -> &'a [lsp_types::TextEdit] {
    let found = actions.iter().find(|(t, _, _)| t == title);
    &found.unwrap_or_else(|| panic!("no '{title}' among {:?}", actions.iter().map(|a| &a.0).collect::<Vec<_>>())).2
}

#[test]
fn publishes_ktlint_diagnostics_on_open_and_change() {
    let dir = TempDir::new("lsp-diagnostics");
    let mut client = Client::start(Value::Null);
    let uri = client.open(&dir.path().join("A.kt"), ONE_VIOLATION);
    let diagnostics = client.diagnostics(&uri, Some(1));
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    let d = &diagnostics[0];
    assert_eq!(d.code, Some(NumberOrString::String("standard:no-multi-spaces".to_owned())));
    assert_eq!(d.range, Range::new(Position::new(0, 3), Position::new(0, 5)));
    assert_eq!((d.severity, d.source.as_deref()), (Some(DiagnosticSeverity::WARNING), Some("ktlint")));
    assert_eq!(d.data, Some(json!({"autocorrectable": true})));
    assert!(client.logs.iter().any(|l| l.contains("diagnostics: ktlint") && l.contains("V1_8")), "{:?}", client.logs);

    for version in 2..=5 {
        client.change(&uri, version, &format!("val  foo = {version}\n"));
    }
    client.change(&uri, 6, "val foo = 1\n");
    assert!(client.diagnostics(&uri, Some(6)).is_empty());
    client.shutdown();
}

#[test]
fn fix_one_and_fix_all_equal_ktlint_format() {
    let dir = TempDir::new("lsp-fix");
    let mut client = Client::start(Value::Null);
    let one = client.open(&dir.path().join("One.kt"), ONE_VIOLATION);
    let actions = client.code_actions(&one, None);
    let fixed = ktrs_lsp::apply_edits(ONE_VIOLATION, action(&actions, "Fix standard:no-multi-spaces"));
    assert_eq!(fixed, ktlint_format("One.kt", ONE_VIOLATION));

    let many = client.open(&dir.path().join("Many.kt"), MANY_VIOLATIONS);
    let actions = client.code_actions(&many, Some(&["source.fixAll"]));
    assert_eq!(actions.len(), 1, "only fix-all is a source.fixAll action");
    assert_eq!(actions[0].1, "source.fixAll.ktlint");
    let fixed = ktrs_lsp::apply_edits(MANY_VIOLATIONS, action(&actions, "Fix all autocorrectable ktlint violations"));
    let expected = ktlint_format("Many.kt", MANY_VIOLATIONS);
    assert_ne!(expected, MANY_VIOLATIONS);
    assert_eq!(fixed, expected);
    client.shutdown();
}

#[test]
fn suppresses_a_rule_on_the_line_or_in_the_file() {
    let dir = TempDir::new("lsp-suppress");
    let mut client = Client::start(Value::Null);
    let uri = client.open(&dir.path().join("A.kt"), ONE_VIOLATION);
    let actions = client.code_actions(&uri, Some(&["quickfix"]));
    let on_line = ktrs_lsp::apply_edits(ONE_VIOLATION, action(&actions, "Suppress standard:no-multi-spaces on this line"));
    assert!(on_line.contains("@Suppress(\"ktlint:standard:no-multi-spaces\")"), "{on_line}");
    let in_file = ktrs_lsp::apply_edits(ONE_VIOLATION, action(&actions, "Suppress standard:no-multi-spaces in this file"));
    assert!(in_file.starts_with("@file:Suppress(\"ktlint:standard:no-multi-spaces\")"), "{in_file}");
    client.change(&uri, 2, &on_line);
    assert!(client.diagnostics(&uri, Some(2)).is_empty());
    client.shutdown();
}

#[test]
fn formats_with_the_configured_tool() {
    let dir = TempDir::new("lsp-format");
    let mut client = Client::start(Value::Null);
    let uri = client.open(&dir.path().join("A.kt"), MANY_VIOLATIONS);
    assert_eq!(client.format(&uri), None, "no formatter by default");
    client.shutdown();

    let mut client = Client::start(json!({"ktrs": {"format": {"tool": "ktlint"}}}));
    let uri = client.open(&dir.path().join("A.kt"), MANY_VIOLATIONS);
    let formatted = ktrs_lsp::apply_edits(MANY_VIOLATIONS, &client.format(&uri).unwrap());
    assert_eq!(formatted, ktlint_format("A.kt", MANY_VIOLATIONS));
    client.shutdown();

    let mut client = Client::start(json!({"format": {"tool": "ktfmt"}, "ktfmt": {"style": "kotlinlang"}}));
    let uri = client.open(&dir.path().join("A.kt"), MANY_VIOLATIONS);
    let formatted = ktrs_lsp::apply_edits(MANY_VIOLATIONS, &client.format(&uri).unwrap());
    assert_eq!(formatted, ktrs_fmt::format(MANY_VIOLATIONS, ktrs_fmt::FileType::Regular, &ktrs_fmt::KOTLINLANG_FORMAT).unwrap());
    client.change(&uri, 2, &formatted);
    assert_eq!(client.format(&uri), Some(Vec::new()), "formatted code has no edits");
    client.shutdown();
}

#[test]
fn range_formatting_is_ktfmts_partial_formatting() {
    const CODE: &str = "fun untouched ( ) =   1\n\nfun test() {\n  val selected    =   2\n  val adjacent    =   3\n}\n";
    let dir = TempDir::new("lsp-range");
    let mut client = Client::start(json!({"format": {"tool": "ktfmt"}}));
    let uri = client.open(&dir.path().join("A.kt"), CODE);
    let selected = Range::new(Position::new(3, 6), Position::new(3, 14));
    let formatted = ktrs_lsp::apply_edits(CODE, &client.format_range(&uri, selected).unwrap());
    assert_eq!(formatted, "fun untouched ( ) =   1\n\nfun test() {\n  val selected = 2\n  val adjacent    =   3\n}\n");
    let cursor = Range::new(Position::new(4, 4), Position::new(4, 4));
    let formatted = ktrs_lsp::apply_edits(CODE, &client.format_range(&uri, cursor).unwrap());
    assert_eq!(formatted, "fun untouched ( ) =   1\n\nfun test() {\n  val selected    =   2\n  val adjacent = 3\n}\n");
    client.shutdown();

    let mut client = Client::start(json!({"format": {"tool": "ktlint"}}));
    let uri = client.open(&dir.path().join("A.kt"), CODE);
    assert_eq!(client.format_range(&uri, selected), None, "ktlint has no range formatting");
    client.shutdown();
}

#[test]
fn relints_open_documents_when_editorconfig_or_settings_change() {
    let dir = TempDir::new("lsp-watch");
    let mut client = Client::start(Value::Null);
    let uri = client.open(&dir.path().join("A.kt"), ONE_VIOLATION);
    assert_eq!(client.diagnostics(&uri, Some(1)).len(), 1);

    let editorconfig = dir.path().join(".editorconfig");
    write_text(&editorconfig, "root = true\n\n[*.{kt,kts}]\nktlint_standard_no-multi-spaces = disabled\n");
    let change = json!({"changes": [{"uri": ktrs_lsp::path_to_uri(&editorconfig), "type": 1}]});
    client.notify("workspace/didChangeWatchedFiles", change);
    assert!(client.diagnostics(&uri, Some(1)).is_empty());

    write_text(&editorconfig, "root = true\n");
    let change = json!({"changes": [{"uri": ktrs_lsp::path_to_uri(&editorconfig), "type": 2}]});
    client.notify("workspace/didChangeWatchedFiles", change);
    assert_eq!(client.diagnostics(&uri, Some(1)).len(), 1);

    client.notify("workspace/didChangeConfiguration", json!({"settings": {"ktrs": {"ktlint": {"enable": false}}}}));
    assert!(client.diagnostics(&uri, Some(1)).is_empty());
    client.shutdown();
}

#[test]
fn warns_once_about_rule_sets_it_cannot_run() {
    let dir = TempDir::new("lsp-rulesets");
    let jar = dir.path().join("custom.jar");
    let mut client = Client::start(json!({"ktlint": {"ruleSets": [jar]}}));
    let a = client.open(&dir.path().join("A.kt"), ONE_VIOLATION);
    assert_eq!(client.diagnostics(&a, Some(1)).len(), 1, "linted without the rule set");
    let b = client.open(&dir.path().join("B.kt"), ONE_VIOLATION);
    client.diagnostics(&b, Some(1));
    let warnings: Vec<&String> = client.shown.iter().filter(|m| m.contains("custom.jar")).collect();
    assert_eq!(warnings.len(), 1, "{:?}", client.shown);
    client.shutdown();
}
