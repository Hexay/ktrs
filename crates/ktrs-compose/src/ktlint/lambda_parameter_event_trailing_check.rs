//! Port of `ktlint/LambdaParameterEventTrailingCheck.kt`.

use crate::ktlint::editor_config_properties::{CONTENT_EMITTERS_DENYLIST, CONTENT_EMITTERS_PROPERTY, ComposeProperty};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::lambda_parameter_event_trailing::LambdaParameterEventTrailing;

pub const CHECK: Option<fn() -> KtlintRule> = Some(lambda_parameter_event_trailing_check);

pub fn lambda_parameter_event_trailing_check() -> KtlintRule {
    KtlintRule::new(
        "compose:lambda-param-event-trailing",
        vec![ComposeProperty::String(&CONTENT_EMITTERS_PROPERTY), ComposeProperty::String(&CONTENT_EMITTERS_DENYLIST)],
        Box::new(LambdaParameterEventTrailing),
    )
}
