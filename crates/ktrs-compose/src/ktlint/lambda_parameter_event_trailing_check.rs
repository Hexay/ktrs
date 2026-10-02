//! Port of `ktlint/LambdaParameterEventTrailingCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(lambda_parameter_event_trailing_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
