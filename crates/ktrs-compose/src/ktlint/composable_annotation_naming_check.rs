//! Port of `ktlint/ComposableAnnotationNamingCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::composable_annotation_naming::ComposableAnnotationNaming;

pub const CHECK: Option<fn() -> KtlintRule> = Some(composable_annotation_naming_check);

pub fn composable_annotation_naming_check() -> KtlintRule {
    KtlintRule::new("compose:composable-annotation-naming", vec![], Box::new(ComposableAnnotationNaming))
}
