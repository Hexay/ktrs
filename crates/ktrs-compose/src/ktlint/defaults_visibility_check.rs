//! Port of `ktlint/DefaultsVisibilityCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::defaults_visibility::DefaultsVisibility;

pub const CHECK: Option<fn() -> KtlintRule> = Some(defaults_visibility_check);

pub fn defaults_visibility_check() -> KtlintRule {
    KtlintRule::new("compose:defaults-visibility", vec![], Box::new(DefaultsVisibility))
}
