//! Port of `ktlint/ParameterNamingCheck.kt`.

use crate::ktlint::editor_config_properties::{ALLOWED_LAMBDA_PARAMETER_NAMES, ComposeProperty, TREAT_AS_LAMBDA};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::parameter_naming::ParameterNaming;

pub const CHECK: Option<fn() -> KtlintRule> = Some(parameter_naming_check);

pub fn parameter_naming_check() -> KtlintRule {
    KtlintRule::new(
        "compose:parameter-naming",
        vec![ComposeProperty::String(&TREAT_AS_LAMBDA), ComposeProperty::String(&ALLOWED_LAMBDA_PARAMETER_NAMES)],
        Box::new(ParameterNaming),
    )
}
