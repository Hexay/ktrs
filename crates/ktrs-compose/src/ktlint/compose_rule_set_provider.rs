//! Port of `ComposeRuleSetProvider.kt` (rule set id `compose`).

use ktrs_lint::rule_provider::RuleV2Provider;

use super::*;
use crate::ktlint::ktlint_rule::KtlintRule;

/// `getRuleProviders()`, in upstream's order; a rule whose `CHECK` is still `None` is not ported yet and left out.
pub fn compose_rule_providers() -> Vec<RuleV2Provider> {
    CHECKS.iter().flatten().map(|&create| KtlintRule::provider(create)).collect()
}

const CHECKS: [Option<fn() -> KtlintRule>; 34] = [
    composable_annotation_naming_check::CHECK,
    composable_nesting_depth_check::CHECK,
    composition_local_allowlist_check::CHECK,
    composition_local_naming_check::CHECK,
    content_emitter_returning_values_check::CHECK,
    content_slot_reused_check::CHECK,
    content_trailing_lambda_check::CHECK,
    defaults_visibility_check::CHECK,
    lambda_parameter_event_trailing_check::CHECK,
    lambda_parameter_in_restartable_effect_check::CHECK,
    material2_check::CHECK,
    modifier_clickable_order_check::CHECK,
    modifier_composed_check::CHECK,
    modifier_missing_check::CHECK,
    modifier_naming_check::CHECK,
    modifier_not_used_at_root_check::CHECK,
    modifier_reused_check::CHECK,
    modifier_without_default_check::CHECK,
    multiple_content_emitters_check::CHECK,
    mutable_parameters_check::CHECK,
    mutable_state_autoboxing_check::CHECK,
    mutable_state_parameter_check::CHECK,
    state_parameter_check::CHECK,
    naming_check::CHECK,
    parameter_naming_check::CHECK,
    parameter_order_check::CHECK,
    preview_annotation_naming_check::CHECK,
    preview_naming_check::CHECK,
    preview_public_check::CHECK,
    remember_content_missing_check::CHECK,
    remember_state_missing_check::CHECK,
    unstable_collections_check::CHECK,
    view_model_forwarding_check::CHECK,
    view_model_injection_check::CHECK,
];
