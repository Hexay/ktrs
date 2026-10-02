//! Port of compose-rules `ComposeRuleSetProvider.kt` (rule set id `compose`).

use ktrs_lint::rule_provider::RuleV2Provider;

/// `getRuleProviders()`, in upstream's order.
pub fn compose_rule_providers() -> Vec<RuleV2Provider> {
    Vec::new()
}
