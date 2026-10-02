//! Port of `ktlint/LambdaParameterInRestartableEffectCheck.kt`.

use crate::ktlint::editor_config_properties::{ComposeProperty, TREAT_AS_LAMBDA};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::lambda_parameter_in_restartable_effect::LambdaParameterInRestartableEffect;

pub const CHECK: Option<fn() -> KtlintRule> = Some(lambda_parameter_in_restartable_effect_check);

pub fn lambda_parameter_in_restartable_effect_check() -> KtlintRule {
    KtlintRule::new(
        "compose:lambda-param-in-effect",
        vec![ComposeProperty::String(&TREAT_AS_LAMBDA)],
        Box::new(LambdaParameterInRestartableEffect),
    )
}
