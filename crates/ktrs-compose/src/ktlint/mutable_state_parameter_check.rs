//! Port of `ktlint/MutableStateParameterCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::mutable_state_parameter::MutableStateParameter;

pub const CHECK: Option<fn() -> KtlintRule> = Some(mutable_state_parameter_check);

pub fn mutable_state_parameter_check() -> KtlintRule {
    KtlintRule::new("compose:mutable-state-param-check", vec![], Box::new(MutableStateParameter))
}
