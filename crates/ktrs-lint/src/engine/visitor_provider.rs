//! Ports of ktlint-rule-engine `internal/VisitorProvider.kt` and `internal/RuleProviderSorter.kt`.

use crate::rule::{RuleId, RuleSetId, RuleV2};
use crate::rule_provider::RuleV2Provider;

/// `RuleProviderSorter.getSortedRuleProviders`: the standard rule set first, then by rule id.
pub fn get_sorted_rule_providers(rule_providers: &[RuleV2Provider]) -> Vec<RuleV2Provider> {
    let mut sorted = rule_providers.to_vec();
    sorted.sort_by_key(|p| {
        (
            p.rule_id().rule_set_id() != RuleSetId::STANDARD,
            p.rule_id().value(),
        )
    });
    sorted
}

/// The standard rules that 1.8's `VisitorModifier`s (`RunAfterRule`, `RunAsLateAsPossible`) move behind all others,
/// in the order its `RuleProviderSorter` gives them (`ktlint-1.8.0 --log-level=debug`).
const KTLINT_1_8_LATE_RULES: [&str; 16] = [
    "standard:annotation",
    "standard:modifier-list-spacing",
    "standard:no-single-line-block-comment",
    "standard:wrapping",
    "standard:no-semi",
    "standard:class-signature",
    "standard:function-signature",
    "standard:argument-list-wrapping",
    "standard:chain-method-continuation",
    "standard:function-literal",
    "standard:trailing-comma-on-call-site",
    "standard:trailing-comma-on-declaration-site",
    "standard:indent",
    "standard:block-comment-initial-star-alignment",
    "standard:string-template-indent",
    "standard:max-line-length",
];

/// A rule's place in 1.8's rule-major execution order: the suppression rule, the others by id, then
/// [`KTLINT_1_8_LATE_RULES`]. Lint uses it to order errors at the same position as 1.8 emits them; the `-F`
/// traversal itself is still 2.0's (phase 2: research/26-ktlint-18-mode.md).
pub(crate) fn ktlint_1_8_rule_rank(rule_id: RuleId) -> (u8, usize, &'static str) {
    if rule_id.rule_set_id() != RuleSetId::STANDARD {
        return (0, 0, rule_id.value());
    }
    match KTLINT_1_8_LATE_RULES.iter().position(|it| *it == rule_id.value()) {
        Some(index) => (2, index, ""),
        None => (1, 0, rule_id.value()),
    }
}

/// `VisitorProvider(ruleProviders).rules`: fresh instances in execution order.
pub struct VisitorProvider {
    rule_providers_sorted: Vec<RuleV2Provider>,
}

impl VisitorProvider {
    pub fn new(rule_providers: &[RuleV2Provider]) -> VisitorProvider {
        VisitorProvider {
            rule_providers_sorted: get_sorted_rule_providers(rule_providers),
        }
    }

    pub fn rules(&self) -> Vec<Box<dyn RuleV2>> {
        self.rule_providers_sorted
            .iter()
            .map(RuleV2Provider::create_new_rule_instance)
            .collect()
    }
}
