//! `code.atOffset(line, col).insertKtlintRuleSuppression(ids)` of `KtlintSuppressionKtTest.kt`, and the
//! engine of `KtlintRuleEngineSuppressionKtTest.kt`. The internal `insertKtlintRuleSuppression` is reached
//! through `insert_suppression`, which inserts `setOf(ruleId.value)`: `"standard:x"` qualifies to the same
//! `"ktlint:standard:x"` the Kotlin tests pass, and a file suppression starts at the root, where
//! `forceFileAnnotation` ends up too.

use ktrs_lint::engine::KtlintSuppression;
use ktrs_lint::{Code, KtLintRuleEngine, RuleId, RuleV2Provider};

use super::assert_that::{DummyRule, trim_indent};

pub const SOME_RULE_ID: RuleId = RuleId("standard:some-rule-id");

pub fn engine() -> KtLintRuleEngine {
    KtLintRuleEngine::new(vec![RuleV2Provider::new(|| {
        Box::new(DummyRule(SOME_RULE_ID.0))
    })])
}

pub fn insert(code: &str, suppression: KtlintSuppression) -> String {
    engine()
        .insert_suppression(&Code::from_snippet(code, false), &suppression)
        .unwrap_or_else(|e| panic!("{e:?}"))
}

/// `code.atOffset(line, col).insertKtlintRuleSuppression(setOf("ktlint:<rule_id>"))`; `code` is trimIndent-ed.
pub fn at_offset(code: &str, line: usize, col: usize, rule_id: &'static str) -> String {
    let code = trim_indent(code);
    let code_line = code.split('\n').nth(line - 1).expect("line <= lines.size");
    assert!(col <= code_line.chars().count(), "col <= codeLine.length");
    insert(
        &code,
        KtlintSuppression::AtOffset {
            line,
            col,
            rule_id: RuleId(rule_id),
        },
    )
}

/// `charAtOffset()` of `atOffset(line, col)`.
pub fn char_at(code: &str, line: usize, col: usize) -> char {
    trim_indent(code)
        .split('\n')
        .nth(line - 1)
        .unwrap()
        .chars()
        .nth(col - 1)
        .unwrap()
}
