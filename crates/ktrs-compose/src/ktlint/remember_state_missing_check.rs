//! Port of `ktlint/RememberStateMissingCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::remember_state_missing::RememberStateMissing;

pub const CHECK: Option<fn() -> KtlintRule> = Some(remember_state_missing_check);

pub fn remember_state_missing_check() -> KtlintRule {
    KtlintRule::new("compose:remember-missing-check", vec![], Box::new(RememberStateMissing))
}
