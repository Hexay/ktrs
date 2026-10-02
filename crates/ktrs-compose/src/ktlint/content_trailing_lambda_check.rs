//! Port of `ktlint/ContentTrailingLambdaCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(content_trailing_lambda_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
