//! Port of `ktlint/RememberContentMissingCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::remember_content_missing::RememberContentMissing;

pub const CHECK: Option<fn() -> KtlintRule> = Some(remember_content_missing_check);

pub fn remember_content_missing_check() -> KtlintRule {
    KtlintRule::new("compose:remember-content-missing-check", vec![], Box::new(RememberContentMissing))
}
