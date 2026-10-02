//! Port of `ktlint/MutableStateAutoboxingCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(mutable_state_autoboxing_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
