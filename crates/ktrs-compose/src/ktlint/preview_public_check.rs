//! Port of `ktlint/PreviewPublicCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::preview_public::PreviewPublic;

pub const CHECK: Option<fn() -> KtlintRule> = Some(preview_public_check);

pub fn preview_public_check() -> KtlintRule {
    KtlintRule::new("compose:preview-public-check", vec![], Box::new(PreviewPublic))
}
