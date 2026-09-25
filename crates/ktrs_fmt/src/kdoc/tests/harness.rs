//! Port of `KDocFormatterTest.checkFormatter` plus the reader for the case files in `data/`.
//!
//! Case file format: each case starts with `=== case`, then `key: value` lines (`options:` holds
//! `maxLineWidth maxCommentWidth` and non-default `name=value` pairs; indents are quoted), then
//! `comment:` and `expected:` bodies whose lines are prefixed with `|`. A body given as
//! `@file.txt` is read from that sibling file instead (keeps files under 300 lines).

use crate::kdoc::kstring::{KStr, ks, to_string};
use crate::kdoc::{FormattingTask, KDocFormatter, KDocFormattingOptions, comment_type};

pub struct Case {
    pub task: FormattingTask,
    pub expected: String,
    pub verify: bool,
}

/// Bodies split out of their case file (`@name` references).
fn side_file(name: &str) -> &'static str {
    match name {
        "testHtml.0.comment.txt" => include_str!("data/testHtml.0.comment.txt"),
        "testHtml.0.expected.txt" => include_str!("data/testHtml.0.expected.txt"),
        _ => panic!("unknown side file {name}"),
    }
}

fn unquote(s: &str) -> String {
    let inner = s.strip_prefix('"').and_then(|s| s.strip_suffix('"')).expect("quoted string");
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        out.push(if c == '\\' { chars.next().expect("escape") } else { c });
    }
    out
}

fn parse_options(value: &str) -> KDocFormattingOptions {
    let mut parts = value.split(' ');
    let max_line_width = parts.next().unwrap().parse().unwrap();
    let max_comment_width = parts.next().unwrap().parse().unwrap();
    let mut o = KDocFormattingOptions::new(max_line_width, max_comment_width);
    for part in parts {
        let (k, v) = part.split_once('=').expect("name=value");
        let b = || v.parse::<bool>().unwrap();
        let n = || v.parse::<i32>().unwrap();
        match k {
            "collapseSingleLine" => o.collapse_single_line = b(),
            "collapseSpaces" => o.collapse_spaces = b(),
            "convertMarkup" => o.convert_markup = b(),
            "addPunctuation" => o.add_punctuation = b(),
            "hangingIndent" => o.hanging_indent = n(),
            "nestedListIndent" => o.set_nested_list_indent(n()),
            "tabWidth" => o.tab_width = n(),
            "optimal" => o.optimal = b(),
            "alignTableColumns" => o.align_table_columns = b(),
            "orderDocTags" => o.order_doc_tags = b(),
            "alternate" => o.alternate = b(),
            "allowParamBrackets" => o.allow_param_brackets = b(),
            _ => panic!("unknown option {k}"),
        }
    }
    o
}

fn read_side_file(value: &str) -> String {
    let file = value.strip_prefix('@').expect("inline body or @file");
    let text = side_file(file).replace("\r\n", "\n");
    text.lines().map(|l| l.strip_prefix('|').expect("| prefix")).collect::<Vec<_>>().join("\n")
}

pub fn load_cases(name: &str, text: &str) -> Vec<Case> {
    let text = text.replace("\r\n", "\n");
    let mut cases = Vec::new();
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        assert_eq!(line, "=== case", "{name}: expected case header");
        let mut options = None;
        let (mut initial, mut secondary, mut names, mut verify) = (None, None, Vec::new(), true);
        let (mut comment, mut expected) = (None, None);
        while let Some(&line) = lines.peek() {
            if line == "=== case" {
                break;
            }
            lines.next();
            let (key, value) = line.split_once(':').expect("key: value");
            let value = value.trim_start();
            match key {
                "options" => options = Some(parse_options(value)),
                "initialIndent" => initial = Some(unquote(value)),
                "secondaryIndent" => secondary = Some(unquote(value)),
                "orderedParameterNames" => names = value.split(',').map(str::to_string).collect(),
                "verify" => verify = value.parse().unwrap(),
                "comment" | "expected" => {
                    let body = if value.is_empty() {
                        let mut body = Vec::new();
                        while let Some(content) = lines.peek().and_then(|l| l.strip_prefix('|')) {
                            body.push(content);
                            lines.next();
                        }
                        body.join("\n")
                    } else {
                        read_side_file(value)
                    };
                    if key == "comment" { comment = Some(body) } else { expected = Some(body) }
                }
                _ => panic!("{name}: unknown key {key}"),
            }
        }
        let comment = comment.expect("comment");
        let initial_indent = initial.expect("initialIndent");
        let task = FormattingTask {
            options: options.expect("options"),
            comment_type: comment_type(&comment),
            comment,
            secondary_indent: secondary.unwrap_or_else(|| initial_indent.clone()),
            initial_indent,
            ordered_parameter_names: names,
        };
        cases.push(Case { task, expected: expected.expect("expected"), verify });
    }
    cases
}

fn reformat_comment(task: &FormattingTask) -> String {
    let formatter = KDocFormatter::new(task.options);
    let formatted = formatter.reformat_comment_task(task);
    format!("{}{}", task.initial_indent, formatted)
}

/// `checkFormatter(task, expected, verify)`; Dokka verification is not ported.
pub fn check_formatter(task: &FormattingTask, expected: &str, verify: bool) {
    let reformatted = reformat_comment(task);
    let indent = &task.initial_indent;

    // Because .trimIndent() will remove it:
    let indented_expected =
        expected.split('\n').map(|it| format!("{indent}{it}")).collect::<Vec<_>>().join("\n");

    assert_eq!(reformatted, indented_expected);

    // Make sure that formatting is stable -- format again and make sure it's the same
    if verify {
        let comment = to_string(ks(&reformatted).trim());
        let again = FormattingTask {
            options: task.options,
            comment_type: comment_type(&comment),
            comment,
            initial_indent: task.initial_indent.clone(),
            secondary_indent: task.secondary_indent.clone(),
            ordered_parameter_names: task.ordered_parameter_names.clone(),
        };
        let formatted_again = reformat_comment(&again);
        if reformatted != formatted_again {
            assert_eq!(
                format!("{indent}// FORMATTED TWICE (implies unstable formatting)\n\n{formatted_again}"),
                format!("{indent}// FORMATTED ONCE\n\n{reformatted}"),
                "Formatting is unstable: if formatted a second time, it changes"
            );
        }
    }
}

pub fn run_cases(name: &str, text: &str) {
    let cases = load_cases(name, text);
    assert!(!cases.is_empty(), "{name}: no cases");
    for case in &cases {
        check_formatter(&case.task, &case.expected, case.verify);
    }
}
