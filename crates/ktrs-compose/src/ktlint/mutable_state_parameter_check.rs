//! Port of `ktlint/MutableStateParameterCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(mutable_state_parameter_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
