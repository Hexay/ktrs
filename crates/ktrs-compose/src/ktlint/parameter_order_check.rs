//! Port of `ktlint/ParameterOrderCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(parameter_order_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
