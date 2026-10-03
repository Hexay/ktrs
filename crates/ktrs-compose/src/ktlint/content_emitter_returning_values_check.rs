//! Port of `ktlint/ContentEmitterReturningValuesCheck.kt`.

use crate::ktlint::editor_config_properties::{CONTENT_EMITTERS_PROPERTY, ComposeProperty};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::content_emitter_returning_values::ContentEmitterReturningValues;

pub const CHECK: Option<fn() -> KtlintRule> = Some(content_emitter_returning_values_check);

pub fn content_emitter_returning_values_check() -> KtlintRule {
    KtlintRule::new(
        "compose:content-emitter-returning-values-check",
        vec![ComposeProperty::String(&CONTENT_EMITTERS_PROPERTY)],
        Box::new(ContentEmitterReturningValues),
    )
}
