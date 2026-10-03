//! Port of `ktlint/ParameterOrderCheck.kt`.

use crate::ktlint::editor_config_properties::{ComposeProperty, TREAT_AS_COMPOSABLE_LAMBDA, TREAT_AS_LAMBDA};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::parameter_order::ParameterOrder;

pub const CHECK: Option<fn() -> KtlintRule> = Some(parameter_order_check);

pub fn parameter_order_check() -> KtlintRule {
    KtlintRule::new(
        "compose:param-order-check",
        vec![ComposeProperty::String(&TREAT_AS_LAMBDA), ComposeProperty::String(&TREAT_AS_COMPOSABLE_LAMBDA)],
        Box::new(ParameterOrder),
    )
}
