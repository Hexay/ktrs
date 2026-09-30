//! `ktlintSuppressionRuleAssertThat` of `KtlintSuppressionRuleTest.kt` and its messages.

use ktrs_lint::engine::internal_rules::KtlintSuppressionRule;

use super::assert_that::{AssertThat, DummyRule, assert_that_rule_builder, trim_indent};

pub const DISABLE: &str =
    "Directive 'ktlint-disable' is deprecated. Replace with @Suppress annotation";
pub const ENABLE: &str =
    "Directive 'ktlint-enable' is obsolete after migrating to suppress annotations";
pub const QUALIFY: &str =
    "Identifier to suppress ktlint rule must be fully qualified with the rule set id";
pub const DANGLING: &str =
    "Directive 'ktlint-disable' in EOL comment is ignored as it is not preceded by a code element";
pub const NO_MATCHING_ENABLE: &str = "Directive 'ktlint-disable' is deprecated. The matching 'ktlint-enable' directive is not found in \
                                      same scope. Replace with @Suppress annotation";

/// A dummy rule for each rule id the tests use in a directive or suppression, so those ids are loaded.
pub fn ktlint_suppression_rule_assert_that(code: &str) -> AssertThat {
    let mut builder = assert_that_rule_builder(|| Box::new(KtlintSuppressionRule::new(Vec::new())));
    for id in [
        "standard:no-wildcard-imports",
        "standard:no-multi-spaces",
        "standard:max-line-length",
        "standard:package-name",
        "custom:foo",
        "standard:bar",
        "standard:foo",
    ] {
        builder = builder.add_additional_rule_provider(move || Box::new(DummyRule(id)));
    }
    builder.code(&trim_indent(code))
}
