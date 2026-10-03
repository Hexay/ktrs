//! Port of `ktlint/ContentTrailingLambdaCheck.kt`.

use crate::ktlint::editor_config_properties::{ComposeProperty, TREAT_AS_COMPOSABLE_LAMBDA, TREAT_AS_LAMBDA};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::content_trailing_lambda::ContentTrailingLambda;

pub const CHECK: Option<fn() -> KtlintRule> = Some(content_trailing_lambda_check);

pub fn content_trailing_lambda_check() -> KtlintRule {
    KtlintRule::new(
        "compose:content-trailing-lambda",
        vec![ComposeProperty::String(&TREAT_AS_LAMBDA), ComposeProperty::String(&TREAT_AS_COMPOSABLE_LAMBDA)],
        Box::new(ContentTrailingLambda),
    )
}
