//! Ports of ktlint-rule-engine `internal/VisitorProvider.kt` and `internal/RuleProviderSorter.kt`.

use crate::rule::{RuleSetId, RuleV2};
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
