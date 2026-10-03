//! Port of `ktlint/PreviewAnnotationNamingCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::preview_annotation_naming::PreviewAnnotationNaming;

pub const CHECK: Option<fn() -> KtlintRule> = Some(preview_annotation_naming_check);

pub fn preview_annotation_naming_check() -> KtlintRule {
    KtlintRule::new("compose:preview-annotation-naming", vec![], Box::new(PreviewAnnotationNaming))
}
