//! Port of ktlint-rule-engine `internal/rules/`: rules only the engine can declare (`InternalRule.kt`).

mod ktlint_directive;
mod ktlint_suppression_rule;

pub use ktlint_suppression_rule::KtlintSuppressionRule;

use crate::rule::{About, RuleId, RuleV2};
use crate::rule_provider::RuleV2Provider;

pub const INTERNAL_RULE_ABOUT: About = About {
    maintainer: "KtLint",
    repository_url: "https://github.com/ktlint/ktlint",
    issue_tracker_url: "https://github.com/ktlint/ktlint/issues",
};

pub const KTLINT_SUPPRESSION_RULE_ID: RuleId = RuleId("internal:ktlint-suppression");

/// The provider `InternalRuleProvidersFilter` adds; the rule knows the ids of the engine's rules.
pub fn ktlint_suppression_rule_provider(allowed_rule_ids: Vec<RuleId>) -> RuleV2Provider {
    RuleV2Provider::new(move || {
        Box::new(KtlintSuppressionRule::new(allowed_rule_ids.clone())) as Box<dyn RuleV2>
    })
}
