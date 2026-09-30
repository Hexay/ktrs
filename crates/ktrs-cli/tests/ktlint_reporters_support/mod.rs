#![allow(dead_code)]

use ktrs_cli::ktlint::console::LINE_SEPARATOR;
use ktrs_cli::ktlint::reporter::{KtlintCliError, ReporterOptions, ReporterV2, Status};

pub use Status::{
    FormatIsAutocorrected as FIXED, KotlinParseException as PARSE, LintCanBeAutocorrected as CAN,
    LintCanNotBeAutocorrected as CANNOT,
};

pub fn e(line: usize, col: usize, rule: &str, detail: &str, status: Status) -> KtlintCliError {
    KtlintCliError::new(line, col, rule, detail, status)
}

/// `"""...""".trimIndent().replace("\n", System.lineSeparator())`.
pub fn lines(text: &str) -> String {
    text.replace('\n', LINE_SEPARATOR)
}

pub fn opts(pairs: &[(&str, &str)]) -> ReporterOptions {
    ReporterOptions(pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect())
}

/// The five errors most upstream reporter tests feed in.
pub fn feed_standard(reporter: &mut dyn ReporterV2, prefix: &str) {
    let f = |name: &str| format!("{prefix}{name}");
    reporter.on_lint_error(&f("one-fixed-and-one-not.kt"), &e(1, 1, "rule-1", "<\"&'>", CAN));
    reporter.on_lint_error(&f("one-fixed-and-one-not.kt"), &e(2, 1, "rule-2", "And if you see my friend", FIXED));
    reporter.on_lint_error(&f("two-not-fixed.kt"), &e(1, 10, "rule-1", "I thought I would again", CAN));
    reporter.on_lint_error(&f("two-not-fixed.kt"), &e(2, 20, "rule-2", "A single thin straight line", CAN));
    reporter.on_lint_error(&f("all-corrected.kt"), &e(1, 1, "rule-1", "I thought we had more time", FIXED));
}
