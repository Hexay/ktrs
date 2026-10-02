//! Port of `ktlint/CompositionLocalNamingCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::composition_local_naming::CompositionLocalNaming;

pub const CHECK: Option<fn() -> KtlintRule> = Some(composition_local_naming_check);

pub fn composition_local_naming_check() -> KtlintRule {
    KtlintRule::new("compose:compositionlocal-naming", vec![], Box::new(CompositionLocalNaming))
}
