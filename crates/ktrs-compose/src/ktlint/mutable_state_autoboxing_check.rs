//! Port of `ktlint/MutableStateAutoboxingCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::mutable_state_autoboxing::MutableStateAutoboxing;

pub const CHECK: Option<fn() -> KtlintRule> = Some(mutable_state_autoboxing_check);

pub fn mutable_state_autoboxing_check() -> KtlintRule {
    KtlintRule::new("compose:mutable-state-autoboxing", vec![], Box::new(MutableStateAutoboxing))
}
