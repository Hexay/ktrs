//! Port of `ktlint/StateParameterCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::state_parameter::StateParameter;

pub const CHECK: Option<fn() -> KtlintRule> = Some(state_parameter_check);

pub fn state_parameter_check() -> KtlintRule {
    KtlintRule::new("compose:state-param-check", vec![], Box::new(StateParameter))
}
