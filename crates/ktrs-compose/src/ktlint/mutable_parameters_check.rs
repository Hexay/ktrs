//! Port of `ktlint/MutableParametersCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::mutable_parameters::MutableParameters;

pub const CHECK: Option<fn() -> KtlintRule> = Some(mutable_parameters_check);

pub fn mutable_parameters_check() -> KtlintRule {
    KtlintRule::new("compose:mutable-params-check", vec![], Box::new(MutableParameters))
}
