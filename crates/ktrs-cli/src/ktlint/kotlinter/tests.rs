//! kotlinter's `SortedThreadSafeReporterWrapperTest`, over a recording reporter.

use std::sync::{Arc, Mutex};

use super::*;

#[derive(Clone, Default)]
struct Calls(Arc<Mutex<Vec<String>>>);

impl Calls {
    fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.0.lock().unwrap())
    }

    fn push(&self, call: String) {
        self.0.lock().unwrap().push(call);
    }
}

impl ReporterV2 for Calls {
    fn before_all(&mut self) {
        self.push("beforeAll".to_owned());
    }

    fn before(&mut self, file: &str) {
        self.push(format!("before {file}"));
    }

    fn on_lint_error(&mut self, file: &str, e: &KtlintCliError) {
        self.push(format!("onLintError {file} {}:{} {}", e.line, e.col, e.rule_id));
    }

    fn after(&mut self, file: &str) {
        self.push(format!("after {file}"));
    }

    fn after_all(&mut self) {
        self.push("afterAll".to_owned());
    }
}

fn wrapper() -> (SortedThreadSafeReporterWrapper, Calls) {
    let calls = Calls::default();
    (SortedThreadSafeReporterWrapper::new(Box::new(calls.clone())), calls)
}

fn error(line: usize, col: usize, rule: &str) -> KtlintCliError {
    KtlintCliError::new(line, col, rule, "", Status::LintCanNotBeAutocorrected)
}

#[test]
fn before_all_is_delegated_to_wrapper() {
    let (mut reporter, calls) = wrapper();
    reporter.before_all();
    assert_eq!(calls.take(), ["beforeAll"]);
}

#[test]
fn before_on_lint_error_and_after_are_delegated_in_after_all_only() {
    let (mut reporter, calls) = wrapper();
    reporter.before("fileName");
    reporter.on_lint_error("fileName", &error(0, 0, "custom:x"));
    reporter.after("fileName");
    assert!(calls.take().is_empty());
    reporter.after_all();
    assert_eq!(calls.take(), ["before fileName", "onLintError fileName 0:0 custom:x", "after fileName", "afterAll"]);
}

#[test]
fn after_all_delegates_in_sorted_order() {
    let (mut reporter, calls) = wrapper();
    reporter.before("b");
    reporter.on_lint_error("b", &error(0, 0, "custom:x"));
    reporter.after("b");
    reporter.before("a");
    reporter.on_lint_error("a", &error(1, 0, "custom:x"));
    reporter.on_lint_error("a", &error(3, 2, "custom:x"));
    reporter.on_lint_error("a", &error(2, 6, "custom:x"));
    reporter.after("a");
    reporter.after_all();
    assert_eq!(
        calls.take(),
        [
            "before a",
            "onLintError a 1:0 custom:x",
            "onLintError a 2:6 custom:x",
            "onLintError a 3:2 custom:x",
            "after a",
            "before b",
            "onLintError b 0:0 custom:x",
            "after b",
            "afterAll",
        ]
    );
}

#[test]
fn a_second_error_at_a_position_is_dropped() {
    let (mut reporter, calls) = wrapper();
    reporter.on_lint_error("a", &error(1, 1, "custom:first"));
    reporter.on_lint_error("a", &error(1, 1, "custom:second"));
    reporter.after_all();
    assert_eq!(calls.take(), ["onLintError a 1:1 custom:first", "afterAll"]);
}

#[test]
fn files_are_keyed_by_the_path_each_call_got() {
    let (mut reporter, calls) = wrapper();
    reporter.before("src/A.kt");
    reporter.on_lint_error("/project/src/A.kt", &error(1, 1, "custom:x"));
    reporter.after("src/A.kt");
    reporter.after_all();
    assert_eq!(calls.take(), ["onLintError /project/src/A.kt 1:1 custom:x", "before src/A.kt", "after src/A.kt", "afterAll"]);
}

#[test]
fn sarif_gets_the_absolute_path() {
    assert_eq!(reporter_path_for(true, "/project/src/A.kt", "src/A.kt"), "/project/src/A.kt");
    assert_eq!(reporter_path_for(false, "/project/src/A.kt", "src/A.kt"), "src/A.kt");
}
